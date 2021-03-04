/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use crate::keys::KEYPAIR;
use anyhow::{Context, Result};
use async_anyhow_logger::catch_context;
use tokio::net::UdpSocket;
use xenon_config::QrConnection;
use xenon_tunnel::XENON_PORT;

async fn qr_connect(qr: QrConnection) -> Result<()> {
	info!("qr code says to connect to {}:{}", qr.ip, qr.port);
	let sock = UdpSocket::bind("0.0.0.0:0")
		.await
		.context("failed to bind udp socket")?;
	sock.connect((qr.ip, qr.port))
		.await
		.context("failed to connect to qr client")?;
	let mut code = qr.code.to_vec();
	let pubkey = &KEYPAIR.read().await.public;
	let hostname = hostname::get()
		.context("failed to get hostname")
		.and_then(|hostname| {
			hostname
				.to_str()
				.map(|hostname| hostname.to_string())
				.context("failed to convert hostname to string")
		})?;
	let hostname_pubkey = (hostname, XENON_PORT, pubkey);
	code.extend_from_slice(
		&rmp_serde::to_vec(&hostname_pubkey)
			.context("failed to encode hostname+pubkey as msgpack")?,
	);
	sock.send(&code)
		.await
		.context("failed to send qr data to client")?;
	Ok(())
}

pub async fn pair_with_qr(qr: String) -> Result<String> {
	let qr = QrConnection::from_base64(qr.strip_prefix("XE42~").context("invalid qr code")?)?;
	tokio::spawn(catch_context(
		"qr pairing connection errored",
		qr_connect(qr),
	));
	Ok("ok".to_string())
}
