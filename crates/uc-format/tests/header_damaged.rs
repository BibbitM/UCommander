//! Package header of truncated and damaged fixtures: the result is an
//! error or a header, never a panic and never a large allocation.

use std::path::PathBuf;

use uc_format::Error;
use uc_format::package::{PackageHeader, PackageIndex, read_header};

const FIXTURES: [&str; 2] = ["BP_SummaryActor.uasset", "L_SummaryLevel.umap"];

/// Entry sizes of a UE 5.8 editor package. Source: `docs/formats/package.md`,
/// observed in fixture `BP_SummaryActor.uasset`.
const IMPORT_SIZE: usize = 40;
const EXPORT_SIZE: usize = 112;

fn fixture(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/editor")
        .join(name);
    std::fs::read(path).unwrap()
}

/// The start of the name map and the end of the last of the three tables.
fn table_range(bytes: &[u8]) -> (usize, usize) {
    let summary = read_header(bytes).unwrap().header.summary;
    let end = |offset: i32, count: i32, size: usize| offset as usize + count as usize * size;
    let imports_end = end(summary.imports.offset, summary.imports.count, IMPORT_SIZE);
    let exports_end = end(summary.exports.offset, summary.exports.count, EXPORT_SIZE);
    (summary.names.offset as usize, imports_end.max(exports_end))
}

fn i32_at(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

/// The size of a string, with its length.
fn string_size(bytes: &[u8], offset: usize) -> usize {
    let length = i32_at(bytes, offset);
    4 + if length >= 0 {
        length as usize
    } else {
        length.unsigned_abs() as usize * 2
    }
}

/// The offsets of the name count, the export count and the import count
/// in the summary of a UE 5.8 editor package. Source: "Summary fields" in
/// `docs/formats/package.md`.
fn table_count_offsets(bytes: &[u8]) -> [usize; 3] {
    // Tag, five versions, saved hash, total header size.
    let mut offset = 4 + 5 * 4 + 20 + 4;
    let custom_versions = i32_at(bytes, offset) as usize;
    offset += 4 + custom_versions * 20;
    offset += string_size(bytes, offset); // package name
    offset += 4; // package flags
    let names = offset;
    offset += 8 + 8; // names, soft object paths
    offset += string_size(bytes, offset); // localization ID
    offset += 8; // gatherable text data
    let exports = offset;
    let imports = offset + 8;
    [names, exports, imports]
}

/// Calls each name and path function for each index of the header.
fn use_all_objects(header: &PackageHeader) {
    let imports = header.imports.len() as i32;
    let exports = header.exports.len() as i32;
    for index in -imports - 1..=exports + 1 {
        let index = PackageIndex(index);
        let _ = header.object_name(index);
        let _ = header.class_name(index);
        let _ = header.object_path(index);
    }
}

#[test]
fn each_truncation_of_the_tables_gives_a_truncated_error() {
    for name in FIXTURES {
        let bytes = fixture(name);
        let (start, end) = table_range(&bytes);

        for len in start..end {
            let result = read_header(&bytes[..len]);

            assert!(
                matches!(result, Err(Error::Truncated { .. })),
                "{name}, length {len}: {result:?}"
            );
        }
        assert!(read_header(&bytes[..end]).is_ok(), "{name}");
    }
}

#[test]
fn a_very_large_table_count_gives_a_truncated_error() {
    for name in FIXTURES {
        let bytes = fixture(name);
        let summary = read_header(&bytes).unwrap().header.summary;
        let counts = [
            summary.names.count,
            summary.exports.count,
            summary.imports.count,
        ];
        let offsets = table_count_offsets(&bytes);
        assert_eq!(offsets.map(|offset| i32_at(&bytes, offset)), counts);

        for offset in offsets {
            let mut damaged = bytes.clone();
            damaged[offset..offset + 4].copy_from_slice(&i32::MAX.to_le_bytes());

            let result = read_header(&damaged);

            assert!(
                matches!(result, Err(Error::Truncated { .. })),
                "{name}, count at {offset}: {result:?}"
            );
        }
    }
}

/// Writes a very large, the smallest and a negative value at each offset
/// of the tables. This changes each name length, name index, package index
/// and size in turn.
#[test]
fn a_damaged_value_in_the_tables_does_not_panic() {
    for name in FIXTURES {
        let bytes = fixture(name);
        let (start, end) = table_range(&bytes);

        for offset in start..end - 3 {
            for value in [i32::MAX, i32::MIN, -1] {
                let mut damaged = bytes.clone();
                damaged[offset..offset + 4].copy_from_slice(&value.to_le_bytes());

                if let Ok(read) = read_header(&damaged) {
                    use_all_objects(&read.header);
                }
            }
        }
    }
}
