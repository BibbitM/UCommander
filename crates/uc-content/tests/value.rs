mod common;

use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use common::{GetValueW, export, load};

const FT_BOOLEAN: i32 = 6;
const FT_NOSUCHFIELD: i32 = -1;
const FT_FILEERROR: i32 = -2;
const FT_FIELDEMPTY: i32 = -3;
const CONTENT_DELAYIFSLOW: i32 = 1;

const IS_PACKAGE: i32 = 0;

/// The package file tag as it is stored in a file.
const TAG: [u8; 4] = [0xC1, 0x83, 0x2A, 0x9E];

/// Writes `bytes` to a new file in a test folder for this process.
fn test_file(name: &str, bytes: &[u8]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("uc-content-tests-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

fn wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain([0]).collect()
}

/// Calls ContentGetValueW for a boolean field. Returns the result and the value.
fn get_bool(path: &Path, field_index: i32) -> (i32, i32) {
    let plugin = load();
    let get: GetValueW = export(&plugin, "ContentGetValueW");
    let path = wide(path);
    let mut value: i32 = 0x5555;

    // SAFETY: `path` is NUL-terminated, and `value` has 4 bytes.
    let result = unsafe {
        get(
            path.as_ptr(),
            field_index,
            0,
            (&raw mut value).cast(),
            4,
            CONTENT_DELAYIFSLOW,
        )
    };
    (result, value)
}

#[test]
fn a_file_that_starts_with_the_tag_is_a_package() {
    let mut bytes = TAG.to_vec();
    bytes.extend_from_slice(&[0; 60]);
    let path = test_file("Tagged.uasset", &bytes);

    let (result, value) = get_bool(&path, IS_PACKAGE);

    assert_eq!(result, FT_BOOLEAN);
    assert_ne!(value, 0);
}

#[test]
fn a_file_with_other_bytes_is_not_a_package() {
    let path = test_file("Text.uasset", b"This is a text file.");

    assert_eq!(get_bool(&path, IS_PACKAGE), (FT_BOOLEAN, 0));
}

#[test]
fn a_file_shorter_than_the_tag_is_not_a_package() {
    let path = test_file("Short.umap", &TAG[..3]);

    assert_eq!(get_bool(&path, IS_PACKAGE), (FT_BOOLEAN, 0));
}

#[test]
fn a_path_with_non_ascii_characters_is_read() {
    let path = test_file("Żółw Größe.uasset", &TAG);

    let (result, value) = get_bool(&path, IS_PACKAGE);

    assert_eq!(result, FT_BOOLEAN);
    assert_ne!(value, 0);
}

#[test]
fn a_missing_file_is_a_file_error() {
    let path = std::env::temp_dir().join("uc-content-tests-no-such-file.uasset");

    assert_eq!(get_bool(&path, IS_PACKAGE).0, FT_FILEERROR);
}

#[test]
fn an_unknown_field_index_is_no_such_field() {
    let path = test_file("Unknown.uasset", &TAG);

    assert_eq!(get_bool(&path, 1).0, FT_NOSUCHFIELD);
    assert_eq!(get_bool(&path, -1).0, FT_NOSUCHFIELD);
}

#[test]
fn null_pointers_do_not_crash_the_plugin() {
    let plugin = load();
    let get: GetValueW = export(&plugin, "ContentGetValueW");
    let path = wide(&test_file("Null.uasset", &TAG));
    let mut value: i32 = 0;

    // SAFETY: the plugin must check for null pointers.
    let null_path = unsafe {
        get(
            std::ptr::null(),
            IS_PACKAGE,
            0,
            (&raw mut value).cast(),
            4,
            0,
        )
    };
    // SAFETY: the plugin must check for null pointers.
    let null_value = unsafe { get(path.as_ptr(), IS_PACKAGE, 0, std::ptr::null_mut(), 4, 0) };

    assert_eq!(null_path, FT_FILEERROR);
    assert_eq!(null_value, FT_FIELDEMPTY);
}
