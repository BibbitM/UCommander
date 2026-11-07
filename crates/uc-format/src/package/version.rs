//! File version numbers. The values are in `docs/formats/package.md`.

/// UE4 file versions.
pub(crate) mod ue4 {
    pub(crate) const OLDEST_LOADABLE_PACKAGE: i32 = 214;
    pub(crate) const WORLD_LEVEL_INFO: i32 = 224;
    pub(crate) const ADDED_CHUNKID_TO_ASSETDATA_AND_UPACKAGE: i32 = 278;
    pub(crate) const CHANGED_CHUNKID_TO_BE_AN_ARRAY_OF_CHUNKIDS: i32 = 326;
    pub(crate) const ENGINE_VERSION_OBJECT: i32 = 336;
    pub(crate) const LOAD_FOR_EDITOR_GAME: i32 = 365;
    pub(crate) const ADD_STRING_ASSET_REFERENCES_MAP: i32 = 384;
    pub(crate) const PACKAGE_SUMMARY_HAS_COMPATIBLE_ENGINE_VERSION: i32 = 444;
    pub(crate) const SERIALIZE_TEXT_IN_PACKAGES: i32 = 459;
    pub(crate) const COOKED_ASSETS_IN_EDITOR_SUPPORT: i32 = 485;
    pub(crate) const NAME_HASHES_SERIALIZED: i32 = 504;
    pub(crate) const PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS: i32 = 507;
    pub(crate) const TEMPLATE_INDEX_IN_COOKED_EXPORTS: i32 = 508;
    pub(crate) const ADDED_SEARCHABLE_NAMES: i32 = 510;
    /// `64BIT_EXPORTMAP_SERIALSIZES` in UE.
    pub(crate) const EXPORTMAP_SERIALSIZES_64BIT: i32 = 511;
    pub(crate) const ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID: i32 = 516;
    pub(crate) const ADDED_PACKAGE_OWNER: i32 = 518;
    pub(crate) const NON_OUTER_PACKAGE_IMPORT: i32 = 520;
    pub(crate) const NEWEST: i32 = 522;
}

/// UE5 file versions.
pub(crate) mod ue5 {
    pub(crate) const INITIAL_VERSION: i32 = 1000;
    pub(crate) const NAMES_REFERENCED_FROM_EXPORT_DATA: i32 = 1001;
    pub(crate) const PAYLOAD_TOC: i32 = 1002;
    pub(crate) const OPTIONAL_RESOURCES: i32 = 1003;
    pub(crate) const REMOVE_OBJECT_EXPORT_PACKAGE_GUID: i32 = 1005;
    pub(crate) const TRACK_OBJECT_EXPORT_IS_INHERITED: i32 = 1006;
    pub(crate) const ADD_SOFTOBJECTPATH_LIST: i32 = 1008;
    pub(crate) const DATA_RESOURCES: i32 = 1009;
    pub(crate) const SCRIPT_SERIALIZATION_OFFSET: i32 = 1010;
    pub(crate) const METADATA_SERIALIZATION_OFFSET: i32 = 1014;
    pub(crate) const VERSE_CELLS: i32 = 1015;
    pub(crate) const PACKAGE_SAVED_HASH: i32 = 1016;
    pub(crate) const IMPORT_TYPE_HIERARCHIES: i32 = 1018;
    pub(crate) const NEWEST: i32 = 1018;
}
