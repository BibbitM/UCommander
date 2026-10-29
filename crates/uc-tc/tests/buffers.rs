use std::ffi::c_char;
use std::path::PathBuf;

use uc_tc::buffers::{read_wide_path, write_ansi, write_utf16};

fn ansi(buffer: &[c_char]) -> Vec<u8> {
    buffer.iter().map(|&c| c as u8).collect()
}

#[test]
fn ansi_text_that_fits_is_written_with_a_nul() {
    let mut buffer = [0x55 as c_char; 8];

    // SAFETY: the pointer and the length come from a live array.
    unsafe { write_ansi(buffer.as_mut_ptr(), buffer.len() as i32, "Is pkg") };

    assert_eq!(&ansi(&buffer)[..7], b"Is pkg\0");
}

#[test]
fn ansi_text_that_is_too_long_is_truncated_with_a_nul() {
    let mut buffer = [0x55 as c_char; 4];

    // SAFETY: the pointer and the length come from a live array.
    unsafe { write_ansi(buffer.as_mut_ptr(), buffer.len() as i32, "Is package") };

    assert_eq!(ansi(&buffer), b"Is \0");
}

#[test]
fn ansi_maxlen_one_writes_only_the_nul() {
    let mut buffer = [0x55 as c_char; 2];

    // SAFETY: the pointer is valid for 1 element.
    unsafe { write_ansi(buffer.as_mut_ptr(), 1, "Is package") };

    assert_eq!(ansi(&buffer), [0, 0x55]);
}

#[test]
fn ansi_maxlen_zero_writes_nothing() {
    let mut buffer = [0x55 as c_char; 1];

    // SAFETY: with maxlen 0 the function does not write.
    unsafe { write_ansi(buffer.as_mut_ptr(), 0, "Is package") };

    assert_eq!(ansi(&buffer), [0x55]);
}

#[test]
fn ansi_null_pointer_writes_nothing() {
    // SAFETY: the function must check for a null pointer.
    unsafe { write_ansi(std::ptr::null_mut(), 8, "Is package") };
}

#[test]
fn non_ascii_characters_become_question_marks_in_ansi() {
    let mut buffer = [0 as c_char; 8];

    // SAFETY: the pointer and the length come from a live array.
    unsafe { write_ansi(buffer.as_mut_ptr(), buffer.len() as i32, "Größe") };

    assert_eq!(&ansi(&buffer)[..6], b"Gr??e\0");
}

#[test]
fn utf16_maxlen_is_in_bytes() {
    let mut buffer = [0x5555u16; 4];

    // SAFETY: the pointer is valid for 4 u16 values = 8 bytes.
    unsafe { write_utf16(buffer.as_mut_ptr(), 8, "Größe") };

    let expected: Vec<u16> = "Grö".encode_utf16().chain([0]).collect();
    assert_eq!(buffer.to_vec(), expected);
}

#[test]
fn utf16_odd_maxlen_uses_whole_characters_only() {
    let mut buffer = [0x5555u16; 2];

    // SAFETY: 3 bytes contain one whole u16 value.
    unsafe { write_utf16(buffer.as_mut_ptr(), 3, "Ab") };

    assert_eq!(buffer, [0, 0x5555]);
}

#[test]
fn wide_path_is_read_up_to_the_nul() {
    let wide: Vec<u16> = "C:\\Gra\\Żółw.uasset\0ignored".encode_utf16().collect();

    // SAFETY: the buffer is NUL-terminated.
    let path = unsafe { read_wide_path(wide.as_ptr()) };

    assert_eq!(path, Some(PathBuf::from("C:\\Gra\\Żółw.uasset")));
}

#[test]
fn null_wide_path_is_none() {
    // SAFETY: the function must check for a null pointer.
    let path = unsafe { read_wide_path(std::ptr::null()) };

    assert_eq!(path, None);
}

#[test]
fn empty_wide_path_is_none() {
    let wide = [0u16];

    // SAFETY: the buffer is NUL-terminated.
    let path = unsafe { read_wide_path(wide.as_ptr()) };

    assert_eq!(path, None);
}
