//! Basic value types of UE files.

use std::fmt;

/// A 128-bit GUID, as four `u32` values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Guid {
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub d: u32,
}

impl fmt::Display for Guid {
    /// Writes `AAAAAAAA-BBBBBBBB-CCCCCCCC-DDDDDDDD` in upper-case hex.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { a, b, c, d } = self;
        write!(f, "{a:08X}-{b:08X}-{c:08X}-{d:08X}")
    }
}

/// A 20-byte hash (`IoHash`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct IoHash(pub [u8; 20]);

impl fmt::Display for IoHash {
    /// Writes the bytes in lower-case hex.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|byte| write!(f, "{byte:02x}"))
    }
}
