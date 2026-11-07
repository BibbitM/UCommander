//! Package summary (`PackageFileSummary`): the header at the start of a
//! package file. The layout is in `docs/formats/package.md`.

use std::fmt;

use super::PACKAGE_FILE_TAG;
use super::version::{ue4, ue5};
use crate::error::Error;
use crate::reader::Reader;
use crate::types::{Guid, IoHash};

/// The byte-swapped package file tag.
const PACKAGE_FILE_TAG_SWAPPED: u32 = PACKAGE_FILE_TAG.swap_bytes();

/// Newest legacy file version in UE 5.8.
const LEGACY_FILE_VERSION_NEWEST: i32 = -9;
/// First legacy file version with the UE5 file version.
const LEGACY_FILE_VERSION_UE5: i32 = -8;

/// Package flag `FilterEditorOnly`: the package has no editor-only data.
pub(crate) const PKG_FILTER_EDITOR_ONLY: u32 = 0x8000_0000;

/// Size of one custom version entry: GUID and `i32`.
const CUSTOM_VERSION_SIZE: usize = 20;
/// Size of one generation: two `i32`.
const GENERATION_SIZE: usize = 8;
/// Smallest size of one string: the `i32` length.
const MIN_STRING_SIZE: usize = 4;

/// The fields of a package summary, in file order.
///
/// A field that the file has only for some versions is an `Option`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct PackageSummary {
    pub legacy_file_version: i32,
    /// UE ignores this value.
    pub legacy_ue3_version: i32,
    pub file_version_ue4: i32,
    pub file_version_ue5: i32,
    pub file_version_licensee_ue: i32,
    /// UE5 `PACKAGE_SAVED_HASH` and later.
    pub saved_hash: Option<IoHash>,
    pub total_header_size: i32,
    pub custom_versions: Vec<CustomVersion>,
    pub package_name: String,
    /// The raw flags, as in the file.
    pub package_flags: u32,
    pub names: Table,
    /// UE5 `ADD_SOFTOBJECTPATH_LIST` and later.
    pub soft_object_paths: Option<Table>,
    /// Not in packages with the `FilterEditorOnly` flag.
    pub localization_id: Option<String>,
    pub gatherable_text_data: Option<Table>,
    pub exports: Table,
    pub imports: Table,
    /// UE5 `VERSE_CELLS` and later.
    pub cell_exports: Option<Table>,
    /// UE5 `VERSE_CELLS` and later.
    pub cell_imports: Option<Table>,
    /// UE5 `METADATA_SERIALIZATION_OFFSET` and later.
    pub metadata_offset: Option<i32>,
    pub depends_offset: i32,
    pub soft_package_references: Option<Table>,
    pub searchable_names_offset: Option<i32>,
    pub thumbnail_table_offset: i32,
    /// UE5 `IMPORT_TYPE_HIERARCHIES` and later.
    pub import_type_hierarchies: Option<Table>,
    /// UE5 before `PACKAGE_SAVED_HASH`. UE copies it into the saved hash.
    pub package_guid: Option<Guid>,
    /// Not in packages with the `FilterEditorOnly` flag.
    pub persistent_guid: Option<Guid>,
    /// Only UE4 versions 518 and 519. UE ignores the value.
    pub owner_persistent_guid: Option<Guid>,
    pub generations: Vec<Generation>,
    pub saved_by_engine_version: Option<EngineVersion>,
    /// Only UE4 versions before `ENGINE_VERSION_OBJECT`.
    pub engine_changelist: Option<i32>,
    pub compatible_with_engine_version: Option<EngineVersion>,
    /// The raw flags, as in the file.
    pub compression_flags: u32,
    pub package_source: u32,
    /// UE does not use this list now.
    pub additional_packages_to_cook: Vec<String>,
    pub asset_registry_data_offset: i32,
    pub bulk_data_start_offset: i64,
    pub world_tile_info_data_offset: Option<i32>,
    pub chunk_ids: Vec<i32>,
    pub preload_dependencies: Option<Table>,
    /// UE5 `NAMES_REFERENCED_FROM_EXPORT_DATA` and later.
    pub names_referenced_from_export_data_count: Option<i32>,
    /// UE5 `PAYLOAD_TOC` and later.
    pub payload_toc_offset: Option<i64>,
    /// UE5 `DATA_RESOURCES` and later.
    pub data_resource_offset: Option<i32>,
}

/// The export count and the name count of one package generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Generation {
    pub export_count: i32,
    pub name_count: i32,
}

/// An engine version, as the summary stores it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EngineVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    /// The raw value. Bit 31 is the licensee flag.
    pub changelist: u32,
    pub branch: String,
}

impl EngineVersion {
    const LICENSEE_BIT: u32 = 0x8000_0000;

    /// `true` when a licensee build saved the package.
    pub fn is_licensee(&self) -> bool {
        self.changelist & Self::LICENSEE_BIT != 0
    }

    /// The changelist without the licensee flag.
    pub fn changelist_number(&self) -> u32 {
        self.changelist & !Self::LICENSEE_BIT
    }
}

impl fmt::Display for EngineVersion {
    /// Writes `major.minor.patch-changelist+branch`, as UE does. The
    /// changelist has no licensee flag.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            major,
            minor,
            patch,
            branch,
            ..
        } = self;
        let changelist = self.changelist_number();
        write!(f, "{major}.{minor}.{patch}-{changelist}+{branch}")
    }
}

/// The entry count and the file offset of a table.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Table {
    pub count: i32,
    pub offset: i32,
}

/// The version of one custom version key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CustomVersion {
    pub guid: Guid,
    pub version: i32,
}

/// A problem that does not stop the parser.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Warning {
    /// A version is newer than the newest known version. The parser uses
    /// the newest known layout.
    NewerVersion {
        field: &'static str,
        value: i32,
        newest: i32,
    },
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NewerVersion {
                field,
                value,
                newest,
            } => write!(
                f,
                "{field} {value} is newer than the newest known version {newest}"
            ),
        }
    }
}

/// The result of [`read_summary`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SummaryRead {
    pub summary: PackageSummary,
    /// The size of the summary in bytes.
    pub size: usize,
    pub warnings: Vec<Warning>,
}

/// Reads the package summary from the first bytes of a package file.
///
/// The function does no I/O. If `bytes` ends before the summary ends, the
/// error is [`Error::Truncated`]. Then the caller can give more bytes and
/// call the function again.
pub fn read_summary(bytes: &[u8]) -> Result<SummaryRead, Error> {
    let mut r = Reader::new(bytes);
    let mut s = PackageSummary::default();
    let mut warnings = Vec::new();

    match r.u32()? {
        PACKAGE_FILE_TAG => {}
        PACKAGE_FILE_TAG_SWAPPED => return Err(Error::ByteSwapped),
        _ => return Err(Error::NotAPackage),
    }

    s.legacy_file_version = r.i32()?;
    if !(LEGACY_FILE_VERSION_NEWEST..=LEGACY_FILE_VERSION_UE5).contains(&s.legacy_file_version) {
        return Err(unsupported("legacy file version", s.legacy_file_version));
    }
    // UE 5 legacy file versions are never -4, so the UE3 version is present.
    s.legacy_ue3_version = r.i32()?;
    s.file_version_ue4 = r.i32()?;
    s.file_version_ue5 = r.i32()?;
    s.file_version_licensee_ue = r.i32()?;

    if s.file_version_ue4 == 0 && s.file_version_ue5 == 0 && s.file_version_licensee_ue == 0 {
        return Err(Error::Unversioned);
    }
    if s.file_version_ue4 < ue4::OLDEST_LOADABLE_PACKAGE {
        return Err(unsupported("file version UE4", s.file_version_ue4));
    }
    if s.file_version_ue5 < ue5::INITIAL_VERSION {
        return Err(unsupported("file version UE5", s.file_version_ue5));
    }
    check_newer(
        "file version UE4",
        s.file_version_ue4,
        ue4::NEWEST,
        &mut warnings,
    );
    check_newer(
        "file version UE5",
        s.file_version_ue5,
        ue5::NEWEST,
        &mut warnings,
    );

    let has_saved_hash = s.file_version_ue5 >= ue5::PACKAGE_SAVED_HASH;
    if has_saved_hash {
        s.saved_hash = Some(r.io_hash()?);
        s.total_header_size = r.i32()?;
    }

    let count = r.array_count(CUSTOM_VERSION_SIZE, "custom version count")?;
    s.custom_versions = Vec::with_capacity(count);
    for _ in 0..count {
        s.custom_versions.push(CustomVersion {
            guid: r.guid()?,
            version: r.i32()?,
        });
    }

    if !has_saved_hash {
        s.total_header_size = r.i32()?;
    }

    read_tables(&mut r, &mut s)?;
    read_engine_data(&mut r, &mut s)?;
    read_end(&mut r, &mut s)?;

    Ok(SummaryRead {
        summary: s,
        size: r.offset(),
        warnings,
    })
}

/// Reads the fields from the package name to the import type hierarchies.
fn read_tables(r: &mut Reader, s: &mut PackageSummary) -> Result<(), Error> {
    let (v4, v5) = (s.file_version_ue4, s.file_version_ue5);
    s.package_name = r.string("package name")?;
    s.package_flags = r.u32()?;
    s.names = table(r)?;
    s.soft_object_paths = table_if(r, v5 >= ue5::ADD_SOFTOBJECTPATH_LIST)?;
    let editor_only_data = s.package_flags & PKG_FILTER_EDITOR_ONLY == 0;
    if editor_only_data && v4 >= ue4::ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID {
        s.localization_id = Some(r.string("localization ID")?);
    }
    s.gatherable_text_data = table_if(r, v4 >= ue4::SERIALIZE_TEXT_IN_PACKAGES)?;
    s.exports = table(r)?;
    s.imports = table(r)?;
    s.cell_exports = table_if(r, v5 >= ue5::VERSE_CELLS)?;
    s.cell_imports = table_if(r, v5 >= ue5::VERSE_CELLS)?;
    s.metadata_offset = i32_if(r, v5 >= ue5::METADATA_SERIALIZATION_OFFSET)?;
    s.depends_offset = r.i32()?;
    s.soft_package_references = table_if(r, v4 >= ue4::ADD_STRING_ASSET_REFERENCES_MAP)?;
    s.searchable_names_offset = i32_if(r, v4 >= ue4::ADDED_SEARCHABLE_NAMES)?;
    s.thumbnail_table_offset = r.i32()?;
    s.import_type_hierarchies = table_if(r, v5 >= ue5::IMPORT_TYPE_HIERARCHIES)?;
    Ok(())
}

/// Reads the fields from the package GUID to the compatible engine version.
fn read_engine_data(r: &mut Reader, s: &mut PackageSummary) -> Result<(), Error> {
    let v4 = s.file_version_ue4;
    if s.file_version_ue5 < ue5::PACKAGE_SAVED_HASH {
        s.package_guid = Some(r.guid()?);
    }
    let editor_only_data = s.package_flags & PKG_FILTER_EDITOR_ONLY == 0;
    if editor_only_data && v4 >= ue4::ADDED_PACKAGE_OWNER {
        s.persistent_guid = Some(r.guid()?);
        if v4 < ue4::NON_OUTER_PACKAGE_IMPORT {
            s.owner_persistent_guid = Some(r.guid()?);
        }
    }

    let count = r.array_count(GENERATION_SIZE, "generation count")?;
    s.generations = Vec::with_capacity(count);
    for _ in 0..count {
        s.generations.push(Generation {
            export_count: r.i32()?,
            name_count: r.i32()?,
        });
    }

    if v4 >= ue4::ENGINE_VERSION_OBJECT {
        s.saved_by_engine_version = Some(engine_version(r)?);
    } else {
        s.engine_changelist = Some(r.i32()?);
    }
    if v4 >= ue4::PACKAGE_SUMMARY_HAS_COMPATIBLE_ENGINE_VERSION {
        s.compatible_with_engine_version = Some(engine_version(r)?);
    }
    Ok(())
}

/// Reads the fields from the compression flags to the end of the summary.
fn read_end(r: &mut Reader, s: &mut PackageSummary) -> Result<(), Error> {
    let (v4, v5) = (s.file_version_ue4, s.file_version_ue5);
    s.compression_flags = r.u32()?;
    let offset = r.offset();
    match r.i32()? {
        0 => {}
        1.. => return Err(Error::Compressed),
        _ => {
            return Err(Error::BadValue {
                offset,
                field: "compressed chunk count",
            });
        }
    }
    s.package_source = r.u32()?;

    let count = r.array_count(MIN_STRING_SIZE, "additional packages to cook count")?;
    // No capacity from the count: a `String` is larger than its 4 input
    // bytes. The list grows only with the strings that the parser reads.
    s.additional_packages_to_cook = Vec::new();
    for _ in 0..count {
        let package = r.string("additional package to cook")?;
        s.additional_packages_to_cook.push(package);
    }

    // UE 5 packages have no texture allocations count: the legacy file
    // version is -8 or lower.
    s.asset_registry_data_offset = r.i32()?;
    s.bulk_data_start_offset = r.i64()?;
    s.world_tile_info_data_offset = i32_if(r, v4 >= ue4::WORLD_LEVEL_INFO)?;

    if v4 >= ue4::CHANGED_CHUNKID_TO_BE_AN_ARRAY_OF_CHUNKIDS {
        let count = r.array_count(4, "chunk ID count")?;
        s.chunk_ids = Vec::with_capacity(count);
        for _ in 0..count {
            s.chunk_ids.push(r.i32()?);
        }
    } else if v4 >= ue4::ADDED_CHUNKID_TO_ASSETDATA_AND_UPACKAGE {
        let chunk_id = r.i32()?;
        if chunk_id >= 0 {
            s.chunk_ids.push(chunk_id);
        }
    }

    s.preload_dependencies = table_if(r, v4 >= ue4::PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS)?;
    s.names_referenced_from_export_data_count =
        i32_if(r, v5 >= ue5::NAMES_REFERENCED_FROM_EXPORT_DATA)?;
    if v5 >= ue5::PAYLOAD_TOC {
        s.payload_toc_offset = Some(r.i64()?);
    }
    s.data_resource_offset = i32_if(r, v5 >= ue5::DATA_RESOURCES)?;
    Ok(())
}

fn engine_version(r: &mut Reader) -> Result<EngineVersion, Error> {
    Ok(EngineVersion {
        major: r.u16()?,
        minor: r.u16()?,
        patch: r.u16()?,
        changelist: r.u32()?,
        branch: r.string("engine version branch")?,
    })
}

fn table(r: &mut Reader) -> Result<Table, Error> {
    Ok(Table {
        count: r.i32()?,
        offset: r.i32()?,
    })
}

fn table_if(r: &mut Reader, present: bool) -> Result<Option<Table>, Error> {
    present.then(|| table(r)).transpose()
}

fn i32_if(r: &mut Reader, present: bool) -> Result<Option<i32>, Error> {
    present.then(|| r.i32()).transpose()
}

fn unsupported(field: &'static str, value: i32) -> Error {
    Error::UnsupportedVersion { field, value }
}

fn check_newer(field: &'static str, value: i32, newest: i32, warnings: &mut Vec<Warning>) {
    if value > newest {
        warnings.push(Warning::NewerVersion {
            field,
            value,
            newest,
        });
    }
}
