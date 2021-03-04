/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

#![deny(clippy::complexity, clippy::correctness, clippy::perf, clippy::style)]

#[macro_use]
extern crate log;

pub mod ipc;
pub mod jetsam;
pub mod keys;
pub mod logger;
pub mod mount;
pub mod server;

use anyhow::{Context, Result};
use async_anyhow_logger::catch_context;
use once_cell::sync::OnceCell;
use std::{collections::HashMap, path::PathBuf};
use std::{io::Write, ops::Deref};
use webdav_handler::DavHandler;
use xenon_config::MountType;

pub static DAV_HANDLERS: OnceCell<HashMap<String, DavHandler>> = OnceCell::new();
pub const CFG_FOLDER: &str = "/var/mobile/Library/me.aspenuwu.xenon";

#[cfg(not(debug_assertions))]
async fn init_logging() -> Result<()> {
	let (logger, rx) = logger::XenonLogger::new();
	let _ = tokio::spawn(logger::file_logging_task(rx));
	log::set_boxed_logger(Box::new(logger)).context("failed to set up logger")
}

#[cfg(debug_assertions)]
#[allow(clippy::unnecessary_wraps, clippy::unit_arg)]
async fn init_logging() -> Result<()> {
	Ok(pretty_env_logger::init())
}

fn log_panics() {
	std::panic::set_hook(Box::new(|info| {
		let thread = std::thread::current();
		let thread = thread.name().unwrap_or("unnamed");
		let msg = match info.payload().downcast_ref::<&'static str>() {
			Some(s) => *s,
			None => match info.payload().downcast_ref::<String>() {
				Some(s) => &**s,
				None => "Box<Any>",
			},
		};
		let error_text = match info.location() {
			Some(location) => {
				format!(
					"thread '{}' panicked at '{}': {}:{}",
					thread,
					msg,
					location.file(),
					location.line(),
				)
			}
			None => {
				format!("thread '{}' panicked at '{}'", thread, msg)
			}
		};
		error!(target: "panic", "{}", error_text);
		let crash_log = PathBuf::from(CFG_FOLDER).join("crash.log");
		let file = std::fs::File::create(&crash_log);
		match file {
			Ok(mut file) => {
				let _ = file.write_all(error_text.as_bytes());
				let _ = file.flush();
				error!("output crash log to {}", crash_log.display());
			}
			Err(e) => {
				error!(
					"failed to output crash log to {}: {:?}",
					crash_log.display(),
					e
				);
			}
		}
	}));
}

#[tokio::main]
async fn main() -> Result<()> {
	init_logging().await?;
	log_panics();
	jetsam::there_will_be_blood_yeaahh();

	let cfg_dir = PathBuf::from(CFG_FOLDER);
	if !cfg_dir.is_dir() {
		if let Err(err) = tokio::fs::create_dir_all(&cfg_dir).await {
			error!(
				"failed to create config directory at {}: {:?}",
				cfg_dir.display(),
				err
			);
		}
	}

	// Initialize this lazy value early.
	let _ = keys::KEYPAIR.deref();

	tokio::spawn(catch_context("unix socket IPC errored", ipc::unix_server()));

	match tokio::fs::read_to_string(PathBuf::from(CFG_FOLDER).join("mounts.json"))
		.await
		.and_then(|contents| {
			serde_json::from_str::<HashMap<String, MountType>>(&contents)
				.map_err(std::io::Error::from)
		}) {
		Ok(mounts) => {
			info!("loaded mounts.json");
			let mut ret = HashMap::<String, DavHandler>::new();
			for (name, mount) in mounts {
				if let Some(dav_handler) = mount::create_dav_handler(&name, mount) {
					ret.insert(name, dav_handler);
				}
			}
			*mount::DAV_MOUNTS.write().await = ret;
		}
		Err(e) => {
			error!("failed to read mounts.json: {}", e);
		}
	};

	server::listener().await.context("server errored")
}
