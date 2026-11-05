//! Errors of the parsers.

use std::fmt;

/// An error that stops the parser.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The input ends before the data ends. `offset` is the start of the
    /// value that is not complete. More input can remove the error.
    Truncated { offset: usize },
    /// The input does not start with the package file tag.
    NotAPackage,
    /// The package uses the byte-swapped tag (big-endian platform).
    ByteSwapped,
    /// A version value that the parser does not support.
    UnsupportedVersion { field: &'static str, value: i32 },
    /// The package has no version numbers (cooked package).
    Unversioned,
    /// A value that cannot be correct, at `offset`.
    BadValue { offset: usize, field: &'static str },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { offset } => write!(f, "truncated at offset {offset}"),
            Self::NotAPackage => f.write_str("not a package (no package file tag)"),
            Self::ByteSwapped => f.write_str("byte-swapped package (not supported)"),
            Self::UnsupportedVersion { field, value } => {
                write!(f, "unsupported version: {field} is {value}")
            }
            Self::Unversioned => f.write_str("unversioned package (not supported)"),
            Self::BadValue { offset, field } => {
                write!(f, "bad value of {field} at offset {offset}")
            }
        }
    }
}

impl std::error::Error for Error {}
