//! The field "Asset class": FFI tests with the built DLL.

mod common;

use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use common::{GetValueW, export, load};

const FT_STRINGW: i32 = 11;
const FT_FILEERROR: i32 = -2;
const FT_FIELDEMPTY: i32 = -3;
const CONTENT_DELAYIFSLOW: i32 = 1;

const ASSET_CLASS: i32 = 1;
/// A buffer value that the plugin does not write.
const CANARY: u8 = 0x55;

/// Package flag `ContainsNoAsset`.
const PKG_CONTAINS_NO_ASSET: u32 = 0x0000_0400;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/editor")
        .join(name)
}

fn fixture_bytes(name: &str) -> Vec<u8> {
    let bytes = std::fs::read(fixture(name)).unwrap();
    // A Git LFS pointer file is small text, not a package.
    assert!(
        !bytes.starts_with(b"version https://git-lfs"),
        "{name} is a Git LFS pointer: run `git lfs pull`"
    );
    bytes
}

/// Writes `bytes` to a new file in a test folder for this process.
fn test_file(name: &str, bytes: &[u8]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("uc-content-asset-tests-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

/// Calls ContentGetValueW for the asset class with a buffer of `maxlen`
/// bytes. The buffer starts at an odd address, and has canary bytes before
/// and after it. Returns the result and the UTF-16 text before the NUL.
fn get_text(path: &Path, maxlen: usize) -> (i32, String) {
    let plugin = load();
    let get: GetValueW = export(&plugin, "ContentGetValueW");
    let path: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    let mut buffer = vec![CANARY; maxlen + 2];

    // SAFETY: `path` is NUL-terminated, and the buffer from offset 1 has
    // `maxlen` bytes.
    let result = unsafe {
        get(
            path.as_ptr(),
            ASSET_CLASS,
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

/// The expected values are the `asset_class` values of the
/// `.expected.json` files of the fixtures.
#[test]
fn the_blueprint_fixture_gives_blueprint() {
    let (result, text) = get_text(&fixture("BP_SummaryActor.uasset"), 256);

    assert_eq!(result, FT_STRINGW);
    assert_eq!(text, "Blueprint");
}

#[test]
fn the_level_fixture_gives_world() {
    let (result, text) = get_text(&fixture("L_SummaryLevel.umap"), 256);

    assert_eq!(result, FT_STRINGW);
    assert_eq!(text, "World");
}

#[test]
fn the_material_fixture_gives_material() {
    let (result, text) = get_text(&fixture("M_Summary.uasset"), 256);

    assert_eq!(result, FT_STRINGW);
    assert_eq!(text, "Material");
}

#[test]
fn a_package_without_an_asset_has_an_empty_field() {
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

    assert_eq!(get_text(&path, 256).0, FT_FIELDEMPTY);
}

#[test]
fn a_file_that_is_not_a_package_has_an_empty_field() {
    let path = test_file("Text.uasset", b"This is a text file.");

    assert_eq!(get_text(&path, 256).0, FT_FIELDEMPTY);
}

#[test]
fn a_damaged_package_has_an_empty_field() {
    let bytes = fixture_bytes("BP_SummaryActor.uasset");
    // In the summary, in the name map, in the import table, in the export
    // table.
    for length in [4, 100, 1000, 5400, 6000] {
        let path = test_file(&format!("Damaged{length}.uasset"), &bytes[..length]);

        assert_eq!(get_text(&path, 256).0, FT_FIELDEMPTY, "length {length}");
    }
}

#[test]
fn a_missing_file_is_a_file_error() {
    let path = std::env::temp_dir().join("uc-content-tests-no-such-file.uasset");

    assert_eq!(get_text(&path, 256).0, FT_FILEERROR);
}

#[test]
fn a_small_buffer_gets_a_truncated_text_and_a_nul() {
    let path = fixture("BP_SummaryActor.uasset");

    // 9 bytes hold 4 UTF-16 units: 3 characters and the NUL.
    assert_eq!(get_text(&path, 9), (FT_STRINGW, "Blu".to_string()));
    assert_eq!(get_text(&path, 2), (FT_STRINGW, String::new()));
}

#[test]
fn a_buffer_without_space_for_the_nul_gives_an_empty_field() {
    let path = fixture("BP_SummaryActor.uasset");

    assert_eq!(get_text(&path, 1).0, FT_FIELDEMPTY);
    assert_eq!(get_text(&path, 0).0, FT_FIELDEMPTY);
}
