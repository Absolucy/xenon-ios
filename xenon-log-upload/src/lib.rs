/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use anyhow::{Context, Result};
use obfstr::obfstr;
use serde::{Deserialize, Serialize};

const API_KEY: &str = include_str!("paste_ee.key");

#[derive(Serialize)]
struct PasteEeUpload {
	description: String,
	sections: Vec<PasteEeSection>,
}

#[derive(Serialize)]
struct PasteEeSection {
	name: String,
	contents: String,
}

#[derive(Deserialize)]
struct PasteEeResponse {
	link: String,
}

pub async fn upload_log_to_paste(name: String, contents: String) -> Result<String> {
	let upload = PasteEeUpload {
		description: obfstr!("Automatically uploaded Xenon log file").to_string(),
		sections: vec![PasteEeSection { name, contents }],
	};
	reqwest::Client::new()
		.post(obfstr!("https://api.paste.ee/v1/pastes"))
		.header(obfstr!("X-Auth-Token"), obfstr!(API_KEY))
		.header(obfstr!("Content-Type"), obfstr!("application/json"))
		.json(&upload)
		.send()
		.await
		.with_context(|| obfstr!("failed to send http request to paste.ee api").to_string())?
		.json::<PasteEeResponse>()
		.await
		.map(|response| response.link)
		.with_context(|| obfstr!("failed to parse paste.ee response").to_string())
}
