//! Events that remove entries from the cache: FFI tests with the built
//! DLL.
//!
//! These tests remove entries that other cache tests need. So they are in
//! their own test binary, and do not run in the same process as
//! `cache.rs`.

mod common;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, SystemTime};

use common::{
    FT_FIELDEMPTY, FT_STRINGW, expected, export, fixture_bytes, get_text, load, test_file,
};

type PluginUnloading = unsafe extern "system" fn();

const ASSET_CLASS: i32 = 1;
/// The number of entries in the cache (point 4 owner decision 7).
const CACHE_SIZE: usize = 16;

/// The tests of this file run one at a time. Else an event of one test can
/// remove the entry that another test examines, and the test passes for
/// the wrong reason.
fn serial() -> MutexGuard<'static, ()> {
    static SERIAL: Mutex<()> = Mutex::new(());
    SERIAL.lock().unwrap_or_else(PoisonError::into_inner)
}

fn asset_class(path: &Path) -> (i32, String) {
    get_text(path, ASSET_CLASS, 256)
}

fn write_time() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000)
}

/// The fixture that the tests copy.
const PACKAGE: &str = "BP_SummaryActor.uasset";

/// The expected result for a copy of [`PACKAGE`]: its `asset_class` value
/// in the `.expected.json` file.
fn class_of_package() -> (i32, String) {
    (FT_STRINGW, expected(PACKAGE).asset_class)
}

/// Writes a copy of [`PACKAGE`] with a fixed last write time.
fn package_file(name: &str) -> PathBuf {
    let path = test_file("reset", name, &fixture_bytes(PACKAGE));
    File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(write_time())
        .unwrap();
    path
}

/// Writes other content with the same size and the same times. The cache
/// cannot see this change (the known limit), so a new value shows that the
/// entry is not in the cache.
fn rewrite_with_the_same_key(path: &Path) {
    let size = std::fs::metadata(path).unwrap().len() as usize;
    std::fs::write(path, vec![0; size]).unwrap();
    File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(write_time())
        .unwrap();
}

#[test]
fn tc_unloading_the_plugin_clears_the_cache() {
    let _serial = serial();
    // Keep the DLL loaded: the test calls the unload function, not
    // FreeLibrary.
    let plugin = load();
    let unloading: PluginUnloading = export(&plugin, "ContentPluginUnloading");
    let path = package_file("Unload.uasset");
    assert_eq!(asset_class(&path), class_of_package());
    rewrite_with_the_same_key(&path);

    // SAFETY: the function has no parameters.
    unsafe { unloading() };

    assert_eq!(asset_class(&path).0, FT_FIELDEMPTY);
}

#[test]
fn the_cache_keeps_at_most_16_files() {
    let _serial = serial();
    let _plugin = load();
    let path = package_file("Oldest.uasset");
    assert_eq!(asset_class(&path), class_of_package());
    rewrite_with_the_same_key(&path);

    for index in 0..CACHE_SIZE {
        asset_class(&package_file(&format!("Newer{index}.uasset")));
    }

    assert_eq!(asset_class(&path).0, FT_FIELDEMPTY);
}
