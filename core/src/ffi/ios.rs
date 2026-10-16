use super::{
    FfiNode, ffi_destroy, ffi_free_string, ffi_poll_event, ffi_publish, ffi_shutdown, ffi_start,
    ffi_subscribe,
};
use std::ffi::CStr;
use std::os::raw::{c_char, c_uchar};

/// # Safety
/// Both paths must be valid NUL-terminated strings for the duration of this call, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn p2p_start(
    key_path: *const c_char,
    db_path: *const c_char,
) -> *mut FfiNode {
    if key_path.is_null() || db_path.is_null() {
        return std::ptr::null_mut();
    }
    let key = unsafe { CStr::from_ptr(key_path) }.to_str().unwrap_or("");
    let db = unsafe { CStr::from_ptr(db_path) }.to_str().unwrap_or("");
    ffi_start(key, db)
}

/// # Safety
/// The handle must be null or a live node returned by p2p_start. Serialize all handle access.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn p2p_poll_event(handle: *mut FfiNode) -> *mut c_char {
    ffi_poll_event(handle)
}

/// # Safety
/// The pointer must be null or an unfreed string returned by p2p_poll_event.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn p2p_free_string(s: *mut c_char) {
    ffi_free_string(s);
}

/// # Safety
/// The handle must be null or live, with serialized access. Topic must be null or a valid NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn p2p_subscribe(handle: *mut FfiNode, topic: *const c_char) {
    if topic.is_null() {
        return;
    }
    let t = unsafe { CStr::from_ptr(topic) }.to_str().unwrap_or("");
    ffi_subscribe(handle, t);
}

/// # Safety
/// The handle must be null or live, with serialized access. Topic must be a valid NUL-terminated string or null. Non-null data must point to at least len readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn p2p_publish(
    handle: *mut FfiNode,
    topic: *const c_char,
    data: *const c_uchar,
    len: usize,
) {
    if topic.is_null() || data.is_null() {
        return;
    }
    let t = unsafe { CStr::from_ptr(topic) }.to_str().unwrap_or("");
    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    ffi_publish(handle, t, bytes);
}

/// # Safety
/// The handle must be null or a live node, with serialized access.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn p2p_shutdown(handle: *mut FfiNode) {
    ffi_shutdown(handle);
}

/// # Safety
/// The handle must be null or a live node, with exclusive access. Never use the handle again after this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn p2p_destroy(handle: *mut FfiNode) {
    ffi_destroy(handle);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn null_inputs_are_rejected_without_dereferencing() {
        // Every exported function explicitly accepts null pointers as a no-op/error.
        unsafe {
            assert!(p2p_start(std::ptr::null(), std::ptr::null()).is_null());
            assert!(p2p_poll_event(std::ptr::null_mut()).is_null());
            p2p_free_string(std::ptr::null_mut());
            p2p_subscribe(std::ptr::null_mut(), std::ptr::null());
            p2p_publish(std::ptr::null_mut(), std::ptr::null(), std::ptr::null(), 1);
            p2p_shutdown(std::ptr::null_mut());
            p2p_destroy(std::ptr::null_mut());
        }
    }
}
