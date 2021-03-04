/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use std::ops::Deref;

use snow::Keypair;

#[derive(Serialize, Deserialize)]
#[serde(remote = "Keypair")]
pub struct KeypairDef {
	#[serde(with = "crate::Base64", rename = "p")]
	pub public: Vec<u8>,
	#[serde(with = "crate::Base64", rename = "s")]
	pub private: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct KeypairHelper(#[serde(with = "KeypairDef")] pub Keypair);

impl From<Keypair> for KeypairHelper {
	fn from(x: Keypair) -> Self {
		Self(x)
	}
}

impl From<&Keypair> for KeypairHelper {
	fn from(x: &Keypair) -> Self {
		Self(Keypair {
			public: x.public.clone(),
			private: x.private.clone(),
		})
	}
}

impl From<KeypairHelper> for Keypair {
	fn from(x: KeypairHelper) -> Self {
		x.0
	}
}

impl Deref for KeypairHelper {
	type Target = Keypair;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
