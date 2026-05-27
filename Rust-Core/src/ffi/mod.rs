#[cfg(target_os = "android")]
pub mod android;

#[cfg(target_os = "ios")]
pub mod ios;

use crate::{identity, node};
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub(super) fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("p2p-worker")
            .build()
            .expect("tokio runtime init failed")
    })
}

pub struct FfiNode {
    pub handle: node::NodeHandle,
}

pub(super) fn ffi_start(key_path: &str, _db_path: &str) -> *mut FfiNode {
    let rt = runtime();

    let keypair = match identity::load_or_create_keypair(key_path) {
        Ok(k) => k,
        Err(e) => {
            log::error!("ffi_start: identity error: {e}");
            return std::ptr::null_mut();
        }
    };

    let handle = match rt.block_on(async { node::start_node(keypair) }) {
        Ok(h) => h,
        Err(e) => {
            log::error!("ffi_start: node error: {e}");
            return std::ptr::null_mut();
        }
    };

    Box::into_raw(Box::new(FfiNode { handle }))
}

pub(super) fn ffi_poll_event(ptr: *mut FfiNode) -> *mut c_char {
    if ptr.is_null() { return std::ptr::null_mut(); }
    let node = unsafe { &mut *ptr };

    match node.handle.event_rx.try_recv() {
        Ok(ev) => {
            let json = serde_json::to_string(&ev).unwrap_or_else(|_| "{}".into());
            CString::new(json).map(CString::into_raw).unwrap_or(std::ptr::null_mut())
        }
        Err(_) => std::ptr::null_mut(),
    }
}

pub(super) fn ffi_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe { drop(CString::from_raw(ptr)) };
    }
}

pub(super) fn ffi_subscribe(ptr: *mut FfiNode, topic: &str) {
    if ptr.is_null() { return; }
    let node = unsafe { &*ptr };
    node.handle.subscribe(topic);
}

pub(super) fn ffi_publish(ptr: *mut FfiNode, topic: &str, data: &[u8]) {
    if ptr.is_null() { return; }
    let node = unsafe { &*ptr };
    node.handle.publish(topic, data.to_vec());
}

pub(super) fn ffi_shutdown(ptr: *mut FfiNode) {
    if ptr.is_null() { return; }
    let node = unsafe { &*ptr };
    node.handle.shutdown();
}

pub(super) fn ffi_destroy(ptr: *mut FfiNode) {
    if !ptr.is_null() {
        unsafe { drop(Box::from_raw(ptr)) };
    }
}