/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

mod bundle;
mod keys;
mod qr;
mod reload;

use anyhow::{Context, Result};
use std::{ffi::CString, path::PathBuf};
use tokio::{
	io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
	net::UnixListener,
};

pub async fn unix_server() -> Result<()> {
	let path = PathBuf::from("/tmp/me.aspenuwu.xenon.sock");
	if path.exists() {
		tokio::fs::remove_file(&path)
			.await
			.context("failed to delete previous socket! is the server still running?")?;
	}
	let socket = UnixListener::bind(path).context("failed to bind to unix socket")?;
	let mut buf = Vec::<u8>::with_capacity(64);
	while let Ok((mut stream, _addr)) = socket.accept().await {
		buf.clear();
		let (read, mut write) = stream.split();
		let mut stream = BufReader::new(read);
		let string = match stream
			.read_until(0, &mut buf)
			.await
			.context("failed to read bytes")
			.map(|bytes| &buf[..bytes - 1])
			.and_then(|bytes| CString::new(bytes).context("failed to process reply into a string"))
			.and_then(|cstr| {
				cstr.into_string()
					.context("failed to convert replied string to UTF-8")
			}) {
			Ok(s) => s,
			Err(e) => {
				error!("processing IPC reply errored: {}", e);
				continue;
			}
		};
		debug!("got string from ipc: {}", string);
		let reply = match string.as_str() {
			"reload-mounts" => reload::reload_mounts()
				.await
				.context("failed to reload mounts"),
			"regenerate-keys" => keys::regen_keys().await.context("failed to generate keys"),
			"pubkey" => keys::get_pubkey()
				.await
				.context("failed to fetch public key"),
			"bundles" => bundle::get_app_bundles()
				.await
				.context("failed to list app bundles"),
			"icloud-bundles" => bundle::get_icloud_bundles()
				.await
				.context("failed to list icloud bundles"),
			"generate-config" => keys::generate_config()
				.await
				.context("failed to generate configuration string"),
			_ => qr::pair_with_qr(string)
				.await
				.context("failed to process qr code"),
		};
		let reply = match reply {
			Ok(o) => o,
			Err(e) => {
				error!("processing IPC reply errored: {}", e);
				continue;
			}
		};
		if let Err(err) = write.write_all(reply.as_bytes()).await {
			error!("IPC response errored: {}", err);
			continue;
		}
	}
	Ok(())
}
