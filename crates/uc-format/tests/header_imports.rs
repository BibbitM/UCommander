//! Package header: the import table.

mod common;

use common::{Import, Package, ue4, ue5};
use uc_format::Error;
use uc_format::package::{NameRef, PackageIndex, read_header};

const NAMES: &[&str] = &[
    "/Script/CoreUObject",
    "/Script/Engine",
    "Actor",
    "Class",
    "Package",
    "None",
];

/// Import 0: package `/Script/Engine`. Import 1: class `Actor` in import 0.
fn two_imports() -> Package {
    Package {
        imports: vec![
            Import {
                class_package: (0, 0),
                class_name: (4, 0),
                outer_index: 0,
                object_name: (1, 0),
                package_name: (5, 0),
                import_optional: 0,
            },
            Import {
                class_package: (0, 0),
                class_name: (3, 0),
                outer_index: -1,
                object_name: (2, 3),
                package_name: (1, 0),
                import_optional: 1,
            },
        ],
        ..Package::new(NAMES)
    }
}

fn name(index: i32, number: i32) -> NameRef {
    NameRef { index, number }
}

#[test]
fn reads_the_imports_in_order() {
    let imports = read_header(&two_imports().to_bytes())
        .unwrap()
        .header
        .imports;

    assert_eq!(imports.len(), 2);
    let import = &imports[1];
    assert_eq!(import.class_package, name(0, 0));
    assert_eq!(import.class_name, name(3, 0));
    assert_eq!(import.outer_index, PackageIndex(-1));
    assert_eq!(import.object_name, name(2, 3));
    assert_eq!(import.package_name, Some(name(1, 0)));
    assert_eq!(import.import_optional, Some(true));
    assert_eq!(imports[0].object_name, name(1, 0));
    assert_eq!(imports[0].import_optional, Some(false));
}

#[test]
fn a_boolean_greater_than_1_is_true() {
    let mut input = two_imports();
    input.imports[0].import_optional = 7;

    let imports = read_header(&input.to_bytes()).unwrap().header.imports;

    assert_eq!(imports[0].import_optional, Some(true));
}

#[test]
fn old_versions_have_no_package_name_and_no_optional_flag() {
    let mut input = two_imports();
    input.summary.file_version_ue4 = ue4::NON_OUTER_PACKAGE_IMPORT - 1;
    input.summary.file_version_ue5 = ue5::OPTIONAL_RESOURCES - 1;

    let imports = read_header(&input.to_bytes()).unwrap().header.imports;

    assert_eq!(imports[1].object_name, name(2, 3));
    assert_eq!(imports[1].package_name, None);
    assert_eq!(imports[1].import_optional, None);
}

/// Sets one name index of an import.
type SetIndex = fn(&mut Import, i32);

#[test]
fn a_name_index_out_of_the_name_map_is_a_bad_value() {
    let names = NAMES.len() as i32;
    let cases: [(SetIndex, &str); 4] = [
        (|i, index| i.class_package.0 = index, "import class package"),
        (|i, index| i.class_name.0 = index, "import class name"),
        (|i, index| i.object_name.0 = index, "import object name"),
        (|i, index| i.package_name.0 = index, "import package name"),
    ];
    for (change, field) in cases {
        for index in [names, -1] {
            let mut input = two_imports();
            change(&mut input.imports[1], index);

            let result = read_header(&input.to_bytes());

            assert!(
                matches!(result, Err(Error::BadValue { field: f, .. }) if f == field),
                "{field}, index {index}: {result:?}"
            );
        }
    }
}

#[test]
fn an_import_table_after_the_input_end_is_truncated() {
    let input = two_imports();
    let bytes = input.to_bytes();
    let import_offset = bytes.len() - 2 * 40;

    for len in import_offset..bytes.len() {
        let result = read_header(&bytes[..len]);

        assert!(
            matches!(result, Err(Error::Truncated { .. })),
            "length {len}: {result:?}"
        );
    }
}

#[test]
fn a_large_import_count_is_truncated_without_allocation() {
    let input = Package {
        import_count: Some(i32::MAX),
        ..two_imports()
    };
    let bytes = input.to_bytes();
    let import_offset = bytes.len() - 2 * 40;

    let result = read_header(&bytes);

    assert_eq!(
        result.unwrap_err(),
        Error::Truncated {
            offset: import_offset
        }
    );
}

#[test]
fn a_negative_import_count_is_a_bad_value() {
    let input = Package {
        import_count: Some(-1),
        ..two_imports()
    };

    let result = read_header(&input.to_bytes());

    assert!(
        matches!(
            result,
            Err(Error::BadValue {
                field: "import",
                ..
            })
        ),
        "{result:?}"
    );
}
