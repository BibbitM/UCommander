//! Text form of GUIDs, hashes and engine versions.

use uc_format::package::EngineVersion;
use uc_format::{Guid, IoHash};

#[test]
fn a_guid_is_four_groups_of_8_hex_digits() {
    let guid = Guid {
        a: 0x9C54D522,
        b: 0xA8264FBE,
        c: 0x94210746,
        d: 0x0000_00D0,
    };

    assert_eq!(guid.to_string(), "9C54D522-A8264FBE-94210746-000000D0");
}

#[test]
fn a_hash_is_40_lower_case_hex_digits() {
    let hash = IoHash(std::array::from_fn(|i| (i as u8) * 13));

    assert_eq!(hash.to_string(), "000d1a2734414e5b6875828f9ca9b6c3d0ddeaf7");
}

#[test]
fn an_engine_version_is_major_minor_patch_changelist_and_branch() {
    let version = EngineVersion {
        major: 5,
        minor: 8,
        patch: 1,
        changelist: 4660,
        branch: "++UE5+Release-5.8".to_string(),
    };

    assert_eq!(version.to_string(), "5.8.1-4660+++UE5+Release-5.8");
}

#[test]
fn the_text_of_an_engine_version_has_no_licensee_flag() {
    let version = EngineVersion {
        major: 5,
        minor: 8,
        patch: 0,
        changelist: 0x8000_0000 | 4660,
        branch: "++Custom".to_string(),
    };

    assert_eq!(version.to_string(), "5.8.0-4660+++Custom");
}
