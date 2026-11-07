//! Package header: the summary and the tables after it (name map). The
//! layout is in `docs/formats/package.md`.

use super::summary::{PackageSummary, Table, Warning, read_summary};
use super::version::ue4;
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

/// The summary and the tables of a package.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct PackageHeader {
    pub summary: PackageSummary,
    pub names: Vec<NameEntry>,
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
    Ok(HeaderRead {
        header: PackageHeader { summary, names },
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
