//! Package summary: GUIDs, generations, engine versions, compression data
//! and the offsets at the end of the summary.

mod common;

use common::{PKG_FILTER_EDITOR_ONLY, Summary, ue4, ue5};
use uc_format::package::{EngineVersion, Generation, Table, read_summary};
use uc_format::{Error, Guid};

fn guid(value: [u32; 4]) -> Guid {
    let [a, b, c, d] = value;
    Guid { a, b, c, d }
}

#[test]
fn reads_the_end_of_the_summary() {
    let bytes = Summary::default().to_bytes();

    let read = read_summary(&bytes).unwrap();

    let s = &read.summary;
    assert_eq!(s.package_guid, None);
    assert_eq!(s.persistent_guid, Some(guid([0xB1, 0xB2, 0xB3, 0xB4])));
    assert_eq!(s.owner_persistent_guid, None);
    assert_eq!(
        s.generations,
        vec![
            Generation {
                export_count: 201,
                name_count: 202
            },
            Generation {
                export_count: 203,
                name_count: 204
            }
        ]
    );
    assert_eq!(s.compression_flags, 0);
    assert_eq!(s.package_source, 0xDEAD_BEEF);
    assert_eq!(s.additional_packages_to_cook, vec!["/Game/A", "/Game/B"]);
    assert_eq!(s.asset_registry_data_offset, 206);
    assert_eq!(s.bulk_data_start_offset, 0x1_0000_0207);
    assert_eq!(s.world_tile_info_data_offset, Some(208));
    assert_eq!(s.chunk_ids, vec![3, 4]);
    assert_eq!(
        s.preload_dependencies,
        Some(Table {
            count: 209,
            offset: 210
        })
    );
    assert_eq!(s.names_referenced_from_export_data_count, Some(211));
    assert_eq!(s.payload_toc_offset, Some(0x2_0000_0212));
    assert_eq!(s.data_resource_offset, Some(213));
    assert_eq!(read.size, bytes.len());
}

#[test]
fn reads_the_engine_versions() {
    let bytes = Summary::default().to_bytes();

    let s = read_summary(&bytes).unwrap().summary;

    let saved = s.saved_by_engine_version.unwrap();
    assert_eq!(
        (saved.major, saved.minor, saved.patch),
        (5, 8, 1),
        "saved by"
    );
    assert_eq!(saved.changelist, 0x8000_1234);
    assert_eq!(saved.branch, "++UE5+Release-5.8");
    assert!(saved.is_licensee());
    assert_eq!(saved.changelist_number(), 0x1234);
    assert_eq!(s.engine_changelist, None);

    let compatible: EngineVersion = s.compatible_with_engine_version.unwrap();
    assert_eq!(
        (compatible.major, compatible.minor, compatible.patch),
        (5, 8, 0)
    );
    assert!(!compatible.is_licensee());
    assert_eq!(compatible.changelist_number(), 0x5678);
}

#[test]
fn an_older_ue5_version_has_the_package_guid_and_fewer_offsets() {
    let input = Summary {
        file_version_ue5: ue5::INITIAL_VERSION,
        ..Summary::default()
    };

    let s = read_summary(&input.to_bytes()).unwrap().summary;

    assert_eq!(s.package_guid, Some(guid([0xA1, 0xA2, 0xA3, 0xA4])));
    assert_eq!(s.names_referenced_from_export_data_count, None);
    assert_eq!(s.payload_toc_offset, None);
    assert_eq!(s.data_resource_offset, None);
    assert_eq!(s.asset_registry_data_offset, 206);
}

#[test]
fn each_ue5_offset_starts_at_its_version() {
    for version in [
        ue5::NAMES_REFERENCED_FROM_EXPORT_DATA,
        ue5::PAYLOAD_TOC,
        ue5::DATA_RESOURCES,
    ] {
        let input = Summary {
            file_version_ue5: version,
            ..Summary::default()
        };

        let s = read_summary(&input.to_bytes()).unwrap().summary;

        assert_eq!(s.names_referenced_from_export_data_count, Some(211));
        assert_eq!(s.payload_toc_offset.is_some(), version >= 1002);
        assert_eq!(s.data_resource_offset.is_some(), version >= 1009);
    }
}

#[test]
fn the_owner_persistent_guid_is_only_in_two_ue4_versions() {
    let input = Summary {
        file_version_ue4: ue4::ADDED_PACKAGE_OWNER,
        ..Summary::default()
    };

    let s = read_summary(&input.to_bytes()).unwrap().summary;

    assert_eq!(s.persistent_guid, Some(guid([0xB1, 0xB2, 0xB3, 0xB4])));
    assert_eq!(
        s.owner_persistent_guid,
        Some(guid([0xC1, 0xC2, 0xC3, 0xC4]))
    );
    assert_eq!(s.chunk_ids, vec![3, 4]);
}

#[test]
fn the_filter_editor_only_flag_removes_the_persistent_guid() {
    let input = Summary {
        package_flags: PKG_FILTER_EDITOR_ONLY,
        ..Summary::default()
    };

    let s = read_summary(&input.to_bytes()).unwrap().summary;

    assert_eq!(s.persistent_guid, None);
    assert_eq!(s.generations.len(), 2);
}

#[test]
fn an_old_ue4_version_has_one_chunk_id_and_an_engine_changelist() {
    let input = Summary {
        file_version_ue4: ue4::ADDED_CHUNKID_TO_ASSETDATA_AND_UPACKAGE,
        ..Summary::default()
    };

    let s = read_summary(&input.to_bytes()).unwrap().summary;

    assert_eq!(s.chunk_ids, vec![3]);
    assert_eq!(s.engine_changelist, Some(205));
    assert_eq!(s.saved_by_engine_version, None);
    assert_eq!(s.compatible_with_engine_version, None);
    assert_eq!(s.preload_dependencies, None);
    assert_eq!(s.world_tile_info_data_offset, Some(208));
}

#[test]
fn a_negative_single_chunk_id_gives_no_chunk_ids() {
    let input = Summary {
        file_version_ue4: ue4::ADDED_CHUNKID_TO_ASSETDATA_AND_UPACKAGE,
        chunk_ids: vec![],
        ..Summary::default()
    };

    let s = read_summary(&input.to_bytes()).unwrap().summary;

    assert!(s.chunk_ids.is_empty());
}

#[test]
fn the_oldest_ue4_version_has_no_chunk_id_and_no_world_tile_info() {
    let input = Summary {
        file_version_ue4: ue4::WORLD_LEVEL_INFO - 1,
        ..Summary::default()
    };

    let s = read_summary(&input.to_bytes()).unwrap().summary;

    assert!(s.chunk_ids.is_empty());
    assert_eq!(s.world_tile_info_data_offset, None);
    assert_eq!(s.bulk_data_start_offset, 0x1_0000_0207);
}

#[test]
fn compressed_chunks_give_an_error() {
    let input = Summary {
        compressed_chunk_count: 1,
        ..Summary::default()
    };

    assert_eq!(
        read_summary(&input.to_bytes()).unwrap_err(),
        Error::Compressed
    );
}

#[test]
fn a_negative_compressed_chunk_count_gives_an_error() {
    let input = Summary {
        compressed_chunk_count: -1,
        ..Summary::default()
    };

    assert!(matches!(
        read_summary(&input.to_bytes()).unwrap_err(),
        Error::BadValue {
            field: "compressed chunk count",
            ..
        }
    ));
}
