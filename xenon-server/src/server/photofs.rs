/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use core::ops::Range;
use futures::{Future, StreamExt};
use http::StatusCode;
use std::{path::PathBuf, pin::Pin};
use webdav_handler::{
	davpath::DavPath,
	fs::{
		DavDirEntry, DavFile, DavFileSystem, DavMetaData, DavProp, FsError, FsFuture, FsStream,
		OpenOptions, ReadDirMeta,
	},
	localfs::LocalFs,
};

// Convert a folder name to a number range
pub fn folder_to_num(name: &str) -> Option<Range<u16>> {
	let num: u16 = name
		.trim()
		.strip_suffix("APPLE")
		.and_then(|x| x.parse().ok())?;
	let base = (num - 100) * 1000;
	Some(base..base + 1000)
}

// Extract a file number from it's name
pub fn file_to_num(name: &str) -> Option<u16> {
	name.trim().strip_prefix("IMG_").and_then(|x| {
		let mut split = x.split('.');
		let before_dot = split.next()?;
		before_dot.parse().ok()
	})
}

// Convert a file number (big) to a folder number (small)
pub fn num_to_folder(num: u16) -> String {
	[
		(((num as f32) / 1000.0).floor() as u16 + 100)
			.to_string()
			.as_str(),
		"APPLE",
	]
	.join("")
}

#[derive(Clone)]
pub struct PhotoFs {
	inner: LocalFs,
}

impl Default for PhotoFs {
	fn default() -> Self {
		Self {
			inner: *LocalFs::new("/var/mobile/Media/DCIM", true, false, true),
		}
	}
}

impl PhotoFs {
	pub fn get_new_path(path: &DavPath) -> Option<DavPath> {
		let path_buf = path.as_pathbuf();
		let name = match path_buf.file_name().and_then(|x| x.to_str()) {
			Some(s) => s,
			None => return None,
		};
		let folder = match file_to_num(name).map(num_to_folder) {
			Some(s) => s,
			None => return None,
		};
		let out = DavPath::new(
			&PathBuf::from(&["/", folder.as_str()].join(""))
				.join("")
				.join(name)
				.display()
				.to_string(),
		)
		.expect("failed to parse path");
		Some(out)
	}
}

impl DavFileSystem for PhotoFs {
	fn open<'a>(&'a self, path: &'a DavPath, options: OpenOptions) -> FsFuture<Box<dyn DavFile>> {
		Box::pin(async move {
			let new_path = match Self::get_new_path(path) {
				Some(s) => s,
				None => return Err(FsError::NotFound),
			};
			self.inner.open(&new_path, options).await
		})
	}

	fn read_dir<'a>(
		&'a self,
		_path: &'a DavPath,
		meta: ReadDirMeta,
	) -> FsFuture<FsStream<Box<dyn DavDirEntry>>> {
		Box::pin(async move {
			let mut read_dir = tokio::fs::read_dir("/var/mobile/Media/DCIM")
				.await
				.expect("failed to read directory");
			let mut contents = Vec::<Box<dyn DavDirEntry>>::new();
			while let Ok(Some(dir)) = read_dir.next_entry().await {
				let path = dir.path();
				let name = match path.file_name().and_then(|x| x.to_str()) {
					Some(s) => s,
					None => continue,
				};
				if folder_to_num(name).is_none() {
					continue;
				};
				let prefixed_path =
					DavPath::new(&["/", name].join("")).expect("failed to parse name");
				let mut entries: FsStream<Box<dyn DavDirEntry>> =
					match self.inner.read_dir(&prefixed_path, meta).await {
						Ok(o) => o,
						Err(err) => {
							error!("failed to read directory '{}': {}", path.display(), err);
							continue;
						}
					};
				while let Some(dir_entry) = entries.next().await {
					contents.push(dir_entry);
				}
			}
			Ok(Box::pin(futures::stream::iter(contents.into_iter()))
				as FsStream<Box<dyn DavDirEntry>>)
		})
	}

	fn metadata<'a>(&'a self, path: &'a DavPath) -> FsFuture<Box<dyn DavMetaData>> {
		let path_buf = path.as_pathbuf();
		if path_buf == PathBuf::from("/") {
			self.inner.metadata(path)
		} else {
			Box::pin(async move {
				let new_path = match Self::get_new_path(path) {
					Some(s) => s,
					None => return Err(FsError::NotFound),
				};
				self.inner.metadata(&new_path).await
			})
		}
	}

	fn remove_dir<'a>(&'a self, _: &'a DavPath) -> FsFuture<()> {
		Box::pin(async move { Err(FsError::NotFound) })
	}

	fn create_dir<'a>(&'a self, _: &'a DavPath) -> FsFuture<()> {
		Box::pin(async move { Err(FsError::NotFound) })
	}

	fn rename<'a>(&'a self, _: &'a DavPath, _: &'a DavPath) -> FsFuture<()> {
		Box::pin(async move { Err(FsError::NotFound) })
	}

	fn copy<'a>(&'a self, _: &'a DavPath, _: &'a DavPath) -> FsFuture<()> {
		Box::pin(async move { Err(FsError::NotFound) })
	}

	fn remove_file<'a>(&'a self, path: &'a DavPath) -> FsFuture<()> {
		Box::pin(async move {
			let new_path = match Self::get_new_path(path) {
				Some(s) => s,
				None => return Err(FsError::NotFound),
			};
			self.inner.remove_file(&new_path).await
		})
	}

	fn have_props<'a>(
		&'a self,
		path: &'a DavPath,
	) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
		Box::pin(async move {
			let new_path = match Self::get_new_path(path) {
				Some(s) => s,
				None => return false,
			};
			self.inner.have_props(&new_path).await
		})
	}

	fn get_props<'a>(&'a self, path: &'a DavPath, do_content: bool) -> FsFuture<Vec<DavProp>> {
		Box::pin(async move {
			let new_path = match Self::get_new_path(path) {
				Some(s) => s,
				None => return Err(FsError::NotFound),
			};
			self.inner.get_props(&new_path, do_content).await
		})
	}

	fn get_prop<'a>(&'a self, path: &'a DavPath, prop: DavProp) -> FsFuture<Vec<u8>> {
		Box::pin(async move {
			let new_path = match Self::get_new_path(path) {
				Some(s) => s,
				None => return Err(FsError::NotFound),
			};
			self.inner.get_prop(&new_path, prop).await
		})
	}

	fn patch_props<'a>(
		&'a self,
		path: &'a DavPath,
		patch: Vec<(bool, DavProp)>,
	) -> FsFuture<Vec<(StatusCode, DavProp)>> {
		Box::pin(async move {
			let new_path = match Self::get_new_path(path) {
				Some(s) => s,
				None => return Err(FsError::NotFound),
			};
			self.inner.patch_props(&new_path, patch).await
		})
	}
}
