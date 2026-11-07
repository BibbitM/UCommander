//! Reads the package header from a reader, with only the bytes that the
//! header needs.

use std::fmt;
use std::io::{self, Read};

use super::header::{HeaderRead, read_header};
use crate::error::Error;

/// The first read size. The parser asks for more if the header is larger.
const FIRST_READ_SIZE: usize = 64 * 1024;

/// The largest number of bytes that [`read_header_from`] reads. A real
/// header is much smaller, so a header that needs more bytes has a damaged
/// count, length or offset.
pub const MAX_HEADER_READ_SIZE: usize = 16 * 1024 * 1024;

/// An error of [`read_header_from`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ReadError {
    /// The reader gave an error.
    Io(io::Error),
    /// The parser gave an error.
    Parse(Error),
    /// The header needs more than [`MAX_HEADER_READ_SIZE`] bytes.
    HeaderTooLarge,
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(f),
            Self::Parse(error) => error.fmt(f),
            Self::HeaderTooLarge => {
                f.write_str("damaged package: the header is larger than 16 MiB")
            }
        }
    }
}

impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Parse(error) => Some(error),
            Self::HeaderTooLarge => None,
        }
    }
}

/// Reads the package header from the start of `reader`.
///
/// The function reads 64 KiB first. If the parser says "truncated", it
/// reads two times more bytes, until the reader ends or the read limit
/// [`MAX_HEADER_READ_SIZE`] is reached. It does not read the rest of the
/// file.
pub fn read_header_from(mut reader: impl Read) -> Result<HeaderRead, ReadError> {
    let mut bytes = Vec::new();
    let mut size = FIRST_READ_SIZE;
    loop {
        let missing = (size - bytes.len()) as u64;
        reader
            .by_ref()
            .take(missing)
            .read_to_end(&mut bytes)
            .map_err(ReadError::Io)?;
        match read_header(&bytes) {
            Err(Error::Truncated { .. }) if bytes.len() == size => {
                if size >= MAX_HEADER_READ_SIZE {
                    return Err(ReadError::HeaderTooLarge);
                }
                size = (size * 2).min(MAX_HEADER_READ_SIZE);
            }
            result => return result.map_err(ReadError::Parse),
        }
    }
}
