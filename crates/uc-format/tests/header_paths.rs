//! Package header: object names, class names and object paths.

mod common;

use common::{Export, Import, Name, Package};
use uc_format::Error;
use uc_format::package::{IndexTarget, MAX_OBJECT_PATH_LENGTH, PackageIndex, read_header};

const NAMES: &[&str] = &[
    "/Script/CoreUObject", // 0
    "/Script/Engine",      // 1
    "Package",             // 2
    "Class",               // 3
    "Actor",               // 4
    "Default__Actor",      // 5
    "Blueprint",           // 6
    "BP_Test",             // 7
    "EventGraph",          // 8
    "K2Node_Event",        // 9
    "Root",                // 10
    "EdGraph",             // 11
    "None",                // 12
];

fn import(class_name: i32, outer_index: i32, object_name: i32) -> Import {
    Import {
        class_package: (0, 0),
        class_name: (class_name, 0),
        outer_index,
        object_name: (object_name, 0),
        package_name: (12, 0),
        import_optional: 0,
    }
}

fn export(class_index: i32, outer_index: i32, object_name: (i32, i32)) -> Export {
    Export {
        class_index,
        outer_index,
        object_name,
        ..Export::default()
    }
}

/// Imports:
/// - -1: `/Script/Engine` (package)
/// - -2: `/Script/Engine.Actor` (class)
/// - -3: `/Script/Engine.Default__Actor` (actor)
/// - -4: `/Script/Engine.Default__Actor:Root` (subobject of an import)
/// - -5: `/Script/Engine.Blueprint` (class)
/// - -6: `/Script/Engine.EdGraph` (class)
///
/// Exports:
/// - 1: `BP_Test` (blueprint)
/// - 2: `BP_Test:EventGraph`
/// - 3: `BP_Test:EventGraph.K2Node_Event_0`
/// - 4: `/Script/Engine.Default__Actor:Root`, an export in an import
/// - 5: `BP_Test_1`, a class (null class index)
/// - 6: `Root`, with export 1 as its class
fn package() -> Package {
    Package {
        imports: vec![
            import(2, 0, 1),
            import(3, -1, 4),
            import(4, -1, 5),
            import(4, -3, 10),
            import(3, -1, 6),
            import(3, -1, 11),
        ],
        exports: vec![
            export(-5, 0, (7, 0)),
            export(-6, 1, (8, 0)),
            export(-2, 2, (9, 1)),
            export(-2, -3, (10, 0)),
            export(0, 0, (7, 2)),
            export(1, 0, (10, 0)),
        ],
        ..Package::new(NAMES)
    }
}

#[test]
fn a_package_index_gives_null_an_import_or_an_export() {
    assert_eq!(PackageIndex(0).target(), IndexTarget::Null);
    assert_eq!(PackageIndex(-1).target(), IndexTarget::Import(0));
    assert_eq!(PackageIndex(1).target(), IndexTarget::Export(0));
    assert_eq!(
        PackageIndex(i32::MIN).target(),
        IndexTarget::Import(i32::MAX as usize)
    );
    assert_eq!(
        PackageIndex(i32::MAX).target(),
        IndexTarget::Export(i32::MAX as usize - 1)
    );
}

#[test]
fn gives_object_names_with_their_number() {
    let header = read_header(&package().to_bytes()).unwrap().header;

    let name = |index| header.object_name(PackageIndex(index));

    assert_eq!(name(-3).as_deref(), Some("Default__Actor"));
    assert_eq!(name(3).as_deref(), Some("K2Node_Event_0"));
    assert_eq!(name(0), None);
    assert_eq!(name(-7), None);
    assert_eq!(name(7), None);
}

#[test]
fn gives_class_names_of_imports_and_exports() {
    let header = read_header(&package().to_bytes()).unwrap().header;

    let class = |index| header.class_name(PackageIndex(index));

    assert_eq!(class(-1).as_deref(), Some("Package"));
    assert_eq!(class(-3).as_deref(), Some("Actor"));
    assert_eq!(class(1).as_deref(), Some("Blueprint"));
    assert_eq!(class(5).as_deref(), Some("Class"));
    assert_eq!(class(6).as_deref(), Some("BP_Test"));
    assert_eq!(class(0), None);
    assert_eq!(class(7), None);
}

#[test]
fn gives_import_paths() {
    let header = read_header(&package().to_bytes()).unwrap().header;

    let path = |index| header.object_path(PackageIndex(index));

    assert_eq!(path(-1).as_deref(), Some("/Script/Engine"));
    assert_eq!(path(-2).as_deref(), Some("/Script/Engine.Actor"));
    assert_eq!(path(-3).as_deref(), Some("/Script/Engine.Default__Actor"));
    assert_eq!(
        path(-4).as_deref(),
        Some("/Script/Engine.Default__Actor:Root")
    );
    assert_eq!(path(0), None);
    assert_eq!(path(-7), None);
}

#[test]
fn gives_export_paths_with_the_package_name() {
    let header = read_header(&package().to_bytes()).unwrap().header;

    let path = |index| header.object_path(PackageIndex(index));

    assert_eq!(path(1).as_deref(), Some("/Game/Fixtures/BP_Test.BP_Test"));
    assert_eq!(
        path(2).as_deref(),
        Some("/Game/Fixtures/BP_Test.BP_Test:EventGraph")
    );
    assert_eq!(
        path(3).as_deref(),
        Some("/Game/Fixtures/BP_Test.BP_Test:EventGraph.K2Node_Event_0")
    );
    assert_eq!(path(5).as_deref(), Some("/Game/Fixtures/BP_Test.BP_Test_1"));
    assert_eq!(path(7), None);
}

#[test]
fn an_export_in_an_import_has_no_package_name() {
    let header = read_header(&package().to_bytes()).unwrap().header;

    let path = header.object_path(PackageIndex(4));

    assert_eq!(path.as_deref(), Some("/Script/Engine.Default__Actor:Root"));
}

#[test]
fn an_outer_chain_with_a_loop_is_a_bad_value() {
    let mut export_loop = package();
    export_loop.exports[0].outer_index = 3;
    let mut self_loop = package();
    self_loop.exports[5].outer_index = 6;
    let mut import_loop = package();
    import_loop.imports[0].outer_index = -4;

    for input in [export_loop, self_loop, import_loop] {
        let result = read_header(&input.to_bytes());

        assert!(
            matches!(
                result,
                Err(Error::BadValue {
                    field: "outer index",
                    ..
                })
            ),
            "{result:?}"
        );
    }
}

/// A package with two names of `first` and `second` characters, and two
/// imports: `<first>`, and its inner object `<first>:<second>`.
fn two_part_import(first: usize, second: usize) -> Package {
    let mut input = Package::new(&["Package", "None"]);
    input.names.push(Name::ansi("A".repeat(first).leak()));
    input.names.push(Name::ansi("B".repeat(second).leak()));
    input.imports = vec![import(1, 0, 2), import(1, -1, 3)];
    for import in &mut input.imports {
        import.package_name = (1, 0);
    }
    input
}

#[test]
fn a_path_of_1024_bytes_is_given() {
    let header = read_header(&two_part_import(1000, 23).to_bytes())
        .unwrap()
        .header;

    let path = header.object_path(PackageIndex(-2)).unwrap();

    assert_eq!(path.len(), MAX_OBJECT_PATH_LENGTH);
    assert_eq!(path, format!("{}:{}", "A".repeat(1000), "B".repeat(23)));
}

#[test]
fn a_path_longer_than_1024_bytes_is_not_given() {
    let header = read_header(&two_part_import(1000, 24).to_bytes())
        .unwrap()
        .header;

    assert_eq!(header.object_path(PackageIndex(-2)), None);
    assert_eq!(header.object_path(PackageIndex(-1)).unwrap().len(), 1000);
}

/// A package with one export: a name of `length` characters. Its path
/// is the package name `/Game/Fixtures/BP_Test` (22 bytes), `.` and the
/// name.
fn one_export(length: usize) -> Package {
    let mut input = Package::new(&["None"]);
    input.names.push(Name::ansi("A".repeat(length).leak()));
    input.exports = vec![export(0, 0, (1, 0))];
    input
}

#[test]
fn the_package_name_is_part_of_the_path_length() {
    let fits = read_header(&one_export(1001).to_bytes()).unwrap().header;
    let too_long = read_header(&one_export(1002).to_bytes()).unwrap().header;

    let path = fits.object_path(PackageIndex(1)).unwrap();
    assert_eq!(path.len(), MAX_OBJECT_PATH_LENGTH);
    assert_eq!(too_long.object_path(PackageIndex(1)), None);
}

/// A deep outer chain of long names is valid input. The path walk stops
/// at the length limit, so the time and the memory do not grow with the
/// chain depth.
#[test]
fn a_deep_chain_of_long_names_gives_no_path_and_stops_early() {
    let mut input = Package::new(&["None"]);
    input.names.push(Name::ansi("A".repeat(1023).leak()));
    input.exports = (0..5_000).map(|i| export(0, i, (1, 0))).collect();
    let header = read_header(&input.to_bytes()).unwrap().header;

    let start = std::time::Instant::now();
    for index in 1..=5_000 {
        assert_eq!(header.object_path(PackageIndex(index)), None, "{index}");
    }

    assert!(start.elapsed() < std::time::Duration::from_secs(5));
}
