//! The text fields of point 4: FFI tests with the built DLL.

mod common;

use std::path::PathBuf;

use common::{FT_FIELDEMPTY, FT_FILEERROR, FT_STRINGW, fixture, fixture_bytes, get_text};

/// A text field and its expected values.
struct Field {
    index: i32,
    /// The values for the fixtures in [`FIXTURES`] order. They are the
    /// values of the `.expected.json` files.
    values: [&'static str; 3],
    /// `true` when a package without an asset has no value.
    needs_asset: bool,
}

const FIXTURES: [&str; 3] = [
    "BP_SummaryActor.uasset",
    "L_SummaryLevel.umap",
    "M_Summary.uasset",
];

const FIELDS: &[Field] = &[Field {
    index: 2,
    values: [
        "/Script/Engine.Blueprint",
        "/Script/Engine.World",
        "/Script/Engine.Material",
    ],
    needs_asset: true,
}];

/// Package flag `ContainsNoAsset`.
const PKG_CONTAINS_NO_ASSET: u32 = 0x0000_0400;

fn test_file(name: &str, bytes: &[u8]) -> PathBuf {
    common::test_file("text", name, bytes)
}

#[test]
fn the_fixtures_give_the_expected_values() {
    for field in FIELDS {
        for (fixture_name, value) in FIXTURES.iter().zip(field.values) {
            assert_eq!(
                get_text(&fixture(fixture_name), field.index, 256),
                (FT_STRINGW, value.to_string()),
                "field {}, {fixture_name}",
                field.index
            );
        }
    }
}

#[test]
fn a_package_without_an_asset_has_only_the_package_values() {
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

    for field in FIELDS {
        let expected = if field.needs_asset {
            FT_FIELDEMPTY
        } else {
            FT_STRINGW
        };

        assert_eq!(
            get_text(&path, field.index, 256).0,
            expected,
            "field {}",
            field.index
        );
    }
}

#[test]
fn a_file_that_is_not_a_package_has_empty_fields() {
    let path = test_file("Text.uasset", b"This is a text file.");

    for field in FIELDS {
        assert_eq!(get_text(&path, field.index, 256).0, FT_FIELDEMPTY);
    }
}

#[test]
fn a_damaged_package_has_empty_fields() {
    let bytes = fixture_bytes("BP_SummaryActor.uasset");
    // In the summary, in the name map, in the import table, in the export
    // table.
    for length in [4, 100, 1000, 5400, 6000] {
        let path = test_file(&format!("Damaged{length}.uasset"), &bytes[..length]);

        for field in FIELDS {
            assert_eq!(
                get_text(&path, field.index, 256).0,
                FT_FIELDEMPTY,
                "field {}, length {length}",
                field.index
            );
        }
    }
}

#[test]
fn a_missing_file_is_a_file_error() {
    let path = std::env::temp_dir().join("uc-content-text-tests-no-such-file.uasset");

    for field in FIELDS {
        assert_eq!(get_text(&path, field.index, 256).0, FT_FILEERROR);
    }
}

#[test]
fn a_small_buffer_gets_a_truncated_text_and_a_nul() {
    let path = fixture(FIXTURES[0]);

    for field in FIELDS {
        // 9 bytes hold 4 UTF-16 units: 3 characters and the NUL.
        let start: String = field.values[0].chars().take(3).collect();

        assert_eq!(get_text(&path, field.index, 9), (FT_STRINGW, start));
        assert_eq!(get_text(&path, field.index, 1).0, FT_FIELDEMPTY);
    }
}
