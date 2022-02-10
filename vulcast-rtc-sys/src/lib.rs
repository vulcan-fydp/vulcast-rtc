#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{ffi::CString, os::raw::c_char};

extern crate link_cplusplus;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// Retake ownership of a string that was previously released with CString::into_raw.
#[no_mangle]
pub unsafe extern "C" fn rust_unmarshal_str(s: *mut c_char) {
    drop(CString::from_raw(s));
}
