use super::{FfiNode, ffi_destroy, ffi_free_string, ffi_poll_event,
            ffi_publish, ffi_shutdown, ffi_start, ffi_subscribe};
use std::ffi::CStr;
use std::os::raw::{c_char, c_uchar};

#[unsafe(no_mangle)]
pub extern "C" fn p2p_start(
    key_path: *const c_char,
    db_path: *const c_char,
) -> *mut FfiNode {
    let key = unsafe { CStr::from_ptr(key_path) }.to_str().unwrap_or("");
    let db  = unsafe { CStr::from_ptr(db_path)  }.to_str().unwrap_or("");
    ffi_start(key, db)
}

#[unsafe(no_mangle)]
pub extern "C" fn p2p_poll_event(handle: *mut FfiNode) -> *mut c_char {
    ffi_poll_event(handle)
}

#[unsafe(no_mangle)]
pub extern "C" fn p2p_free_string(s: *mut c_char) {
    ffi_free_string(s);
}

#[unsafe(no_mangle)]
pub extern "C" fn p2p_subscribe(handle: *mut FfiNode, topic: *const c_char) {
    let t = unsafe { CStr::from_ptr(topic) }.to_str().unwrap_or("");
    ffi_subscribe(handle, t);
}

#[unsafe(no_mangle)]
pub extern "C" fn p2p_publish(
    handle: *mut FfiNode,
    topic: *const c_char,
    data: *const c_uchar,
    len: usize,
) {
    let t    = unsafe { CStr::from_ptr(topic) }.to_str().unwrap_or("");
    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    ffi_publish(handle, t, bytes);
}

#[unsafe(no_mangle)]
pub extern "C" fn p2p_shutdown(handle: *mut FfiNode) {
    ffi_shutdown(handle);
}

#[unsafe(no_mangle)]
pub extern "C" fn p2p_destroy(handle: *mut FfiNode) {
    ffi_destroy(handle);
}