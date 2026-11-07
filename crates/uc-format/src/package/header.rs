//! Package header: the summary and the tables after it (name map, import
//! table). The layout is in `docs/formats/package.md`.

use super::summary::{PackageSummary, Table, Warning, read_summary};
use super::version::{ue4, ue5};
use crate::error::Error;
use crate::reader::Reader;

/// The longest name map entry, in characters with the null terminator
/// (`NAME_SIZE`).
const NAME_SIZE: u32 = 1024;
/// Smallest size of one name map entry: the `i32` length.
const MIN_NAME_SIZE: usize = 4;

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

/// The summary and the tables of a package.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct PackageHeader {
    pub summary: PackageSummary,
    pub names: Vec<NameEntry>,
    pub imports: Vec<ObjectImport>,
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
    let imports = read_imports(bytes, &summary, names.len())?;
    Ok(HeaderRead {
        header: PackageHeader {
            summary,
            names,
            imports,
        },
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
        let text = r.string_limited("name", NAME_SIZE)?;
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
    name_count: usize,
) -> Result<Vec<ObjectImport>, Error> {
    let has_package_name = summary.file_version_ue4 >= ue4::NON_OUTER_PACKAGE_IMPORT;
    let has_optional = summary.file_version_ue5 >= ue5::OPTIONAL_RESOURCES;
    let entry_size = 28 + 8 * usize::from(has_package_name) + 4 * usize::from(has_optional);
    let (mut r, count) = table_reader(bytes, summary.imports, entry_size, "import")?;
    // No capacity from the count: the entry size in memory is not
    // checked against its size in the file.
    let mut imports = Vec::new();
    for _ in 0..count {
        let class_package = name_ref(&mut r, name_count, "import class package")?;
        let class_name = name_ref(&mut r, name_count, "import class name")?;
        let outer_index = PackageIndex(r.i32()?);
        let object_name = name_ref(&mut r, name_count, "import object name")?;
        let package_name = has_package_name
            .then(|| name_ref(&mut r, name_count, "import package name"))
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
    Ok(imports)
}

/// Reads a name reference. A name index out of the name map is a bad
/// value.
fn name_ref(r: &mut Reader, name_count: usize, field: &'static str) -> Result<NameRef, Error> {
    let offset = r.offset();
    let index = r.i32()?;
    let number = r.i32()?;
    if usize::try_from(index).map_or(true, |index| index >= name_count) {
        return Err(Error::BadValue { offset, field });
    }
    Ok(NameRef { index, number })
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
