/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use anyhow::{Context, Result};

pub async fn check_for_updates() -> Result<()> {
	let release: serde_json::Value = reqwest::Client::new()
		.get("https://api.github.com/repos/aspenluxxxy/xenon-docs/releases/latest")
		.header(
			"User-Agent",
			"xenon (aspenluxxxy/xenon-docs) client update checker",
		)
		.send()
		.await
		.context("failed to request github api")?
		.json()
		.await
		.context("failed to parse github response")?;
	let latest_tag = release
		.get("tag_name")
		.context("failed to get tag_name in github response")?
		.as_str()
		.context("failed to convert tag_name to a string")?;
	let latest_version = latest_tag.trim().replace('v', "");
	info!(
		"latest version is {}, our version is {}",
		latest_version,
		env!("CARGO_PKG_VERSION")
	);
	if latest_version != env!("CARGO_PKG_VERSION") {
		notifica::notify(
			"Update Available",
			&format!(
				"An update for the Xenon client is now available!\nVersion {} -> {}",
				env!("CARGO_PKG_VERSION"),
				latest_version
			),
		)
		.context("failed to send notification")?;
	}
	Ok(())
}
