use super::{
    FfiNode, ffi_destroy, ffi_free_string, ffi_poll_event,
    ffi_publish, ffi_shutdown, ffi_start, ffi_subscribe,
};
use jni::errors::ThrowRuntimeExAndDefault;
use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jlong, jstring};
use jni::EnvUnowned;
use std::ffi::CStr;

#[unsafe(no_mangle)]
#[allow(deprecated)]
pub extern "system" fn java_com_example_p2p_p2p_lib_start<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    key_path_j: JString<'local>,
    db_path_j: JString<'local>,
) -> jlong {
    unowned_env
        .with_env(|env| -> jni::errors::Result<jlong> {
            let key_path: String = env.get_string(&key_path_j)?.into();
            let db_path: String = env.get_string(&db_path_j)?.into();
            Ok(ffi_start(&key_path, &db_path) as jlong)
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
#[allow(deprecated)]
pub extern "system" fn java_com_example_p2p_p2p_lib_poll_event<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jstring {
    let ptr = handle as *mut FfiNode;
    let raw = ffi_poll_event(ptr);
    if raw.is_null() {
        return std::ptr::null_mut();
    }
    let s = unsafe { CStr::from_ptr(raw) }.to_str().unwrap_or("{}").to_owned();
    let out = unowned_env
        .with_env(|env| -> jni::errors::Result<jstring> {
            Ok(env.new_string(&s)?.into_raw())
        })
        .resolve::<ThrowRuntimeExAndDefault>();
    ffi_free_string(raw);
    out
}

#[unsafe(no_mangle)]
#[allow(deprecated)]
pub extern "system" fn java_com_example_p2p_p2p_lib_subscribe<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
    topic_j: JString<'local>,
) {
    unowned_env
        .with_env(|env| -> jni::errors::Result<()> {
            let topic: String = env.get_string(&topic_j)?.into();
            ffi_subscribe(handle as *mut FfiNode, &topic);
            Ok(())
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
#[allow(deprecated)]
pub extern "system" fn java_com_example_p2p_p2p_lib_publish<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
    topic_j: JString<'local>,
    data_j: JByteArray<'local>,
) {
    unowned_env
        .with_env(|env| -> jni::errors::Result<()> {
            let topic: String = env.get_string(&topic_j)?.into();
            let data: Vec<u8> = env.convert_byte_array(&data_j)?;
            ffi_publish(handle as *mut FfiNode, &topic, &data);
            Ok(())
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn java_com_example_p2p_p2p_lib_shutdown<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) {
    ffi_shutdown(handle as *mut FfiNode);
}

#[unsafe(no_mangle)]
pub extern "system" fn java_com_example_p2p_p2p_lib_destroy<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) {
    ffi_destroy(handle as *mut FfiNode);
}