//! Safe access to the string buffers and paths that TC passes to a plugin.

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
    // SAFETY: the caller guarantees `maxlen` writable bytes at `dst`.
    let Some(buffer) = (unsafe { buffer_mut(dst, maxlen) }) else {
        return;
    };
    let ascii = text.chars().map(|c| {
        if c.is_ascii() {
            c as u8 as c_char
        } else {
            b'?' as c_char
        }
    });
    fill_nul_terminated(buffer, ascii);
}

/// Writes `text` into a TC UTF-16 buffer of `maxlen_bytes` bytes, with a
/// trailing NUL.
///
/// TC gives the size in bytes, so the buffer holds `maxlen_bytes / 2` UTF-16
/// units. A null `dst` or a buffer below 1 unit writes nothing.
///
/// # Safety
///
/// If `dst` is not null, it must be valid for writes of `maxlen_bytes` bytes,
/// and aligned for `u16`.
pub unsafe fn write_utf16(dst: *mut u16, maxlen_bytes: i32, text: &str) {
    // SAFETY: the caller guarantees `maxlen_bytes` writable, aligned bytes
    // at `dst`, so `maxlen_bytes / 2` u16 values fit.
    let Some(buffer) = (unsafe { buffer_mut(dst, maxlen_bytes / 2) }) else {
        return;
    };
    fill_nul_terminated(buffer, text.encode_utf16());
}

/// Reads a NUL-terminated UTF-16 path from TC.
///
/// Returns `None` for a null pointer, an empty path, or a path without a NUL
/// in the first 32,767 units.
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

/// # Safety
///
/// If `ptr` is not null, it must be valid for writes of `len` elements.
unsafe fn buffer_mut<'a, T>(ptr: *mut T, len: i32) -> Option<&'a mut [T]> {
    let len = usize::try_from(len).ok().filter(|&len| len > 0)?;
    if ptr.is_null() {
        return None;
    }
    // SAFETY: the pointer is not null, and the caller guarantees `len`
    // writable elements.
    Some(unsafe { std::slice::from_raw_parts_mut(ptr, len) })
}

/// Copies `src` into `dst` and adds a NUL. Truncates to `dst.len() - 1`.
fn fill_nul_terminated<T: Copy + Default>(dst: &mut [T], src: impl Iterator<Item = T>) {
    let capacity = dst.len() - 1;
    let mut written = 0;
    for (slot, value) in dst[..capacity].iter_mut().zip(src) {
        *slot = value;
        written += 1;
    }
    dst[written] = T::default();
}
