/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use crate::CFG_FOLDER;
use once_cell::sync::Lazy;
use snow::{Builder, Keypair};
use std::path::PathBuf;
use tokio::sync::RwLock;
use xenon_config::KeypairHelper;
use xenon_tunnel::NOISE_PARAMS;

pub static KEYPAIR: Lazy<RwLock<Keypair>> = Lazy::new(|| {
	let path = PathBuf::from(CFG_FOLDER);
	if !path.is_dir() {
		std::fs::create_dir_all(&path).expect("failed to create storage directory");
	}
	let path = path.join("xenon.key");
	match std::fs::read(&path)
		.ok()
		.and_then(|v| serde_json::from_slice::<KeypairHelper>(&v).ok())
	{
		Some(s) => RwLock::new(Keypair::from(s)),
		_ => {
			let keypair = Builder::new(NOISE_PARAMS.clone())
				.generate_keypair()
				.expect("failed to generate keys");
			std::fs::write(
				path,
				serde_json::to_string(&KeypairHelper::from(&keypair))
					.expect("failed to serialize keypair"),
			)
			.expect("failed to save keypair");
			RwLock::new(keypair)
		}
	}
});
