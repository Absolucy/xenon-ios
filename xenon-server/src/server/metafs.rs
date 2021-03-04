/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use crate::mount::DAV_MOUNTS;
use once_cell::sync::Lazy;
use std::time::SystemTime;
use webdav_handler::{
	davpath::DavPath,
	fs::{
		DavDirEntry, DavFile, DavFileSystem, DavMetaData, FsError, FsFuture, FsResult, FsStream,
		OpenOptions, ReadDirMeta,
	},
	DavHandler,
};

pub static METAFS: Lazy<DavHandler> = Lazy::new(|| {
	DavHandler::builder()
		.filesystem(Box::new(MetaFs))
		.build_handler()
});

#[derive(Copy, Clone)]
pub struct MetaFs;

impl DavFileSystem for MetaFs {
	fn open<'a>(&'a self, path: &'a DavPath, _: OpenOptions) -> FsFuture<Box<dyn DavFile>> {
		error!("returning 404 for open({})", path);
		Box::pin(async move { Err(FsError::NotFound) })
	}

	fn read_dir<'a>(
		&'a self,
		_: &'a DavPath,
		_: ReadDirMeta,
	) -> FsFuture<FsStream<Box<dyn DavDirEntry>>> {
		Box::pin(async move {
			let mut mounts = Vec::<Box<dyn DavDirEntry>>::new();
			let global_mounts = DAV_MOUNTS.read().await;
			for mount in global_mounts.keys() {
				mounts.push(Box::new(MetaFsEntry {
					name: mount.as_bytes().to_vec(),
				}))
			}
			Ok(Box::pin(futures::stream::iter(mounts.into_iter()))
				as FsStream<Box<dyn DavDirEntry>>)
		})
	}

	fn metadata<'a>(&'a self, _: &'a DavPath) -> FsFuture<Box<dyn DavMetaData>> {
		Box::pin(async move { Ok(Box::new(MetaFsMetadata) as Box<dyn DavMetaData>) })
	}
}

pub struct MetaFsEntry {
	name: Vec<u8>,
}

impl DavDirEntry for MetaFsEntry {
	fn name(&self) -> Vec<u8> {
		self.name.clone()
	}

	fn metadata(&self) -> FsFuture<Box<dyn DavMetaData>> {
		Box::pin(async move { Ok(Box::new(MetaFsMetadata) as Box<dyn DavMetaData>) })
	}
}

#[derive(Debug, Clone, Copy)]
pub struct MetaFsMetadata;

impl DavMetaData for MetaFsMetadata {
	fn len(&self) -> u64 {
		0
	}

	fn modified(&self) -> FsResult<SystemTime> {
		Ok(SystemTime::now())
	}

	fn is_dir(&self) -> bool {
		true
	}
}
