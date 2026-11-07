//! Package summary: tag, file versions and custom versions.

mod common;

use common::{Summary, TAG_SWAPPED};
use uc_format::package::{Warning, read_summary};
use uc_format::{Error, Guid, IoHash};

#[test]
fn reads_the_file_versions() {
    let bytes = Summary::default().to_bytes();

    let read = read_summary(&bytes).unwrap();

    let summary = &read.summary;
    assert_eq!(summary.legacy_file_version, -9);
    assert_eq!(summary.legacy_ue3_version, 864);
    assert_eq!(summary.file_version_ue4, 522);
    assert_eq!(summary.file_version_ue5, 1018);
    assert_eq!(summary.file_version_licensee_ue, 0);
    assert!(read.warnings.is_empty());
}

#[test]
fn reads_the_licensee_version() {
    let input = Summary {
        file_version_licensee_ue: 17,
        ..Summary::default()
    };

    let read = read_summary(&input.to_bytes()).unwrap();

    assert_eq!(read.summary.file_version_licensee_ue, 17);
}

#[test]
fn reads_the_saved_hash_and_the_total_header_size() {
    let bytes = Summary::default().to_bytes();

    let summary = read_summary(&bytes).unwrap().summary;

    let expected: [u8; 20] = std::array::from_fn(|i| i as u8 + 1);
    assert_eq!(summary.saved_hash, Some(IoHash(expected)));
    assert_eq!(summary.total_header_size, 0x1234);
}

#[test]
fn an_older_ue5_version_has_no_saved_hash_and_the_old_total_header_size_position() {
    let input = Summary {
        file_version_ue5: 1015,
        ..Summary::default()
    };

    let summary = read_summary(&input.to_bytes()).unwrap().summary;

    assert_eq!(summary.saved_hash, None);
    assert_eq!(summary.total_header_size, 0x1234);
    assert_eq!(summary.custom_versions.len(), 2);
}

#[test]
fn reads_the_custom_versions() {
    let bytes = Summary::default().to_bytes();

    let summary = read_summary(&bytes).unwrap().summary;

    assert_eq!(summary.custom_versions.len(), 2);
    let first = &summary.custom_versions[0];
    assert_eq!(
        first.guid,
        Guid {
            a: 0x9C54D522,
            b: 0xA8264FBE,
            c: 0x94210746,
            d: 0x61B482D0
        }
    );
    assert_eq!(first.version, 52);
    assert_eq!(summary.custom_versions[1].version, 7);
}

#[test]
fn the_summary_size_is_the_number_of_bytes_read() {
    let bytes = Summary::default().to_bytes();

    let read = read_summary(&bytes).unwrap();

    assert_eq!(read.size, bytes.len());
}

#[test]
fn bytes_after_the_summary_are_not_read() {
    let mut bytes = Summary::default().to_bytes();
    let size = bytes.len();
    bytes.extend_from_slice(&[0xAB; 100]);

    let read = read_summary(&bytes).unwrap();

    assert_eq!(read.size, size);
}

#[test]
fn legacy_file_version_minus_8_is_read() {
    let input = Summary {
        legacy_file_version: -8,
        file_version_ue5: 1015,
        ..Summary::default()
    };

    let summary = read_summary(&input.to_bytes()).unwrap().summary;

    assert_eq!(summary.legacy_file_version, -8);
    assert_eq!(summary.file_version_ue5, 1015);
}

#[test]
fn other_bytes_are_not_a_package() {
    let bytes = b"This is a text file, not a package.";

    assert_eq!(read_summary(bytes).unwrap_err(), Error::NotAPackage);
}

#[test]
fn the_byte_swapped_tag_gives_an_error() {
    let input = Summary {
        tag: TAG_SWAPPED,
        ..Summary::default()
    };

    assert_eq!(
        read_summary(&input.to_bytes()).unwrap_err(),
        Error::ByteSwapped
    );
}

#[test]
fn a_ue4_legacy_file_version_gives_an_error() {
    let input = Summary {
        legacy_file_version: -7,
        ..Summary::default()
    };

    assert_eq!(
        read_summary(&input.to_bytes()).unwrap_err(),
        Error::UnsupportedVersion {
            field: "legacy file version",
            value: -7
        }
    );
}

#[test]
fn a_ue3_legacy_file_version_gives_an_error() {
    let input = Summary {
        legacy_file_version: 864,
        ..Summary::default()
    };

    assert_eq!(
        read_summary(&input.to_bytes()).unwrap_err(),
        Error::UnsupportedVersion {
            field: "legacy file version",
            value: 864
        }
    );
}

#[test]
fn a_newer_legacy_file_version_gives_an_error() {
    let input = Summary {
        legacy_file_version: -10,
        ..Summary::default()
    };

    assert_eq!(
        read_summary(&input.to_bytes()).unwrap_err(),
        Error::UnsupportedVersion {
            field: "legacy file version",
            value: -10
        }
    );
}

#[test]
fn a_ue5_version_before_ue5_gives_an_error() {
    let input = Summary {
        file_version_ue5: 999,
        ..Summary::default()
    };

    assert_eq!(
        read_summary(&input.to_bytes()).unwrap_err(),
        Error::UnsupportedVersion {
            field: "file version UE5",
            value: 999
        }
    );
}

#[test]
fn a_ue4_version_older_than_the_oldest_loadable_gives_an_error() {
    let input = Summary {
        file_version_ue4: 213,
        ..Summary::default()
    };

    assert_eq!(
        read_summary(&input.to_bytes()).unwrap_err(),
        Error::UnsupportedVersion {
            field: "file version UE4",
            value: 213
        }
    );
}

#[test]
fn an_unversioned_package_gives_an_error() {
    let input = Summary {
        file_version_ue4: 0,
        file_version_ue5: 0,
        file_version_licensee_ue: 0,
        ..Summary::default()
    };

    assert_eq!(
        read_summary(&input.to_bytes()).unwrap_err(),
        Error::Unversioned
    );
}

#[test]
fn a_newer_ue5_version_gives_a_warning() {
    let input = Summary {
        file_version_ue5: 1019,
        ..Summary::default()
    };

    let read = read_summary(&input.to_bytes()).unwrap();

    assert_eq!(read.summary.file_version_ue5, 1019);
    assert_eq!(
        read.warnings,
        vec![Warning::NewerVersion {
            field: "file version UE5",
            value: 1019,
            newest: 1018
        }]
    );
}

#[test]
fn a_newer_ue4_version_gives_a_warning() {
    let input = Summary {
        file_version_ue4: 523,
        ..Summary::default()
    };

    let read = read_summary(&input.to_bytes()).unwrap();

    assert_eq!(
        read.warnings,
        vec![Warning::NewerVersion {
            field: "file version UE4",
            value: 523,
            newest: 522
        }]
    );
}

#[test]
fn each_truncation_gives_a_truncated_error() {
    let bytes = Summary::default().to_bytes();

    for len in 0..bytes.len() {
        let error = read_summary(&bytes[..len]).unwrap_err();
        match error {
            Error::Truncated { offset } => assert!(offset <= len, "length {len}"),
            other => panic!("length {len}: {other:?}"),
        }
    }
}

#[test]
fn a_truncated_error_gives_the_offset_of_the_missing_value() {
    let bytes = Summary::default().to_bytes();

    // Tag and legacy file version are present. The legacy UE3 version is not.
    assert_eq!(
        read_summary(&bytes[..9]).unwrap_err(),
        Error::Truncated { offset: 8 }
    );
}

#[test]
fn a_very_large_custom_version_count_gives_an_error_without_allocation() {
    let input = Summary {
        custom_version_count: Some(i32::MAX),
        ..Summary::default()
    };

    assert!(matches!(
        read_summary(&input.to_bytes()).unwrap_err(),
        Error::Truncated { .. }
    ));
}

#[test]
fn a_negative_custom_version_count_gives_an_error() {
    let input = Summary {
        custom_version_count: Some(-1),
        ..Summary::default()
    };

    assert!(matches!(
        read_summary(&input.to_bytes()).unwrap_err(),
        Error::BadValue {
            field: "custom version count",
            ..
        }
    ));
}
