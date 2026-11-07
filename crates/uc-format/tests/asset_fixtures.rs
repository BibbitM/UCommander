//! Asset class of real fixtures, compared with their `.expected.json`
//! files. The expected values come from the asset registry section of the
//! `PkgInfo` output: the class of the asset with the package name.

use std::path::PathBuf;

use serde::Deserialize;
use uc_format::package::read_header;

#[derive(Debug, Deserialize)]
struct Expected {
    asset_class: String,
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/editor")
        .join(name)
}

fn check(package: &str, expected: &str) {
    let bytes = std::fs::read(fixture(package)).unwrap();
    // A Git LFS pointer file is small text, not a package.
    assert!(
        !bytes.starts_with(b"version https://git-lfs"),
        "{package} is a Git LFS pointer: run `git lfs pull`"
    );
    let header = read_header(&bytes).unwrap().header;
    let text = std::fs::read_to_string(fixture(expected)).unwrap();
    let expected: Expected = serde_json::from_str(&text).unwrap();

    assert_eq!(header.asset_class(), Some(expected.asset_class));
}

#[test]
fn bp_summary_actor_asset_class() {
    check("BP_SummaryActor.uasset", "BP_SummaryActor.expected.json");
}

#[test]
fn l_summary_level_asset_class() {
    check("L_SummaryLevel.umap", "L_SummaryLevel.expected.json");
}
