//! Package summary of truncated and damaged fixtures: the result is an
//! error, never a panic and never a large allocation.

use std::path::PathBuf;

use uc_format::Error;
use uc_format::package::read_summary;

const FIXTURES: [&str; 3] = [
    "BP_SummaryActor.uasset",
    "L_SummaryLevel.umap",
    "M_Summary.uasset",
];

fn fixture(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/editor")
        .join(name);
    std::fs::read(path).unwrap()
}

/// The offset of the custom version count: tag, five `i32` versions,
/// saved hash (20 bytes) and total header size. Source:
/// `docs/formats/package.md`.
const CUSTOM_VERSION_COUNT_OFFSET: usize = 4 + 5 * 4 + 20 + 4;

fn with_i32(bytes: &[u8], offset: usize, value: i32) -> Vec<u8> {
    let mut bytes = bytes.to_vec();
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    bytes
}

#[test]
fn each_truncation_of_a_fixture_gives_a_truncated_error() {
    for name in FIXTURES {
        let bytes = fixture(name);
        let size = read_summary(&bytes).unwrap().size;

        for len in 0..size {
            match read_summary(&bytes[..len]) {
                Err(Error::Truncated { offset }) => {
                    assert!(offset <= len, "{name}, length {len}")
                }
                other => panic!("{name}, length {len}: {other:?}"),
            }
        }
    }
}

#[test]
fn a_very_large_custom_version_count_gives_a_truncated_error() {
    for name in FIXTURES {
        let bytes = fixture(name);
        let damaged = with_i32(&bytes, CUSTOM_VERSION_COUNT_OFFSET, i32::MAX);

        assert!(
            matches!(read_summary(&damaged), Err(Error::Truncated { .. })),
            "{name}"
        );
    }
}

/// Writes a very large, a negative and the smallest value at each offset of
/// the summary. This changes each count and each string length in turn.
/// The parser must give a result in the bounds of the input, and must not
/// panic.
#[test]
fn a_damaged_value_at_each_offset_gives_no_panic() {
    for name in FIXTURES {
        let bytes = fixture(name);
        let size = read_summary(&bytes).unwrap().size;

        for offset in 0..size - 3 {
            for value in [i32::MAX, 0x7FFF_0000, -1, i32::MIN] {
                let damaged = with_i32(&bytes, offset, value);

                if let Ok(read) = read_summary(&damaged) {
                    assert!(read.size <= damaged.len(), "{name}, offset {offset}");
                }
            }
        }
    }
}
