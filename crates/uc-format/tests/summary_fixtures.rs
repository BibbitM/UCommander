//! Package summary of real fixtures, compared with their `.expected.json`
//! files. The expected values come from the `PkgInfo` commandlet of UE.

use std::path::PathBuf;

use serde::Deserialize;
use uc_format::Guid;
use uc_format::package::{SummaryRead, read_summary};

#[derive(Debug, Deserialize)]
struct Expected {
    file_size: u64,
    summary: ExpectedSummary,
}

#[derive(Debug, Deserialize)]
struct ExpectedSummary {
    file_version_ue5: i32,
    saved_by_engine_version: String,
    compatible_with_engine_version: String,
    /// Hex digits without a prefix, as `PkgInfo` writes them.
    package_flags: String,
    name_count: i32,
    name_offset: i32,
    import_count: i32,
    import_offset: i32,
    export_count: i32,
    export_offset: i32,
    compression_flags: u32,
    custom_versions: Vec<ExpectedCustomVersion>,
    saved_hash: String,
    persistent_guid: String,
    generations: Vec<ExpectedGeneration>,
    gatherable_text_data_count: i32,
}

#[derive(Debug, Deserialize)]
struct ExpectedCustomVersion {
    /// Lower-case hex with hyphens, as UE writes a custom version key.
    guid: String,
    version: i32,
}

#[derive(Debug, Deserialize)]
struct ExpectedGeneration {
    export_count: i32,
    name_count: i32,
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/editor")
        .join(name)
}

/// Reads the fixture, its summary and its expected values.
fn load(package: &str, expected: &str) -> (Vec<u8>, SummaryRead, Expected) {
    let bytes = std::fs::read(fixture(package)).unwrap();
    // A Git LFS pointer file is small text, not a package.
    assert!(
        !bytes.starts_with(b"version https://git-lfs"),
        "{package} is a Git LFS pointer: run `git lfs pull`"
    );
    let read = read_summary(&bytes).unwrap();
    let text = std::fs::read_to_string(fixture(expected)).unwrap();
    let expected: Expected = serde_json::from_str(&text).unwrap();
    (bytes, read, expected)
}

/// UE text form `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` (lower case).
fn hyphens_lower(guid: &Guid) -> String {
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:04x}{:08x}",
        guid.a,
        guid.b >> 16,
        guid.b & 0xFFFF,
        guid.c >> 16,
        guid.c & 0xFFFF,
        guid.d
    )
}

/// UE text form of 32 upper-case hex digits.
fn digits(guid: &Guid) -> String {
    format!("{:08X}{:08X}{:08X}{:08X}", guid.a, guid.b, guid.c, guid.d)
}

fn check(package: &str, expected: &str) {
    let (bytes, read, expected) = load(package, expected);
    let s = &read.summary;
    let e = &expected.summary;

    assert_eq!(bytes.len() as u64, expected.file_size);
    assert!(read.warnings.is_empty(), "{:?}", read.warnings);
    assert_eq!(s.file_version_ue5, e.file_version_ue5);
    assert_eq!(
        s.saved_by_engine_version.as_ref().unwrap().to_string(),
        e.saved_by_engine_version
    );
    assert_eq!(
        s.compatible_with_engine_version
            .as_ref()
            .unwrap()
            .to_string(),
        e.compatible_with_engine_version
    );
    assert_eq!(
        s.package_flags,
        u32::from_str_radix(&e.package_flags, 16).unwrap()
    );
    assert_eq!(
        (s.names.count, s.names.offset),
        (e.name_count, e.name_offset)
    );
    assert_eq!(
        (s.imports.count, s.imports.offset),
        (e.import_count, e.import_offset)
    );
    assert_eq!(
        (s.exports.count, s.exports.offset),
        (e.export_count, e.export_offset)
    );
    assert_eq!(s.compression_flags, e.compression_flags);

    let actual: Vec<(String, i32)> = s
        .custom_versions
        .iter()
        .map(|c| (hyphens_lower(&c.guid), c.version))
        .collect();
    let wanted: Vec<(String, i32)> = e
        .custom_versions
        .iter()
        .map(|c| (c.guid.clone(), c.version))
        .collect();
    assert_eq!(actual, wanted);

    assert_eq!(s.saved_hash.unwrap().to_string(), e.saved_hash);
    assert_eq!(digits(&s.persistent_guid.unwrap()), e.persistent_guid);
    let generations: Vec<(i32, i32)> = s
        .generations
        .iter()
        .map(|g| (g.export_count, g.name_count))
        .collect();
    let wanted: Vec<(i32, i32)> = e
        .generations
        .iter()
        .map(|g| (g.export_count, g.name_count))
        .collect();
    assert_eq!(generations, wanted);
    assert_eq!(
        s.gatherable_text_data.unwrap().count,
        e.gatherable_text_data_count
    );
}

#[test]
fn blueprint_fixture_gives_the_expected_summary() {
    check("BP_SummaryActor.uasset", "BP_SummaryActor.expected.json");
}

#[test]
fn level_fixture_gives_the_expected_summary() {
    check("L_SummaryLevel.umap", "L_SummaryLevel.expected.json");
}

#[test]
fn the_fixture_starts_with_the_little_endian_tag() {
    let bytes = std::fs::read(fixture("BP_SummaryActor.uasset")).unwrap();

    assert_eq!(bytes[..4], [0xC1, 0x83, 0x2A, 0x9E]);
}

#[test]
fn the_name_map_starts_after_the_summary() {
    for package in ["BP_SummaryActor.uasset", "L_SummaryLevel.umap"] {
        let bytes = std::fs::read(fixture(package)).unwrap();

        let read = read_summary(&bytes).unwrap();

        assert_eq!(read.size, read.summary.names.offset as usize, "{package}");
    }
}
