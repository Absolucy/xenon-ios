/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use crate::{mount, CFG_FOLDER};
use std::{collections::HashMap, path::PathBuf};
use webdav_handler::DavHandler;
use xenon_config::MountType;

pub async fn reload_mounts() -> Option<String> {
	let mounts: HashMap<String, MountType> =
		match tokio::fs::read_to_string(PathBuf::from(CFG_FOLDER).join("mounts.json"))
			.await
			.and_then(|contents| serde_json::from_str(&contents).map_err(std::io::Error::from))
		{
			Ok(o) => o,
			Err(e) => {
				error!("failed to read mounts.json: {}", e);
				return None;
			}
		};
	info!("loaded mounts.json");
	let mut ret = HashMap::<String, DavHandler>::new();
	for (name, mount) in mounts {
		if let Some(dav_handler) = mount::create_dav_handler(&name, mount) {
			ret.insert(name, dav_handler);
		}
	}
	*mount::DAV_MOUNTS.write().await = ret;
	Some("ok".to_string())
}
