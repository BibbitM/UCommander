//! OFPA actor data of a package: the external actor in the asset registry
//! data, its actor path, its map path and its descriptor values.

mod common;

use common::Str;
use common::desc::{Desc, FORTNITE_MAIN, FORTNITE_SEASON, UE5_MAIN, UE5_RELEASE, base64};
use common::registry::{Object, SECTION_END, package, section};
use uc_format::Error;
use uc_format::package::{Warning, read_asset_registry, read_header};
use uc_format::world_partition::{OfpaActorRead, read_ofpa_actor};

const ACTOR_PATH: &str = "/Game/Ofpa/L_Map.L_Map:PersistentLevel.PointLight_UAID_1";

/// An asset registry object with the `ActorMetaData` tag.
fn actor_object(path: &'static str, metadata: String) -> Object {
    // `Str` keeps static text. The test leaks the short text.
    let metadata: &'static str = Box::leak(metadata.into_boxed_str());
    Object {
        path: Str::Ansi(path),
        class: Str::Ansi("/Script/Engine.PointLight"),
        tags: vec![
            (
                Str::Ansi("ActorMetaDataClass"),
                Str::Ansi("/Script/Engine.PointLight"),
            ),
            (Str::Ansi("ActorMetaData"), Str::Ansi(metadata)),
        ],
    }
}

fn read(objects: &[Object]) -> Result<OfpaActorRead, Error> {
    let bytes = package(section(Some(SECTION_END), objects)).to_bytes();
    let header = read_header(&bytes).unwrap().header;
    let data = read_asset_registry(&bytes, &header.summary).unwrap();
    read_ofpa_actor(&data)
}

#[test]
fn reads_the_actor_of_an_external_actor_package() {
    let desc = Desc {
        label: Some(Str::Utf16("Lampa żółta")),
        ..Desc::default()
    };

    let read = read(&[actor_object(ACTOR_PATH, base64(&desc.to_bytes()))]).unwrap();

    let actor = read.actor.unwrap();
    assert_eq!(actor.actor_path, ACTOR_PATH);
    assert_eq!(actor.map_path, "/Game/Ofpa/L_Map");
    assert_eq!(actor.desc.native_class_path, "/Script/Engine.PointLight");
    assert_eq!(actor.desc.label.as_deref(), Some("Lampa żółta"));
    assert!(read.warnings.is_empty());
}

#[test]
fn a_package_with_no_actor_metadata_gives_no_actor() {
    let level = Object {
        path: Str::Ansi("L_Map"),
        class: Str::Ansi("/Script/Engine.World"),
        tags: vec![(Str::Ansi("ActorsMetaData"), Str::Ansi("A,B"))],
    };

    for objects in [vec![], vec![level]] {
        let read = read(&objects).unwrap();

        assert_eq!(read.actor, None);
        assert!(read.warnings.is_empty());
    }
}

#[test]
fn a_relative_object_path_gives_no_actor() {
    // A Blueprint package of an actor class (F1) has the tag on an object
    // with a path relative to the package.
    let object = actor_object("BP_Actor", base64(&Desc::default().to_bytes()));

    assert_eq!(read(&[object]).unwrap().actor, None);
}

#[test]
fn a_class_descriptor_gives_no_actor() {
    let desc = Desc {
        is_default: 1,
        ..Desc::default()
    };

    let read = read(&[actor_object(ACTOR_PATH, base64(&desc.to_bytes()))]).unwrap();

    assert_eq!(read.actor, None);
}

#[test]
fn more_than_one_actor_gives_the_first_and_a_warning() {
    let first = Desc {
        label: Some(Str::Ansi("First")),
        ..Desc::default()
    };
    let second = Desc {
        guid: [1, 2, 3, 4],
        label: Some(Str::Ansi("Second")),
        ..Desc::default()
    };
    let objects = [
        actor_object(ACTOR_PATH, base64(&first.to_bytes())),
        actor_object(
            "/Game/Ofpa/L_Map.L_Map:PersistentLevel.Other",
            base64(&second.to_bytes()),
        ),
    ];

    let read = read(&objects).unwrap();

    assert_eq!(read.actor.unwrap().desc.label.as_deref(), Some("First"));
    assert_eq!(read.warnings, [Warning::MoreActors { count: 2 }]);
}

#[test]
fn the_warnings_of_the_descriptor_are_kept() {
    let desc = Desc {
        custom_versions: vec![
            (UE5_MAIN, 123),
            (FORTNITE_SEASON, 16),
            (FORTNITE_MAIN, 268),
            (UE5_RELEASE, 68),
        ],
        ..Desc::default()
    };

    let read = read(&[actor_object(ACTOR_PATH, base64(&desc.to_bytes()))]).unwrap();

    assert!(read.actor.is_some());
    assert_eq!(
        read.warnings,
        [Warning::NewerVersion {
            field: "FortniteSeasonBranchObjectVersion",
            value: 16,
            newest: 15
        }]
    );
}

#[test]
fn damaged_base64_text_gives_an_error() {
    let object = actor_object(ACTOR_PATH, "BAAA*AAA".to_owned());

    assert_eq!(
        read(&[object]),
        Err(Error::BadValue {
            offset: 4,
            field: "Base64 text"
        })
    );
}

#[test]
fn a_truncated_descriptor_gives_an_error() {
    let bytes = Desc::default().to_bytes();
    let object = actor_object(ACTOR_PATH, base64(&bytes[..100]));

    assert!(matches!(read(&[object]), Err(Error::Truncated { .. })));
}

#[test]
fn each_truncation_of_an_external_actor_package_gives_an_error() {
    let metadata = base64(&Desc::default().to_bytes());
    let objects = [actor_object(ACTOR_PATH, metadata)];
    let bytes = package(section(Some(SECTION_END), &objects)).to_bytes();
    let header = read_header(&bytes).unwrap().header;
    let start = header.summary.asset_registry_data_offset as usize;

    // The tables end before the section, so the header stays valid. The
    // asset registry data gives the error.
    for end in start..bytes.len() {
        let result = read_asset_registry(&bytes[..end], &header.summary);

        assert!(
            matches!(result, Err(Error::Truncated { .. })),
            "end {end}: {result:?}"
        );
    }
    let data = read_asset_registry(&bytes, &header.summary).unwrap();
    assert!(read_ofpa_actor(&data).unwrap().actor.is_some());
}
