/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use anyhow::{Context, Result};
use std::net::{IpAddr, UdpSocket};

pub fn get_local_ip() -> Result<IpAddr> {
	let socket = UdpSocket::bind("0.0.0.0:0").context("failed to bind udp socket")?;

	socket
		.connect("1.1.1.1:80")
		.context("failed to connect socket")?;

	socket
		.local_addr()
		.map(|addr| addr.ip())
		.context("failed to get local address")
}

#[derive(Serialize, Deserialize)]
pub struct QrConnection {
	pub ip: IpAddr,
	pub port: u16,
	pub code: [u8; 32],
}

impl QrConnection {
	pub fn create() -> Result<(Self, UdpSocket)> {
		let ip = get_local_ip().context("failed to find local ip")?;
		let socket = UdpSocket::bind(format!("{}:0", ip)).context("failed to open udp socket")?;
		let qr = Self {
			ip,
			port: socket.local_addr().context("failed to get port")?.port(),
			code: rand::random(),
		};
		Ok((qr, socket))
	}

	pub fn to_base64(&self) -> String {
		base64::encode_config(
			&rmp_serde::to_vec(self).expect("failed to encode connection data"),
			base64::URL_SAFE_NO_PAD,
		)
	}

	pub fn from_base64(b64: &str) -> Result<Self> {
		base64::decode_config(b64, base64::URL_SAFE_NO_PAD)
			.context("failed to decode base64")
			.and_then(|decoded| {
				rmp_serde::from_slice(&decoded).context("failed to decode msgpack data")
			})
	}
}
