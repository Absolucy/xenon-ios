/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use crate::config::CONFIG;

#[cfg(target_os = "windows")]
pub async fn mount_webdav() {
	use async_anyhow_logger::catch_context;
	use std::process::Stdio;
	use tokio::process::Command;

	let drive_letter = CONFIG
		.read()
		.await
		.general
		.windows_mount_point
		.clone()
		.filter(|letter| (letter.len() == 2 && letter.ends_with(':')) || letter.len() == 1)
		.map(|letter| {
			if letter.len() == 1 {
				letter + ":"
			} else {
				letter
			}
		})
		.unwrap_or_else(|| "W:".to_string());

	match Command::new("net")
		.arg("use")
		.arg(drive_letter)
		.arg(format!(
			"http://localhost:{}",
			CONFIG.read().await.general.port
		))
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.stdin(Stdio::null())
		.spawn()
	{
		Ok(child) => {
			tokio::spawn(catch_context("failed to wait for 'net use'", async move {
				let output = child.wait_with_output().await?;
				let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
				let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
				if !stdout.is_empty() {
					debug!("'net use' output:\n{}", stdout);
				}
				if !stderr.is_empty() {
					debug!("'net use' err output:\n{}", stderr);
				}
				Ok(())
			}));
		}
		Err(err) => {
			error!("failed to run 'net use': {:?}", err);
		}
	}
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub async fn mount_webdav() {
	info!("You need to manually mount WebDAV at http://localhost:{}, as this OS does not have a standardized method of mounting.", CONFIG.read().await.general.port);
}

#[cfg(target_os = "macos")]
pub async fn mount_webdav() {
	use std::process::Command;

	let ourself = match std::env::current_exe() {
		Ok(o) => o,
		Err(err) => {
			error!("failed to get own path: {:?}", err);
			return;
		}
	};

	let applescript = ourself
		.parent()
		.expect("somehow xenon-client is orphaned")
		.join("../Resources/mount.applescript");

	match Command::new("osascript")
		.arg(applescript)
		.arg(CONFIG.read().await.general.port.to_string())
		.spawn()
	{
		Ok(_) => (),
		Err(err) => {
			error!("failed to run osascript: {:?}", err);
		}
	}
}
