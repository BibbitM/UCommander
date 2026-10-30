//! UCommanderContent: Total Commander content plugin (WDX) for Unreal Engine files.
//!
//! Each export with code calls it through [`uc_tc::guard`], so a panic never
//! reaches TC. The API notes are in `docs/tc-api/content.md`.

#![allow(non_snake_case)]

use std::ffi::{c_char, c_void};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use uc_format::package::has_package_file_tag;
use uc_tc::buffers::{read_wide_path, write_ansi};
use uc_tc::content::{DefaultParams, FieldType, NO_MORE_FIELDS, ValueError};
use uc_tc::guard;

/// TC calls the plugin only for files that match this expression.
const DETECT_STRING: &str = r#"EXT="UASSET" | EXT="UMAP""#;

struct Field {
    name: &'static str,
    field_type: FieldType,
}

/// The fields in TC field index order. Names must stay the same after 1.0,
/// because users save them in column sets and searches.
const FIELDS: &[Field] = &[Field {
    name: "Is package",
    field_type: FieldType::Boolean,
}];

fn field(index: i32) -> Option<&'static Field> {
    usize::try_from(index)
        .ok()
        .and_then(|index| FIELDS.get(index))
}

/// # Safety
///
/// TC guarantees that `field_name` and `units` are valid for `maxlen` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn ContentGetSupportedField(
    field_index: i32,
    field_name: *mut c_char,
    units: *mut c_char,
    maxlen: i32,
) -> i32 {
    guard(NO_MORE_FIELDS, || {
        let Some(field) = field(field_index) else {
            return NO_MORE_FIELDS;
        };
        // SAFETY: TC guarantees `maxlen` writable bytes in both buffers.
        unsafe {
            write_ansi(field_name, maxlen, field.name);
            write_ansi(units, maxlen, "");
        }
        field.field_type.code()
    })
}

/// # Safety
///
/// TC guarantees that `detect_string` is valid for `maxlen` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn ContentGetDetectString(
    detect_string: *mut c_char,
    maxlen: i32,
) -> i32 {
    guard(0, || {
        // SAFETY: TC guarantees `maxlen` writable bytes.
        unsafe { write_ansi(detect_string, maxlen, DETECT_STRING) };
        0
    })
}

/// TC calls this function immediately after it loads the DLL. The plugin does
/// not use the parameters yet.
#[unsafe(no_mangle)]
pub extern "system" fn ContentSetDefaultParams(_params: *const DefaultParams) {}

/// TC calls this function just before it unloads the DLL. There is nothing to
/// release yet.
#[unsafe(no_mangle)]
pub extern "system" fn ContentPluginUnloading() {}

/// # Safety
///
/// TC guarantees that `file_name` is a NUL-terminated UTF-16 path, and that
/// `field_value` is valid for `maxlen` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn ContentGetValueW(
    file_name: *const u16,
    field_index: i32,
    _unit_index: i32,
    field_value: *mut c_void,
    maxlen: i32,
    _flags: i32,
) -> i32 {
    guard(ValueError::FileError.code(), || {
        let Some(field) = field(field_index) else {
            return ValueError::NoSuchField.code();
        };
        // SAFETY: TC guarantees a NUL-terminated UTF-16 path.
        let Some(path) = (unsafe { read_wide_path(file_name) }) else {
            return ValueError::FileError.code();
        };
        if field_value.is_null() || maxlen < size_of::<i32>() as i32 {
            return ValueError::FieldEmpty.code();
        }
        match is_package(&path) {
            Ok(value) => {
                // SAFETY: `field_value` is not null and has at least 4 bytes.
                // TC gives no alignment guarantee, so write unaligned.
                unsafe { field_value.cast::<i32>().write_unaligned(i32::from(value)) };
                field.field_type.code()
            }
            Err(_) => ValueError::FileError.code(),
        }
    })
}

/// Reads the first 4 bytes of the file, and checks the package file tag.
fn is_package(path: &Path) -> io::Result<bool> {
    let mut head = Vec::with_capacity(4);
    File::open(path)?.take(4).read_to_end(&mut head)?;
    Ok(has_package_file_tag(&head))
}
