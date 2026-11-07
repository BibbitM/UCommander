//! Package header: read from a reader, with only the bytes of the header.

mod common;

use std::io::{self, Cursor, Read};

use common::{Export, Package};
use uc_format::Error;
use uc_format::package::{MAX_HEADER_READ_SIZE, ReadError, read_header, read_header_from};

fn package() -> Package {
    Package {
        exports: vec![Export {
            object_name: (1, 0),
            ..Export::default()
        }],
        ..Package::new(&["/Script/Engine", "Root"])
    }
}

/// A reader that counts the bytes that the caller reads.
struct Counting<R> {
    inner: R,
    count: usize,
}

impl<R: Read> Read for Counting<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.count += n;
        Ok(n)
    }
}

#[test]
fn gives_the_same_header_as_read_header() {
    let bytes = package().to_bytes();

    let read = read_header_from(Cursor::new(&bytes)).unwrap();

    assert_eq!(read, read_header(&bytes).unwrap());
}

#[test]
fn reads_a_header_that_is_larger_than_64_kib() {
    let mut input = package();
    input.summary.custom_versions = vec![([1, 2, 3, 4], 5); 4000];
    let bytes = input.to_bytes();
    assert!(bytes.len() > 64 * 1024);

    let read = read_header_from(Cursor::new(&bytes)).unwrap();

    assert_eq!(read.header.summary.custom_versions.len(), 4000);
    assert_eq!(read.header.exports.len(), 1);
}

#[test]
fn reads_64_kib_of_a_large_file_with_a_small_header() {
    let mut bytes = package().to_bytes();
    bytes.resize(1024 * 1024, 0xAB);
    let mut reader = Counting {
        inner: Cursor::new(&bytes),
        count: 0,
    };

    read_header_from(&mut reader).unwrap();

    assert_eq!(reader.count, 64 * 1024);
}

#[test]
fn a_damaged_count_stops_at_the_read_limit() {
    let mut input = package();
    input.summary.custom_version_count = Some(i32::MAX);
    let bytes = input.to_bytes();
    // The reader never ends.
    let mut reader = Counting {
        inner: Cursor::new(bytes).chain(io::repeat(0)),
        count: 0,
    };

    let result = read_header_from(&mut reader);

    assert!(
        matches!(result, Err(ReadError::HeaderTooLarge)),
        "{result:?}"
    );
    assert_eq!(reader.count, MAX_HEADER_READ_SIZE);
    assert_eq!(MAX_HEADER_READ_SIZE, 16 * 1024 * 1024);
}

#[test]
fn a_file_that_ends_in_the_header_gives_truncated() {
    let bytes = package().to_bytes();

    let result = read_header_from(Cursor::new(&bytes[..bytes.len() - 1]));

    assert!(
        matches!(result, Err(ReadError::Parse(Error::Truncated { .. }))),
        "{result:?}"
    );
}

#[test]
fn a_file_that_is_not_a_package_gives_the_parser_error() {
    let result = read_header_from(Cursor::new(b"This is not a package."));

    assert!(
        matches!(result, Err(ReadError::Parse(Error::NotAPackage))),
        "{result:?}"
    );
}

#[test]
fn an_io_error_is_given_to_the_caller() {
    struct Failing;
    impl Read for Failing {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("disk error"))
        }
    }

    let result = read_header_from(Failing);

    match result {
        Err(ReadError::Io(error)) => assert_eq!(error.to_string(), "disk error"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_errors_have_a_message() {
    assert_eq!(
        ReadError::HeaderTooLarge.to_string(),
        "damaged package: the header is larger than 16 MiB"
    );
    assert_eq!(
        ReadError::Parse(Error::NotAPackage).to_string(),
        Error::NotAPackage.to_string()
    );
    assert_eq!(
        ReadError::Io(io::Error::other("disk error")).to_string(),
        "disk error"
    );
}
