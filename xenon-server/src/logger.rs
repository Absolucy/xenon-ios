/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use crate::CFG_FOLDER;
use log::{Level, LevelFilter, Log, Metadata, Record};
use oslog::OsLogger;
use std::path::PathBuf;
use tokio::{
	fs::File,
	io::AsyncWriteExt,
	sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};

pub enum LogMessage {
	Message(String),
	Flush,
}

pub struct XenonLogger {
	os: OsLogger,
	tx: UnboundedSender<LogMessage>,
}

impl XenonLogger {
	pub fn new() -> (Self, UnboundedReceiver<LogMessage>) {
		let (tx, rx) = unbounded_channel();
		(
			XenonLogger {
				os: OsLogger::new("me.aspenuwu.xenon").level_filter(LevelFilter::Info),
				tx,
			},
			rx,
		)
	}
}

impl Log for XenonLogger {
	#[cfg(not(any(debug_assertions, feature = "beta")))]
	fn enabled(&self, metadata: &Metadata) -> bool {
		metadata.level() <= Level::Info
	}

	#[cfg(any(debug_assertions, feature = "beta"))]
	fn enabled(&self, metadata: &Metadata) -> bool {
		metadata.level() <= Level::Debug
	}

	fn log(&self, record: &Record) {
		self.os.log(record);
		let _ = self.tx.send(LogMessage::Message(format!(
			"[{}] {}\n",
			record.level(),
			record.args()
		)));
	}

	fn flush(&self) {
		let _ = self.tx.send(LogMessage::Flush);
	}
}

pub async fn file_logging_task(mut rx: UnboundedReceiver<LogMessage>) {
	let cfg_dir = PathBuf::from(CFG_FOLDER);
	if !cfg_dir.is_dir() {
		let _ = tokio::fs::create_dir_all(&cfg_dir).await;
	}
	let mut log_file = File::create(cfg_dir.join("daemon.log"))
		.await
		.expect("failed to open log file");
	while let Some(msg) = rx.recv().await {
		match msg {
			LogMessage::Message(log) => {
				let _ = log_file.write_all(log.as_bytes()).await;
			}
			LogMessage::Flush => {
				let _ = log_file.flush().await;
			}
		}
	}
}
