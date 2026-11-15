//! World Partition actor descriptor: the values up to the actor label.

mod common;

use std::path::PathBuf;

use common::Str;
use common::desc::{Desc, FORTNITE_MAIN, FORTNITE_SEASON, UE5_MAIN, UE5_RELEASE};
use uc_format::package::{Warning, read_asset_registry, read_header};
use uc_format::world_partition::{ActorDescRead, read_actor_desc};
use uc_format::{Error, Guid, base64};

fn read(desc: &Desc) -> Result<ActorDescRead, Error> {
    read_actor_desc(&desc.to_bytes())
}

#[test]
fn reads_a_native_actor() {
    let read = read(&Desc::default()).unwrap();

    let desc = read.desc;
    assert_eq!(
        desc.guid,
        Guid {
            a: 0x6413279F,
            b: 0x42A6D887,
            c: 0xDF5B61B2,
            d: 0x5DEB3B08
        }
    );
    assert_eq!(desc.native_class_path, "/Script/Engine.PointLight");
    assert_eq!(desc.base_class_path, None);
    assert_eq!(desc.label.as_deref(), Some("PointLight"));
    assert!(read.warnings.is_empty());
}

#[test]
fn reads_a_blueprint_actor() {
    let desc = Desc {
        base_class: (
            Str::Ansi("/Game/Summary/BP_SummaryActor"),
            Str::Ansi("BP_SummaryActor_C"),
        ),
        native_class: (Str::Ansi("/Script/Engine"), Str::Ansi("Actor")),
        label: Some(Str::Ansi("Summary Actor")),
        ..Desc::default()
    };

    let desc = read(&desc).unwrap().desc;

    assert_eq!(
        desc.base_class_path.as_deref(),
        Some("/Game/Summary/BP_SummaryActor.BP_SummaryActor_C")
    );
    assert_eq!(desc.native_class_path, "/Script/Engine.Actor");
    assert_eq!(desc.label.as_deref(), Some("Summary Actor"));
}

#[test]
fn reads_the_label_after_the_delta_fields_and_arrays() {
    let desc = Desc {
        is_spatially_loaded: Some(0),
        runtime_grid: Some(Str::Ansi("MainGrid")),
        references: vec![[1, 2, 3, 4], [5, 6, 7, 8]],
        tags: Some(vec![Str::Ansi("Tag_1"), Str::Utf16("Znacznik")]),
        data_layers: Some(vec![Str::Ansi("/Game/DL.DL")]),
        label: Some(Str::Ansi("After")),
        ..Desc::default()
    };

    assert_eq!(read(&desc).unwrap().desc.label.as_deref(), Some("After"));
}

#[test]
fn reads_a_utf16_label() {
    let desc = Desc {
        label: Some(Str::Utf16("Lampa żółta")),
        ..Desc::default()
    };

    assert_eq!(
        read(&desc).unwrap().desc.label.as_deref(),
        Some("Lampa żółta")
    );
}

#[test]
fn a_label_with_flag_0_or_the_text_none_gives_no_label() {
    for label in [None, Some(Str::Ansi("None")), Some(Str::Ansi("none"))] {
        let desc = Desc {
            label,
            ..Desc::default()
        };

        assert_eq!(read(&desc).unwrap().desc.label, None);
    }
}

#[test]
fn a_base_class_with_no_asset_name_gives_the_package_name() {
    let desc = Desc {
        base_class: (Str::Ansi("/Game/X"), Str::Ansi("None")),
        ..Desc::default()
    };

    assert_eq!(
        read(&desc).unwrap().desc.base_class_path.as_deref(),
        Some("/Game/X")
    );
}

#[test]
fn a_newer_custom_version_gives_a_warning() {
    let desc = Desc {
        custom_versions: vec![
            (UE5_MAIN, 124),
            (FORTNITE_SEASON, 15),
            (FORTNITE_MAIN, 268),
            (UE5_RELEASE, 68),
        ],
        ..Desc::default()
    };

    let read = read(&desc).unwrap();

    assert_eq!(read.desc.label.as_deref(), Some("PointLight"));
    assert_eq!(
        read.warnings,
        [Warning::NewerVersion {
            field: "UE5MainStreamObjectVersion",
            value: 124,
            newest: 123
        }]
    );
}

#[test]
fn other_custom_versions_give_no_warning() {
    let mut desc = Desc::default();
    desc.custom_versions.push(([9, 9, 9, 9], 1000));

    assert!(read(&desc).unwrap().warnings.is_empty());
}

#[test]
fn the_ue_5_5_layout_gives_unsupported() {
    let desc = Desc {
        custom_versions: vec![
            (UE5_MAIN, 123),
            (FORTNITE_SEASON, 15),
            (FORTNITE_MAIN, 176),
            (UE5_RELEASE, 68),
        ],
        ..Desc::default()
    };

    assert_eq!(
        read(&desc),
        Err(Error::UnsupportedVersion {
            field: "FortniteMainBranchObjectVersion",
            value: 176
        })
    );
}

#[test]
fn a_missing_custom_version_gives_unsupported() {
    let desc = Desc {
        custom_versions: vec![(UE5_MAIN, 123), (FORTNITE_MAIN, 268), (UE5_RELEASE, 68)],
        ..Desc::default()
    };

    assert_eq!(
        read(&desc),
        Err(Error::UnsupportedVersion {
            field: "FortniteSeasonBranchObjectVersion",
            value: -1
        })
    );
}

#[test]
fn a_class_descriptor_gives_an_error() {
    let desc = Desc {
        is_default: 1,
        ..Desc::default()
    };

    assert_eq!(read(&desc), Err(Error::ClassDescriptor));
}

#[test]
fn the_class_descriptor_of_a_blueprint_fixture_gives_an_error() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/editor/BP_SummaryActor.uasset");
    let bytes = std::fs::read(path).unwrap();
    let header = read_header(&bytes).unwrap().header;
    let data = read_asset_registry(&bytes, &header.summary).unwrap();
    let text = &data.objects[0]
        .tags
        .iter()
        .find(|tag| tag.key == "ActorMetaData")
        .unwrap()
        .value;

    let desc = base64::decode(text).unwrap();

    assert_eq!(read_actor_desc(&desc), Err(Error::ClassDescriptor));
}

#[test]
fn a_truncated_descriptor_gives_an_error() {
    let bytes = Desc::default().to_bytes();
    let rest = Desc::default().rest.len();

    for end in [0, 3, 30, 100, 300, bytes.len() - rest - 1] {
        let result = read_actor_desc(&bytes[..end]);

        assert!(
            matches!(result, Err(Error::Truncated { .. })),
            "end {end}: {result:?}"
        );
    }
}

#[test]
fn a_large_array_count_gives_truncated() {
    let mut bytes = Desc::default().to_bytes();
    // The custom version count.
    bytes[..4].copy_from_slice(&1_000_000i32.to_le_bytes());

    assert!(matches!(
        read_actor_desc(&bytes),
        Err(Error::Truncated { .. })
    ));
}
