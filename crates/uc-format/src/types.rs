//! Basic value types of UE files.

/// A 128-bit GUID, as four `u32` values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Guid {
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub d: u32,
}

/// A 20-byte hash (`IoHash`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct IoHash(pub [u8; 20]);
