/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use crate::XENON_DIR;
use anyhow::{Context, Result};
use keyring::Keyring;
use once_cell::sync::Lazy;
use snow::{Builder, Keypair};
use xenon_config::KeypairHelper;
use xenon_tunnel::NOISE_PARAMS;

fn generate_keypair(keyring: Keyring) -> Result<Keypair> {
	let keypair = Builder::new(NOISE_PARAMS.clone())
		.generate_keypair()
		.context("Failed to generate keys!")?;
	let encoded = rmp_serde::to_vec(&(keypair.public.clone(), keypair.private.clone()))
		.context("failed to encode keypair")?;
	if keyring.set_password(&base64::encode(&encoded)).is_err() {
		std::fs::write(
			XENON_DIR.data_dir().join("xenon.key"),
			serde_json::to_string(&KeypairHelper::from(&keypair))
				.context("failed to serialize keypair! aborting")?,
		)
		.context("failed to save keypair! aborting")?;
	}
	Ok(keypair)
}

fn get_keypair(keyring: Keyring) -> Result<Keypair> {
	match keyring
		.get_password()
		.ok()
		.and_then(|x| base64::decode(&x).ok())
		.and_then(|x| rmp_serde::from_slice::<(Vec<u8>, Vec<u8>)>(&x).ok())
	{
		Some((public, private)) => {
			if public.len() == 32 && private.len() == 32 {
				Ok(Keypair { public, private })
			} else {
				generate_keypair(keyring).context("failed to generate keypair")
			}
		}
		_ => match std::fs::read(XENON_DIR.data_dir().join("xenon.key"))
			.ok()
			.and_then(|v| serde_json::from_slice::<KeypairHelper>(&v).ok())
		{
			Some(s) => Ok(s.into()),
			_ => generate_keypair(keyring).context("failed to generate keypair"),
		},
	}
}

pub static CLIENT_KEYPAIR: Lazy<Keypair> = Lazy::new(|| {
	let keyring = Keyring::new("me.aspenuwu.xenon", "keypair");

	get_keypair(keyring).expect("failed to get keypair")
});
