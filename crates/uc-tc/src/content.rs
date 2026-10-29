//! Types of the TC content plugin API (WDX).
//!
//! Source: Content-Plugin Guide 2.12. See `docs/tc-api/content.md`.

use std::ffi::c_char;

/// Return value of `ContentGetSupportedField` after the last field.
pub const NO_MORE_FIELDS: i32 = 0;

/// `flags` bit of `ContentGetValueW`: TC calls in the foreground, and a slow
/// field can return `ft_delayed`.
pub const CONTENT_DELAYIFSLOW: i32 = 1;

/// Field type (`ft_…`). The return value of `ContentGetSupportedField`, and of
/// `ContentGetValueW` on success.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    Numeric32 = 1,
    Numeric64 = 2,
    NumericFloating = 3,
    Date = 4,
    Time = 5,
    Boolean = 6,
    MultipleChoice = 7,
    String = 8,
    FullText = 9,
    DateTime = 10,
    StringW = 11,
    FullTextW = 12,
}

impl FieldType {
    /// The value that TC expects.
    pub const fn code(self) -> i32 {
        self as i32
    }
}

/// Error return value of `ContentGetValueW`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueError {
    /// The field index is not valid.
    NoSuchField = -1,
    /// File I/O error.
    FileError = -2,
    /// The field is valid, but the file has no value for it.
    FieldEmpty = -3,
    /// Get the value only when the user presses Space.
    OnDemand = -4,
    /// The function is not supported.
    NotSupported = -5,
    /// The user cancelled the field editor.
    SetCancel = -6,
}

impl ValueError {
    /// The value that TC expects.
    pub const fn code(self) -> i32 {
        self as i32
    }
}

/// Parameter of `ContentSetDefaultParams` (`ContentDefaultParamStruct`).
#[repr(C)]
#[derive(Debug)]
pub struct DefaultParams {
    /// Size of the structure in bytes.
    pub size: i32,
    /// Interface version digits after the point × 100 (2.12 → 12).
    pub version_low: u32,
    /// Interface version digits before the point (2.12 → 2).
    pub version_high: u32,
    /// Suggested INI file path (ANSI, NUL-terminated).
    pub default_ini_name: [c_char; 260],
}
