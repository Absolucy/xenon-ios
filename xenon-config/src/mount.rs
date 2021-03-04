/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MountPreset {
	Photos,
	LocalFiles,
	Home,
	Documents,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MountType {
	Path(PathBuf),
	ICloudBundle(String),
	Bundle(String),
	Preset(MountPreset),
}
