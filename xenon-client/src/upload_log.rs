/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use crate::XENON_DIR;

pub async fn upload_log() {
	let logfile = XENON_DIR.data_dir().join("xenon-client.log");
	let logs = match tokio::fs::read_to_string(logfile).await {
		Ok(o) => o,
		Err(err) => {
			let _ = notifica::notify(
				"Error uploading log file",
				&format!("Failed to read log file: {:?}", err),
			);
			return;
		}
	};
	let url = match xenon_log_upload::upload_log_to_paste("xenon client log".into(), logs).await {
		Ok(o) => o,
		Err(err) => {
			let _ = notifica::notify(
				"Error uploading log file",
				&format!("Failed to upload log file: {:?}", err),
			);
			return;
		}
	};
	match cli_clipboard::set_contents(url.clone()) {
		Ok(_) => {
			let _ = notifica::notify(
				"Uploaded log file",
				&format!(
					"Log file succesfully uploaded at '{}' and copied to clipboard",
					url
				),
			);
		}
		Err(_) => {
			let _ = notifica::notify(
				"Error copying log file",
				&format!(
							"Log file succesfully uploaded at '{}', but it couldn't be copied to the clipboard!",
							url
						),
			);
		}
	}
}
