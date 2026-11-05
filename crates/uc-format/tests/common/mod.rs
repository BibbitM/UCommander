//! Builds synthetic package summaries, field by field.
//!
//! The layout follows `docs/formats/package.md`. The default values give a
//! valid summary of a UE 5.8 editor package.

#![allow(dead_code)]

/// The package file tag, little-endian.
pub const TAG: u32 = 0x9E2A83C1;
/// The byte-swapped package file tag.
pub const TAG_SWAPPED: u32 = 0xC1832A9E;

/// Package flag `FilterEditorOnly`.
pub const PKG_FILTER_EDITOR_ONLY: u32 = 0x8000_0000;

/// UE4 file versions.
pub mod ue4 {
    pub const ADD_STRING_ASSET_REFERENCES_MAP: i32 = 384;
    pub const SERIALIZE_TEXT_IN_PACKAGES: i32 = 459;
    pub const ADDED_SEARCHABLE_NAMES: i32 = 510;
    pub const ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID: i32 = 516;
}

/// UE5 file versions.
pub mod ue5 {
    pub const ADD_SOFTOBJECTPATH_LIST: i32 = 1008;
    pub const METADATA_SERIALIZATION_OFFSET: i32 = 1014;
    pub const VERSE_CELLS: i32 = 1015;
    pub const PACKAGE_SAVED_HASH: i32 = 1016;
    pub const IMPORT_TYPE_HIERARCHIES: i32 = 1018;
}

/// A string value as it is stored in the file.
#[derive(Debug, Clone)]
pub enum Str {
    /// 8-bit characters with a null terminator.
    Ansi(&'static str),
    /// UTF-16 characters with a null terminator.
    Utf16(&'static str),
    /// Only the `i32` length, and no character data.
    Length(i32),
}

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
    pub package_name: Str,
    pub package_flags: u32,
    pub name_count: i32,
    pub name_offset: i32,
    pub soft_object_paths_count: i32,
    pub soft_object_paths_offset: i32,
    pub localization_id: Str,
    pub gatherable_text_data_count: i32,
    pub gatherable_text_data_offset: i32,
    pub export_count: i32,
    pub export_offset: i32,
    pub import_count: i32,
    pub import_offset: i32,
    pub cell_export_count: i32,
    pub cell_export_offset: i32,
    pub cell_import_count: i32,
    pub cell_import_offset: i32,
    pub metadata_offset: i32,
    pub depends_offset: i32,
    pub soft_package_references_count: i32,
    pub soft_package_references_offset: i32,
    pub searchable_names_offset: i32,
    pub thumbnail_table_offset: i32,
    pub import_type_hierarchies_count: i32,
    pub import_type_hierarchies_offset: i32,
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
            package_name: Str::Ansi("/Game/Fixtures/BP_Test"),
            package_flags: 0x0000_0400,
            // Each count and offset has a different value, so that a field
            // at a wrong position gives a wrong value.
            name_count: 101,
            name_offset: 102,
            soft_object_paths_count: 103,
            soft_object_paths_offset: 104,
            localization_id: Str::Ansi("0123456789ABCDEF"),
            gatherable_text_data_count: 105,
            gatherable_text_data_offset: 106,
            export_count: 107,
            export_offset: 108,
            import_count: 109,
            import_offset: 110,
            cell_export_count: 111,
            cell_export_offset: 112,
            cell_import_count: 113,
            cell_import_offset: 114,
            metadata_offset: 115,
            depends_offset: 116,
            soft_package_references_count: 117,
            soft_package_references_offset: 118,
            searchable_names_offset: 119,
            thumbnail_table_offset: 120,
            import_type_hierarchies_count: 121,
            import_type_hierarchies_offset: 122,
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
        if self.file_version_ue5 >= ue5::PACKAGE_SAVED_HASH {
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
        if self.file_version_ue5 < ue5::PACKAGE_SAVED_HASH {
            w.i32(self.total_header_size);
        }
        let (v4, v5) = (self.file_version_ue4, self.file_version_ue5);
        w.string(&self.package_name);
        w.u32(self.package_flags);
        w.i32(self.name_count);
        w.i32(self.name_offset);
        if v5 >= ue5::ADD_SOFTOBJECTPATH_LIST {
            w.i32(self.soft_object_paths_count);
            w.i32(self.soft_object_paths_offset);
        }
        if self.package_flags & PKG_FILTER_EDITOR_ONLY == 0
            && v4 >= ue4::ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
        {
            w.string(&self.localization_id);
        }
        if v4 >= ue4::SERIALIZE_TEXT_IN_PACKAGES {
            w.i32(self.gatherable_text_data_count);
            w.i32(self.gatherable_text_data_offset);
        }
        w.i32(self.export_count);
        w.i32(self.export_offset);
        w.i32(self.import_count);
        w.i32(self.import_offset);
        if v5 >= ue5::VERSE_CELLS {
            w.i32(self.cell_export_count);
            w.i32(self.cell_export_offset);
            w.i32(self.cell_import_count);
            w.i32(self.cell_import_offset);
        }
        if v5 >= ue5::METADATA_SERIALIZATION_OFFSET {
            w.i32(self.metadata_offset);
        }
        w.i32(self.depends_offset);
        if v4 >= ue4::ADD_STRING_ASSET_REFERENCES_MAP {
            w.i32(self.soft_package_references_count);
            w.i32(self.soft_package_references_offset);
        }
        if v4 >= ue4::ADDED_SEARCHABLE_NAMES {
            w.i32(self.searchable_names_offset);
        }
        w.i32(self.thumbnail_table_offset);
        if v5 >= ue5::IMPORT_TYPE_HIERARCHIES {
            w.i32(self.import_type_hierarchies_count);
            w.i32(self.import_type_hierarchies_offset);
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

    pub fn string(&mut self, value: &Str) {
        match value {
            Str::Ansi(text) => {
                self.i32(text.len() as i32 + 1);
                self.bytes(text.as_bytes());
                self.bytes(&[0]);
            }
            Str::Utf16(text) => {
                let units: Vec<u16> = text.encode_utf16().chain([0]).collect();
                self.i32(-(units.len() as i32));
                for unit in units {
                    self.bytes(&unit.to_le_bytes());
                }
            }
            Str::Length(length) => self.i32(*length),
        }
    }
}
