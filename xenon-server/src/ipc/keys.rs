/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use crate::{keys::KEYPAIR, CFG_FOLDER};
use anyhow::{Context, Result};
use snow::Builder;
use std::path::PathBuf;
use xenon_config::{get_local_ip, ConnectionConfig, KeypairHelper};
use xenon_tunnel::{NOISE_PARAMS, XENON_PORT};

pub async fn regen_keys() -> Result<String> {
	let path = PathBuf::from(CFG_FOLDER);
	if !path.is_dir() {
		std::fs::create_dir_all(&path).context("failed to create storage directory")?;
	}
	let path = path.join("xenon.key");
	if path.exists() {
		tokio::fs::remove_file(&path)
			.await
			.context("failed to remove key!")?;
	}
	let keypair = Builder::new(NOISE_PARAMS.clone())
		.generate_keypair()
		.context("failed to generate keys")?;
	tokio::fs::write(
		path,
		serde_json::to_string(&KeypairHelper::from(&keypair))
			.context("failed to serialize keypair! aborting")?,
	)
	.await
	.context("failed to save keypair! aborting")?;
	let pubkey = base64::encode_config(&keypair.public, base64::URL_SAFE_NO_PAD);
	info!("regenerated keys, new public key is {}", pubkey);
	*KEYPAIR.write().await = keypair;
	Ok(pubkey)
}

pub async fn generate_config() -> Result<String> {
	let ip = get_local_ip().context("failed to get own ip address")?;
	let hostname = hostname::get()
		.context("failed to get hostname")
		.and_then(|hostname| {
			hostname
				.to_str()
				.map(|hostname| hostname.to_string())
				.context("failed to convert hostname to string")
		})?;
	let pubkey = KEYPAIR.read().await.public.clone();
	let config = ConnectionConfig {
		ip,
		port: XENON_PORT,
		hostname,
		pubkey,
	};
	toml::to_string_pretty(&config).context("failed to encode new config as toml")
}

pub async fn get_pubkey() -> Result<String> {
	let keypair = KEYPAIR.read().await;
	Ok(base64::encode_config(
		&keypair.public,
		base64::URL_SAFE_NO_PAD,
	))
}
