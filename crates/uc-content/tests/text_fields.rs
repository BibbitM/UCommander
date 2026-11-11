//! The text fields of point 4: FFI tests with the built DLL.

mod common;

use std::path::PathBuf;

use common::{
    FT_FIELDEMPTY, FT_FILEERROR, FT_STRINGW, fixture, fixture_bytes, get_text, unsupported_packages,
};

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

const FIELDS: &[Field] = &[
    Field {
        index: 2,
        values: [
            "/Script/Engine.Blueprint",
            "/Script/Engine.World",
            "/Script/Engine.Material",
        ],
        needs_asset: true,
    },
    Field {
        index: 3,
        values: ["BP_SummaryActor", "L_SummaryLevel", "M_Summary"],
        needs_asset: true,
    },
    Field {
        index: 4,
        values: [
            "/Game/Summary/BP_SummaryActor.BP_SummaryActor",
            "/Game/Summary/L_SummaryLevel.L_SummaryLevel",
            "/Game/Summary/M_Summary.M_Summary",
        ],
        needs_asset: true,
    },
    Field {
        index: 5,
        values: [
            "/Game/Summary/BP_SummaryActor",
            "/Game/Summary/L_SummaryLevel",
            "/Game/Summary/M_Summary",
        ],
        needs_asset: false,
    },
    Field {
        index: ENGINE_VERSION,
        values: ["5.8.3"; 3],
        needs_asset: false,
    },
    Field {
        index: FULL_ENGINE_VERSION,
        values: ["5.8.3-58210709+++UE5+Release-5.8"; 3],
        needs_asset: false,
    },
];

const OBJECT_PATH: i32 = 4;
const PACKAGE_PATH: i32 = 5;
const ENGINE_VERSION: i32 = 6;
const FULL_ENGINE_VERSION: i32 = 7;

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
fn an_empty_engine_version_gives_empty_engine_version_fields() {
    let mut bytes = fixture_bytes("BP_SummaryActor.uasset");
    // The saved-by engine version: major 5, minor 8, patch 3 (3 × u16),
    // then the changelist 58210709 (u32). The compatible-with version has
    // another changelist.
    let mut version = [5u16, 8, 3].map(u16::to_le_bytes).concat();
    version.extend_from_slice(&58210709u32.to_le_bytes());
    let start = bytes
        .windows(version.len())
        .position(|window| window == version)
        .unwrap();
    bytes[start..start + 6].fill(0);
    let path = test_file("EmptyVersion.uasset", &bytes);

    for field in FIELDS {
        let expected = if [ENGINE_VERSION, FULL_ENGINE_VERSION].contains(&field.index) {
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
fn a_package_name_that_is_not_valid_gives_no_package_and_object_path() {
    let mut bytes = fixture_bytes("BP_SummaryActor.uasset");
    // The first copy of the name is the package name of the summary. The
    // name map comes after the summary.
    let name = b"/Game/Summary/BP_SummaryActor\0";
    let start = bytes
        .windows(name.len())
        .position(|window| window == name)
        .unwrap();
    bytes[start] = b'X';
    let path = test_file("NotValidName.uasset", &bytes);

    for field in FIELDS {
        let expected = if [OBJECT_PATH, PACKAGE_PATH].contains(&field.index) {
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
fn a_package_that_the_parser_does_not_support_has_empty_fields() {
    for (name, bytes) in unsupported_packages() {
        let path = test_file(name, &bytes);

        for field in FIELDS {
            assert_eq!(
                get_text(&path, field.index, 256).0,
                FT_FIELDEMPTY,
                "field {}, {name}",
                field.index
            );
        }
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
    let path = common::missing_file();

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
