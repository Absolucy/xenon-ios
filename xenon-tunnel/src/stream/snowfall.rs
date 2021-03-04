/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use crate::SIZE_LIMIT;
use anyhow::Result;
use bytes::{Buf, BytesMut};
use nano_leb128::{LEB128DecodeError, ULEB128};
use snow::TransportState;
use std::io::{Cursor, Error as IoError, ErrorKind as IoErrorKind};
use tokio_util::codec::{Decoder, Encoder};

pub struct SnowfallStream {
	snowfall: TransportState,
	encryption_buf: Vec<u8>,
	compression_buf: Vec<u8>,
}

impl SnowfallStream {
	pub fn new(snowfall: TransportState) -> Self {
		Self {
			snowfall,
			encryption_buf: Vec::with_capacity(65535),
			compression_buf: Vec::with_capacity(65535),
		}
	}
}

impl Encoder<&[u8]> for SnowfallStream {
	type Error = IoError;

	fn encode(&mut self, item: &[u8], dst: &mut BytesMut) -> Result<(), Self::Error> {
		// Resize the compression buffer to the size of a u64, which should be the upper limit of a ULEB128
		self.compression_buf.resize(std::mem::size_of::<u64>(), 0);
		// Encode the size of our uncompressed data as an ULEB128 into our compression buffer
		ULEB128::from(item.len() as u64)
			.write_into(&mut self.compression_buf)
			.map(|len| self.compression_buf.truncate(len))
			.map_err(|_| IoError::new(IoErrorKind::InvalidInput, "failed to encode ULEB128"))
			.expect("failed to encode uleb128");
		// Compress into our compression buffer, appending it to the dcompressed length
		lz4_flex::compress_into(&item, &mut self.compression_buf);
		// Resize the encryption buffer, to be the size of the compressed data plus the 16-byte NPF tag.
		self.encryption_buf
			.resize(self.compression_buf.len() + 16, 0);
		// Encrypt our now-compressed data, getting the slice of the encrypted+compressed data.
		let ec_req = self
			.snowfall
			.write_message(&self.compression_buf, &mut self.encryption_buf)
			.map(|len| &self.encryption_buf[..len])
			.map_err(|e| IoError::new(IoErrorKind::InvalidInput, e.to_string()))
			.expect("failed to encrypt compressed data");
		// Encode the size of our encrypted+compressed data into the output buffer.
		let mut length_header = [0u8; std::mem::size_of::<u64>()];
		let length_header = ULEB128::from(ec_req.len() as u64)
			.write_into(&mut length_header)
			.map(|len| &mut length_header[..len])
			.map_err(|_| IoError::new(IoErrorKind::InvalidInput, "failed to encode ULEB128"))
			.expect("failed to encode uleb128");
		// Now, copy our encrypted-compressed data into it.
		dst.extend_from_slice(&length_header);
		dst.extend_from_slice(ec_req);
		Ok(())
	}
}

impl Decoder for SnowfallStream {
	type Item = Cursor<Vec<u8>>;
	type Error = IoError;

	fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
		// Read the length, handling possible errors or invalid lengths.
		let (length, offset) = match ULEB128::read_from(&src) {
			Ok((length, offset)) => {
				let length = u64::from(length) as usize;
				match length {
					0 => Err(IoError::new(IoErrorKind::InvalidData, "length is zero")),
					SIZE_LIMIT..=usize::MAX => Err(IoError::new(
						IoErrorKind::InvalidData,
						"length exceeds 65535",
					)),
					_ => Ok((length as usize, offset)),
				}?
			}
			Err(err) => match err {
				LEB128DecodeError::IntegerOverflow => {
					return Err(IoError::new(IoErrorKind::InvalidData, "length overflowed"))
				}
				LEB128DecodeError::BufferOverflow => return Ok(None),
			},
		};
		// If the remaining length isn't there, reserve the space needed for it and wait
		if src.len() < offset + length {
			src.reserve(offset + length - src.len());
			return Ok(None);
		}
		// Alright, we have our whole message now.
		let data = src[offset..offset + length].to_vec();
		src.advance(offset + length);
		// Ensure our decryption buffer is the right size
		self.encryption_buf.resize(length, 0);
		let encryption_buf = &mut self.encryption_buf;
		// Decrypt the message, putting it into our encryption buffer
		let decrypted_msg = self
			.snowfall
			.read_message(&data, encryption_buf)
			.map(|len| &encryption_buf[..len])
			.map_err(|err| IoError::new(IoErrorKind::InvalidData, err.to_string()))?;
		// Now, we read the decompressed size from the decrypted message.
		let (decompressed_size, start_at) = ULEB128::read_from(decrypted_msg)
			.map(|(num, len)| (u64::from(num) as usize, len))
			.map_err(IoError::from)?;
		// Get a slice of where the decompression length part ended
		let compressed_msg = &decrypted_msg[start_at..];
		// Resize our decompression buffer to the size of the decompressed content.
		self.compression_buf.resize(decompressed_size, 0);
		// Alright, actually decompress.
		lz4_flex::decompress_into(&compressed_msg, &mut self.compression_buf)
			.map_err(|err| IoError::new(IoErrorKind::InvalidData, err.to_string()))?;
		// And we're done here! Return the decompressed data.
		Ok(Some(Cursor::new(self.compression_buf.clone())))
	}
}
