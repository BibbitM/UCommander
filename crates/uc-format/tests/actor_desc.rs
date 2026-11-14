//! World Partition actor descriptor: the values up to the actor label.

mod common;

use std::path::PathBuf;

use common::{Str, Writer};
use uc_format::package::{Warning, read_asset_registry, read_header};
use uc_format::world_partition::{ActorDescRead, read_actor_desc};
use uc_format::{Error, Guid, base64};

const UE5_MAIN: [u32; 4] = [0x697DD581, 0xE64F41AB, 0xAA4A51EC, 0xBEB7B628];
const UE5_RELEASE: [u32; 4] = [0xD89B5E42, 0x24BD4D46, 0x8412ACA8, 0xDF641779];
const FORTNITE_MAIN: [u32; 4] = [0x601D1886, 0xAC644F84, 0xAA16D3DE, 0x0DEAC7D6];
const FORTNITE_SEASON: [u32; 4] = [0x5B4C06B7, 0x24634AF8, 0x805BBF70, 0xCDF5D0DD];
const ACTOR_GUID: [u32; 4] = [0x6413279F, 0x42A6D887, 0xDF5B61B2, 0x5DEB3B08];

/// A delta field: `None` writes flag 0, `Some` writes flag 1 and the value.
type Delta<T> = Option<T>;

/// A synthetic descriptor with the UE 5.8.3 layout up to the label.
struct Desc {
    custom_versions: Vec<([u32; 4], i32)>,
    is_default: u32,
    base_class: (Str, Str),
    native_class: (Str, Str),
    is_spatially_loaded: Delta<u32>,
    runtime_grid: Delta<Str>,
    references: Vec<[u32; 4]>,
    tags: Delta<Vec<Str>>,
    data_layers: Delta<Vec<Str>>,
    label: Delta<Str>,
    /// Bytes after the label, which the parser does not read.
    rest: Vec<u8>,
}

impl Default for Desc {
    /// The F2 descriptor of `PointLight_1`: the newest UE 5.8.3 versions,
    /// a native class and a label.
    fn default() -> Self {
        Self {
            custom_versions: vec![
                (UE5_MAIN, 123),
                (FORTNITE_SEASON, 15),
                (FORTNITE_MAIN, 268),
                (UE5_RELEASE, 68),
            ],
            is_default: 0,
            base_class: (Str::Ansi("None"), Str::Ansi("None")),
            native_class: (Str::Ansi("/Script/Engine"), Str::Ansi("PointLight")),
            is_spatially_loaded: None,
            runtime_grid: None,
            references: Vec::new(),
            tags: None,
            data_layers: None,
            label: Some(Str::Ansi("PointLight")),
            rest: vec![0; 40],
        }
    }
}

impl Desc {
    fn to_bytes(&self) -> Vec<u8> {
        let mut w = Writer::default();
        w.i32(self.custom_versions.len() as i32);
        for (guid, version) in &self.custom_versions {
            w.guid(*guid);
            w.i32(*version);
        }
        w.u32(self.is_default);
        w.string(&self.base_class.0);
        w.string(&self.base_class.1);
        w.string(&self.native_class.0);
        w.string(&self.native_class.1);
        w.guid(ACTOR_GUID);
        // Transform: identity rotation, no translation, scale 1.
        for value in [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0] {
            w.bytes(&f64::to_le_bytes(value));
        }
        // Runtime and editor bounds, relative.
        for _ in 0..2 {
            for value in [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1000.0, 1000.0, 1000.0] {
                w.bytes(&f64::to_le_bytes(value));
            }
            w.u32(1);
        }
        delta(&mut w, &self.is_spatially_loaded, |w, value| w.u32(*value));
        delta(&mut w, &self.runtime_grid, Writer::string);
        delta(&mut w, &None::<u32>, |w, value| w.u32(*value));
        delta(&mut w, &None::<u32>, |w, value| w.u32(*value));
        for references in [&self.references, &Vec::new()] {
            w.i32(references.len() as i32);
            for guid in references {
                w.guid(*guid);
            }
        }
        delta(&mut w, &self.tags, names);
        delta(&mut w, &self.data_layers, names);
        delta(&mut w, &None::<u32>, |w, value| w.u32(*value));
        delta(&mut w, &self.label, Writer::string);
        w.bytes(&self.rest);
        w.0
    }
}

fn delta<T>(w: &mut Writer, value: &Delta<T>, write: impl Fn(&mut Writer, &T)) {
    match value {
        None => w.bytes(&[0]),
        Some(value) => {
            w.bytes(&[1]);
            write(w, value);
        }
    }
}

fn names(w: &mut Writer, names: &Vec<Str>) {
    w.i32(names.len() as i32);
    for name in names {
        w.string(name);
    }
}

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
