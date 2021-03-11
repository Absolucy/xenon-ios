/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

#![deny(
	clippy::complexity,
	clippy::correctness,
	clippy::perf,
	clippy::style,
	unsafe_code
)]

#[macro_use]
extern crate obfstr;

pub mod handshake;
pub mod net;
pub mod stream;

pub use stream::EncryptedTcpStream;

use once_cell::sync::Lazy;
use snow::params::NoiseParams;

pub const MAGIC_INITIALIZER: &str = include_str!("magic.key");
// (42 xor 7,500,000) modulo 65535
pub const XENON_PORT: u16 = 28988;
pub const SIZE_LIMIT: usize = u16::MAX as usize;
// NPF maximum size is 65535. So this should be enough to cover compression + headers.
pub const MAX_FRAME_SIZE: u32 = 61440;

pub static NOISE_PARAMS: Lazy<NoiseParams> = Lazy::new(|| {
	obfstr!("Noise_XK_25519_ChaChaPoly_SHA256")
		.parse::<NoiseParams>()
		.unwrap_or_else(|e| unreachable!(e))
});
