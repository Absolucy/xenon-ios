/*
	Copyright (c) aspen 2021
	All rights reserved.
*/

use std::ffi::CStr;
use std::os::raw::{c_char, c_void};

const MEMORYSTATUS_CMD_SET_JETSAM_TASK_LIMIT: u32 = 6;
const XENON_MEMORY_LIMIT_MB: u32 = 100;

extern "C" {
	fn memorystatus_control(
		command: u32,
		pid: i32,
		flags: u32,
		buffer: *mut c_void,
		buffersize: usize,
	) -> i32;
	fn getpid() -> i32;
	fn strerror(err: i32) -> *const c_char;
}

pub fn there_will_be_blood_yeaahh() {
	let ret = unsafe {
		memorystatus_control(
			MEMORYSTATUS_CMD_SET_JETSAM_TASK_LIMIT,
			getpid(),
			XENON_MEMORY_LIMIT_MB,
			std::ptr::null_mut(),
			0,
		)
	};
	if ret != 0 {
		match unsafe { CStr::from_ptr(strerror(ret)) }.to_str() {
			Ok(err) => {
				panic!("!! SETTING MEMORY LIMITS FAILED !!: {}", err);
			}
			_ => {
				panic!("!! SETTING MEMORY LIMITS FAILED !!: error code {}", ret);
			}
		}
	}
	debug!(
		"Memories broken, the truth goes unspoken, I've even forgotten my name, I don't know the season or what is the reason, I'm standing here holding my blade, setting my memory limits to {} MB.",
		XENON_MEMORY_LIMIT_MB
	)
}
