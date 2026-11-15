//! Builds synthetic asset registry sections (`docs/formats/package.md`).

use super::{Export, Package, Str, Writer};

/// One object of the section, as it is stored in the file.
pub struct Object {
    pub path: Str,
    pub class: Str,
    pub tags: Vec<(Str, Str)>,
}

/// A placeholder for the dependency data offset. [`Package::to_bytes`]
/// writes the end of the section in its place, as UE does.
pub const SECTION_END: i64 = i64::MIN;

/// Writes the section: the dependency data offset (when present), the
/// object count, then the objects.
pub fn section(dependency_data_offset: Option<i64>, objects: &[Object]) -> Vec<u8> {
    let mut w = Writer::default();
    if let Some(offset) = dependency_data_offset {
        w.i64(offset);
    }
    w.i32(objects.len() as i32);
    for object in objects {
        w.string(&object.path);
        w.string(&object.class);
        w.i32(object.tags.len() as i32);
        for (key, value) in &object.tags {
            w.string(key);
            w.string(value);
        }
    }
    w.0
}

/// An editor package with one export and the asset registry `section`.
pub fn package(section: Vec<u8>) -> Package {
    Package {
        exports: vec![Export {
            object_name: (1, 0),
            ..Export::default()
        }],
        asset_registry: Some(section),
        ..Package::new(&["/Script/Engine", "Root"])
    }
}
