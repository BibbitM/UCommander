//! Package header: the name map.

mod common;

use common::{Name, Package, Str, ue4};
use uc_format::Error;
use uc_format::package::{NameRef, read_header};

#[test]
fn reads_the_names_in_order() {
    let input = Package {
        names: vec![
            Name::ansi("/Script/CoreUObject"),
            Name(Str::Utf16("Żółw"), 0xA1B2, 0xC3D4),
            Name::ansi("None"),
        ],
        ..Package::new(&[])
    };

    let names = read_header(&input.to_bytes()).unwrap().header.names;

    let texts: Vec<_> = names.iter().map(|name| name.text.as_str()).collect();
    assert_eq!(texts, ["/Script/CoreUObject", "Żółw", "None"]);
    assert_eq!(names[0].non_case_preserving_hash, Some(0x1000 + 19));
    assert_eq!(names[0].case_preserving_hash, Some(0x2000 + 19));
    assert_eq!(names[1].non_case_preserving_hash, Some(0xA1B2));
    assert_eq!(names[1].case_preserving_hash, Some(0xC3D4));
}

#[test]
fn names_have_no_hashes_before_name_hashes_serialized() {
    let mut input = Package::new(&["Actor", "None"]);
    input.summary.file_version_ue4 = ue4::NAME_HASHES_SERIALIZED - 1;

    let names = read_header(&input.to_bytes()).unwrap().header.names;

    let texts: Vec<_> = names.iter().map(|name| name.text.as_str()).collect();
    assert_eq!(texts, ["Actor", "None"]);
    assert_eq!(names[1].non_case_preserving_hash, None);
    assert_eq!(names[1].case_preserving_hash, None);
}

#[test]
fn an_empty_name_map_ignores_the_offset() {
    let input = Package {
        name_offset: Some(i32::MAX),
        ..Package::new(&[])
    };

    let names = read_header(&input.to_bytes()).unwrap().header.names;

    assert!(names.is_empty());
}

#[test]
fn name_text_adds_the_number_minus_one() {
    let input = Package::new(&["None", "K2Node_Event"]);
    let header = read_header(&input.to_bytes()).unwrap().header;

    let text = |index, number| header.name_text(NameRef { index, number });

    assert_eq!(text(1, 0).as_deref(), Some("K2Node_Event"));
    assert_eq!(text(1, 1).as_deref(), Some("K2Node_Event_0"));
    assert_eq!(text(1, 11).as_deref(), Some("K2Node_Event_10"));
    assert_eq!(text(0, 0).as_deref(), Some("None"));
    assert_eq!(text(2, 0), None);
    assert_eq!(text(-1, 0), None);
}

#[test]
fn keeps_the_summary_and_its_warnings() {
    let mut input = Package::new(&["None"]);
    input.summary.file_version_ue5 = 1019;

    let read = read_header(&input.to_bytes()).unwrap();

    assert_eq!(read.header.summary.package_name, "/Game/Fixtures/BP_Test");
    assert_eq!(read.summary_size, input.summary_size());
    assert_eq!(read.warnings.len(), 1);
}

#[test]
fn a_name_map_after_the_input_end_is_truncated() {
    let input = Package::new(&["None"]);
    let bytes = input.to_bytes();
    let start = input.summary_size();

    for len in start..bytes.len() {
        let result = read_header(&bytes[..len]);

        assert!(
            matches!(result, Err(Error::Truncated { .. })),
            "length {len}: {result:?}"
        );
    }
}

#[test]
fn a_name_offset_after_the_input_end_is_truncated() {
    let input = Package {
        name_offset: Some(100_000),
        ..Package::new(&["None"])
    };

    let result = read_header(&input.to_bytes());

    assert_eq!(result.unwrap_err(), Error::Truncated { offset: 100_000 });
}

#[test]
fn a_negative_name_count_or_offset_is_a_bad_value() {
    for input in [
        Package {
            name_count: Some(-1),
            ..Package::new(&["None"])
        },
        Package {
            name_offset: Some(-1),
            ..Package::new(&["None"])
        },
    ] {
        let result = read_header(&input.to_bytes());

        assert!(matches!(result, Err(Error::BadValue { .. })), "{result:?}");
    }
}

#[test]
fn a_large_name_count_is_truncated_without_allocation() {
    let input = Package {
        name_count: Some(i32::MAX),
        ..Package::new(&["None"])
    };
    let start = input.summary_size();

    let result = read_header(&input.to_bytes());

    assert_eq!(result.unwrap_err(), Error::Truncated { offset: start });
}

#[test]
fn a_name_longer_than_1024_characters_is_a_bad_value() {
    let long = "N".repeat(1023);
    let longer = "N".repeat(1024);
    let ok = Package::new(&[long.leak()]);
    let bad_ansi = Package::new(&[longer.clone().leak()]);
    let bad_utf16 = Package {
        names: vec![Name(Str::Utf16(longer.leak()), 0, 0)],
        ..Package::new(&[])
    };

    assert!(read_header(&ok.to_bytes()).is_ok());
    for input in [bad_ansi, bad_utf16] {
        let start = input.summary_size();

        let result = read_header(&input.to_bytes());

        assert_eq!(
            result.unwrap_err(),
            Error::BadValue {
                offset: start,
                field: "name"
            }
        );
    }
}

#[test]
fn a_name_ends_at_the_first_null_character() {
    let input = Package {
        names: vec![
            Name::ansi("Foo\0Bar"),
            Name(Str::Utf16("Żół\0w"), 1, 2),
            Name::ansi("None"),
        ],
        ..Package::new(&[])
    };

    let names = read_header(&input.to_bytes()).unwrap().header.names;

    let texts: Vec<_> = names.iter().map(|name| name.text.as_str()).collect();
    assert_eq!(texts, ["Foo", "Żół", "None"]);
}
