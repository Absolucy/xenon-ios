/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use crate::SIZE_LIMIT;
use anyhow::{Context, Result};
use nano_leb128::ULEB128;
use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	net::TcpStream,
};

pub async fn recv_size(socket: &mut TcpStream) -> Option<usize> {
	let mut len_buf = [0u8];
	let mut buf = Vec::<u8>::new();
	for _ in 0..std::mem::size_of::<u64>() {
		socket.read_exact(&mut len_buf).await.ok()?;
		let byte = (len_buf[0] ^ 42).reverse_bits();
		buf.push(byte);
		if byte & (1 << 7) == 0 {
			break;
		}
	}
	ULEB128::read_from(&buf)
		.ok()
		.filter(|(_, s)| *s > 0 && *s < SIZE_LIMIT)
		.map(|(size, _)| u64::from(size) as usize)
}

pub async fn send_size(socket: &mut TcpStream, size: usize) -> Option<()> {
	if size > SIZE_LIMIT {
		return None;
	}
	let mut buf = [0u8; std::mem::size_of::<u64>()];
	let encoded_length = ULEB128::from(size as u64)
		.write_into(&mut buf)
		.ok()
		.filter(|el| *el > 0)?;
	let length_buf = &mut buf[..encoded_length];
	length_buf
		.iter_mut()
		.for_each(|byte| *byte = byte.reverse_bits() ^ 42);
	socket.write_all(length_buf).await.ok()
}

#[allow(clippy::needless_lifetimes)]
pub async fn read_msg<'a>(socket: &mut TcpStream, buf: &'a mut Vec<u8>) -> Option<&'a [u8]> {
	let size = recv_size(socket).await?;
	if size > buf.len() {
		buf.resize(size, 0);
	}
	match socket.read_exact(&mut buf[..size]).await {
		Ok(o) if o != 0 => Some(&buf[..size]),
		_ => return None,
	}
}

pub async fn write_msg(socket: &mut TcpStream, buf: &[u8]) -> Result<()> {
	send_size(socket, buf.len())
		.await
		.context("failed to send size")?;
	socket.write_all(buf).await?;
	Ok(())
}
