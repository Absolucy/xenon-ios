use std::path::PathBuf;

fn main() {
	let root_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("..");
	if cfg!(target_os = "windows") {
		let mut res = winres::WindowsResource::new();
		res.set_icon_with_id(
			&root_dir.join("res/windows/xenon.ico").display().to_string(),
			"xenon-tray",
		)
		.set_language(0x0409)
		.compile()
		.expect("failed to windows metadata!");
	}
}
