/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#![deny(
	clippy::complexity,
	clippy::correctness,
	clippy::perf,
	clippy::style,
	unsafe_code
)]

#[macro_use]
extern crate log;

pub mod config;
pub mod conn;
pub mod keys;
pub mod qrgen;
pub mod updater;
pub mod upload_log;
pub mod windows;

use anyhow::{Context, Result};
use async_anyhow_logger::catch_context;
use directories_next::ProjectDirs;
use once_cell::sync::{Lazy, OnceCell};
use std::ops::DerefMut;
use tokio::{
	runtime::Runtime,
	sync::Mutex,
	task::JoinHandle,
	time::{interval, Duration},
};
use tray_item::TrayItem;

pub static SERVER_TASK: OnceCell<Mutex<JoinHandle<()>>> = OnceCell::new();
pub static XENON_DIR: Lazy<ProjectDirs> = Lazy::new(|| {
	let dirs = ProjectDirs::from("me", "aspenuwu", "xenon").expect("no home directory found");
	if !dirs.config_dir().is_dir() {
		std::fs::create_dir_all(dirs.config_dir()).expect("failed to create config directory");
	}
	if !dirs.data_dir().is_dir() {
		std::fs::create_dir_all(dirs.data_dir()).expect("failed to create data directory");
	}
	dirs
});
pub static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.expect("failed to build Tokio runtime")
});

async fn async_main() {
	let mut interval = interval(Duration::from_secs(5));
	loop {
		interval.tick().await;
		catch_context(
			"connection to server errored",
			conn::handshake::initiate_connection(),
		)
		.await;
		warn!("server stopped listening, restarting in 5 seconds!");
	}
}

#[allow(clippy::unnecessary_wraps, clippy::unit_arg)]
#[cfg(debug_assertions)]
async fn init_logging() -> Result<()> {
	if std::env::var("RUST_LOG").is_err() {
		std::env::set_var("RUST_LOG", "debug");
	}
	Ok(pretty_env_logger::init())
}

#[cfg(all(not(debug_assertions), not(windows)))]
async fn init_logging() -> Result<()> {
	use crate::config::CONFIG;
	use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger};
	use std::fs::File;
	use xenon_config::LogLevel;

	let log_level = match CONFIG.read().await.general.log_level {
		LogLevel::Error => LevelFilter::Error,
		LogLevel::Warn => LevelFilter::Warn,
		LogLevel::Info => LevelFilter::Info,
		LogLevel::Debug => LevelFilter::Debug,
	};

	let log_path = XENON_DIR.data_dir().join("xenon-client.log");
	CombinedLogger::init(vec![
		TermLogger::new(log_level, Config::default(), TerminalMode::Mixed),
		WriteLogger::new(
			log_level,
			Config::default(),
			File::create(&log_path)
				.with_context(|| format!("failed to create {}", log_path.display()))?,
		),
	])
	.context("failed to initialize logger")
}

#[cfg(all(not(debug_assertions), windows))]
async fn init_logging() -> Result<()> {
	use crate::config::CONFIG;
	use simplelog::{Config, LevelFilter, WriteLogger};
	use std::fs::File;
	use xenon_config::LogLevel;

	let log_level = match CONFIG.read().await.general.log_level {
		LogLevel::Error => LevelFilter::Error,
		LogLevel::Warn => LevelFilter::Warn,
		LogLevel::Info => LevelFilter::Info,
		LogLevel::Debug => LevelFilter::Debug,
	};
	let log_path = XENON_DIR.data_dir().join("xenon-client.log");
	WriteLogger::init(
		log_level,
		Config::default(),
		File::create(&log_path)
			.with_context(|| format!("failed to create {}", log_path.display()))?,
	)
	.context("failed to initialize log file")
}

pub async fn start_webserver() {
	if let Some(s) = SERVER_TASK.get() {
		let mut handle = s.lock().await;
		let handle = handle.deref_mut();
		handle.abort();
		debug!("stopped webserver, waiting for it to die");
		let _ = handle.await;
		debug!("old webserver is dead");
	}
	let task = RUNTIME.spawn(async_main());
	info!("started webserver");
	match SERVER_TASK.get() {
		Some(s) => {
			*s.lock().await = task;
		}
		None => {
			SERVER_TASK
				.set(Mutex::new(task))
				.unwrap_or_else(|_| unreachable!());
		}
	}
}

pub async fn upload_log_to_paste_ee() {}

fn main() -> Result<()> {
	RUNTIME
		.block_on(init_logging())
		.context("failed to initialize logger")?;
	log_panics::init();

	RUNTIME.spawn(catch_context(
		"failed to check for updates",
		updater::check_for_updates(),
	));

	#[cfg(target_os = "linux")]
	gtk::init().context("failed to initialize gtk")?;

	RUNTIME.spawn(start_webserver());

	#[cfg(any(target_os = "windows", target_os = "macos"))]
	let tray_icon = "xenon-tray";
	#[cfg(any(target_os = "linux"))]
	let tray_icon = "/usr/share/xenon-client/xenon.png";

	let mut tray = TrayItem::new("Xenon Client", tray_icon).unwrap();
	tray.add_menu_item("Pair Device", move || {
		RUNTIME.spawn(catch_context("failed to pair", qrgen::qr_connection()));
	})
	.expect("failed to add tray item");

	if XENON_DIR.config_dir() == XENON_DIR.data_dir() {
		tray.add_menu_item("Open Config/Data Folder", move || {
			if let Err(err) = opener::open(XENON_DIR.config_dir()) {
				error!("failed to open config.toml: {:?}", err);
			}
		})
		.expect("failed to add tray item");
	} else {
		tray.add_menu_item("Open Config Folder", move || {
			if let Err(err) = opener::open(XENON_DIR.config_dir()) {
				error!("failed to open config.toml: {:?}", err);
			}
		})
		.expect("failed to add tray item");
		tray.add_menu_item("Open Data Folder", move || {
			if let Err(err) = opener::open(XENON_DIR.data_dir()) {
				error!("failed to open config.toml: {:?}", err);
			}
		})
		.expect("failed to add tray item");
	}

	tray.add_menu_item("View Log", move || {
		if let Err(err) = opener::open(XENON_DIR.data_dir().join("xenon-client.log")) {
			error!("failed to open xenon-client.log: {:?}", err);
		}
	})
	.expect("failed to add tray item");

	tray.add_menu_item("Upload Log", move || {
		RUNTIME.spawn(upload_log::upload_log());
	})
	.expect("failed to add tray item");

	tray.add_menu_item("Restart Server", move || {
		RUNTIME.spawn(start_webserver());
	})
	.expect("failed to add tray item");

	#[cfg(target_os = "macos")]
	{
		let inner = tray.inner_mut();
		inner.add_quit_item("Quit");
		inner.display();
		panic!("tray exited early");
	}

	#[cfg(any(target_os = "windows", target_os = "linux"))]
	{
		tray.add_menu_item("Quit", move || {
			info!("user clicked on 'Quit' button, exiting now!");
			std::process::exit(0);
		})
		.expect("failed to add tray item");

		#[cfg(target_os = "windows")]
		loop {
			std::thread::park();
		}
		#[cfg(target_os = "linux")]
		Ok(gtk::main())
	}
}
