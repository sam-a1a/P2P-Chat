//! iOS C ABI exports.
//!
//! Swift bridging header usage:
//!
//! void* p2p_start(const char* key_path, const char* db_path);
//! char* p2p_poll_event(void* handle);
//! void  p2p_free_string(char* s);
//! void  p2p_subscribe(void* handle, const char* topic);
//! void  p2p_publish(void* handle, const char* topic,
//!                   const uint8_t* data, uintptr_t len);
//! void  p2p_shutdown(void* handle);
//! void  p2p_destroy(void* handle);

use super::{FfiNode, ffi_destroy, ffi_free_string, ffi_poll_event,
            ffi_publish, ffi_shutdown, ffi_start, ffi_subscribe};
use std::ffi::CStr;
use std::os::raw::{c_char, c_uchar};

#[no_mangle]
pub extern "C" fn p2p_start(
    key_path: *const c_char,
    db_path:  *const c_char,
) -> *mut FfiNode {
    let key = unsafe { CStr::from_ptr(key_path) }.to_str().unwrap_or("");
    let db  = unsafe { CStr::from_ptr(db_path)  }.to_str().unwrap_or("");
    ffi_start(key, db)
}

#[no_mangle]
pub extern "C" fn p2p_poll_event(handle: *mut FfiNode) -> *mut c_char {
    ffi_poll_event(handle)
}

#[no_mangle]
pub extern "C" fn p2p_free_string(s: *mut c_char) {
    ffi_free_string(s);
}

#[no_mangle]
pub extern "C" fn p2p_subscribe(handle: *mut FfiNode, topic: *const c_char) {
    let t = unsafe { CStr::from_ptr(topic) }.to_str().unwrap_or("");
    ffi_subscribe(handle, t);
}

#[no_mangle]
pub extern "C" fn p2p_publish(
    handle: *mut FfiNode,
    topic:  *const c_char,
    data:   *const c_uchar,
    len:    usize,
) {
    let t    = unsafe { CStr::from_ptr(topic) }.to_str().unwrap_or("");
    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    ffi_publish(handle, t, bytes);
}

#[no_mangle]
pub extern "C" fn p2p_shutdown(handle: *mut FfiNode) {
    ffi_shutdown(handle);
}

#[no_mangle]
pub extern "C" fn p2p_destroy(handle: *mut FfiNode) {
    ffi_destroy(handle);
}