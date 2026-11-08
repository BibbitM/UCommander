//! UCommanderContent: Total Commander content plugin (WDX) for Unreal Engine files.
//!
//! Each export with code calls it through [`uc_tc::guard`], so a panic never
//! reaches TC. The API notes are in `docs/tc-api/content.md`.

#![allow(non_snake_case)]

mod cache;

use std::ffi::{c_char, c_void};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use cache::{PackageValues, package_values};
use uc_format::package::has_package_file_tag;
use uc_tc::buffers::{read_wide_path, write_ansi, write_utf16};
use uc_tc::content::{DefaultParams, FieldType, NO_MORE_FIELDS, ValueError};
use uc_tc::guard;

/// TC calls the plugin only for files that match this expression.
const DETECT_STRING: &str = r#"EXT="UASSET" | EXT="UMAP""#;

struct Field {
    name: &'static str,
    /// The type that `ContentGetSupportedField` gives.
    field_type: FieldType,
    /// Gives the value of the field for a file. `None` when the file has no
    /// value. An error is an I/O error.
    value: fn(&Path) -> io::Result<Option<Value>>,
}

/// A field value, before the plugin writes it into the TC buffer.
enum Value {
    Boolean(bool),
    Numeric32(i32),
    /// Written as UTF-16 (`ft_stringw`). The field declares `ft_string`.
    Text(String),
}

/// The fields in TC field index order. Names must stay the same after 1.0,
/// because users save them in column sets and searches.
const FIELDS: &[Field] = &[
    Field {
        name: "Is package",
        field_type: FieldType::Boolean,
        value: is_package,
    },
    Field {
        name: "Asset class",
        field_type: FieldType::String,
        value: asset_class,
    },
    Field {
        name: "Asset class path",
        field_type: FieldType::String,
        value: asset_class_path,
    },
    Field {
        name: "Asset name",
        field_type: FieldType::String,
        value: asset_name,
    },
    Field {
        name: "Object path",
        field_type: FieldType::String,
        value: object_path,
    },
    Field {
        name: "Package path",
        field_type: FieldType::String,
        value: package_path,
    },
    Field {
        name: "Engine version",
        field_type: FieldType::String,
        value: engine_version,
    },
    Field {
        name: "Full engine version",
        field_type: FieldType::String,
        value: full_engine_version,
    },
    Field {
        name: "File version UE5",
        field_type: FieldType::Numeric32,
        value: file_version_ue5,
    },
    Field {
        name: "File version UE4",
        field_type: FieldType::Numeric32,
        value: file_version_ue4,
    },
    Field {
        name: "File version licensee",
        field_type: FieldType::Numeric32,
        value: file_version_licensee,
    },
];

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
        if field_value.is_null() {
            return ValueError::FieldEmpty.code();
        }
        match (field.value)(&path) {
            // SAFETY: `field_value` is not null, and TC guarantees `maxlen`
            // writable bytes.
            Ok(Some(value)) => unsafe { write_value(&value, field_value, maxlen) },
            Ok(None) => ValueError::FieldEmpty.code(),
            Err(_) => ValueError::FileError.code(),
        }
    })
}

/// Writes `value` in the format of its type, and gives the return value of
/// `ContentGetValueW`. A buffer that is too small gives `ft_fieldempty`.
///
/// # Safety
///
/// `field_value` must not be null, and must be valid for writes of
/// `maxlen` bytes. It can be unaligned.
unsafe fn write_value(value: &Value, field_value: *mut c_void, maxlen: i32) -> i32 {
    match *value {
        Value::Boolean(value) => {
            // SAFETY: the caller guarantees `maxlen` writable bytes.
            unsafe { write_i32(field_value, maxlen, i32::from(value), FieldType::Boolean) }
        }
        Value::Numeric32(value) => {
            // SAFETY: the caller guarantees `maxlen` writable bytes.
            unsafe { write_i32(field_value, maxlen, value, FieldType::Numeric32) }
        }
        Value::Text(ref text) => {
            // One UTF-16 unit for the NUL at least.
            if maxlen < size_of::<u16>() as i32 {
                return ValueError::FieldEmpty.code();
            }
            // SAFETY: the caller guarantees `maxlen` writable bytes.
            unsafe { write_utf16(field_value.cast(), maxlen, text) };
            FieldType::StringW.code()
        }
    }
}

/// Writes an `i32` value, and gives `field_type`. A buffer smaller than 4
/// bytes gives `ft_fieldempty`, and the plugin writes nothing.
///
/// # Safety
///
/// `field_value` must not be null, and must be valid for writes of
/// `maxlen` bytes. It can be unaligned.
unsafe fn write_i32(
    field_value: *mut c_void,
    maxlen: i32,
    value: i32,
    field_type: FieldType,
) -> i32 {
    if maxlen < size_of::<i32>() as i32 {
        return ValueError::FieldEmpty.code();
    }
    // SAFETY: the caller guarantees `maxlen` bytes, at least 4. TC gives no
    // alignment guarantee, so write unaligned.
    unsafe { field_value.cast::<i32>().write_unaligned(value) };
    field_type.code()
}

/// Reads the first 4 bytes of the file, and checks the package file tag.
fn is_package(path: &Path) -> io::Result<Option<Value>> {
    let mut head = Vec::with_capacity(4);
    File::open(path)?.take(4).read_to_end(&mut head)?;
    Ok(Some(Value::Boolean(has_package_file_tag(&head))))
}

/// Gives a text value from the package values. A file that the parser
/// cannot read has no value.
fn text_value(
    path: &Path,
    value: impl FnOnce(&PackageValues) -> Option<String>,
) -> io::Result<Option<Value>> {
    let values = package_values(path)?;
    Ok(values.and_then(|values| value(&values)).map(Value::Text))
}

/// The short class name of the main asset, for example `Blueprint`.
fn asset_class(path: &Path) -> io::Result<Option<Value>> {
    text_value(path, |values| values.asset_class.clone())
}

/// The class path of the main asset, for example `/Script/Engine.Blueprint`.
fn asset_class_path(path: &Path) -> io::Result<Option<Value>> {
    text_value(path, |values| values.asset_class_path.clone())
}

/// The object name of the main asset, for example `BP_SummaryActor`.
fn asset_name(path: &Path) -> io::Result<Option<Value>> {
    text_value(path, |values| values.asset_name.clone())
}

/// The object path of the main asset, for example
/// `/Game/Summary/BP_SummaryActor.BP_SummaryActor`.
fn object_path(path: &Path) -> io::Result<Option<Value>> {
    text_value(path, |values| values.object_path.clone())
}

/// The package name of the summary, for example
/// `/Game/Summary/BP_SummaryActor`. It does not need a main asset.
fn package_path(path: &Path) -> io::Result<Option<Value>> {
    text_value(path, |values| Some(values.package_path.clone()))
}

/// The saved-by engine version, for example `5.8.3`.
fn engine_version(path: &Path) -> io::Result<Option<Value>> {
    text_value(path, |values| values.engine_version.clone())
}

/// The full saved-by engine version, for example
/// `5.8.3-58210709+++UE5+Release-5.8`.
fn full_engine_version(path: &Path) -> io::Result<Option<Value>> {
    text_value(path, |values| values.full_engine_version.clone())
}

/// Gives a number from the package values. A file that the parser cannot
/// read has no value.
fn numeric_value(
    path: &Path,
    value: impl FnOnce(&PackageValues) -> i32,
) -> io::Result<Option<Value>> {
    let values = package_values(path)?;
    Ok(values.map(|values| Value::Numeric32(value(&values))))
}

fn file_version_ue5(path: &Path) -> io::Result<Option<Value>> {
    numeric_value(path, |values| values.file_version_ue5)
}

fn file_version_ue4(path: &Path) -> io::Result<Option<Value>> {
    numeric_value(path, |values| values.file_version_ue4)
}

fn file_version_licensee(path: &Path) -> io::Result<Option<Value>> {
    numeric_value(path, |values| values.file_version_licensee)
}
