//! Builds synthetic package summaries, field by field.
//!
//! The layout follows `docs/formats/package.md`. The default values give a
//! valid summary of a UE 5.8 editor package.

#![allow(dead_code)]

/// The package file tag, little-endian.
pub const TAG: u32 = 0x9E2A83C1;
/// The byte-swapped package file tag.
pub const TAG_SWAPPED: u32 = 0xC1832A9E;

/// The fields of a synthetic summary.
#[derive(Debug, Clone)]
pub struct Summary {
    pub tag: u32,
    pub legacy_file_version: i32,
    pub legacy_ue3_version: i32,
    pub file_version_ue4: i32,
    pub file_version_ue5: i32,
    pub file_version_licensee_ue: i32,
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
}
