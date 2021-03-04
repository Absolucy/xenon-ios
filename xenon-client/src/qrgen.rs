/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use crate::{config::CONFIG, start_webserver, windows, XENON_DIR};
use anyhow::{Context, Result};
use async_anyhow_logger::catch_context;
use qrcode::QrCode;
use tokio::time::{timeout, Duration};
use xenon_config::{ConnectionConfig, QrConnection};

pub async fn qr_connection() -> Result<()> {
	let (qr, socket) = QrConnection::create().context("failed to initialize QR code")?;

	let qr_data = ["XE42", qr.to_base64().as_str()].join("~");
	let qr_image = QrCode::new(qr_data.as_bytes())
		.context("failed to create QR code")?
		.render::<image::Rgb<u8>>()
		.min_dimensions(256, 256)
		.build();
	let tempfile = tempfile::Builder::new()
		.suffix(".png")
		.tempfile()
		.context("failed to get temporary file")?
		.keep()
		.context("failed to persist temporary file")?
		.1;
	qr_image
		.save_with_format(&tempfile, image::ImageFormat::Png)
		.context("failed to save rendered QR code to file")?;
	debug!("qr code saved to {}", tempfile.display());
	opener::open(&tempfile).context("failed to open qr code image")?;

	tokio::spawn(catch_context(
		"failed to check for windows firewall",
		windows::firewall_check(),
	));

	tokio::spawn(async move {
		let timer = Duration::from_secs(60 * 5);
		let x = timeout(
			timer,
			catch_context("failed to receive qr code", async move {
				let mut buf = vec![0u8; 4096];
				loop {
					let (recv_amt, addr) = socket
						.recv_from(&mut buf)
						.context("failed to receive bytes")?;
					if recv_amt < 64 {
						error!(
							"received {} bytes, even though we should always get at least 64",
							recv_amt
						);
						continue;
					}
					debug!("received {} bytes from {}", recv_amt, addr);
					if buf[..32] == qr.code {
						let (hostname, port, pubkey): (String, u16, Vec<u8>) =
							rmp_serde::from_slice(&buf[32..])
								.context("failed to decode msgpack from message")?;
						let ip = addr.ip();
						info!(
							"configuring server: '{}' at {}, public key [{}]",
							hostname,
							ip,
							base64::encode_config(&pubkey, base64::URL_SAFE_NO_PAD)
						);
						let config = ConnectionConfig {
							ip,
							port,
							hostname,
							pubkey,
						};
						debug!("writing new connection config: {:#?}", config);
						CONFIG.write().await.connection = Some(config.clone());
						let path = XENON_DIR.config_dir().join("config.toml");
						std::fs::write(
							&path,
							toml::to_string_pretty(&*CONFIG.read().await)
								.context("failed to encode new configuration as TOML")?,
						)
						.with_context(|| {
							format!("failed to write new configuration to {}", path.display())
						})?;
						let _ = tokio::fs::remove_file(tempfile).await;
						tokio::spawn(start_webserver());
						break;
					}
				}
				Ok(())
			}),
		)
		.await;
		if x.is_err() {
			warn!("waiting for QR code message timed out");
		}
	});

	Ok(())
}
