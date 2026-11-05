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

    /// The offset of the next value.
    pub(crate) fn offset(&self) -> usize {
        self.offset
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
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

    pub(crate) fn u32(&mut self) -> Result<u32, Error> {
        self.take().map(u32::from_le_bytes)
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
        let count = usize::try_from(count).map_err(|_| Error::BadValue { offset, field })?;
        if count.saturating_mul(element_size) > self.remaining() {
            return Err(Error::Truncated {
                offset: self.offset,
            });
        }
        Ok(count)
    }
}
