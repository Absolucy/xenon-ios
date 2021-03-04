/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use anyhow::Result;

#[cfg(windows)]
pub async fn firewall_check() -> Result<()> {
	use anyhow::Context;
	use tokio::process::Command;

	let output = Command::new("netsh")
		.args(&["advfirewall", "show", "currentprofile"])
		.output()
		.await
		.context("failed to run 'netsh advfirewall show currentprofile'")?
		.stdout;
	let output = String::from_utf8(output).context("netsh output wasn't valid UTF-8")?;
	let firewall_on = output.lines().all(|line| {
		let line = line.split_whitespace().collect::<Vec<&str>>();
		if let (Some(key), Some(value)) = (line.get(0), line.get(1)) {
			*key != "State" && *value != "ON"
		} else {
			true
		}
	});
	if !firewall_on {
		notifica::notify("Xenon Pairing", "Windows Firewall may cause issues while pairing. Disable it to ensure it does not intefere, you may re-enable it afterwards if you want.").context("failed to send notification")?;
	}
	Ok(())
}

#[cfg(not(windows))]
pub async fn firewall_check() -> Result<()> {
	Ok(())
}
