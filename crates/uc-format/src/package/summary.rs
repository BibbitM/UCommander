//! Package summary (`PackageFileSummary`): the header at the start of a
//! package file. The layout is in `docs/formats/package.md`.

use std::fmt;

use super::PACKAGE_FILE_TAG;
use crate::error::Error;
use crate::reader::Reader;

/// The byte-swapped package file tag.
const PACKAGE_FILE_TAG_SWAPPED: u32 = PACKAGE_FILE_TAG.swap_bytes();

/// Newest legacy file version in UE 5.8.
const LEGACY_FILE_VERSION_NEWEST: i32 = -9;
/// First legacy file version with the UE5 file version.
const LEGACY_FILE_VERSION_UE5: i32 = -8;

/// UE4 file versions.
mod ue4 {
    pub(super) const OLDEST_LOADABLE_PACKAGE: i32 = 214;
    pub(super) const NEWEST: i32 = 522;
}

/// UE5 file versions.
mod ue5 {
    pub(super) const INITIAL_VERSION: i32 = 1000;
    pub(super) const NEWEST: i32 = 1018;
}

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

    Ok(SummaryRead {
        summary: s,
        size: r.offset(),
        warnings,
    })
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
