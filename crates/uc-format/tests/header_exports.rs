//! Package header: the export table, and the package index checks.

mod common;

use common::{Export, Import, PKG_UNVERSIONED_PROPERTIES, Package, ue4, ue5};
use uc_format::package::{ExportDependencies, NameRef, PackageIndex, read_header};
use uc_format::{Error, Guid};

const NAMES: &[&str] = &["/Script/Engine", "Actor", "Class", "BP_Test", "None"];

/// UE 5.8 editor package: import 0 is class `Actor`, export 0 is
/// `BP_Test`, export 1 is `BP_Test_0` in export 0.
fn package() -> Package {
    let export = Export {
        class_index: -1,
        super_index: -1,
        template_index: 0,
        outer_index: 0,
        object_name: (3, 0),
        object_flags: 0x0000_000B,
        serial_size: 0x1_0000_0733,
        serial_offset: 0x2_0000_4991,
        forced_export: 0,
        not_for_client: 1,
        not_for_server: 1,
        package_guid: [1, 2, 3, 4],
        is_inherited_instance: 1,
        package_flags: 0x8000_0001,
        not_always_loaded_for_editor_game: 1,
        is_asset: 1,
        generate_public_hash: 1,
        dependencies: [-1, 2, 3, 4, 5],
        script_serialization_start_offset: 0x3_0000_0011,
        script_serialization_end_offset: 0x3_0000_0022,
    };
    Package {
        imports: vec![Import {
            class_package: (0, 0),
            class_name: (2, 0),
            outer_index: 0,
            object_name: (1, 0),
            package_name: (4, 0),
            import_optional: 0,
        }],
        exports: vec![
            export.clone(),
            Export {
                class_index: -1,
                super_index: 0,
                template_index: 1,
                outer_index: 1,
                object_name: (3, 1),
                ..export
            },
        ],
        ..Package::new(NAMES)
    }
}

fn exports_of(input: &Package) -> Vec<uc_format::package::ObjectExport> {
    read_header(&input.to_bytes()).unwrap().header.exports
}

#[test]
fn reads_all_fields_of_a_ue58_export() {
    let exports = exports_of(&package());

    assert_eq!(exports.len(), 2);
    let e = &exports[0];
    assert_eq!(e.class_index, PackageIndex(-1));
    assert_eq!(e.super_index, PackageIndex(-1));
    assert_eq!(e.template_index, Some(PackageIndex(0)));
    assert_eq!(e.outer_index, PackageIndex(0));
    assert_eq!(
        e.object_name,
        NameRef {
            index: 3,
            number: 0
        }
    );
    assert_eq!(e.object_flags, 0x0000_000B);
    assert_eq!(e.serial_size, 0x1_0000_0733);
    assert_eq!(e.serial_offset, 0x2_0000_4991);
    assert!(!e.forced_export);
    assert!(e.not_for_client);
    assert!(e.not_for_server);
    assert_eq!(e.package_guid, None);
    assert_eq!(e.is_inherited_instance, Some(true));
    assert_eq!(e.package_flags, 0x8000_0001);
    assert_eq!(e.not_always_loaded_for_editor_game, Some(true));
    assert_eq!(e.is_asset, Some(true));
    assert_eq!(e.generate_public_hash, Some(true));
    assert_eq!(
        e.dependencies,
        Some(ExportDependencies {
            first_export_dependency: -1,
            serialization_before_serialization: 2,
            create_before_serialization: 3,
            serialization_before_create: 4,
            create_before_create: 5,
        })
    );
    assert_eq!(e.script_serialization_start_offset, Some(0x3_0000_0011));
    assert_eq!(e.script_serialization_end_offset, Some(0x3_0000_0022));
    assert_eq!(exports[1].template_index, Some(PackageIndex(1)));
    assert_eq!(exports[1].outer_index, PackageIndex(1));
    assert_eq!(
        exports[1].object_name,
        NameRef {
            index: 3,
            number: 1
        }
    );
}

#[test]
fn old_ue5_versions_have_the_package_guid_and_no_new_fields() {
    let mut input = package();
    input.summary.file_version_ue5 = ue5::REMOVE_OBJECT_EXPORT_PACKAGE_GUID - 1;

    let e = &exports_of(&input)[0];

    assert_eq!(
        e.package_guid,
        Some(Guid {
            a: 1,
            b: 2,
            c: 3,
            d: 4
        })
    );
    assert_eq!(e.is_inherited_instance, None);
    assert_eq!(e.generate_public_hash, Some(true));
    assert_eq!(e.script_serialization_start_offset, None);
    assert_eq!(e.package_flags, 0x8000_0001);
}

#[test]
fn the_first_ue5_version_has_no_public_hash_flag() {
    let mut input = package();
    input.summary.file_version_ue5 = ue5::OPTIONAL_RESOURCES - 1;

    let e = &exports_of(&input)[0];

    assert_eq!(e.generate_public_hash, None);
    assert_eq!(e.dependencies.unwrap().create_before_create, 5);
}

#[test]
fn old_ue4_versions_have_32_bit_sizes_and_no_template() {
    let mut input = package();
    input.summary.file_version_ue4 = ue4::PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS;
    input.exports[0].serial_size = 0x733;
    input.exports[0].serial_offset = -0x4991;

    let e = &exports_of(&input)[0];

    assert_eq!(e.template_index, None);
    assert_eq!(e.serial_size, 0x733);
    assert_eq!(e.serial_offset, -0x4991);
    assert_eq!(e.dependencies.unwrap().first_export_dependency, -1);
}

#[test]
fn very_old_ue4_versions_have_no_flags_and_no_dependencies() {
    let mut input = package();
    input.summary.file_version_ue4 = ue4::LOAD_FOR_EDITOR_GAME - 1;

    let e = &exports_of(&input)[0];

    assert_eq!(e.not_always_loaded_for_editor_game, None);
    assert_eq!(e.is_asset, None);
    assert_eq!(e.dependencies, None);
    assert_eq!(e.script_serialization_end_offset, Some(0x3_0000_0022));
}

#[test]
fn unversioned_properties_have_no_script_serialization_offsets() {
    let mut input = package();
    input.summary.package_flags |= PKG_UNVERSIONED_PROPERTIES;

    let exports = exports_of(&input);

    assert_eq!(exports[0].script_serialization_start_offset, None);
    assert_eq!(exports[0].script_serialization_end_offset, None);
    assert_eq!(
        exports[1].object_name,
        NameRef {
            index: 3,
            number: 1
        }
    );
}

#[test]
fn an_object_name_out_of_the_name_map_is_a_bad_value() {
    let mut input = package();
    input.exports[1].object_name = (NAMES.len() as i32, 0);

    let result = read_header(&input.to_bytes());

    assert!(
        matches!(
            result,
            Err(Error::BadValue {
                field: "export object name",
                ..
            })
        ),
        "{result:?}"
    );
}

/// Sets one package index of the package.
type SetIndex = fn(&mut Package, i32);

#[test]
fn a_package_index_out_of_the_tables_is_a_bad_value() {
    let cases: [(SetIndex, &str); 5] = [
        (
            |p, index| p.imports[0].outer_index = index,
            "import outer index",
        ),
        (
            |p, index| p.exports[1].class_index = index,
            "export class index",
        ),
        (
            |p, index| p.exports[1].super_index = index,
            "export super index",
        ),
        (
            |p, index| p.exports[1].template_index = index,
            "export template index",
        ),
        (
            |p, index| p.exports[1].outer_index = index,
            "export outer index",
        ),
    ];
    // 1 import and 2 exports: -2 and 3 are out of the tables.
    for (change, field) in cases {
        for index in [-2, 3, i32::MIN, i32::MAX] {
            let mut input = package();
            change(&mut input, index);

            let result = read_header(&input.to_bytes());

            assert!(
                matches!(result, Err(Error::BadValue { field: f, .. }) if f == field),
                "{field}, index {index}: {result:?}"
            );
        }
    }
}

#[test]
fn an_export_table_after_the_input_end_is_truncated() {
    let bytes = package().to_bytes();
    let export_offset = bytes.len() - 2 * 112;

    for len in export_offset..bytes.len() {
        let result = read_header(&bytes[..len]);

        assert!(
            matches!(result, Err(Error::Truncated { .. })),
            "length {len}: {result:?}"
        );
    }
}

#[test]
fn a_large_export_count_is_truncated_without_allocation() {
    let input = Package {
        export_count: Some(i32::MAX),
        ..package()
    };
    let bytes = input.to_bytes();
    let export_offset = bytes.len() - 2 * 112;

    let result = read_header(&bytes);

    assert_eq!(
        result.unwrap_err(),
        Error::Truncated {
            offset: export_offset
        }
    );
}
