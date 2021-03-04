/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use crate::XENON_DIR;
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use tokio::sync::RwLock;
use xenon_config::UserConfig;

pub static CONFIG: Lazy<RwLock<UserConfig>> = Lazy::new(|| {
	let xenon_config = XENON_DIR.config_dir().join("config.toml");
	if xenon_config.is_file() {
		let config = std::fs::read_to_string(&xenon_config).expect("failed to read config.toml");
		toml::from_str::<UserConfig>(&config)
			.map(RwLock::new)
			.unwrap_or_else(|_| {
				warn!("config.toml invalid; recreating default!");
				let config = UserConfig::default();
				std::fs::write(
					xenon_config,
					toml::to_string_pretty(&config)
						.expect("failed to serialize example config.toml"),
				)
				.expect("failed to create example config.toml");
				RwLock::new(config)
			})
	} else {
		let config = UserConfig::default();
		info!("config.toml doesn't exist, creating an example!");
		std::fs::write(
			xenon_config,
			toml::to_string_pretty(&config).expect("failed to serialize example config.toml"),
		)
		.expect("failed to create example config.toml");
		RwLock::new(config)
	}
});

pub async fn update_config() -> Result<()> {
	let xenon_config = XENON_DIR.config_dir().join("config.toml");
	if xenon_config.is_file() {
		let config = tokio::fs::read_to_string(&xenon_config)
			.await
			.context("failed to read config.toml")?;
		let config =
			toml::from_str::<UserConfig>(&config).context("failed to parse config.toml")?;
		let mut global_config = CONFIG.write().await;
		*global_config = config;
	}
	Ok(())
}
