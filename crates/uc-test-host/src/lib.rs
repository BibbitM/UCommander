//! Test helper: loads a plugin DLL and calls its exports, as Total Commander
//! does. Only tests use this crate.

use std::ffi::{CString, c_char, c_void};
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

type Module = *mut c_void;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryW(file_name: *const u16) -> Module;
    fn GetProcAddress(module: Module, proc_name: *const c_char) -> *mut c_void;
    fn FreeLibrary(module: Module) -> i32;
}

/// A loaded DLL. Dropping it unloads the DLL.
#[derive(Debug)]
pub struct Plugin {
    module: Module,
}

impl Plugin {
    /// Loads the DLL at `path`.
    pub fn load(path: &Path) -> io::Result<Self> {
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
        // SAFETY: `wide` is a NUL-terminated UTF-16 string.
        let module = unsafe { LoadLibraryW(wide.as_ptr()) };
        if module.is_null() {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { module })
    }

    /// Returns the export `name` as a function pointer of type `F`, or `None`
    /// if the DLL does not export `name`.
    ///
    /// # Safety
    ///
    /// `F` must be an `unsafe extern "system" fn` type with the real signature
    /// of the export.
    pub unsafe fn export<F: Copy>(&self, name: &str) -> Option<F> {
        assert_eq!(
            size_of::<F>(),
            size_of::<*mut c_void>(),
            "F must be a fn pointer"
        );
        let name = CString::new(name).ok()?;
        // SAFETY: `module` is a loaded module, and `name` is NUL-terminated.
        let address = unsafe { GetProcAddress(self.module, name.as_ptr()) };
        if address.is_null() {
            return None;
        }
        // SAFETY: the address is not null, `F` has the size of a pointer, and
        // the caller guarantees that `F` is the signature of the export.
        Some(unsafe { std::mem::transmute_copy::<*mut c_void, F>(&address) })
    }
}

impl Drop for Plugin {
    fn drop(&mut self) {
        // SAFETY: `module` came from LoadLibraryW and is freed only here.
        unsafe { FreeLibrary(self.module) };
    }
}

/// Path of the plugin DLL `<crate_name>.dll` that Cargo built for the current
/// test, for example `uc_content`.
///
/// Cargo puts the DLL next to the test executable, in `target/<profile>/deps/`.
/// `cargo test` builds it there, because the plugin crate has the `rlib` crate
/// type too, and each integration test depends on that library target. The
/// copy in `target/<profile>/` comes only from `cargo build`.
pub fn plugin_dll_path(crate_name: &str) -> PathBuf {
    let exe = std::env::current_exe().expect("path of the test executable");
    let deps_dir = exe.parent().expect("test executable has a parent folder");
    deps_dir.join(format!("{crate_name}.dll"))
}
