//! Loads the built UCommanderContent DLL for FFI tests.

#![allow(dead_code)]

use std::ffi::c_char;

use uc_test_host::{Plugin, fresh_plugin_dll};

pub type GetSupportedField = unsafe extern "system" fn(i32, *mut c_char, *mut c_char, i32) -> i32;
pub type GetDetectString = unsafe extern "system" fn(*mut c_char, i32) -> i32;
pub type GetValueW = unsafe extern "system" fn(*const u16, i32, i32, *mut u8, i32, i32) -> i32;

pub fn load() -> Plugin {
    let path = fresh_plugin_dll("uc_content");
    Plugin::load(&path).unwrap_or_else(|e| panic!("load {}: {e}", path.display()))
}

pub fn export<F: Copy>(plugin: &Plugin, name: &str) -> F {
    // SAFETY: each caller uses the type alias that matches the guide.
    unsafe { plugin.export::<F>(name) }.unwrap_or_else(|| panic!("missing export {name}"))
}

/// Text before the first NUL.
pub fn ansi_text(buffer: &[c_char]) -> String {
    buffer
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8 as char)
        .collect()
}
