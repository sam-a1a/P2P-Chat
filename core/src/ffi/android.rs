use super::{FfiNode, ffi_destroy, ffi_free_string, ffi_poll_event,
            ffi_publish, ffi_shutdown, ffi_start, ffi_subscribe};
use jni::objects::{JClass, JString, JByteArray};
use jni::sys::{jlong, jstring};
use jni::EnvUnowned;
use jni::errors::ThrowRuntimeExAndDefault;

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_p2p_P2pLib_start<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    key_path_j: JString<'local>,
    db_path_j: JString<'local>,
) -> jlong {
    let outcome = unowned_env.with_env(|env| -> Result<jlong, jni::errors::Error> {
        let key_path: String = key_path_j.to_string();
        let db_path: String = db_path_j.to_string();
        Ok(ffi_start(&key_path, &db_path) as jlong)
    });
    outcome.resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_p2p_P2pLib_pollEvent<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jstring {
    let ptr = handle as *mut FfiNode;
    let raw = ffi_poll_event(ptr);
    if raw.is_null() {
        return std::ptr::null_mut();
    }
    let s = unsafe { std::ffi::CStr::from_ptr(raw) }.to_str().unwrap_or("{}");
    let outcome = unowned_env.with_env(|env| -> Result<jstring, jni::errors::Error> {
        Ok(env.new_string(s)?.into_raw())
    });
    ffi_free_string(raw);
    outcome.resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_p2p_P2pLib_subscribe<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
    topic_j: JString<'local>,
) {
    let outcome = unowned_env.with_env(|env| -> Result<(), jni::errors::Error> {
        let topic: String = topic_j.to_string();
        ffi_subscribe(handle as *mut FfiNode, &topic);
        Ok(())
    });
    outcome.resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_p2p_P2pLib_publish<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
    topic_j: JString<'local>,
    data_j: JByteArray<'local>,
) {
    let outcome = unowned_env.with_env(|env| -> Result<(), jni::errors::Error> {
        let topic: String = topic_j.to_string();
        let data: Vec<u8> = env.convert_byte_array(&data_j)?;
        ffi_publish(handle as *mut FfiNode, &topic, &data);
        Ok(())
    });
    outcome.resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_p2p_P2pLib_shutdown<'local>(
    _unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) {
    ffi_shutdown(handle as *mut FfiNode);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_p2p_P2pLib_destroy<'local>(
    _unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) {
    ffi_destroy(handle as *mut FfiNode);
}