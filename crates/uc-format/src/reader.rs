//! Bounds-checked reader of little-endian values.

use crate::error::Error;
use crate::types::{Guid, IoHash};

/// Reads values from a byte slice. Each read checks the bounds first.
#[derive(Debug)]
pub(crate) struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    /// Creates a reader that starts at `offset`. The offset can be after
    /// the end of `bytes`: then the first read gives "truncated".
    pub(crate) fn at(bytes: &'a [u8], offset: usize) -> Self {
        Self { bytes, offset }
    }

    /// The offset of the next value.
    pub(crate) fn offset(&self) -> usize {
        self.offset
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.offset)
    }

    fn take<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        let truncated = Error::Truncated {
            offset: self.offset,
        };
        let rest = self.bytes.get(self.offset..).ok_or(truncated.clone())?;
        let value = *rest.first_chunk::<N>().ok_or(truncated)?;
        self.offset += N;
        Ok(value)
    }

    /// Takes the next `len` bytes, without a copy.
    fn slice(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let truncated = Error::Truncated {
            offset: self.offset,
        };
        let value = self
            .bytes
            .get(self.offset..)
            .and_then(|rest| rest.get(..len))
            .ok_or(truncated)?;
        self.offset += len;
        Ok(value)
    }

    /// Reads a string (`FString`). A positive length gives 8-bit
    /// characters, read as Latin-1. A negative length gives UTF-16
    /// characters. The last character is the null terminator, and is
    /// not part of the result. A UTF-16 error gives U+FFFD.
    pub(crate) fn string(&mut self, field: &'static str) -> Result<String, Error> {
        self.string_limited(field, u32::MAX)
    }

    /// Reads a string, as [`Self::string`]. A length with an absolute
    /// value greater than `max_length` is a bad value.
    pub(crate) fn string_limited(
        &mut self,
        field: &'static str,
        max_length: u32,
    ) -> Result<String, Error> {
        let offset = self.offset;
        let length = self.i32()?;
        if length.unsigned_abs() > max_length {
            return Err(Error::BadValue { offset, field });
        }
        if length >= 0 {
            let bytes = self.slice(length.unsigned_abs() as usize)?;
            let text = bytes.split_last().map_or(&[][..], |(_, text)| text);
            return Ok(text.iter().map(|&byte| char::from(byte)).collect());
        }
        if length == i32::MIN {
            return Err(Error::BadValue { offset, field });
        }
        // At least one character: the length is -1 or less.
        let bytes = self.slice(length.unsigned_abs() as usize * 2)?;
        let text = &bytes[..bytes.len() - 2];
        let units = text
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&pair| u16::from_le_bytes(pair));
        Ok(char::decode_utf16(units)
            .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect())
    }

    pub(crate) fn u32(&mut self) -> Result<u32, Error> {
        self.take().map(u32::from_le_bytes)
    }

    /// Reads a boolean: a `u32`. Each value other than 0 is true.
    pub(crate) fn bool(&mut self) -> Result<bool, Error> {
        self.u32().map(|value| value != 0)
    }

    pub(crate) fn u16(&mut self) -> Result<u16, Error> {
        self.take().map(u16::from_le_bytes)
    }

    pub(crate) fn i64(&mut self) -> Result<i64, Error> {
        self.take().map(i64::from_le_bytes)
    }

    pub(crate) fn i32(&mut self) -> Result<i32, Error> {
        self.take().map(i32::from_le_bytes)
    }

    pub(crate) fn guid(&mut self) -> Result<Guid, Error> {
        Ok(Guid {
            a: self.u32()?,
            b: self.u32()?,
            c: self.u32()?,
            d: self.u32()?,
        })
    }

    pub(crate) fn io_hash(&mut self) -> Result<IoHash, Error> {
        self.take().map(IoHash)
    }

    /// Reads an `i32` element count of an array, for elements of
    /// `element_size` bytes. A negative count is a bad value. If the
    /// elements cannot fit in the remaining input, the result is
    /// "truncated". So the caller can allocate `count` elements safely.
    pub(crate) fn array_count(
        &mut self,
        element_size: usize,
        field: &'static str,
    ) -> Result<usize, Error> {
        let offset = self.offset;
        let count = self.i32()?;
        self.check_count(count, element_size, offset, field)
    }

    /// Checks an element count from the file, for elements of
    /// `element_size` bytes from the current offset. A negative count is a
    /// bad value at `field_offset`. If the elements cannot fit in the
    /// remaining input, the result is "truncated".
    pub(crate) fn check_count(
        &self,
        count: i32,
        element_size: usize,
        field_offset: usize,
        field: &'static str,
    ) -> Result<usize, Error> {
        let count = usize::try_from(count).map_err(|_| Error::BadValue {
            offset: field_offset,
            field,
        })?;
        if count.saturating_mul(element_size) > self.remaining() {
            return Err(Error::Truncated {
                offset: self.offset,
            });
        }
        Ok(count)
    }
}
