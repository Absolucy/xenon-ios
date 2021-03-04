/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use crate::mount::BUNDLES;
use anyhow::{Context, Result};

pub async fn get_app_bundles() -> Result<String> {
	serde_json::to_string(
		&BUNDLES
			.keys()
			.filter(|name| !name.trim().is_empty())
			.collect::<Vec<_>>(),
	)
	.context("failed to serialize json")
}

pub async fn get_icloud_bundles() -> Result<String> {
	let mut read_dir = tokio::fs::read_dir("/var/mobile/Library/Mobile Documents")
		.await
		.context("failed to read iCloud bundles!")?;
	let mut bundles = Vec::<String>::new();
	while let Ok(Some(entry)) = read_dir.next_entry().await {
		let path = entry.path();
		if let Some(name) = path
			.file_name()
			.and_then(|name| name.to_str())
			.filter(|name| !name.trim().is_empty())
			.map(|name| name.replace("~", "."))
		{
			bundles.push(name);
		}
	}

	serde_json::to_string(&bundles).context("failed to serialize json")
}
