/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use crate::config::CONFIG;
use anyhow::{Context, Result};
use http::Request;
use hyper::{
	client::conn::Builder as HyperBuilder,
	service::{make_service_fn, service_fn},
	Body, Server,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::{sync::Mutex, time, try_join};
use xenon_tunnel::EncryptedTcpStream;

pub async fn http_forwarder(stream: EncryptedTcpStream) -> Result<()> {
	let (request_sender, connection) = HyperBuilder::new()
		.http2_only(true)
		.http2_max_frame_size(xenon_tunnel::MAX_FRAME_SIZE)
		.handshake::<EncryptedTcpStream, Body>(stream)
		.await
		.context("failed to build HTTP/2 server")?;

	let request_sender = Arc::new(Mutex::new(request_sender));

	let make_svc = make_service_fn(move |_conn| {
		let request_sender = request_sender.clone();
		async {
			Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
				let request_sender = request_sender.clone();
				async move {
					let request_sender = request_sender.clone();
					let mut sender = request_sender.lock().await;
					sender.send_request(req).await
				}
			}))
		}
	});

	let addr = SocketAddr::from(([127, 0, 0, 1], CONFIG.read().await.general.port));
	let server = Server::bind(&addr).serve(make_svc);

	tokio::spawn(async move {
		time::sleep(time::Duration::from_secs(1)).await;
		super::webdav::mount_webdav().await;
	});

	try_join!(
		async move { server.await.context("local http server errored") },
		async move {
			connection
				.await
				.with_context(|| format!("http connection to xenon-server at [{}] errored", addr))
		}
	)
	.context("http webdav bridge errored")
	.map(|_| ())
}
