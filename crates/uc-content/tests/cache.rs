//! The cache of package values: FFI tests with the built DLL.
//!
//! The cache key is the path, the size, the last write time and the
//! creation time of the file. The rules are in
//! `docs/steps/04-content-name-version.md`, "Cache".

mod common;

use std::fs::File;
use std::path::Path;
use std::time::{Duration, SystemTime};

use common::{FT_FIELDEMPTY, FT_STRINGW, fixture, fixture_bytes, get_text, test_file};

const ASSET_CLASS: i32 = 1;

fn asset_class(path: &Path) -> (i32, String) {
    get_text(path, ASSET_CLASS, 256)
}

fn text(value: &str) -> (i32, String) {
    (FT_STRINGW, value.to_owned())
}

/// A last write time that the tests set. It is not the current time, so
/// that two writes in the same clock tick do not give the same time.
fn write_time(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000 + seconds)
}

/// Sets the last write time of the file, and checks it.
fn set_write_time(path: &Path, time: SystemTime) {
    File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(time)
        .unwrap();
    assert_eq!(std::fs::metadata(path).unwrap().modified().unwrap(), time);
}

/// Writes `bytes` into the existing file. The creation time does not
/// change. Then sets the last write time.
fn rewrite(path: &Path, bytes: &[u8], time: SystemTime) {
    let created = std::fs::metadata(path).unwrap().created().unwrap();
    std::fs::write(path, bytes).unwrap();
    set_write_time(path, time);
    assert_eq!(std::fs::metadata(path).unwrap().created().unwrap(), created);
}

#[test]
fn a_file_with_another_size_gives_the_new_value() {
    let path = test_file(
        "cache",
        "OtherSize.uasset",
        &fixture_bytes("BP_SummaryActor.uasset"),
    );
    assert_eq!(asset_class(&path), text("Blueprint"));

    std::fs::write(&path, fixture_bytes("M_Summary.uasset")).unwrap();

    assert_eq!(asset_class(&path), text("Material"));
}

#[test]
fn a_file_with_another_write_time_gives_the_new_value() {
    let bytes = fixture_bytes("BP_SummaryActor.uasset");
    let path = test_file("cache", "OtherTime.uasset", &bytes);
    set_write_time(&path, write_time(0));
    assert_eq!(asset_class(&path), text("Blueprint"));

    // The same size, but not a package.
    rewrite(&path, &vec![0; bytes.len()], write_time(1));

    assert_eq!(asset_class(&path).0, FT_FIELDEMPTY);
}

/// The known limit of the cache: the plugin does not read the content to
/// find a change.
#[test]
fn a_file_with_the_same_size_and_times_gives_the_cached_value() {
    let bytes = fixture_bytes("BP_SummaryActor.uasset");
    let path = test_file("cache", "SameTime.uasset", &bytes);
    set_write_time(&path, write_time(0));
    assert_eq!(asset_class(&path), text("Blueprint"));

    rewrite(&path, &vec![0; bytes.len()], write_time(0));

    assert_eq!(asset_class(&path), text("Blueprint"));
}

#[test]
fn calls_from_many_threads_give_the_correct_values() {
    let cases = [
        ("BP_SummaryActor.uasset", "Blueprint"),
        ("L_SummaryLevel.umap", "World"),
        ("M_Summary.uasset", "Material"),
    ];

    std::thread::scope(|scope| {
        for thread in 0..8 {
            scope.spawn(move || {
                for call in 0..30 {
                    let (name, class) = cases[(thread + call) % cases.len()];

                    assert_eq!(asset_class(&fixture(name)), text(class), "{name}");
                }
            });
        }
    });
}
