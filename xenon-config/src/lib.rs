/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

#[macro_use]
extern crate serde;
#[macro_use]
extern crate base64_serde;

pub mod keypair;
pub mod mount;
pub mod qr;
pub mod user;

pub use keypair::*;
pub use mount::*;
pub use qr::*;
pub use user::*;

use base64::URL_SAFE_NO_PAD;
base64_serde_type!(Base64, URL_SAFE_NO_PAD);
