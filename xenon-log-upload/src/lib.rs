/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const API_KEY: &str = "get yer own api key!";

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
		description: "Automatically uploaded Xenon log file".to_string(),
		sections: vec![PasteEeSection { name, contents }],
	};
	reqwest::Client::new()
		.post("https://api.paste.ee/v1/pastes")
		.header("X-Auth-Token", API_KEY)
		.header("Content-Type", "application/json")
		.json(&upload)
		.send()
		.await
		.context("failed to send http request to paste.ee api")?
		.json::<PasteEeResponse>()
		.await
		.map(|response| response.link)
		.context("failed to parse paste.ee response")
}
