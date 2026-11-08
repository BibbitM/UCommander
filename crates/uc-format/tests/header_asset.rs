//! Package header: the main asset and the asset class.

mod common;

use common::{Export, Import, Package};
use uc_format::package::{PackageIndex, read_header};

/// Package flag `ContainsNoAsset`.
const PKG_CONTAINS_NO_ASSET: u32 = 0x0000_0400;
/// Package flag `FilterEditorOnly`.
const PKG_FILTER_EDITOR_ONLY: u32 = 0x8000_0000;

const NAMES: &[&str] = &[
    "/Script/CoreUObject",           // 0
    "/Script/Engine",                // 1
    "Package",                       // 2
    "Class",                         // 3
    "Blueprint",                     // 4
    "BlueprintGeneratedClass",       // 5
    "WidgetBlueprintGeneratedClass", // 6
    "StaticMesh",                    // 7
    "BP_Test",                       // 8
    "bp_test",                       // 9
    "BP_Test_C",                     // 10
    "Other",                         // 11
    "None",                          // 12
    "WidgetBlueprint",               // 13
    "/Game/Other/BP_Other",          // 14
    "BP_Other_C",                    // 15
];

/// The class imports, as package indexes.
const BLUEPRINT: i32 = -2;
const GENERATED_CLASS: i32 = -3;
const WIDGET_GENERATED_CLASS: i32 = -4;
const STATIC_MESH: i32 = -5;
const WIDGET_BLUEPRINT: i32 = -6;
/// A Blueprint generated class in the package `/Game/Other/BP_Other` (-7).
const OTHER_PACKAGE_CLASS: i32 = -8;

fn class_import(object_name: i32) -> Import {
    Import {
        class_package: (0, 0),
        class_name: (3, 0),
        outer_index: -1,
        object_name: (object_name, 0),
        package_name: (12, 0),
        import_optional: 0,
    }
}

fn export(class_index: i32, outer_index: i32, object_name: (i32, i32), is_asset: bool) -> Export {
    Export {
        class_index,
        outer_index,
        object_name,
        is_asset: u32::from(is_asset),
        ..Export::default()
    }
}

/// A package `/Game/Fixtures/BP_Test` with the exports `exports`. The
/// imports are `/Script/Engine` (-1) and the class constants above.
fn package(exports: Vec<Export>) -> Package {
    let mut package = Package {
        imports: vec![
            Import {
                class_package: (0, 0),
                class_name: (2, 0),
                outer_index: 0,
                object_name: (1, 0),
                package_name: (12, 0),
                import_optional: 0,
            },
            class_import(4),
            class_import(5),
            class_import(6),
            class_import(7),
            class_import(13),
            Import {
                class_package: (0, 0),
                class_name: (2, 0),
                outer_index: 0,
                object_name: (14, 0),
                package_name: (12, 0),
                import_optional: 0,
            },
            Import {
                class_package: (1, 0),
                class_name: (5, 0),
                outer_index: -7,
                object_name: (15, 0),
                package_name: (12, 0),
                import_optional: 0,
            },
        ],
        exports,
        ..Package::new(NAMES)
    };
    package.summary.package_flags = 0;
    package
}

fn main_asset(package: &Package) -> Option<PackageIndex> {
    read_header(&package.to_bytes())
        .unwrap()
        .header
        .main_asset()
}

fn asset_class(package: &Package) -> Option<String> {
    read_header(&package.to_bytes())
        .unwrap()
        .header
        .asset_class()
}

#[test]
fn the_asset_with_the_short_package_name_is_the_main_asset() {
    let package = package(vec![
        export(GENERATED_CLASS, 0, (10, 0), true),
        export(STATIC_MESH, 0, (11, 0), true),
        export(BLUEPRINT, 0, (8, 0), true),
    ]);

    assert_eq!(main_asset(&package), Some(PackageIndex(3)));
    assert_eq!(asset_class(&package).as_deref(), Some("Blueprint"));
}

#[test]
fn the_name_comparison_ignores_the_case() {
    let package = package(vec![
        export(STATIC_MESH, 0, (11, 0), true),
        export(BLUEPRINT, 0, (9, 0), true),
    ]);

    assert_eq!(main_asset(&package), Some(PackageIndex(2)));
}

#[test]
fn an_export_without_the_is_asset_flag_is_not_the_main_asset() {
    let package = package(vec![
        export(BLUEPRINT, 0, (8, 0), false),
        export(STATIC_MESH, 0, (11, 0), true),
    ]);

    assert_eq!(main_asset(&package), Some(PackageIndex(2)));
    assert_eq!(asset_class(&package).as_deref(), Some("StaticMesh"));
}

#[test]
fn the_main_name_needs_a_null_outer_and_no_number() {
    let package = package(vec![
        export(STATIC_MESH, 0, (11, 0), true),
        export(BLUEPRINT, 1, (8, 0), true),
        export(BLUEPRINT, 0, (8, 1), true),
    ]);

    // No export has the main name, so the first asset is the result.
    assert_eq!(main_asset(&package), Some(PackageIndex(1)));
}

#[test]
fn without_the_main_name_the_first_asset_that_is_not_skipped_is_the_result() {
    let package = package(vec![
        export(BLUEPRINT, 0, (11, 0), false),
        export(GENERATED_CLASS, 0, (10, 0), true),
        export(WIDGET_GENERATED_CLASS, 0, (10, 1), true),
        export(WIDGET_BLUEPRINT, 0, (11, 1), true),
        export(STATIC_MESH, 0, (11, 2), true),
    ]);

    assert_eq!(main_asset(&package), Some(PackageIndex(4)));
    assert_eq!(asset_class(&package).as_deref(), Some("WidgetBlueprint"));
}

#[test]
fn a_skipped_class_is_never_the_main_asset() {
    let package = package(vec![export(GENERATED_CLASS, 0, (8, 0), true)]);

    assert_eq!(main_asset(&package), None);
}

#[test]
fn a_cooked_package_skips_blueprints_and_not_generated_classes() {
    let mut package = package(vec![
        export(BLUEPRINT, 0, (8, 0), true),
        export(WIDGET_BLUEPRINT, 0, (11, 0), true),
        export(GENERATED_CLASS, 0, (10, 0), true),
    ]);
    package.summary.package_flags = PKG_FILTER_EDITOR_ONLY;

    assert_eq!(main_asset(&package), Some(PackageIndex(3)));
    assert_eq!(
        asset_class(&package).as_deref(),
        Some("BlueprintGeneratedClass")
    );
}

#[test]
fn a_package_with_the_contains_no_asset_flag_has_no_main_asset() {
    let mut package = package(vec![export(BLUEPRINT, 0, (8, 0), true)]);
    package.summary.package_flags = PKG_CONTAINS_NO_ASSET;

    assert_eq!(main_asset(&package), None);
    assert_eq!(asset_class(&package), None);
}

#[test]
fn a_package_without_assets_has_no_main_asset() {
    let package = package(vec![
        export(BLUEPRINT, 0, (8, 0), false),
        export(STATIC_MESH, 1, (11, 0), false),
    ]);

    assert_eq!(main_asset(&package), None);
    assert_eq!(asset_class(&package), None);
}

#[test]
fn a_package_without_exports_has_no_main_asset() {
    let package = package(Vec::new());

    assert_eq!(main_asset(&package), None);
    assert_eq!(asset_class(&package), None);
}

/// The class path, the name and the object path of the main asset.
fn asset_paths(package: &Package) -> [Option<String>; 3] {
    let header = read_header(&package.to_bytes()).unwrap().header;
    [
        header.asset_class_path(),
        header.asset_name(),
        header.asset_object_path(),
    ]
}

#[test]
fn a_top_level_asset_has_a_class_path_a_name_and_an_object_path() {
    let package = package(vec![
        export(STATIC_MESH, 0, (11, 0), true),
        export(BLUEPRINT, 0, (8, 0), true),
    ]);

    assert_eq!(
        asset_paths(&package),
        [
            Some("/Script/Engine.Blueprint".to_owned()),
            Some("BP_Test".to_owned()),
            Some("/Game/Fixtures/BP_Test.BP_Test".to_owned()),
        ]
    );
}

#[test]
fn the_class_path_of_a_class_from_another_package_is_its_object_path() {
    let package = package(vec![export(OTHER_PACKAGE_CLASS, 0, (8, 0), true)]);

    assert_eq!(
        asset_paths(&package)[0].as_deref(),
        Some("/Game/Other/BP_Other.BP_Other_C")
    );
}

#[test]
fn a_null_class_index_gives_the_class_path_of_class() {
    let package = package(vec![export(0, 0, (8, 0), true)]);

    assert_eq!(
        asset_paths(&package)[0].as_deref(),
        Some("/Script/CoreUObject.Class")
    );
}

#[test]
fn a_package_without_a_main_asset_has_no_asset_paths() {
    let package = package(vec![export(BLUEPRINT, 0, (8, 0), false)]);

    assert_eq!(asset_paths(&package), [None, None, None]);
}
