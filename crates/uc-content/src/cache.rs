//! The cache of package values. The plugin reads a package header one time
//! for all the fields of a file. The rules are in
//! `docs/steps/04-content-name-version.md`, "Cache".

use std::collections::VecDeque;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::SystemTime;

use uc_format::package::{PackageHeader, ReadError, read_header_from};

/// The maximum number of entries. TC requests the values file by file
/// (`docs/tc-api/content.md`), so a small cache is enough.
const CAPACITY: usize = 16;

/// The values of the content fields that come from the package header.
#[derive(Debug)]
pub struct PackageValues {
    pub asset_class: Option<String>,
    pub asset_class_path: Option<String>,
    pub asset_name: Option<String>,
    pub object_path: Option<String>,
    pub package_path: Option<String>,
    /// `major.minor.patch` of the saved-by engine version.
    pub engine_version: Option<String>,
    /// The full text of the saved-by engine version.
    pub full_engine_version: Option<String>,
    pub file_version_ue5: i32,
    pub file_version_ue4: i32,
    pub file_version_licensee: i32,
}

impl PackageValues {
    fn new(header: &PackageHeader) -> Self {
        // An empty engine version has no value (UE rule).
        let engine_version = header
            .summary
            .saved_by_engine_version
            .as_ref()
            .filter(|version| !version.is_empty());
        Self {
            engine_version: engine_version.map(|version| version.short_text()),
            full_engine_version: engine_version.map(|version| version.to_string()),
            file_version_ue5: header.summary.file_version_ue5,
            file_version_ue4: header.summary.file_version_ue4,
            file_version_licensee: header.summary.file_version_licensee_ue,
            asset_class: header.asset_class(),
            asset_class_path: header.asset_class_path(),
            asset_name: header.asset_name(),
            object_path: header.asset_object_path(),
            package_path: header.long_package_name().map(str::to_owned),
        }
    }
}

/// The values of a file. `None` when the parser cannot read the file: not
/// a package, not supported, or damaged.
pub type Entry = Option<Arc<PackageValues>>;

/// Identifies one version of a file. A changed file has a different key.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Key {
    path: PathBuf,
    size: u64,
    modified: Option<SystemTime>,
    created: Option<SystemTime>,
}

impl Key {
    fn of(path: &Path) -> io::Result<Self> {
        let metadata = fs::metadata(path)?;
        Ok(Self {
            path: path.to_owned(),
            size: metadata.len(),
            modified: metadata.modified().ok(),
            created: metadata.created().ok(),
        })
    }
}

/// The entries, the oldest first.
static CACHE: Mutex<VecDeque<(Key, Entry)>> = Mutex::new(VecDeque::new());

/// Gives the values of the file at `path`, from the cache or from the file.
/// An error is an I/O error. The cache does not keep errors.
pub fn package_values(path: &Path) -> io::Result<Entry> {
    let key = Key::of(path)?;
    if let Some(entry) = find(&key) {
        return Ok(entry);
    }
    let entry = read(path)?;
    // A file that changed during the read can give a mixed result. Keep
    // only the result of a file that did not change.
    if Key::of(path).is_ok_and(|after| after == key) {
        insert(key, entry.clone());
    }
    Ok(entry)
}

fn read(path: &Path) -> io::Result<Entry> {
    match read_header_from(File::open(path)?) {
        Ok(read) => Ok(Some(Arc::new(PackageValues::new(&read.header)))),
        Err(ReadError::Io(error)) => Err(error),
        Err(_) => Ok(None),
    }
}

fn lock() -> MutexGuard<'static, VecDeque<(Key, Entry)>> {
    // Each entry is complete before it goes into the cache, so the data
    // stays valid after a panic in another thread.
    CACHE.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Removes all entries, and releases their memory.
pub fn clear() {
    let mut cache = lock();
    cache.clear();
    cache.shrink_to_fit();
}

fn find(key: &Key) -> Option<Entry> {
    lock()
        .iter()
        .find(|(entry_key, _)| entry_key == key)
        .map(|(_, entry)| entry.clone())
}

/// Adds an entry. It replaces an entry of an older version of the same
/// file, else the oldest entry when the cache is full.
fn insert(key: Key, entry: Entry) {
    let mut cache = lock();
    cache.retain(|(entry_key, _)| entry_key.path != key.path);
    if cache.len() >= CAPACITY {
        cache.pop_front();
    }
    cache.push_back((key, entry));
}
