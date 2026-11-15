//! Package asset registry data: the objects and their tags.

mod common;

use common::registry::{Object, SECTION_END, package, section};
use common::{PKG_FILTER_EDITOR_ONLY, Package, Str, Writer};
use uc_format::Error;
use uc_format::package::{AssetRegistryData, read_asset_registry, read_header};

fn read(package: &Package) -> Result<AssetRegistryData, Error> {
    let bytes = package.to_bytes();
    let header = read_header(&bytes).unwrap().header;
    read_asset_registry(&bytes, &header.summary)
}

fn level_object() -> Object {
    Object {
        path: Str::Ansi("L_Level"),
        class: Str::Ansi("/Script/Engine.World"),
        tags: vec![
            (Str::Ansi("ActorsMetaData"), Str::Ansi("A,B")),
            (Str::Ansi("DateModified"), Str::Ansi("2026.11.06")),
        ],
    }
}

#[test]
fn reads_the_objects_and_their_tags_in_file_order() {
    let objects = [
        level_object(),
        Object {
            path: Str::Ansi("/Game/Map.Map:PersistentLevel.Actor_1"),
            class: Str::Ansi("/Script/Engine.PointLight"),
            tags: vec![
                (Str::Ansi("Z"), Str::Ansi("1")),
                (Str::Ansi("A"), Str::Ansi("2")),
            ],
        },
    ];

    let bytes = package(section(Some(SECTION_END), &objects)).to_bytes();
    let header = read_header(&bytes).unwrap().header;

    let data = read_asset_registry(&bytes, &header.summary).unwrap();

    assert_eq!(data.dependency_data_offset, Some(bytes.len() as i64));
    assert_eq!(data.objects.len(), 2);
    let level = &data.objects[0];
    assert_eq!(level.object_path, "L_Level");
    assert_eq!(level.class_name, "/Script/Engine.World");
    let tags: Vec<_> = level
        .tags
        .iter()
        .map(|tag| (tag.key.as_str(), tag.value.as_str()))
        .collect();
    assert_eq!(
        tags,
        [("ActorsMetaData", "A,B"), ("DateModified", "2026.11.06")]
    );
    let actor = &data.objects[1];
    assert_eq!(actor.object_path, "/Game/Map.Map:PersistentLevel.Actor_1");
    let keys: Vec<_> = actor.tags.iter().map(|tag| tag.key.as_str()).collect();
    assert_eq!(keys, ["Z", "A"]);
}

#[test]
fn no_objects_give_an_empty_list() {
    let data = read(&package(section(Some(SECTION_END), &[]))).unwrap();

    assert!(data.objects.is_empty());
}

#[test]
fn reads_utf16_text_and_keeps_empty_tags() {
    let object = Object {
        path: Str::Utf16("Lampa_żółta"),
        class: Str::Ansi("/Script/Engine.PointLight"),
        tags: vec![
            (Str::Utf16("Klucz"), Str::Utf16("żółw")),
            (Str::Ansi(""), Str::Ansi("")),
        ],
    };

    let data = read(&package(section(Some(SECTION_END), &[object]))).unwrap();

    let object = &data.objects[0];
    assert_eq!(object.object_path, "Lampa_żółta");
    assert_eq!(object.tags[0].key, "Klucz");
    assert_eq!(object.tags[0].value, "żółw");
    assert_eq!(object.tags[1].key, "");
    assert_eq!(object.tags[1].value, "");
}

#[test]
fn a_cooked_package_has_no_dependency_data_offset() {
    let mut input = package(section(None, &[level_object()]));
    input.summary.package_flags |= PKG_FILTER_EDITOR_ONLY;

    let data = read(&input).unwrap();

    assert_eq!(data.dependency_data_offset, None);
    assert_eq!(data.objects[0].object_path, "L_Level");
}

#[test]
fn a_package_before_dependency_flags_has_no_dependency_data_offset() {
    let mut input = package(section(None, &[level_object()]));
    input.summary.file_version_ue4 = 520;

    let data = read(&input).unwrap();

    assert_eq!(data.dependency_data_offset, None);
    assert_eq!(data.objects[0].object_path, "L_Level");
}

#[test]
fn an_offset_of_zero_or_less_gives_no_data() {
    for offset in [0, -1] {
        let mut input = package(section(Some(SECTION_END), &[level_object()]));
        input.asset_registry = None;
        input.summary.asset_registry_data_offset = offset;

        let data = read(&input).unwrap();

        assert_eq!(data.dependency_data_offset, None, "offset {offset}");
        assert!(data.objects.is_empty(), "offset {offset}");
    }
}

#[test]
fn a_truncated_section_gives_an_error() {
    let bytes = package(section(Some(SECTION_END), &[level_object()])).to_bytes();
    let header = read_header(&bytes).unwrap().header;

    for cut in [1, 5, 20] {
        let result = read_asset_registry(&bytes[..bytes.len() - cut], &header.summary);

        assert!(
            matches!(result, Err(Error::Truncated { .. })),
            "cut {cut}: {result:?}"
        );
    }
}

#[test]
fn an_offset_after_the_input_gives_truncated() {
    let mut input = package(section(Some(SECTION_END), &[]));
    input.asset_registry = None;
    input.summary.asset_registry_data_offset = 100_000;

    assert!(matches!(read(&input), Err(Error::Truncated { .. })));
}

#[test]
fn a_negative_object_count_gives_a_bad_value() {
    let mut bytes = Writer::default();
    bytes.i64(0);
    bytes.i32(-1);

    let result = read(&package(bytes.0));

    assert!(
        matches!(
            result,
            Err(Error::BadValue {
                field: "asset registry object count",
                ..
            })
        ),
        "{result:?}"
    );
}

#[test]
fn an_object_count_larger_than_the_input_gives_truncated() {
    let mut bytes = Writer::default();
    bytes.i64(0);
    bytes.i32(1_000_000);

    assert!(matches!(
        read(&package(bytes.0)),
        Err(Error::Truncated { .. })
    ));
}

#[test]
fn a_negative_tag_count_gives_a_bad_value() {
    let mut bytes = Writer::default();
    bytes.i64(0);
    bytes.i32(1);
    bytes.string(&Str::Ansi("L_Level"));
    bytes.string(&Str::Ansi("/Script/Engine.World"));
    bytes.i32(-5);

    let result = read(&package(bytes.0));

    assert!(
        matches!(
            result,
            Err(Error::BadValue {
                field: "asset registry tag count",
                ..
            })
        ),
        "{result:?}"
    );
}

#[test]
fn a_tag_count_larger_than_the_input_gives_truncated() {
    let mut bytes = Writer::default();
    bytes.i64(0);
    bytes.i32(1);
    bytes.string(&Str::Ansi("L_Level"));
    bytes.string(&Str::Ansi("/Script/Engine.World"));
    bytes.i32(1_000_000);

    assert!(matches!(
        read(&package(bytes.0)),
        Err(Error::Truncated { .. })
    ));
}

#[test]
fn a_dependency_data_offset_that_is_not_the_section_end_gives_a_bad_value() {
    let input = package(section(Some(SECTION_END), &[level_object()]));
    let end = input.to_bytes().len() as i64;

    // A damaged object count can stop the objects before the end of the
    // section. UE writes the dependency data directly after the objects.
    for offset in [0, end - 1, end + 1] {
        let bytes = package(section(Some(offset), &[level_object()])).to_bytes();
        let header = read_header(&bytes).unwrap().header;

        let result = read_asset_registry(&bytes, &header.summary);

        assert_eq!(
            result,
            Err(Error::BadValue {
                offset: header.summary.asset_registry_data_offset as usize,
                field: "asset registry dependency data offset",
            }),
            "offset {offset}"
        );
    }
}
