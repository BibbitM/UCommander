//! The field "Asset class": FFI tests with the built DLL.

mod common;

use std::path::{Path, PathBuf};

use common::{FIXTURES, FT_FIELDEMPTY, FT_FILEERROR, FT_STRINGW, expected, fixture, fixture_bytes};

const ASSET_CLASS: i32 = 1;

/// Package flag `ContainsNoAsset`.
const PKG_CONTAINS_NO_ASSET: u32 = 0x0000_0400;

fn test_file(name: &str, bytes: &[u8]) -> PathBuf {
    common::test_file("asset", name, bytes)
}

fn get_text(path: &Path, maxlen: usize) -> (i32, String) {
    common::get_text(path, ASSET_CLASS, maxlen)
}

/// The expected values are the `asset_class` values of the
/// `.expected.json` files of the fixtures.
#[test]
fn the_fixtures_give_the_expected_asset_class() {
    for name in FIXTURES {
        assert_eq!(
            get_text(&fixture(name), 256),
            (FT_STRINGW, expected(name).asset_class),
            "{name}"
        );
    }
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
fn a_package_that_the_parser_does_not_support_has_an_empty_field() {
    let bytes = fixture_bytes("BP_SummaryActor.uasset");
    // Bytes 4..8: legacy file version. Bytes 12..24: UE4, UE5 and licensee
    // versions.
    let mut ue4 = bytes.clone();
    ue4[4..8].copy_from_slice(&(-7i32).to_le_bytes());
    let mut unversioned = bytes;
    unversioned[12..24].fill(0);

    for (name, bytes) in [("Ue4.uasset", ue4), ("Unversioned.uasset", unversioned)] {
        let path = test_file(name, &bytes);

        assert_eq!(get_text(&path, 256).0, FT_FIELDEMPTY, "{name}");
    }
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
    let path = common::missing_file();

    assert_eq!(get_text(&path, 256).0, FT_FILEERROR);
}

#[test]
fn a_small_buffer_gets_a_truncated_text_and_a_nul() {
    let name = "BP_SummaryActor.uasset";
    let path = fixture(name);
    let start: String = expected(name).asset_class.chars().take(3).collect();

    // 9 bytes hold 4 UTF-16 units: 3 characters and the NUL.
    assert_eq!(get_text(&path, 9), (FT_STRINGW, start));
    assert_eq!(get_text(&path, 2), (FT_STRINGW, String::new()));
}

#[test]
fn a_buffer_without_space_for_the_nul_gives_an_empty_field() {
    let path = fixture("BP_SummaryActor.uasset");

    assert_eq!(get_text(&path, 1).0, FT_FIELDEMPTY);
    assert_eq!(get_text(&path, 0).0, FT_FIELDEMPTY);
}
