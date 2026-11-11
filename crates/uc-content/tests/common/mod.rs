//! Loads the built UCommanderContent DLL for FFI tests.

#![allow(dead_code)]

use std::ffi::c_char;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use uc_test_host::{Plugin, plugin_dll_path};

pub const FT_STRINGW: i32 = 11;
pub const FT_FILEERROR: i32 = -2;
pub const FT_FIELDEMPTY: i32 = -3;
pub const CONTENT_DELAYIFSLOW: i32 = 1;
/// A buffer value that the plugin does not write.
pub const CANARY: u8 = 0x55;

pub type GetSupportedField = unsafe extern "system" fn(i32, *mut c_char, *mut c_char, i32) -> i32;
pub type GetDetectString = unsafe extern "system" fn(*mut c_char, i32) -> i32;
pub type GetValueW = unsafe extern "system" fn(*const u16, i32, i32, *mut u8, i32, i32) -> i32;

pub fn load() -> Plugin {
    let path = plugin_dll_path("uc_content");
    Plugin::load(&path).unwrap_or_else(|e| panic!("load {}: {e}", path.display()))
}

pub fn export<F: Copy>(plugin: &Plugin, name: &str) -> F {
    // SAFETY: each caller uses the type alias that matches the guide.
    unsafe { plugin.export::<F>(name) }.unwrap_or_else(|| panic!("missing export {name}"))
}

pub fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/editor")
        .join(name)
}

pub fn fixture_bytes(name: &str) -> Vec<u8> {
    let bytes = std::fs::read(fixture(name)).unwrap();
    // A Git LFS pointer file is small text, not a package.
    assert!(
        !bytes.starts_with(b"version https://git-lfs"),
        "{name} is a Git LFS pointer: run `git lfs pull`"
    );
    bytes
}

/// Packages that the parser does not support: a UE4 package and an
/// unversioned package, made from fixture F1. Gives a file name and the
/// bytes of each.
pub fn unsupported_packages() -> [(&'static str, Vec<u8>); 2] {
    let bytes = fixture_bytes("BP_SummaryActor.uasset");
    // Bytes 4..8: legacy file version. Bytes 12..24: UE4, UE5 and licensee
    // versions.
    let mut ue4 = bytes.clone();
    ue4[4..8].copy_from_slice(&(-7i32).to_le_bytes());
    let mut unversioned = bytes;
    unversioned[12..24].fill(0);
    [("Ue4.uasset", ue4), ("Unversioned.uasset", unversioned)]
}

/// The folder for test files: in the Cargo temporary folder of the target
/// folder, not in the system temporary folder. `cargo clean` removes it.
fn test_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("uc-content")
}

/// Writes `bytes` to the file `name` in the test folder `folder`. Use one
/// folder for each test file, and a different file name in each test. A
/// new run writes the same file again.
pub fn test_file(folder: &str, name: &str, bytes: &[u8]) -> PathBuf {
    let dir = test_dir().join(folder);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

/// A path in the test folder with no file.
pub fn missing_file() -> PathBuf {
    let path = test_dir().join("no-such-file.uasset");
    assert!(!path.exists(), "{} must not exist", path.display());
    path
}

/// Calls ContentGetValueW for a text field with a buffer of `maxlen`
/// bytes. The buffer starts at an odd address, and has canary bytes before
/// and after it. Returns the result and the UTF-16 text before the NUL.
pub fn get_text(path: &Path, field: i32, maxlen: usize) -> (i32, String) {
    let plugin = load();
    let get: GetValueW = export(&plugin, "ContentGetValueW");
    let path: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    let mut buffer = vec![CANARY; maxlen + 2];

    // SAFETY: `path` is NUL-terminated, and the buffer from offset 1 has
    // `maxlen` bytes.
    let result = unsafe {
        get(
            path.as_ptr(),
            field,
            0,
            buffer.as_mut_ptr().add(1),
            maxlen as i32,
            CONTENT_DELAYIFSLOW,
        )
    };

    assert_eq!(buffer[0], CANARY, "write before the buffer");
    assert_eq!(buffer[maxlen + 1], CANARY, "write after the buffer");
    let units: Vec<u16> = buffer[1..=maxlen]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&pair| u16::from_le_bytes(pair))
        .take_while(|&unit| unit != 0)
        .collect();
    (result, String::from_utf16(&units).unwrap_or_default())
}

/// Text before the first NUL.
pub fn ansi_text(buffer: &[c_char]) -> String {
    buffer
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8 as char)
        .collect()
}
