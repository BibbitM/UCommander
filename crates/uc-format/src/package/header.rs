//! Package header: the summary and the tables after it (name map, import
//! table, export table). The layout is in `docs/formats/package.md`.

use super::objects::check_outer_loops;
use super::summary::{PackageSummary, Table, Warning, read_summary};
use super::version::{ue4, ue5};
use crate::error::Error;
use crate::reader::Reader;
use crate::types::Guid;

/// The longest name map entry, in characters with the null terminator
/// (`NAME_SIZE`).
const NAME_SIZE: u32 = 1024;
/// Smallest size of one name map entry: the `i32` length.
const MIN_NAME_SIZE: usize = 4;
/// Package flag `UnversionedProperties`.
const PKG_UNVERSIONED_PROPERTIES: u32 = 0x0000_2000;

/// One entry of the name map.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct NameEntry {
    pub text: String,
    /// UE4 `NAME_HASHES_SERIALIZED` and later. UE does not use it.
    pub non_case_preserving_hash: Option<u16>,
    /// UE4 `NAME_HASHES_SERIALIZED` and later. UE does not use it.
    pub case_preserving_hash: Option<u16>,
}

/// A name in the tables (`FName`): an index into the name map, and a
/// number.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct NameRef {
    pub index: i32,
    /// 0 means no number. Other values add `_<number - 1>` to the text.
    pub number: i32,
}

/// A reference to an object (`FPackageIndex`): 0 is null, `-(i + 1)` is
/// import `i`, `i + 1` is export `i`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct PackageIndex(pub i32);

/// One entry of the import table: an object in another package.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ObjectImport {
    pub class_package: NameRef,
    pub class_name: NameRef,
    pub outer_index: PackageIndex,
    pub object_name: NameRef,
    /// UE4 `NON_OUTER_PACKAGE_IMPORT` and later.
    pub package_name: Option<NameRef>,
    /// UE5 `OPTIONAL_RESOURCES` and later.
    pub import_optional: Option<bool>,
}

/// The preload dependency counts of an export, from the first export
/// dependency position.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExportDependencies {
    pub first_export_dependency: i32,
    pub serialization_before_serialization: i32,
    pub create_before_serialization: i32,
    pub serialization_before_create: i32,
    pub create_before_create: i32,
}

/// One entry of the export table: an object in this package.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ObjectExport {
    pub class_index: PackageIndex,
    pub super_index: PackageIndex,
    /// UE4 `TEMPLATE_INDEX_IN_COOKED_EXPORTS` and later.
    pub template_index: Option<PackageIndex>,
    pub outer_index: PackageIndex,
    pub object_name: NameRef,
    /// The raw flags, as in the file.
    pub object_flags: u32,
    pub serial_size: i64,
    pub serial_offset: i64,
    pub forced_export: bool,
    pub not_for_client: bool,
    pub not_for_server: bool,
    /// UE5 before `REMOVE_OBJECT_EXPORT_PACKAGE_GUID`. UE ignores it.
    pub package_guid: Option<Guid>,
    /// UE5 `TRACK_OBJECT_EXPORT_IS_INHERITED` and later.
    pub is_inherited_instance: Option<bool>,
    pub package_flags: u32,
    /// UE4 `LOAD_FOR_EDITOR_GAME` and later.
    pub not_always_loaded_for_editor_game: Option<bool>,
    /// UE4 `COOKED_ASSETS_IN_EDITOR_SUPPORT` and later.
    pub is_asset: Option<bool>,
    /// UE5 `OPTIONAL_RESOURCES` and later.
    pub generate_public_hash: Option<bool>,
    /// UE4 `PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS` and later.
    pub dependencies: Option<ExportDependencies>,
    /// UE5 `SCRIPT_SERIALIZATION_OFFSET` and later, without the package
    /// flag `UnversionedProperties`.
    pub script_serialization_start_offset: Option<i64>,
    /// The same condition as the start offset.
    pub script_serialization_end_offset: Option<i64>,
}

/// The summary and the tables of a package.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct PackageHeader {
    pub summary: PackageSummary,
    pub names: Vec<NameEntry>,
    pub imports: Vec<ObjectImport>,
    pub exports: Vec<ObjectExport>,
}

impl PackageHeader {
    /// Gives the display text of a name: the name map text, and
    /// `_<number - 1>` when the number is not 0. `None` when the index is
    /// not in the name map.
    pub fn name_text(&self, name: NameRef) -> Option<String> {
        let index = usize::try_from(name.index).ok()?;
        let text = &self.names.get(index)?.text;
        Some(match name.number {
            0 => text.clone(),
            number => format!("{text}_{}", i64::from(number) - 1),
        })
    }
}

/// The result of [`read_header`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HeaderRead {
    pub header: PackageHeader,
    /// The size of the summary in bytes.
    pub summary_size: usize,
    pub warnings: Vec<Warning>,
}

/// Reads the package summary and the tables from the first bytes of a
/// package file.
///
/// The function does no I/O. The tables are before the total header size
/// of the summary. If `bytes` ends before a table ends, the error is
/// [`Error::Truncated`]. Then the caller can give more bytes and call the
/// function again.
pub fn read_header(bytes: &[u8]) -> Result<HeaderRead, Error> {
    let read = read_summary(bytes)?;
    let summary = read.summary;
    let names = read_names(bytes, &summary)?;
    // A negative count gives an error when the parser reads its table.
    let limits = Limits {
        names: names.len(),
        imports: usize::try_from(summary.imports.count).unwrap_or(0),
        exports: usize::try_from(summary.exports.count).unwrap_or(0),
    };
    let (imports, import_entry_size) = read_imports(bytes, &summary, &limits)?;
    let (exports, export_entry_size) = read_exports(bytes, &summary, &limits)?;
    let header = PackageHeader {
        summary,
        names,
        imports,
        exports,
    };
    check_outer_loops(&header, import_entry_size, export_entry_size)?;
    Ok(HeaderRead {
        header,
        summary_size: read.size,
        warnings: read.warnings,
    })
}

fn read_names(bytes: &[u8], summary: &PackageSummary) -> Result<Vec<NameEntry>, Error> {
    let (mut r, count) = table_reader(bytes, summary.names, MIN_NAME_SIZE, "name")?;
    let has_hashes = summary.file_version_ue4 >= ue4::NAME_HASHES_SERIALIZED;
    // No capacity from the count: a `NameEntry` is larger than its 4
    // smallest input bytes. The list grows only with the names that the
    // parser reads.
    let mut names = Vec::new();
    for _ in 0..count {
        let mut text = r.string_limited("name", NAME_SIZE)?;
        // UE reads the name as a null-terminated string.
        if let Some(end) = text.find('\0') {
            text.truncate(end);
        }
        let (non_case_preserving_hash, case_preserving_hash) = if has_hashes {
            (Some(r.u16()?), Some(r.u16()?))
        } else {
            (None, None)
        };
        names.push(NameEntry {
            text,
            non_case_preserving_hash,
            case_preserving_hash,
        });
    }
    Ok(names)
}

fn read_imports(
    bytes: &[u8],
    summary: &PackageSummary,
    limits: &Limits,
) -> Result<(Vec<ObjectImport>, usize), Error> {
    let has_package_name = summary.file_version_ue4 >= ue4::NON_OUTER_PACKAGE_IMPORT;
    let has_optional = summary.file_version_ue5 >= ue5::OPTIONAL_RESOURCES;
    let entry_size = 28 + 8 * usize::from(has_package_name) + 4 * usize::from(has_optional);
    let (mut r, count) = table_reader(bytes, summary.imports, entry_size, "import")?;
    // No capacity from the count: the entry size in memory is not
    // checked against its size in the file.
    let mut imports = Vec::new();
    for _ in 0..count {
        let class_package = limits.name_ref(&mut r, "import class package")?;
        let class_name = limits.name_ref(&mut r, "import class name")?;
        let outer_index = limits.package_index(&mut r, "import outer index")?;
        let object_name = limits.name_ref(&mut r, "import object name")?;
        let package_name = has_package_name
            .then(|| limits.name_ref(&mut r, "import package name"))
            .transpose()?;
        let import_optional = has_optional.then(|| r.bool()).transpose()?;
        imports.push(ObjectImport {
            class_package,
            class_name,
            outer_index,
            object_name,
            package_name,
            import_optional,
        });
    }
    Ok((imports, entry_size))
}

fn read_exports(
    bytes: &[u8],
    summary: &PackageSummary,
    limits: &Limits,
) -> Result<(Vec<ObjectExport>, usize), Error> {
    let (v4, v5) = (summary.file_version_ue4, summary.file_version_ue5);
    let has_template = v4 >= ue4::TEMPLATE_INDEX_IN_COOKED_EXPORTS;
    let has_64bit_sizes = v4 >= ue4::EXPORTMAP_SERIALSIZES_64BIT;
    let has_package_guid = v5 < ue5::REMOVE_OBJECT_EXPORT_PACKAGE_GUID;
    let has_inherited = v5 >= ue5::TRACK_OBJECT_EXPORT_IS_INHERITED;
    let has_editor_game = v4 >= ue4::LOAD_FOR_EDITOR_GAME;
    let has_is_asset = v4 >= ue4::COOKED_ASSETS_IN_EDITOR_SUPPORT;
    let has_public_hash = v5 >= ue5::OPTIONAL_RESOURCES;
    let has_dependencies = v4 >= ue4::PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS;
    let has_script_offsets = v5 >= ue5::SCRIPT_SERIALIZATION_OFFSET
        && summary.package_flags & PKG_UNVERSIONED_PROPERTIES == 0;
    let entry_size: usize = 40
        + [
            (has_template, 4),
            (has_64bit_sizes, 8),
            (has_package_guid, 16),
            (has_inherited, 4),
            (has_editor_game, 4),
            (has_is_asset, 4),
            (has_public_hash, 4),
            (has_dependencies, 20),
            (has_script_offsets, 16),
        ]
        .iter()
        .filter(|(present, _)| *present)
        .map(|(_, size)| size)
        .sum::<usize>();
    let (mut r, count) = table_reader(bytes, summary.exports, entry_size, "export")?;
    // No capacity from the count: the entry size in memory is not
    // checked against its size in the file.
    let mut exports = Vec::new();
    for _ in 0..count {
        let class_index = limits.package_index(&mut r, "export class index")?;
        let super_index = limits.package_index(&mut r, "export super index")?;
        let template_index = has_template
            .then(|| limits.package_index(&mut r, "export template index"))
            .transpose()?;
        let outer_index = limits.package_index(&mut r, "export outer index")?;
        let object_name = limits.name_ref(&mut r, "export object name")?;
        let object_flags = r.u32()?;
        let (serial_size, serial_offset) = if has_64bit_sizes {
            (r.i64()?, r.i64()?)
        } else {
            (i64::from(r.i32()?), i64::from(r.i32()?))
        };
        let forced_export = r.bool()?;
        let not_for_client = r.bool()?;
        let not_for_server = r.bool()?;
        let package_guid = has_package_guid.then(|| r.guid()).transpose()?;
        let is_inherited_instance = has_inherited.then(|| r.bool()).transpose()?;
        let package_flags = r.u32()?;
        let not_always_loaded_for_editor_game = has_editor_game.then(|| r.bool()).transpose()?;
        let is_asset = has_is_asset.then(|| r.bool()).transpose()?;
        let generate_public_hash = has_public_hash.then(|| r.bool()).transpose()?;
        let dependencies = has_dependencies
            .then(|| export_dependencies(&mut r))
            .transpose()?;
        let (script_serialization_start_offset, script_serialization_end_offset) =
            if has_script_offsets {
                (Some(r.i64()?), Some(r.i64()?))
            } else {
                (None, None)
            };
        exports.push(ObjectExport {
            class_index,
            super_index,
            template_index,
            outer_index,
            object_name,
            object_flags,
            serial_size,
            serial_offset,
            forced_export,
            not_for_client,
            not_for_server,
            package_guid,
            is_inherited_instance,
            package_flags,
            not_always_loaded_for_editor_game,
            is_asset,
            generate_public_hash,
            dependencies,
            script_serialization_start_offset,
            script_serialization_end_offset,
        });
    }
    Ok((exports, entry_size))
}

fn export_dependencies(r: &mut Reader) -> Result<ExportDependencies, Error> {
    Ok(ExportDependencies {
        first_export_dependency: r.i32()?,
        serialization_before_serialization: r.i32()?,
        create_before_serialization: r.i32()?,
        serialization_before_create: r.i32()?,
        create_before_create: r.i32()?,
    })
}

/// The entry counts of the tables, to check the indexes in the tables.
#[derive(Debug)]
struct Limits {
    names: usize,
    imports: usize,
    exports: usize,
}

impl Limits {
    /// Reads a name reference. A name index out of the name map is a bad
    /// value.
    fn name_ref(&self, r: &mut Reader, field: &'static str) -> Result<NameRef, Error> {
        let offset = r.offset();
        let index = r.i32()?;
        let number = r.i32()?;
        if usize::try_from(index).map_or(true, |index| index >= self.names) {
            return Err(Error::BadValue { offset, field });
        }
        Ok(NameRef { index, number })
    }

    /// Reads a package index. An index out of the import table or the
    /// export table is a bad value.
    fn package_index(&self, r: &mut Reader, field: &'static str) -> Result<PackageIndex, Error> {
        let offset = r.offset();
        let raw = r.i32()?;
        let value = i64::from(raw);
        let (position, count) = match value {
            0 => return Ok(PackageIndex(0)),
            ..0 => (-value - 1, self.imports),
            _ => (value - 1, self.exports),
        };
        if usize::try_from(position).map_or(true, |position| position >= count) {
            return Err(Error::BadValue { offset, field });
        }
        Ok(PackageIndex(raw))
    }
}

/// Gives a reader at the start of a table, and the checked entry count.
///
/// An empty table gives no reader position check: UE does not read its
/// offset. A negative count or offset is a bad value. The error offset is
/// the table offset, or 0 for a negative table offset, because the count
/// and the offset are values of the summary.
fn table_reader<'a>(
    bytes: &'a [u8],
    table: Table,
    min_entry_size: usize,
    field: &'static str,
) -> Result<(Reader<'a>, usize), Error> {
    if table.count == 0 {
        return Ok((Reader::at(bytes, 0), 0));
    }
    let offset = usize::try_from(table.offset).map_err(|_| Error::BadValue { offset: 0, field })?;
    let r = Reader::at(bytes, offset);
    let count = r.check_count(table.count, min_entry_size, offset, field)?;
    Ok((r, count))
}
