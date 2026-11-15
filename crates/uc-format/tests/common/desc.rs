//! Builds synthetic World Partition actor descriptors, with the UE 5.8.3
//! layout up to the label (`docs/formats/world-partition.md`).

use super::{Str, Writer};

pub const UE5_MAIN: [u32; 4] = [0x697DD581, 0xE64F41AB, 0xAA4A51EC, 0xBEB7B628];
pub const UE5_RELEASE: [u32; 4] = [0xD89B5E42, 0x24BD4D46, 0x8412ACA8, 0xDF641779];
pub const FORTNITE_MAIN: [u32; 4] = [0x601D1886, 0xAC644F84, 0xAA16D3DE, 0x0DEAC7D6];
pub const FORTNITE_SEASON: [u32; 4] = [0x5B4C06B7, 0x24634AF8, 0x805BBF70, 0xCDF5D0DD];
pub const ACTOR_GUID: [u32; 4] = [0x6413279F, 0x42A6D887, 0xDF5B61B2, 0x5DEB3B08];

/// A delta field: `None` writes flag 0, `Some` writes flag 1 and the value.
pub type Delta<T> = Option<T>;

/// A synthetic descriptor with the UE 5.8.3 layout up to the label.
pub struct Desc {
    pub custom_versions: Vec<([u32; 4], i32)>,
    pub is_default: u32,
    pub base_class: (Str, Str),
    pub native_class: (Str, Str),
    pub guid: [u32; 4],
    pub is_spatially_loaded: Delta<u32>,
    pub runtime_grid: Delta<Str>,
    pub is_editor_only: Delta<u32>,
    pub is_runtime_only: Delta<u32>,
    pub references: Vec<[u32; 4]>,
    pub editor_only_references: Vec<[u32; 4]>,
    pub tags: Delta<Vec<Str>>,
    pub data_layers: Delta<Vec<Str>>,
    pub is_using_data_layer_asset: Delta<u32>,
    pub label: Delta<Str>,
    /// Bytes after the label, which the parser does not read.
    pub rest: Vec<u8>,
}

impl Default for Desc {
    /// The F2 descriptor of `PointLight_1`: the newest UE 5.8.3 versions,
    /// a native class and a label.
    fn default() -> Self {
        Self {
            custom_versions: vec![
                (UE5_MAIN, 123),
                (FORTNITE_SEASON, 15),
                (FORTNITE_MAIN, 268),
                (UE5_RELEASE, 68),
            ],
            is_default: 0,
            base_class: (Str::Ansi("None"), Str::Ansi("None")),
            native_class: (Str::Ansi("/Script/Engine"), Str::Ansi("PointLight")),
            guid: ACTOR_GUID,
            is_spatially_loaded: None,
            runtime_grid: None,
            is_editor_only: None,
            is_runtime_only: None,
            references: Vec::new(),
            editor_only_references: Vec::new(),
            tags: None,
            data_layers: None,
            is_using_data_layer_asset: None,
            label: Some(Str::Ansi("PointLight")),
            rest: vec![0; 40],
        }
    }
}

impl Desc {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut w = Writer::default();
        w.i32(self.custom_versions.len() as i32);
        for (guid, version) in &self.custom_versions {
            w.guid(*guid);
            w.i32(*version);
        }
        w.u32(self.is_default);
        w.string(&self.base_class.0);
        w.string(&self.base_class.1);
        w.string(&self.native_class.0);
        w.string(&self.native_class.1);
        w.guid(self.guid);
        // Transform: identity rotation, no translation, scale 1.
        for value in [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0] {
            w.bytes(&f64::to_le_bytes(value));
        }
        // Runtime and editor bounds, relative.
        for _ in 0..2 {
            for value in [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1000.0, 1000.0, 1000.0] {
                w.bytes(&f64::to_le_bytes(value));
            }
            w.u32(1);
        }
        delta(&mut w, &self.is_spatially_loaded, |w, value| w.u32(*value));
        delta(&mut w, &self.runtime_grid, Writer::string);
        delta(&mut w, &self.is_editor_only, |w, value| w.u32(*value));
        delta(&mut w, &self.is_runtime_only, |w, value| w.u32(*value));
        for references in [&self.references, &self.editor_only_references] {
            w.i32(references.len() as i32);
            for guid in references {
                w.guid(*guid);
            }
        }
        delta(&mut w, &self.tags, names);
        delta(&mut w, &self.data_layers, names);
        delta(&mut w, &self.is_using_data_layer_asset, |w, value| {
            w.u32(*value)
        });
        delta(&mut w, &self.label, Writer::string);
        w.bytes(&self.rest);
        w.0
    }
}

/// Encodes bytes as Base64 text (RFC 4648, standard alphabet, with `=`
/// padding), as UE writes the `ActorMetaData` tag.
pub fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut text = String::new();
    for chunk in bytes.chunks(3) {
        let mut group = [0u8; 3];
        group[..chunk.len()].copy_from_slice(chunk);
        let bits = u32::from_be_bytes([0, group[0], group[1], group[2]]);
        for i in 0..4 {
            if i <= chunk.len() {
                let index = (bits >> (18 - 6 * i)) & 0x3F;
                text.push(char::from(ALPHABET[index as usize]));
            } else {
                text.push('=');
            }
        }
    }
    text
}

fn delta<T>(w: &mut Writer, value: &Delta<T>, write: impl Fn(&mut Writer, &T)) {
    match value {
        None => w.bytes(&[0]),
        Some(value) => {
            w.bytes(&[1]);
            write(w, value);
        }
    }
}

fn names(w: &mut Writer, names: &Vec<Str>) {
    w.i32(names.len() as i32);
    for name in names {
        w.string(name);
    }
}
