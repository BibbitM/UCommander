//! Helpers for the xtask tests.

use std::ops::Deref;
use std::path::{Path, PathBuf};

/// A new, empty folder for one test. The folder is deleted at the end of the
/// test.
pub struct TestDir(PathBuf);

impl Deref for TestDir {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub fn test_dir(name: &str) -> TestDir {
    let dir = std::env::temp_dir().join(format!("uc-xtask-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    TestDir(dir)
}
