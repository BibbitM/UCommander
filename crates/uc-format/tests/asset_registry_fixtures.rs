//! Asset registry data of real fixtures, compared with the asset registry
//! section of their `PkgInfo` output (`.pkginfo.txt`). The test reads the
//! expected objects and tags from that UE output directly, because some
//! tag values (for example `FiBData`) contain control characters that a
//! hand copy cannot keep.

use std::collections::BTreeMap;
use std::path::PathBuf;

use uc_format::package::{read_asset_registry, read_header};

/// One object of the asset registry section of a `PkgInfo` output.
#[derive(Debug)]
struct PkgInfoObject {
    class_path: String,
    object_path: String,
    tags: BTreeMap<String, String>,
}

/// A fixture file. `name` is relative to `tests/fixtures`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

/// Reads the asset registry section of a `PkgInfo` output.
///
/// The section starts after the line `Asset Registry Data` and ends at a
/// line of `-`. An object line is `N) <class path>'<object path>' (K
/// Tags)`. A tag line is `"<key>": "<value>"`. The lines end with CR LF.
/// A tag value can contain LF or CR alone, but not CR LF.
fn pkginfo_objects(text: &str) -> Vec<PkgInfoObject> {
    let start = text
        .find("Asset Registry Data\r\n")
        .expect("no asset registry section");
    let mut objects: Vec<PkgInfoObject> = Vec::new();
    for line in text[start..].split("\r\n").skip(3) {
        let line = line.trim_start();
        if line.starts_with('-') {
            break;
        }
        if let Some(tag) = line.strip_prefix('"') {
            let (key, value) = tag.split_once("\": \"").expect("tag line");
            let value = value.strip_suffix('"').expect("tag value end");
            let object = objects.last_mut().expect("tag before object");
            object.tags.insert(key.to_owned(), value.to_owned());
        } else {
            let (_, rest) = line.split_once(") ").expect("object line");
            let (class_path, rest) = rest.split_once('\'').expect("class path");
            let (object_path, _) = rest.split_once('\'').expect("object path");
            objects.push(PkgInfoObject {
                class_path: class_path.to_owned(),
                object_path: object_path.to_owned(),
                tags: BTreeMap::new(),
            });
        }
    }
    objects
}

/// Checks a package with the asset registry section of its `PkgInfo`
/// output, which has the same name and the extension `.pkginfo.txt`.
fn check(package: &str) {
    let pkginfo = PathBuf::from(package).with_extension("pkginfo.txt");
    let bytes = std::fs::read(fixture(package)).unwrap();
    // A Git LFS pointer file is small text, not a package.
    assert!(
        !bytes.starts_with(b"version https://git-lfs"),
        "{package} is a Git LFS pointer: run `git lfs pull`"
    );
    let header = read_header(&bytes).unwrap().header;
    let text = std::fs::read_to_string(fixture(pkginfo.to_str().unwrap())).unwrap();
    let expected = pkginfo_objects(&text);

    let data = read_asset_registry(&bytes, &header.summary).unwrap();

    assert_eq!(data.objects.len(), expected.len());
    for (object, expected) in data.objects.iter().zip(&expected) {
        // The file has a path relative to the package, or a full path (an
        // external actor). `PkgInfo` shows the full path, with the
        // temporary mount point `/Temp` for a relative path.
        let full_path = if object.object_path.starts_with('/') {
            object.object_path.clone()
        } else {
            format!("{}.{}", header.summary.package_name, object.object_path)
        };
        let expected_path = expected.object_path.replacen("/Temp/", "/Game/", 1);
        assert_eq!(full_path, expected_path);
        assert_eq!(object.class_name, expected.class_path);
        // UE ignores a tag with an empty key or value, and `PkgInfo`
        // shows the tags in another order.
        let tags: BTreeMap<String, String> = object
            .tags
            .iter()
            .filter(|tag| !tag.key.is_empty() && !tag.value.is_empty())
            .map(|tag| (tag.key.clone(), tag.value.clone()))
            .collect();
        assert_eq!(tags, expected.tags, "{}", object.object_path);
    }
}

#[test]
fn bp_summary_actor_asset_registry() {
    check("editor/BP_SummaryActor.uasset");
}

#[test]
fn l_summary_level_asset_registry() {
    check("editor/L_SummaryLevel.umap");
}

#[test]
fn m_summary_asset_registry() {
    check("editor/M_Summary.uasset");
}

#[test]
fn ofpa_map_asset_registry() {
    check("editor/Ofpa/L_OfpaMap.umap");
}

#[test]
fn data_layer_asset_registry() {
    check("editor/Ofpa/DL_OfpaFixture.uasset");
}

#[test]
fn external_actors_asset_registry() {
    for package in OFPA_ACTORS {
        check(package);
    }
}

#[test]
fn external_objects_asset_registry() {
    check("editor/__ExternalObjects__/Ofpa/L_OfpaMap/6/F4/WX7Q02ROY466F4EAK3HYSA.uasset");
    check("editor/__ExternalObjects__/Ofpa/L_OfpaMap/7/BO/WCOVYDCVHSOAQI6426EICH.uasset");
}

/// The external actor packages of the OFPA fixtures (UE 5.8 and UE 5.7).
const OFPA_ACTORS: [&str; 11] = [
    "editor/__ExternalActors__/Ofpa/L_OfpaMap/1/5X/LDR80LGXHNUA4U4CBHWYNV.uasset",
    "editor/__ExternalActors__/Ofpa/L_OfpaMap/4/TV/YHL37N60V5A37O1U3ASTQ7.uasset",
    "editor/__ExternalActors__/Ofpa/L_OfpaMap/6/VM/S206VJ57URX2IZEGIAFTJ8.uasset",
    "editor/__ExternalActors__/Ofpa/L_OfpaMap/7/GM/CD2EGZM9LI8RXFXKWX2IC8.uasset",
    "editor/__ExternalActors__/Ofpa/L_OfpaMap/9/UY/KPHXLC180ULM321RTVNKBI.uasset",
    "editor/__ExternalActors__/Ofpa/L_OfpaMap/B/DH/DPUOQ7MHWJ6AFJ48NJZJ24.uasset",
    "editor/__ExternalActors__/Ofpa/L_OfpaMap/C/LG/7A5579ZXJXD32CJYP3WICE.uasset",
    "editor/__ExternalActors__/Ofpa/L_OfpaMap/C/N1/EXGB2JTMGHKCGRQDEZBUFA.uasset",
    "editor-5.7/__ExternalActors__/Ofpa/L_OfpaMap57/3/5F/UY7CAHIHPCHXPYHP6UAZC4.uasset",
    "editor-5.7/__ExternalActors__/Ofpa/L_OfpaMap57/5/2D/65F9D6M5A1KPQ0QFCEMT5H.uasset",
    "editor-5.7/__ExternalActors__/Ofpa/L_OfpaMap57/B/I4/4LDIAYPOR4RGIIJ2QNMPSY.uasset",
];
