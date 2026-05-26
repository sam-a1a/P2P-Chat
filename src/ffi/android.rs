//! Android JNI exports.
//!
//! Kotlin usage:
//!
//! class P2pLib {
//!     external fun start(keyPath: String, dbPath: String): Long
//!     external fun pollEvent(handle: Long): String?
//!     external fun subscribe(handle: Long, topic: String)
//!     external fun publish(handle: Long, topic: String, data: ByteArray)
//!     external fun shutdown(handle: Long)
//!     external fun destroy(handle: Long)
//!
//!     companion object { init { System.loadLibrary("p2p") } }
//! }

use super::{FfiNode, ffi_destroy, ffi_free_string, ffi_poll_event,
            ffi_publish, ffi_shutdown, ffi_start, ffi_subscribe};
use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jbyteArray, jlong, jstring};
use jni::JNIEnv;

#[no_mangle]
pub extern "system" fn Java_com_example_p2p_P2pLib_start(
    mut env:      JNIEnv,
    _class:       JClass,
    key_path_j:   JString,
    db_path_j:    JString,
) -> jlong {
    let key_path: String = env.get_string(&key_path_j).unwrap().into();
    let db_path:  String = env.get_string(&db_path_j).unwrap().into();
    ffi_start(&key_path, &db_path) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_example_p2p_P2pLib_pollEvent(
    mut env:  JNIEnv,
    _class:   JClass,
    handle:   jlong,
) -> jstring {
    let ptr = handle as *mut FfiNode;
    let raw = ffi_poll_event(ptr);
    if raw.is_null() {
        return std::ptr::null_mut();
    }
    let s = unsafe { std::ffi::CStr::from_ptr(raw) }
        .to_str()
        .unwrap_or("{}");
    let out = env.new_string(s).unwrap().into_raw();
    ffi_free_string(raw);
    out
}

#[no_mangle]
pub extern "system" fn Java_com_example_p2p_P2pLib_subscribe(
    mut env:  JNIEnv,
    _class:   JClass,
    handle:   jlong,
    topic_j:  JString,
) {
    let topic: String = env.get_string(&topic_j).unwrap().into();
    ffi_subscribe(handle as *mut FfiNode, &topic);
}

#[no_mangle]
pub extern "system" fn Java_com_example_p2p_P2pLib_publish(
    mut env:  JNIEnv,
    _class:   JClass,
    handle:   jlong,
    topic_j:  JString,
    data_j:   JByteArray,
) {
    let topic: String = env.get_string(&topic_j).unwrap().into();
    let data: Vec<u8> = env.convert_byte_array(&data_j).unwrap();
    ffi_publish(handle as *mut FfiNode, &topic, &data);
}

#[no_mangle]
pub extern "system" fn Java_com_example_p2p_P2pLib_shutdown(
    _env:    JNIEnv,
    _class:  JClass,
    handle:  jlong,
) {
    ffi_shutdown(handle as *mut FfiNode);
}

#[no_mangle]
pub extern "system" fn Java_com_example_p2p_P2pLib_destroy(
    _env:    JNIEnv,
    _class:  JClass,
    handle:  jlong,
) {
    ffi_destroy(handle as *mut FfiNode);
}