//! Package summary: package name, flags, and table counts and offsets.

mod common;

use common::{PKG_FILTER_EDITOR_ONLY, Str, Summary, ue4, ue5};
use uc_format::Error;
use uc_format::package::{Table, read_summary};

#[test]
fn reads_the_package_name_and_the_flags() {
    let bytes = Summary::default().to_bytes();

    let summary = read_summary(&bytes).unwrap().summary;

    assert_eq!(summary.package_name, "/Game/Fixtures/BP_Test");
    assert_eq!(summary.package_flags, 0x0000_0400);
}

#[test]
fn reads_a_utf16_package_name() {
    let input = Summary {
        package_name: Str::Utf16("/Game/Żółw_ä"),
        ..Summary::default()
    };

    let summary = read_summary(&input.to_bytes()).unwrap().summary;

    assert_eq!(summary.package_name, "/Game/Żółw_ä");
}

#[test]
fn a_string_of_length_0_or_1_is_empty() {
    for length in [0, 1] {
        let mut bytes = Summary {
            package_name: Str::Length(length),
            ..Summary::default()
        }
        .to_bytes();
        // Length 1 has one character: the null terminator. Put it after
        // the length.
        if length == 1 {
            let at = name_offset() + 4;
            bytes.insert(at, 0);
        }

        let summary = read_summary(&bytes).unwrap().summary;

        assert_eq!(summary.package_name, "", "length {length}");
    }
}

#[test]
fn reads_8_bit_characters_as_latin_1() {
    let mut bytes = Summary {
        package_name: Str::Ansi("/Game/X"),
        ..Summary::default()
    }
    .to_bytes();
    let at = name_offset() + 4 + "/Game/".len();
    bytes[at] = 0xE9;

    let summary = read_summary(&bytes).unwrap().summary;

    assert_eq!(summary.package_name, "/Game/é");
}

#[test]
fn reads_the_table_counts_and_offsets() {
    let bytes = Summary::default().to_bytes();

    let s = read_summary(&bytes).unwrap().summary;

    assert_eq!(s.names, table(101, 102));
    assert_eq!(s.soft_object_paths, Some(table(103, 104)));
    assert_eq!(s.localization_id.as_deref(), Some("0123456789ABCDEF"));
    assert_eq!(s.gatherable_text_data, Some(table(105, 106)));
    assert_eq!(s.exports, table(107, 108));
    assert_eq!(s.imports, table(109, 110));
    assert_eq!(s.cell_exports, Some(table(111, 112)));
    assert_eq!(s.cell_imports, Some(table(113, 114)));
    assert_eq!(s.metadata_offset, Some(115));
    assert_eq!(s.depends_offset, 116);
    assert_eq!(s.soft_package_references, Some(table(117, 118)));
    assert_eq!(s.searchable_names_offset, Some(119));
    assert_eq!(s.thumbnail_table_offset, 120);
    assert_eq!(s.import_type_hierarchies, Some(table(121, 122)));
}

#[test]
fn an_older_ue5_version_has_fewer_tables() {
    let input = Summary {
        file_version_ue5: ue5::ADD_SOFTOBJECTPATH_LIST - 1,
        ..Summary::default()
    };

    let s = read_summary(&input.to_bytes()).unwrap().summary;

    assert_eq!(s.soft_object_paths, None);
    assert_eq!(s.cell_exports, None);
    assert_eq!(s.cell_imports, None);
    assert_eq!(s.metadata_offset, None);
    assert_eq!(s.import_type_hierarchies, None);
    assert_eq!(s.exports, table(107, 108));
    assert_eq!(s.thumbnail_table_offset, 120);
}

#[test]
fn each_ue5_table_starts_at_its_version() {
    let cases = [
        ue5::ADD_SOFTOBJECTPATH_LIST,
        ue5::METADATA_SERIALIZATION_OFFSET,
        ue5::VERSE_CELLS,
        ue5::IMPORT_TYPE_HIERARCHIES,
    ];
    for version in cases {
        let input = Summary {
            file_version_ue5: version,
            ..Summary::default()
        };

        let s = read_summary(&input.to_bytes()).unwrap().summary;

        assert_eq!(s.soft_object_paths.is_some(), version >= 1008);
        assert_eq!(s.metadata_offset.is_some(), version >= 1014);
        assert_eq!(s.cell_exports.is_some(), version >= 1015);
        assert_eq!(s.import_type_hierarchies.is_some(), version >= 1018);
        assert_eq!(s.thumbnail_table_offset, 120, "version {version}");
    }
}

#[test]
fn an_older_ue4_version_has_fewer_tables() {
    let input = Summary {
        file_version_ue4: ue4::ADD_STRING_ASSET_REFERENCES_MAP - 1,
        ..Summary::default()
    };

    let s = read_summary(&input.to_bytes()).unwrap().summary;

    assert_eq!(s.localization_id, None);
    assert_eq!(s.gatherable_text_data, None);
    assert_eq!(s.soft_package_references, None);
    assert_eq!(s.searchable_names_offset, None);
    assert_eq!(s.depends_offset, 116);
    assert_eq!(s.thumbnail_table_offset, 120);
}

#[test]
fn the_filter_editor_only_flag_removes_the_localization_id() {
    let input = Summary {
        package_flags: PKG_FILTER_EDITOR_ONLY,
        ..Summary::default()
    };

    let s = read_summary(&input.to_bytes()).unwrap().summary;

    assert_eq!(s.localization_id, None);
    assert_eq!(s.gatherable_text_data, Some(table(105, 106)));
}

#[test]
fn the_smallest_string_length_gives_an_error() {
    let input = Summary {
        package_name: Str::Length(i32::MIN),
        ..Summary::default()
    };
    let bytes = input.to_bytes();

    assert_eq!(
        read_summary(&bytes).unwrap_err(),
        Error::BadValue {
            offset: name_offset(),
            field: "package name"
        }
    );
}

#[test]
fn a_very_long_string_gives_a_truncated_error() {
    for length in [i32::MAX, -i32::MAX] {
        let input = Summary {
            package_name: Str::Length(length),
            ..Summary::default()
        };

        assert!(
            matches!(
                read_summary(&input.to_bytes()).unwrap_err(),
                Error::Truncated { .. }
            ),
            "length {length}"
        );
    }
}

/// The offset of the package name length in a default summary.
fn name_offset() -> usize {
    let with_name = Summary::default().to_bytes();
    let without = Summary {
        package_name: Str::Length(0),
        ..Summary::default()
    }
    .to_bytes();
    // The two summaries are equal up to the length of the package name.
    with_name
        .iter()
        .zip(&without)
        .position(|(a, b)| a != b)
        .unwrap()
}

fn table(count: i32, offset: i32) -> Table {
    Table { count, offset }
}
