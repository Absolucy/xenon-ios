/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use crate::{
	config::{update_config, CONFIG},
	keys,
};
use anyhow::{Context, Result};
use async_anyhow_logger::catch_context;
use snow::Builder;
use tokio::{net::TcpStream, time};
use xenon_tunnel::{handshake, net, EncryptedTcpStream, NOISE_PARAMS};

pub async fn initiate_connection() -> Result<()> {
	let mut buf = Vec::<u8>::new();
	let mut handshake_buf = vec![0u8; 65535];
	let mut timer = time::interval(time::Duration::from_secs(15));
	let (mut stream, config, connection) = loop {
		timer.tick().await;
		catch_context("failed to update config", update_config()).await;
		let config = CONFIG.read().await.clone();
		let connection = match config.connection.clone() {
			Some(conn) => conn,
			None => continue,
		};
		match TcpStream::connect((connection.ip, connection.port)).await {
			Ok(stream) => break (stream, config, connection),
			Err(err) => {
				warn!(
					"Failed to connect to {}:{}; trying again in 15 seconds: {:?}",
					connection.ip, connection.port, err
				)
			}
		}
	};
	handshake::write_magic(&mut stream)
		.await
		.context("failed to initiate handshake")?;

	let mut blizzard = Builder::new(NOISE_PARAMS.clone())
		.local_private_key(&keys::CLIENT_KEYPAIR.public)
		.remote_public_key(&connection.pubkey)
		.build_initiator()
		.context("failed to initialize encryption")?;

	let len = blizzard
		.write_message(&[], &mut handshake_buf)
		.context("failed to write handshake message 0,2")?;
	net::write_msg(&mut stream, &handshake_buf[..len])
		.await
		.context("failed to send handshake message 0,2")?;
	debug!("sent 0,2 -> {}", connection.ip);

	let msg = net::read_msg(&mut stream, &mut buf)
		.await
		.context("failed to read handshake message 2,1")?;
	blizzard
		.read_message(msg, &mut handshake_buf)
		.context("failed to parse handshake message 2,1")?;
	debug!("{} -> got 2,1", connection.ip);

	let len = blizzard
		.write_message(&[], &mut handshake_buf)
		.context("failed to write handshake message 4,5")?;
	net::write_msg(&mut stream, &handshake_buf[..len])
		.await
		.context("failed to send handshake message 4,5")?;
	debug!("sent 4,5 -> {}", connection.ip);

	info!(
		"succesfully connected to {}, bringing up webserver",
		connection.ip
	);
	if config.general.notifications {
		if let Err(err) = notifica::notify(
			"Xenon connected",
			&format!("Connected to '{}' at {}:{} successfully!\nHosting WebDAV server on localhost port {}.", connection.hostname,
			connection.ip,
			connection.port,
			config.general.port)
		).context("failed to send notification") {
			warn!("failed to send notification: {:?}", err);
		}
	}

	let stream = EncryptedTcpStream::new(blizzard.into_transport_mode().unwrap(), stream);
	super::http::http_forwarder(stream)
		.await
		.context("http forwarder errored")
}
