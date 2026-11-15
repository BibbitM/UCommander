//! OFPA actor data of real fixtures, compared with their `.expected.json`
//! files. The expected values come from `wp.Editor.DumpActorDescs`, the
//! editor and the `PkgInfo` output. The sources are in each file.

use std::path::PathBuf;

use serde::Deserialize;
use uc_format::package::{read_asset_registry, read_header};
use uc_format::world_partition::{OfpaActorRead, read_ofpa_actor};

#[derive(Debug, Deserialize)]
struct Expected {
    actor_guid: String,
    actor_label: String,
    actor_native_class_path: String,
    actor_blueprint_class_path: Option<String>,
    actor_path: String,
    map_path: String,
    asset_class: String,
    asset_class_path: String,
    package_path: String,
}

/// A fixture file. `name` is relative to `tests/fixtures`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

/// Reads the package header and the OFPA actor data of a fixture.
fn read(package: &str) -> (uc_format::package::PackageHeader, OfpaActorRead) {
    let bytes = std::fs::read(fixture(package)).unwrap();
    // A Git LFS pointer file is small text, not a package.
    assert!(
        !bytes.starts_with(b"version https://git-lfs"),
        "{package} is a Git LFS pointer: run `git lfs pull`"
    );
    let header = read_header(&bytes).unwrap().header;
    let data = read_asset_registry(&bytes, &header.summary).unwrap();
    let read = read_ofpa_actor(&data).unwrap();
    (header, read)
}

fn check(package: &str) {
    let path = fixture(package).with_extension("expected.json");
    let text = std::fs::read_to_string(&path).unwrap();
    let expected: Expected =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));

    let (header, read) = read(package);

    let actor = read.actor.unwrap_or_else(|| panic!("{package}: no actor"));
    let guid = actor.desc.guid;
    let guid = format!("{:08X}{:08X}{:08X}{:08X}", guid.a, guid.b, guid.c, guid.d);
    assert_eq!(guid, expected.actor_guid, "{package}");
    assert_eq!(actor.desc.label, Some(expected.actor_label), "{package}");
    assert_eq!(
        actor.desc.native_class_path, expected.actor_native_class_path,
        "{package}"
    );
    assert_eq!(
        actor.desc.base_class_path, expected.actor_blueprint_class_path,
        "{package}"
    );
    assert_eq!(actor.actor_path, expected.actor_path, "{package}");
    assert_eq!(actor.map_path, expected.map_path, "{package}");
    assert!(read.warnings.is_empty(), "{package}: {:?}", read.warnings);
    assert_eq!(
        header.asset_class(),
        Some(expected.asset_class),
        "{package}"
    );
    assert_eq!(
        header.asset_class_path(),
        Some(expected.asset_class_path),
        "{package}"
    );
    assert_eq!(
        header.summary.package_name, expected.package_path,
        "{package}"
    );
}

const MAP_58: &str = "editor/__ExternalActors__/Ofpa/L_OfpaMap";
const MAP_57: &str = "editor-5.7/__ExternalActors__/Ofpa/L_OfpaMap57";

#[test]
fn f5_point_light_with_a_utf16_label() {
    check(&format!("{MAP_58}/9/UY/KPHXLC180ULM321RTVNKBI.uasset"));
}

#[test]
fn f6_blueprint_actor_in_a_folder() {
    check(&format!("{MAP_58}/6/VM/S206VJ57URX2IZEGIAFTJ8.uasset"));
}

#[test]
fn f8_actor_that_is_not_spatially_loaded() {
    check(&format!("{MAP_58}/1/5X/LDR80LGXHNUA4U4CBHWYNV.uasset"));
}

#[test]
fn f9_actor_with_a_runtime_grid() {
    check(&format!("{MAP_58}/4/TV/YHL37N60V5A37O1U3ASTQ7.uasset"));
}

#[test]
fn f10_actor_of_a_child_blueprint() {
    check(&format!("{MAP_58}/7/GM/CD2EGZM9LI8RXFXKWX2IC8.uasset"));
}

#[test]
fn f11_actor_in_a_data_layer() {
    check(&format!("{MAP_58}/C/LG/7A5579ZXJXD32CJYP3WICE.uasset"));
}

#[test]
fn f12_world_data_layers() {
    check(&format!("{MAP_58}/C/N1/EXGB2JTMGHKCGRQDEZBUFA.uasset"));
}

#[test]
fn f13_mini_map() {
    check(&format!("{MAP_58}/B/DH/DPUOQ7MHWJ6AFJ48NJZJ24.uasset"));
}

#[test]
fn f7_ue_5_7_point_light() {
    check(&format!("{MAP_57}/B/I4/4LDIAYPOR4RGIIJ2QNMPSY.uasset"));
}

#[test]
fn f14_ue_5_7_world_data_layers() {
    check(&format!("{MAP_57}/3/5F/UY7CAHIHPCHXPYHP6UAZC4.uasset"));
}

#[test]
fn f15_ue_5_7_mini_map() {
    check(&format!("{MAP_57}/5/2D/65F9D6M5A1KPQ0QFCEMT5H.uasset"));
}

#[test]
fn other_packages_give_no_actor() {
    for package in [
        "editor/BP_SummaryActor.uasset",
        "editor/L_SummaryLevel.umap",
        "editor/M_Summary.uasset",
        "editor/Ofpa/L_OfpaMap.umap",
        "editor/Ofpa/DL_OfpaFixture.uasset",
        "editor/__ExternalObjects__/Ofpa/L_OfpaMap/6/F4/WX7Q02ROY466F4EAK3HYSA.uasset",
        "editor/__ExternalObjects__/Ofpa/L_OfpaMap/7/BO/WCOVYDCVHSOAQI6426EICH.uasset",
    ] {
        let (_, read) = read(package);

        assert_eq!(read.actor, None, "{package}");
        assert!(read.warnings.is_empty(), "{package}: {:?}", read.warnings);
    }
}
