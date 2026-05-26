//! p2p — LAN chat engine for Android & iOS.
//!
//! Compiled as staticlib (iOS) + `cdylib` (Android).
//! The public surface for mobile is entirely in ffi::ios / ffi::android
//! The public surface for Rust callers is node::start_node

pub mod behaviour;
pub mod crypto;
pub mod error;
pub mod ffi;
pub mod identity;
pub mod node;
pub mod storage;
pub mod types;