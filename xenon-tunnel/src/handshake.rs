/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use crate::MAGIC_INITIALIZER;
use anyhow::Result;
use rand::RngCore;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	net::TcpStream,
};

pub async fn read_magic(socket: &mut TcpStream, buf: &mut Vec<u8>) -> Result<bool> {
	let magic = MAGIC_INITIALIZER;
	let magic_len = magic.len();
	let mut magic_key = vec![0u8; magic_len];
	let mut magicked_epoch = [0u8; std::mem::size_of::<u128>()];
	let mut epoch_key = [0u8; std::mem::size_of::<u128>()];
	socket.read_exact(&mut epoch_key).await?;
	socket.read_exact(&mut magicked_epoch).await?;
	socket.read_exact(&mut magic_key).await?;

	magic_key.reverse();
	epoch_key.reverse();

	epoch_key
		.iter_mut()
		.for_each(|byte| *byte = byte.reverse_bits() ^ 42);

	magicked_epoch
		.iter_mut()
		.enumerate()
		.for_each(|(idx, byte)| *byte = byte.reverse_bits() ^ epoch_key[idx]);

	magic_key.iter_mut().enumerate().for_each(|(idx, byte)| {
		*byte = byte.reverse_bits() ^ magicked_epoch[idx % std::mem::size_of::<u128>()];
	});

	let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
	let epoch = u128::from_le_bytes(magicked_epoch);

	if now > epoch + 5000000000 {
		return Ok(false);
	}

	if magic_len > buf.len() {
		buf.resize(magic_len, 0);
	}

	let buf = &mut buf[..magic_len];
	socket.read_exact(buf).await?;

	buf.iter_mut()
		.enumerate()
		.for_each(|(idx, byte)| *byte = byte.reverse_bits() ^ magic_key[idx]);

	Ok(magic.as_bytes() == buf)
}

pub async fn write_magic(socket: &mut TcpStream) -> Result<()> {
	let magic = MAGIC_INITIALIZER;
	let magic_len = magic.len();
	let mut magic_key = vec![0u8; magic_len];
	let mut epoch_key = [0u8; std::mem::size_of::<u128>()];
	rand::thread_rng().fill_bytes(&mut magic_key);
	rand::thread_rng().fill_bytes(&mut epoch_key);

	let epoch_val = SystemTime::now()
		.duration_since(UNIX_EPOCH)?
		.as_nanos()
		.to_le_bytes();
	let epoch = epoch_val
		.iter()
		.enumerate()
		.map(|(idx, byte)| (byte ^ epoch_key[idx]).reverse_bits())
		.collect::<Vec<u8>>();
	let magic = magic
		.as_bytes()
		.iter()
		.enumerate()
		.map(|(idx, byte)| (byte ^ magic_key[idx]).reverse_bits())
		.collect::<Vec<u8>>();

	magic_key.iter_mut().enumerate().for_each(|(idx, byte)| {
		*byte = (*byte ^ epoch_val[idx % std::mem::size_of::<u128>()]).reverse_bits()
	});
	epoch_key
		.iter_mut()
		.for_each(|byte| *byte = (*byte ^ 42).reverse_bits());
	magic_key.reverse();
	epoch_key.reverse();

	socket.write_all(&epoch_key).await?;
	socket.write_all(&epoch).await?;
	socket.write_all(&magic_key).await?;
	socket.write_all(&magic).await?;
	socket.flush().await?;

	Ok(())
}
