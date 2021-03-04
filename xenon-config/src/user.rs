/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use std::net::{IpAddr, Ipv4Addr};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
	Error,
	Warn,
	Info,
	Debug,
}

impl Default for LogLevel {
	fn default() -> Self {
		LogLevel::Info
	}
}

const fn default_webdav_port() -> u16 {
	4200
}

const fn default_notifications() -> bool {
	true
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct UserConfig {
	#[serde(flatten)]
	pub general: GeneralConfig,
	pub connection: Option<ConnectionConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct GeneralConfig {
	#[serde(default = "default_webdav_port")]
	pub port: u16,
	#[serde(default)]
	pub log_level: LogLevel,
	#[serde(default = "default_notifications")]
	pub notifications: bool,
	#[serde(rename = "mount-point")]
	pub windows_mount_point: Option<String>,
}

impl Default for GeneralConfig {
	fn default() -> Self {
		GeneralConfig {
			port: default_webdav_port(),
			log_level: LogLevel::default(),
			notifications: true,
			windows_mount_point: None,
		}
	}
}

pub const fn default_connection_port() -> u16 {
	28988
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConnectionConfig {
	pub ip: IpAddr,
	#[serde(default = "default_connection_port")]
	pub port: u16,
	pub hostname: String,
	#[serde(with = "crate::Base64")]
	pub pubkey: Vec<u8>,
}

impl Default for ConnectionConfig {
	fn default() -> Self {
		Self {
			ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
			port: default_connection_port(),
			hostname: "My iPhone".to_string(),
			pubkey: vec![
				1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
				24, 25, 26, 27, 28, 29, 30, 31, 32,
			],
		}
	}
}
