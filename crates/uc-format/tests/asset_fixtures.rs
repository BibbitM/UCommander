//! Main asset values of real fixtures, compared with their `.expected.json`
//! files. The asset class and the asset name come from the asset registry
//! section of the `PkgInfo` output. The paths come from the UE editor
//! ("Copy Reference" and "Copy Package Path").

use std::path::PathBuf;

use serde::Deserialize;
use uc_format::package::read_header;

#[derive(Debug, Deserialize)]
struct Expected {
    asset_class: String,
    asset_class_path: String,
    asset_name: String,
    object_path: String,
    package_path: String,
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
    assert_eq!(header.asset_class_path(), Some(expected.asset_class_path));
    assert_eq!(header.asset_name(), Some(expected.asset_name));
    assert_eq!(header.asset_object_path(), Some(expected.object_path));
    assert_eq!(header.summary.package_name, expected.package_path);
}

#[test]
fn bp_summary_actor_main_asset() {
    check("BP_SummaryActor.uasset", "BP_SummaryActor.expected.json");
}

#[test]
fn l_summary_level_main_asset() {
    check("L_SummaryLevel.umap", "L_SummaryLevel.expected.json");
}

#[test]
fn m_summary_main_asset() {
    check("M_Summary.uasset", "M_Summary.expected.json");
}
