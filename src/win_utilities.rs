use std::{
	ffi::OsStr,
	iter::once,
	os::windows::ffi::OsStrExt,
}; // OS string // OS String Extended (wide character

pub fn win32_string(value : &str) -> Vec<u16>
{
	OsStr::new(value).encode_wide().chain(once(0)).collect()
}
