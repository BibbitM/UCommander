//! Builds synthetic package summaries, field by field.
//!
//! The layout follows `docs/formats/package.md`. The default values give a
//! valid summary of a UE 5.8 editor package.

#![allow(dead_code)]

/// The package file tag, little-endian.
pub const TAG: u32 = 0x9E2A83C1;
/// The byte-swapped package file tag.
pub const TAG_SWAPPED: u32 = 0xC1832A9E;

/// UE5 file version `PACKAGE_SAVED_HASH`.
pub const UE5_PACKAGE_SAVED_HASH: i32 = 1016;

/// The fields of a synthetic summary.
#[derive(Debug, Clone)]
pub struct Summary {
    pub tag: u32,
    pub legacy_file_version: i32,
    pub legacy_ue3_version: i32,
    pub file_version_ue4: i32,
    pub file_version_ue5: i32,
    pub file_version_licensee_ue: i32,
    pub saved_hash: [u8; 20],
    pub total_header_size: i32,
    /// GUID as (A, B, C, D) and version.
    pub custom_versions: Vec<([u32; 4], i32)>,
    /// When set, the custom version count is this value, not the list length.
    pub custom_version_count: Option<i32>,
}

impl Default for Summary {
    fn default() -> Self {
        Self {
            tag: TAG,
            legacy_file_version: -9,
            legacy_ue3_version: 864,
            file_version_ue4: 522,
            file_version_ue5: 1018,
            file_version_licensee_ue: 0,
            saved_hash: std::array::from_fn(|i| i as u8 + 1),
            total_header_size: 0x1234,
            custom_versions: vec![
                ([0x9C54D522, 0xA8264FBE, 0x94210746, 0x61B482D0], 52),
                ([0x11111111, 0x22222222, 0x33333333, 0x44444444], 7),
            ],
            custom_version_count: None,
        }
    }
}

impl Summary {
    /// Writes the fields in file order, with the version conditions.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut w = Writer::default();
        w.u32(self.tag);
        w.i32(self.legacy_file_version);
        if self.legacy_file_version != -4 {
            w.i32(self.legacy_ue3_version);
        }
        w.i32(self.file_version_ue4);
        if self.legacy_file_version <= -8 {
            w.i32(self.file_version_ue5);
        }
        w.i32(self.file_version_licensee_ue);
        if self.file_version_ue5 >= UE5_PACKAGE_SAVED_HASH {
            w.bytes(&self.saved_hash);
            w.i32(self.total_header_size);
        }
        let count = self
            .custom_version_count
            .unwrap_or(self.custom_versions.len() as i32);
        w.i32(count);
        for (guid, version) in &self.custom_versions {
            for part in guid {
                w.u32(*part);
            }
            w.i32(*version);
        }
        if self.file_version_ue5 < UE5_PACKAGE_SAVED_HASH {
            w.i32(self.total_header_size);
        }
        w.0
    }
}

/// Little-endian byte writer.
#[derive(Debug, Default)]
pub struct Writer(pub Vec<u8>);

impl Writer {
    pub fn u32(&mut self, value: u32) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }

    pub fn i32(&mut self, value: i32) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }

    pub fn bytes(&mut self, value: &[u8]) {
        self.0.extend_from_slice(value);
    }
}
