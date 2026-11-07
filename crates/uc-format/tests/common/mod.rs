//! Builds synthetic package summaries and tables, field by field.
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
    pub const NAME_HASHES_SERIALIZED: i32 = 504;
    pub const WORLD_LEVEL_INFO: i32 = 224;
    pub const ADDED_CHUNKID_TO_ASSETDATA_AND_UPACKAGE: i32 = 278;
    pub const CHANGED_CHUNKID_TO_BE_AN_ARRAY_OF_CHUNKIDS: i32 = 326;
    pub const ENGINE_VERSION_OBJECT: i32 = 336;
    pub const PACKAGE_SUMMARY_HAS_COMPATIBLE_ENGINE_VERSION: i32 = 444;
    pub const PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS: i32 = 507;
    pub const ADDED_PACKAGE_OWNER: i32 = 518;
    pub const NON_OUTER_PACKAGE_IMPORT: i32 = 520;
}

/// UE5 file versions.
pub mod ue5 {
    pub const INITIAL_VERSION: i32 = 1000;
    pub const NAMES_REFERENCED_FROM_EXPORT_DATA: i32 = 1001;
    pub const PAYLOAD_TOC: i32 = 1002;
    pub const DATA_RESOURCES: i32 = 1009;
    pub const ADD_SOFTOBJECTPATH_LIST: i32 = 1008;
    pub const METADATA_SERIALIZATION_OFFSET: i32 = 1014;
    pub const VERSE_CELLS: i32 = 1015;
    pub const PACKAGE_SAVED_HASH: i32 = 1016;
    pub const IMPORT_TYPE_HIERARCHIES: i32 = 1018;
}

/// An engine version: major, minor, patch, changelist, branch.
#[derive(Debug, Clone)]
pub struct EngineVersion(pub u16, pub u16, pub u16, pub u32, pub Str);

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
    pub package_guid: [u32; 4],
    pub persistent_guid: [u32; 4],
    pub owner_persistent_guid: [u32; 4],
    /// Export count and name count.
    pub generations: Vec<(i32, i32)>,
    pub saved_by_engine_version: EngineVersion,
    pub engine_changelist: i32,
    pub compatible_with_engine_version: EngineVersion,
    pub compression_flags: u32,
    /// The number of compressed chunks. Each chunk is 16 bytes of zeros.
    pub compressed_chunk_count: i32,
    pub package_source: u32,
    pub additional_packages_to_cook: Vec<Str>,
    /// When set, the additional package count is this value, not the list
    /// length.
    pub additional_package_count: Option<i32>,
    pub asset_registry_data_offset: i32,
    pub bulk_data_start_offset: i64,
    pub world_tile_info_data_offset: i32,
    /// The chunk IDs. Old versions store only the first one, or -1.
    pub chunk_ids: Vec<i32>,
    pub preload_dependency_count: i32,
    pub preload_dependency_offset: i32,
    pub names_referenced_from_export_data_count: i32,
    pub payload_toc_offset: i64,
    pub data_resource_offset: i32,
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
            package_guid: [0xA1, 0xA2, 0xA3, 0xA4],
            persistent_guid: [0xB1, 0xB2, 0xB3, 0xB4],
            owner_persistent_guid: [0xC1, 0xC2, 0xC3, 0xC4],
            generations: vec![(201, 202), (203, 204)],
            saved_by_engine_version: EngineVersion(
                5,
                8,
                1,
                0x8000_1234,
                Str::Ansi("++UE5+Release-5.8"),
            ),
            engine_changelist: 205,
            compatible_with_engine_version: EngineVersion(
                5,
                8,
                0,
                0x0000_5678,
                Str::Ansi("++UE5+Release-5.8"),
            ),
            compression_flags: 0,
            compressed_chunk_count: 0,
            package_source: 0xDEAD_BEEF,
            additional_packages_to_cook: vec![Str::Ansi("/Game/A"), Str::Utf16("/Game/B")],
            additional_package_count: None,
            asset_registry_data_offset: 206,
            bulk_data_start_offset: 0x1_0000_0207,
            world_tile_info_data_offset: 208,
            chunk_ids: vec![3, 4],
            preload_dependency_count: 209,
            preload_dependency_offset: 210,
            names_referenced_from_export_data_count: 211,
            payload_toc_offset: 0x2_0000_0212,
            data_resource_offset: 213,
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
        if v5 < ue5::PACKAGE_SAVED_HASH {
            w.guid(self.package_guid);
        }
        if self.package_flags & PKG_FILTER_EDITOR_ONLY == 0 && v4 >= ue4::ADDED_PACKAGE_OWNER {
            w.guid(self.persistent_guid);
            if v4 < ue4::NON_OUTER_PACKAGE_IMPORT {
                w.guid(self.owner_persistent_guid);
            }
        }
        w.i32(self.generations.len() as i32);
        for (export_count, name_count) in &self.generations {
            w.i32(*export_count);
            w.i32(*name_count);
        }
        if v4 >= ue4::ENGINE_VERSION_OBJECT {
            w.engine_version(&self.saved_by_engine_version);
        } else {
            w.i32(self.engine_changelist);
        }
        if v4 >= ue4::PACKAGE_SUMMARY_HAS_COMPATIBLE_ENGINE_VERSION {
            w.engine_version(&self.compatible_with_engine_version);
        }
        w.u32(self.compression_flags);
        w.i32(self.compressed_chunk_count);
        for _ in 0..self.compressed_chunk_count.max(0) {
            w.bytes(&[0; 16]);
        }
        w.u32(self.package_source);
        let count = self
            .additional_package_count
            .unwrap_or(self.additional_packages_to_cook.len() as i32);
        w.i32(count);
        for package in &self.additional_packages_to_cook {
            w.string(package);
        }
        w.i32(self.asset_registry_data_offset);
        w.0.extend_from_slice(&self.bulk_data_start_offset.to_le_bytes());
        if v4 >= ue4::WORLD_LEVEL_INFO {
            w.i32(self.world_tile_info_data_offset);
        }
        if v4 >= ue4::CHANGED_CHUNKID_TO_BE_AN_ARRAY_OF_CHUNKIDS {
            w.i32(self.chunk_ids.len() as i32);
            for id in &self.chunk_ids {
                w.i32(*id);
            }
        } else if v4 >= ue4::ADDED_CHUNKID_TO_ASSETDATA_AND_UPACKAGE {
            w.i32(self.chunk_ids.first().copied().unwrap_or(-1));
        }
        if v4 >= ue4::PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS {
            w.i32(self.preload_dependency_count);
            w.i32(self.preload_dependency_offset);
        }
        if v5 >= ue5::NAMES_REFERENCED_FROM_EXPORT_DATA {
            w.i32(self.names_referenced_from_export_data_count);
        }
        if v5 >= ue5::PAYLOAD_TOC {
            w.0.extend_from_slice(&self.payload_toc_offset.to_le_bytes());
        }
        if v5 >= ue5::DATA_RESOURCES {
            w.i32(self.data_resource_offset);
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

    pub fn guid(&mut self, value: [u32; 4]) {
        for part in value {
            self.u32(part);
        }
    }

    pub fn engine_version(&mut self, value: &EngineVersion) {
        for part in [value.0, value.1, value.2] {
            self.bytes(&part.to_le_bytes());
        }
        self.u32(value.3);
        self.string(&value.4);
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

/// One name map entry: text and the two hash values.
#[derive(Debug, Clone)]
pub struct Name(pub Str, pub u16, pub u16);

impl Name {
    /// An 8-bit name. The hash values come from the text length, so that
    /// each name has different values.
    pub fn ansi(text: &'static str) -> Self {
        let len = text.len() as u16;
        Self(Str::Ansi(text), 0x1000 + len, 0x2000 + len)
    }
}

/// A synthetic package: a summary, then the name map.
///
/// `to_bytes` sets the counts, the offsets and the total header size of
/// the summary from the tables, unless an override is set.
#[derive(Debug, Clone)]
pub struct Package {
    pub summary: Summary,
    pub names: Vec<Name>,
    /// When set, the name count is this value, not the list length.
    pub name_count: Option<i32>,
    /// When set, the name offset is this value, not the table position.
    pub name_offset: Option<i32>,
}

impl Package {
    pub fn new(names: &[&'static str]) -> Self {
        Self {
            summary: Summary::default(),
            names: names.iter().map(|text| Name::ansi(text)).collect(),
            name_count: None,
            name_offset: None,
        }
    }

    /// The size of the summary. The size does not change with the counts
    /// and offsets, because they all have a fixed size.
    pub fn summary_size(&self) -> usize {
        self.summary.to_bytes().len()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let start = self.summary_size();
        let mut tables = Writer::default();
        for Name(text, non_case_preserving, case_preserving) in &self.names {
            tables.string(text);
            if self.summary.file_version_ue4 >= ue4::NAME_HASHES_SERIALIZED {
                tables.bytes(&non_case_preserving.to_le_bytes());
                tables.bytes(&case_preserving.to_le_bytes());
            }
        }

        let mut summary = self.summary.clone();
        summary.name_count = self.name_count.unwrap_or(self.names.len() as i32);
        summary.name_offset = self.name_offset.unwrap_or(start as i32);
        summary.import_count = 0;
        summary.export_count = 0;
        summary.total_header_size = (start + tables.0.len()) as i32;

        let mut bytes = summary.to_bytes();
        bytes.extend_from_slice(&tables.0);
        bytes
    }
}
