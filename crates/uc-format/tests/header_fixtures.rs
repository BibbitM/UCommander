//! Name map, import table and export table of real fixtures, compared with
//! their `.expected.json` files. The expected values come from the
//! `PkgInfo` commandlet of UE. The paths use the real package name, not
//! the temporary `/Temp/` name of `PkgInfo` (point 2 owner decision 1).

use std::path::PathBuf;

use serde::Deserialize;
use uc_format::package::{PackageHeader, PackageIndex, read_header};

#[derive(Debug, Deserialize)]
struct Expected {
    names: Vec<String>,
    imports: Vec<ExpectedImport>,
    exports: Vec<ExpectedExport>,
}

#[derive(Debug, Deserialize)]
struct ExpectedImport {
    object_name: String,
    outer_index: i32,
    outer_path: Option<String>,
    class_name: String,
    class_package: String,
}

#[derive(Debug, Deserialize)]
struct ExpectedExport {
    object_name: String,
    class_index: i32,
    class_name: String,
    super_index: i32,
    super_path: Option<String>,
    template_index: i32,
    template_path: Option<String>,
    outer_index: i32,
    outer_path: Option<String>,
    /// `0x` and 8 upper-case hex digits, as `PkgInfo` writes them.
    object_flags: String,
    serial_size: i64,
    serial_offset: i64,
    not_for_client: bool,
    not_for_server: bool,
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/editor")
        .join(name)
}

/// Reads the fixture header and its expected values.
fn load(package: &str, expected: &str) -> (PackageHeader, Expected) {
    let bytes = std::fs::read(fixture(package)).unwrap();
    // A Git LFS pointer file is small text, not a package.
    assert!(
        !bytes.starts_with(b"version https://git-lfs"),
        "{package} is a Git LFS pointer: run `git lfs pull`"
    );
    let header = read_header(&bytes).unwrap().header;
    let text = std::fs::read_to_string(fixture(expected)).unwrap();
    (header, serde_json::from_str(&text).unwrap())
}

fn check(package: &str, expected: &str) {
    let (header, expected) = load(package, expected);
    let path = |index| header.object_path(PackageIndex(index));

    let names: Vec<_> = header.names.iter().map(|name| name.text.clone()).collect();
    assert_eq!(names, expected.names);

    assert_eq!(header.imports.len(), expected.imports.len());
    for (i, (import, e)) in header.imports.iter().zip(&expected.imports).enumerate() {
        let index = -(i as i32) - 1;
        assert_eq!(
            header.object_name(PackageIndex(index)),
            Some(e.object_name.clone()),
            "import {i}"
        );
        assert_eq!(
            import.outer_index,
            PackageIndex(e.outer_index),
            "import {i}"
        );
        assert_eq!(path(e.outer_index), e.outer_path, "import {i}");
        assert_eq!(
            header.name_text(import.class_name),
            Some(e.class_name.clone()),
            "import {i}"
        );
        assert_eq!(
            header.name_text(import.class_package),
            Some(e.class_package.clone()),
            "import {i}"
        );
    }

    assert_eq!(header.exports.len(), expected.exports.len());
    for (i, (export, e)) in header.exports.iter().zip(&expected.exports).enumerate() {
        let index = i as i32 + 1;
        assert_eq!(
            header.object_name(PackageIndex(index)),
            Some(e.object_name.clone()),
            "export {i}"
        );
        assert_eq!(
            export.class_index,
            PackageIndex(e.class_index),
            "export {i}"
        );
        assert_eq!(
            header.class_name(PackageIndex(index)),
            Some(e.class_name.clone()),
            "export {i}"
        );
        assert_eq!(
            export.super_index,
            PackageIndex(e.super_index),
            "export {i}"
        );
        assert_eq!(path(e.super_index), e.super_path, "export {i}");
        assert_eq!(
            export.template_index,
            Some(PackageIndex(e.template_index)),
            "export {i}"
        );
        assert_eq!(path(e.template_index), e.template_path, "export {i}");
        assert_eq!(
            export.outer_index,
            PackageIndex(e.outer_index),
            "export {i}"
        );
        assert_eq!(path(e.outer_index), e.outer_path, "export {i}");
        assert_eq!(
            format!("0x{:08X}", export.object_flags),
            e.object_flags,
            "export {i}"
        );
        assert_eq!(export.serial_size, e.serial_size, "export {i}");
        assert_eq!(export.serial_offset, e.serial_offset, "export {i}");
        assert_eq!(export.not_for_client, e.not_for_client, "export {i}");
        assert_eq!(export.not_for_server, e.not_for_server, "export {i}");
    }
}

#[test]
fn bp_summary_actor_tables() {
    check("BP_SummaryActor.uasset", "BP_SummaryActor.expected.json");
}

#[test]
fn l_summary_level_tables() {
    check("L_SummaryLevel.umap", "L_SummaryLevel.expected.json");
}
