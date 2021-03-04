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

pub mod handshake;
pub mod net;
pub mod stream;

pub use stream::EncryptedTcpStream;

use once_cell::sync::Lazy;
use snow::params::NoiseParams;

pub const MAGIC_INITIALIZER: &str = "get your own dang initializer!";
// (42 xor 7,500,000) modulo 65535
pub const XENON_PORT: u16 = 28988;
pub const SIZE_LIMIT: usize = u16::MAX as usize;
pub const MAX_FRAME_SIZE: u32 = 61440;

pub static NOISE_PARAMS: Lazy<NoiseParams> = Lazy::new(|| {
	"Noise_XK_25519_ChaChaPoly_SHA256"
		.parse::<NoiseParams>()
		.unwrap_or_else(|e| unreachable!(e))
});
