//! Locate OFPA actor metadata in editor package bytes.

mod common;

use common::{Str, Summary};
use uc_format::Error;
use uc_format::package::{read_actor_metadata, read_header};

fn string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(i32::try_from(value.len()).unwrap() + 1).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
    bytes.push(0);
}

fn external_actor_package() -> Vec<u8> {
    let mut records = Vec::new();
    records.extend_from_slice(&1_i32.to_le_bytes());
    string(&mut records, "/Game/Map.Map:PersistentLevel.PointLight_1");
    string(&mut records, "/Script/Engine.PointLight");
    records.extend_from_slice(&2_i32.to_le_bytes());
    string(&mut records, "ActorMetaDataClass");
    string(&mut records, "/Script/Engine.PointLight");
    string(&mut records, "ActorMetaData");
    string(&mut records, "AQID");

    let mut summary = Summary {
        package_name: Str::Ansi("/Game/__ExternalActors__/Map/A/BC/DEF"),
        ..Summary::default()
    };
    let start = summary.to_bytes().len();
    let end = start + 8 + records.len();
    summary.asset_registry_data_offset = i32::try_from(start).unwrap();
    summary.total_header_size = i32::try_from(end + 8).unwrap();
    let mut bytes = summary.to_bytes();
    bytes.extend_from_slice(&i64::try_from(end).unwrap().to_le_bytes());
    bytes.extend_from_slice(&records);
    bytes
}

#[test]
fn reads_actor_metadata_without_dependency_data() {
    let actor = read_actor_metadata(&external_actor_package())
        .unwrap()
        .unwrap();
    assert_eq!(
        actor.object_path,
        "/Game/Map.Map:PersistentLevel.PointLight_1"
    );
    assert_eq!(actor.class_path, "/Script/Engine.PointLight");
    assert_eq!(actor.native_class_path, "/Script/Engine.PointLight");
    assert_eq!(actor.descriptor_base64, "AQID");
}

#[test]
fn the_blueprint_fixture_has_no_external_actor() {
    let bytes = include_bytes!("../../../tests/fixtures/editor/BP_SummaryActor.uasset");
    assert_eq!(read_actor_metadata(bytes).unwrap(), None);
}

#[test]
fn reads_the_direct_parent_of_the_blueprint_generated_class() {
    let bytes = include_bytes!("../../../tests/fixtures/editor/BP_SummaryActor.uasset");
    let header = read_header(bytes).unwrap().header;
    assert_eq!(
        header
            .blueprint_parent_class("/Game/Summary/BP_SummaryActor.BP_SummaryActor_C")
            .as_deref(),
        Some("/Script/Engine.Actor"),
    );
    assert_eq!(
        header.blueprint_parent_class("/Game/Other/BP_SummaryActor.BP_SummaryActor_C"),
        None,
    );
}

#[test]
fn each_truncation_of_the_actor_package_gives_an_error() {
    let bytes = external_actor_package();
    for end in 0..bytes.len() {
        assert!(
            matches!(
                read_actor_metadata(&bytes[..end]),
                Err(Error::Truncated { .. })
            ),
            "length {end}",
        );
    }
}
