/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use crate::server::photofs::PhotoFs;
use once_cell::sync::Lazy;
use std::{collections::HashMap, path::PathBuf};
use tokio::sync::RwLock;
use webdav_handler::{localfs::LocalFs, memls::MemLs, DavHandler};
use xenon_config::{MountPreset, MountType};

pub static DAV_MOUNTS: Lazy<RwLock<HashMap<String, DavHandler>>> =
	Lazy::new(|| RwLock::new(HashMap::new()));

pub static BUNDLES: Lazy<HashMap<String, PathBuf>> = Lazy::new(|| {
	let mut out = HashMap::<String, PathBuf>::new();

	for entry in std::fs::read_dir("/var/mobile/Containers/Shared/AppGroup")
		.expect("failed to read AppGroup container")
	{
		if let Ok(entry) = entry {
			let path = entry.path();
			if let Some(name) = get_name_from_plist(&path) {
				out.insert(name, path);
			}
		}
	}

	for entry in std::fs::read_dir("/var/mobile/Containers/Data/Application")
		.expect("failed to read Applications container")
	{
		if let Ok(entry) = entry {
			let path = entry.path();
			if let Some(name) = get_name_from_plist(&path) {
				out.insert(name, path);
			}
		}
	}

	out
});

pub fn get_name_from_plist(dir: &PathBuf) -> Option<String> {
	plist::Value::from_file(dir.join(".com.apple.mobile_container_manager.metadata.plist"))
		.ok()?
		.into_dictionary()?
		.get("MCMMetadataIdentifier")
		.and_then(|x| plist::Value::into_string(x.clone()))
}

pub fn create_dav_handler(name: &str, mount: MountType) -> Option<DavHandler> {
	if !name
		.chars()
		.all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
		|| name.eq_ignore_ascii_case("xenon")
	{
		warn!("Mount [{}] has an invalid name, skipping!", name);
		return None;
	}
	match mount {
		MountType::Bundle(bundle) => {
			create_dav_handler(name, MountType::Path(BUNDLES.get(&bundle)?.to_owned()))
		}
		MountType::ICloudBundle(bundle) => {
			let bundle_path = PathBuf::from("/var/mobile/Library/Mobile Documents")
				.join(bundle.replace('.', "~"));
			create_dav_handler(name, MountType::Path(bundle_path))
		}
		MountType::Path(path) => {
			if path.is_dir() {
				info!("Mount '{}' -> {}", name, path.display());
				Some(
					DavHandler::builder()
						.locksystem(MemLs::new())
						.filesystem(LocalFs::new(path, true, false, true))
						.strip_prefix(["/", name].join(""))
						.build_handler(),
				)
			} else {
				error!("Mount '{}' -> {} DOESN'T EXIST", name, path.display());
				None
			}
		}
		MountType::Preset(preset) => match preset {
			MountPreset::Photos => {
				info!("Mount '{}' -> Photos", name);
				Some(
					DavHandler::builder()
						.locksystem(MemLs::new())
						.filesystem(Box::new(PhotoFs::default()))
						.strip_prefix(["/", name].join(""))
						.build_handler(),
				)
			}
			MountPreset::LocalFiles => create_dav_handler(
				name,
				MountType::Path(
					BUNDLES
						.get("group.com.apple.FileProvider.LocalStorage")?
						.join("File Provider Storage"),
				),
			),
			MountPreset::Home => create_dav_handler(name, MountType::Path("/var/mobile".into())),
			MountPreset::Documents => {
				create_dav_handler(name, MountType::Path("/var/mobile/Documents".into()))
			}
		},
	}
}
