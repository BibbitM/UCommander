//! Bounds-checked reader of little-endian values.

use crate::error::Error;

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
}
