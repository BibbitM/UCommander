//! UCommanderContent: Total Commander content plugin (WDX) for Unreal Engine files.
//!
//! Each export calls its code through [`uc_tc::guard`], so a panic never
//! reaches TC. The API notes are in `docs/tc-api/content.md`.

#![allow(non_snake_case)]

use std::ffi::c_char;

use uc_tc::buffers::write_ansi;
use uc_tc::content::{DefaultParams, FieldType, NO_MORE_FIELDS};
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
