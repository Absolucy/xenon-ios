/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

pub mod metafs;
pub mod photofs;

use self::metafs::METAFS;
use crate::mount::DAV_MOUNTS;
use anyhow::{Context, Result};
use async_anyhow_logger::catch_context;
use http::Request;
use hyper::{server::conn::Http, service::service_fn, Body, Response, StatusCode};
use snow::{Builder, HandshakeState, TransportState};
use std::{convert::Infallible, net::SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use xenon_tunnel::{handshake, net, EncryptedTcpStream, NOISE_PARAMS, XENON_PORT};

async fn server(snowfall: TransportState, stream: TcpStream, addr: SocketAddr) -> Result<()> {
	let stream = EncryptedTcpStream::new(snowfall, stream);

	Http::new()
		.http2_only(true)
		.http2_max_frame_size(xenon_tunnel::MAX_FRAME_SIZE)
		.serve_connection(
			stream,
			service_fn(|req: Request<Body>| async move {
				let first_part = format!("{} -> {}", addr, req.uri());
				let path = req.uri().path().trim();
				let path = path.strip_prefix("/").unwrap_or(path);
				if path.is_empty() {
					debug!("{} -> MetaFS", path);
					return Ok::<_, Infallible>(METAFS.handle(req).await);
				}
				let global_mounts = DAV_MOUNTS.read().await;
				let response = match global_mounts
					.iter()
					.find(|(key, _)| path.starts_with(key.as_str()))
				{
					Some((name, dav)) => {
						debug!("{} -> real FS {}", path, name);
						dav.handle(req).await
					}
					None => Response::builder()
						.status(StatusCode::NOT_FOUND)
						.body(webdav_handler::body::Body::from("not found"))
						.expect("failed to send 404"),
				};
				debug!("{} -> {}", first_part, response.status());
				Ok::<_, Infallible>(response)
			}),
		)
		.await
		.context("connection errored")?;
	Ok(())
}

pub async fn listener() -> Result<()> {
	let listener = TcpListener::bind(("0.0.0.0", XENON_PORT))
		.await
		.context("failed to bind port")?;
	info!("listening on port {}", XENON_PORT);
	loop {
		let (socket, addr) = match listener.accept().await {
			Ok(o) => o,
			Err(_) => continue,
		};
		debug!("accepted connection from {}", addr);
		let snowfall = Builder::new(NOISE_PARAMS.clone())
			.local_private_key(&crate::keys::KEYPAIR.read().await.private)
			.build_responder()
			.context("failed to build encryption")?;
		tokio::spawn(catch_context(
			"encrypted tunnel errored",
			connection(snowfall, socket, addr),
		));
	}
}

async fn connection(
	mut snowfall: HandshakeState,
	mut socket: TcpStream,
	addr: SocketAddr,
) -> Result<()> {
	let mut buf = Vec::<u8>::new();
	let mut handshake_buf = vec![0u8; 65535];
	// Before we set up Noise, we need the magic packet (wavexor'd)
	if !handshake::read_magic(&mut socket, &mut buf)
		.await
		.context("failed to initialize handshake")?
	{
		return Ok(());
	}
	trace!("{} -> magic is ok", addr);
	// Set up our Noise connection: https://noiseexplorer.com/patterns/XK
	// 0,2
	let msg = net::read_msg(&mut socket, &mut buf)
		.await
		.context("failed to read handshake message 0,2")?;
	snowfall
		.read_message(msg, &mut handshake_buf)
		.context("failed to parse handshake message 0,2")?;
	trace!("{} -> got 0,2", addr);
	// 2,1
	let len = snowfall
		.write_message(&[42], &mut handshake_buf)
		.context("failed to write handshake message 2,1")?;
	net::write_msg(&mut socket, &handshake_buf[..len])
		.await
		.context("failed to send handshake message 2,1")?;
	trace!("sent 2,1 -> {}", addr);
	// 4,5
	let msg = net::read_msg(&mut socket, &mut buf)
		.await
		.context("failed to read handshake message 4,s")?;
	snowfall
		.read_message(msg, &mut handshake_buf)
		.context("failed to parse handshake message 4,5")?;
	trace!("{} -> got 4,5", addr);
	info!("connection established by {}", addr);
	// Handshake complete, start the actual connection.
	tokio::spawn(catch_context(
		"encrypted connection errored",
		server(
			snowfall
				.into_transport_mode()
				.context("failed to finalize encrypted connection")?,
			socket,
			addr,
		),
	));
	Ok(())
}
