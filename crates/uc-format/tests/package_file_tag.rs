use uc_format::package::has_package_file_tag;

/// The package file tag as it is stored in a file: `0x9E2A83C1`, little-endian.
const TAG: [u8; 4] = [0xC1, 0x83, 0x2A, 0x9E];

#[test]
fn bytes_that_start_with_the_tag_are_a_package() {
    let mut bytes = TAG.to_vec();
    bytes.extend_from_slice(&[0xFF; 28]);

    assert!(has_package_file_tag(&bytes));
}

#[test]
fn the_tag_alone_is_a_package() {
    assert!(has_package_file_tag(&TAG));
}

#[test]
fn other_bytes_are_not_a_package() {
    assert!(!has_package_file_tag(b"This is a text file."));
}

#[test]
fn the_byte_swapped_tag_is_not_a_package() {
    assert!(!has_package_file_tag(&[0x9E, 0x2A, 0x83, 0xC1]));
}

#[test]
fn input_shorter_than_the_tag_is_not_a_package() {
    assert!(!has_package_file_tag(&TAG[..3]));
}

#[test]
fn empty_input_is_not_a_package() {
    assert!(!has_package_file_tag(&[]));
}
