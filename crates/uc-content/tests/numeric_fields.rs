//! The file version fields (`ft_numeric_32`): FFI tests with the built DLL.

mod common;

use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use common::{
    CANARY, CONTENT_DELAYIFSLOW, FT_FIELDEMPTY, FT_FILEERROR, GetValueW, export, fixture,
    fixture_bytes, load,
};

const FT_NUMERIC_32: i32 = 1;

/// The field index and the expected value for each fixture. The values are
/// the `summary` values of the `.expected.json` files: `file_version_ue5`,
/// `file_version_ue4` and `file_version_licensee`. All fixtures have the
/// same values.
const FIELDS: [(i32, i32); 3] = [(8, 1018), (9, 522), (10, 0)];

const FIXTURES: [&str; 3] = [
    "BP_SummaryActor.uasset",
    "L_SummaryLevel.umap",
    "M_Summary.uasset",
];

/// Package flag `ContainsNoAsset`.
const PKG_CONTAINS_NO_ASSET: u32 = 0x0000_0400;

fn test_file(name: &str, bytes: &[u8]) -> PathBuf {
    common::test_file("numeric", name, bytes)
}

/// Calls ContentGetValueW with a buffer of `maxlen` bytes. The buffer
/// starts at an odd address, and has canary bytes before and after it.
/// Returns the result and the `i32` at the start of the buffer. The
/// canary value is not a value that the plugin writes.
fn get_i32(path: &Path, field: i32, maxlen: usize) -> (i32, i32) {
    let plugin = load();
    let get: GetValueW = export(&plugin, "ContentGetValueW");
    let path: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    let mut buffer = vec![CANARY; maxlen.max(4) + 2];

    // SAFETY: `path` is NUL-terminated, and the buffer from offset 1 has
    // at least `maxlen` bytes.
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
    assert!(
        buffer[maxlen + 1..].iter().all(|&byte| byte == CANARY),
        "write after the buffer"
    );
    let value = i32::from_le_bytes(buffer[1..5].try_into().unwrap());
    (result, value)
}

#[test]
fn the_fixtures_give_the_expected_values() {
    for (index, value) in FIELDS {
        for name in FIXTURES {
            assert_eq!(
                get_i32(&fixture(name), index, 4),
                (FT_NUMERIC_32, value),
                "field {index}, {name}"
            );
        }
    }
}

#[test]
fn a_package_without_an_asset_has_the_file_versions() {
    let mut bytes = fixture_bytes("BP_SummaryActor.uasset");
    // The package flags follow the package name in the summary.
    let name = b"/Game/Summary/BP_SummaryActor\0";
    let flags = bytes
        .windows(name.len())
        .position(|window| window == name)
        .unwrap()
        + name.len();
    let value = u32::from_le_bytes(bytes[flags..flags + 4].try_into().unwrap());
    bytes[flags..flags + 4].copy_from_slice(&(value | PKG_CONTAINS_NO_ASSET).to_le_bytes());
    let path = test_file("NoAsset.uasset", &bytes);

    for (index, value) in FIELDS {
        assert_eq!(get_i32(&path, index, 4), (FT_NUMERIC_32, value));
    }
}

#[test]
fn a_file_that_is_not_a_package_has_empty_fields() {
    let path = test_file("Text.uasset", b"This is a text file.");

    for (index, _) in FIELDS {
        assert_eq!(get_i32(&path, index, 4).0, FT_FIELDEMPTY);
    }
}

#[test]
fn a_damaged_package_has_empty_fields() {
    let bytes = fixture_bytes("BP_SummaryActor.uasset");
    for length in [4, 100, 1000, 5400, 6000] {
        let path = test_file(&format!("Damaged{length}.uasset"), &bytes[..length]);

        for (index, _) in FIELDS {
            assert_eq!(
                get_i32(&path, index, 4).0,
                FT_FIELDEMPTY,
                "field {index}, length {length}"
            );
        }
    }
}

#[test]
fn a_missing_file_is_a_file_error() {
    let path = std::env::temp_dir().join("uc-content-numeric-tests-no-such-file.uasset");

    for (index, _) in FIELDS {
        assert_eq!(get_i32(&path, index, 4).0, FT_FILEERROR);
    }
}

#[test]
fn a_buffer_smaller_than_4_bytes_gives_an_empty_field() {
    let path = fixture(FIXTURES[0]);
    let canary = i32::from_le_bytes([CANARY; 4]);

    for (index, _) in FIELDS {
        for maxlen in [0, 3] {
            assert_eq!(get_i32(&path, index, maxlen), (FT_FIELDEMPTY, canary));
        }
    }
}
