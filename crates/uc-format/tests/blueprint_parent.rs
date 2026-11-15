//! The direct parent class of a Blueprint class, from the Blueprint
//! package.

mod common;

use std::path::PathBuf;

use common::{Export, Import, Package};
use uc_format::Error;
use uc_format::package::{PackageHeader, read_header};

const CLASS_PATH: &str = "/Game/Fixtures/BP_Test.BP_Test_C";

/// The Blueprint package `/Game/Fixtures/BP_Test` with the generated class
/// `BP_Test_C`. The super index of the class is `super_index`. Import -2
/// is `/Script/Engine.Actor`.
fn header(super_index: i32) -> PackageHeader {
    let bytes = Package {
        imports: vec![
            Import {
                class_package: (0, 0),
                class_name: (3, 0),
                outer_index: 0,
                object_name: (0, 0),
                package_name: (5, 0),
                import_optional: 0,
            },
            Import {
                class_package: (0, 0),
                class_name: (2, 0),
                outer_index: -1,
                object_name: (1, 0),
                package_name: (5, 0),
                import_optional: 0,
            },
        ],
        exports: vec![Export {
            class_index: -2,
            super_index,
            object_name: (4, 0),
            ..Export::default()
        }],
        ..Package::new(&[
            "/Script/Engine",
            "Actor",
            "Class",
            "Package",
            "BP_Test_C",
            "None",
        ])
    }
    .to_bytes();
    read_header(&bytes).unwrap().header
}

#[test]
fn gives_the_super_class_of_the_generated_class() {
    let header = header(-2);

    assert_eq!(
        header.blueprint_parent_class_path(CLASS_PATH).as_deref(),
        Some("/Script/Engine.Actor")
    );
}

#[test]
fn ignores_the_case_of_ascii_letters() {
    let header = header(-2);

    assert_eq!(
        header
            .blueprint_parent_class_path("/game/fixtures/bp_test.bp_test_c")
            .as_deref(),
        Some("/Script/Engine.Actor")
    );
}

#[test]
fn a_class_of_another_package_or_another_name_gives_none() {
    let header = header(-2);

    for path in [
        "/Game/Other/BP_Test.BP_Test_C",
        "/Game/Fixtures/BP_Test.BP_Other_C",
        "/Game/Fixtures/BP_Test",
        "",
    ] {
        assert_eq!(header.blueprint_parent_class_path(path), None, "{path}");
    }
}

#[test]
fn a_null_super_index_gives_none() {
    let header = header(0);

    assert_eq!(header.blueprint_parent_class_path(CLASS_PATH), None);
}

#[test]
fn a_super_index_out_of_the_tables_gives_an_error_of_the_header() {
    // The header parser checks each super index, so the function gets
    // only indices of the tables.
    let bytes = Package {
        exports: vec![Export {
            super_index: -100,
            object_name: (1, 0),
            ..Export::default()
        }],
        ..Package::new(&["/Script/Engine", "BP_Test_C"])
    }
    .to_bytes();

    assert!(matches!(
        read_header(&bytes),
        Err(Error::BadValue {
            field: "export super index",
            ..
        })
    ));
}

/// A fixture file. `name` is relative to `tests/fixtures`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

/// Gives the class path in the first `ParentClass` tag of a `PkgInfo`
/// output. The tag value has the form `<class of class>'<class path>'`.
fn pkginfo_parent_class(pkginfo: &str) -> String {
    let text = std::fs::read_to_string(fixture(pkginfo)).unwrap();
    let (_, value) = text.split_once("\"ParentClass\": \"").unwrap();
    let (_, value) = value.split_once('\'').unwrap();
    let (path, _) = value.split_once('\'').unwrap();
    path.to_owned()
}

/// Checks the parent class of the generated class `class_path` of the
/// Blueprint fixture `package` with the `ParentClass` tag of its `PkgInfo`
/// output.
fn check(package: &str, class_path: &str) -> Result<(), Error> {
    let bytes = std::fs::read(fixture(package)).unwrap();
    // A Git LFS pointer file is small text, not a package.
    assert!(
        !bytes.starts_with(b"version https://git-lfs"),
        "{package} is a Git LFS pointer: run `git lfs pull`"
    );
    let header = read_header(&bytes)?.header;
    let pkginfo = PathBuf::from(package).with_extension("pkginfo.txt");
    let expected = pkginfo_parent_class(pkginfo.to_str().unwrap());

    assert_eq!(
        header.blueprint_parent_class_path(class_path),
        Some(expected),
        "{package}"
    );
    Ok(())
}

#[test]
fn bp_summary_actor_parent_class() {
    check(
        "editor/BP_SummaryActor.uasset",
        "/Game/Summary/BP_SummaryActor.BP_SummaryActor_C",
    )
    .unwrap();
}
