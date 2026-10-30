//! Safe access to the string buffers and paths that TC passes to a plugin.
//!
//! The write functions use raw pointers, not slices. A TC buffer can be
//! uninitialized and unaligned, and a Rust slice must not be either.

use std::ffi::{OsString, c_char};
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;

/// Longest path that Windows supports, in UTF-16 units, without the NUL.
const MAX_WIDE_PATH: usize = 32_767;

/// Writes `text` into a TC ANSI buffer of `maxlen` bytes, with a trailing NUL.
///
/// The text is truncated to `maxlen - 1` bytes. Characters that are not ASCII
/// become `?`. A null `dst` or a `maxlen` below 1 writes nothing.
///
/// # Safety
///
/// If `dst` is not null, it must be valid for writes of `maxlen` bytes.
pub unsafe fn write_ansi(dst: *mut c_char, maxlen: i32, text: &str) {
    let ascii = text.chars().map(|c| {
        if c.is_ascii() {
            c as u8 as c_char
        } else {
            b'?' as c_char
        }
    });
    // SAFETY: the caller guarantees `maxlen` writable bytes at `dst`.
    unsafe { write_nul_terminated(dst, maxlen, ascii) };
}

/// Writes `text` into a TC UTF-16 buffer of `maxlen_bytes` bytes, with a
/// trailing NUL.
///
/// TC gives the size in bytes, so the buffer holds `maxlen_bytes / 2` UTF-16
/// units. The buffer can be unaligned. A null `dst` or a buffer below 1 unit
/// writes nothing.
///
/// # Safety
///
/// If `dst` is not null, it must be valid for writes of `maxlen_bytes` bytes.
pub unsafe fn write_utf16(dst: *mut u8, maxlen_bytes: i32, text: &str) {
    // SAFETY: the caller guarantees `maxlen_bytes` writable bytes at `dst`, so
    // `maxlen_bytes / 2` u16 values fit. The function writes them unaligned.
    unsafe { write_nul_terminated(dst.cast::<u16>(), maxlen_bytes / 2, text.encode_utf16()) };
}

/// Reads a NUL-terminated UTF-16 path from TC.
///
/// Returns `None` for a null pointer, an empty path, or a path longer than
/// 32,767 units. The function reads at most 32,768 units.
///
/// # Safety
///
/// If `src` is not null, it must point to a NUL-terminated UTF-16 string.
pub unsafe fn read_wide_path(src: *const u16) -> Option<PathBuf> {
    if src.is_null() {
        return None;
    }
    let mut len = 0;
    // SAFETY: the caller guarantees a NUL-terminated string. The loop stops
    // at the NUL, so it reads only elements of that string.
    while unsafe { *src.add(len) } != 0 {
        len += 1;
        if len > MAX_WIDE_PATH {
            return None;
        }
    }
    if len == 0 {
        return None;
    }
    // SAFETY: the loop above read `len` valid elements.
    let wide = unsafe { std::slice::from_raw_parts(src, len) };
    Some(PathBuf::from(OsString::from_wide(wide)))
}

/// Copies `src` to `dst` and adds a NUL. Truncates to `len - 1` elements. A
/// null `dst` or a `len` below 1 writes nothing.
///
/// # Safety
///
/// If `dst` is not null, it must be valid for writes of `len` elements. It can
/// be unaligned and uninitialized.
unsafe fn write_nul_terminated<T: Copy + Default>(
    dst: *mut T,
    len: i32,
    src: impl Iterator<Item = T>,
) {
    let Some(len) = usize::try_from(len).ok().filter(|&len| len > 0) else {
        return;
    };
    if dst.is_null() {
        return;
    }
    let mut written = 0;
    for value in src.take(len - 1) {
        // SAFETY: `written` is below `len - 1`, and the caller guarantees
        // `len` writable elements.
        unsafe { dst.add(written).write_unaligned(value) };
        written += 1;
    }
    // SAFETY: `written` is at most `len - 1`.
    unsafe { dst.add(written).write_unaligned(T::default()) };
}
