use std::path::Path;

use uc_test_host::Plugin;

type GetTickCount = unsafe extern "system" fn() -> u32;

#[test]
fn a_system_dll_loads_and_its_export_can_be_called() {
    let kernel32 = Plugin::load(Path::new("kernel32.dll")).unwrap();

    // SAFETY: GetTickCount has this signature.
    let get_tick_count = unsafe { kernel32.export::<GetTickCount>("GetTickCount") }.unwrap();

    // SAFETY: GetTickCount has no preconditions.
    let _ticks = unsafe { get_tick_count() };
}

#[test]
fn a_missing_export_is_none() {
    let kernel32 = Plugin::load(Path::new("kernel32.dll")).unwrap();

    // SAFETY: the export does not exist, so it is never called.
    let missing = unsafe { kernel32.export::<GetTickCount>("NoSuchExport") };

    assert!(missing.is_none());
}

#[test]
fn a_missing_dll_is_an_error() {
    assert!(Plugin::load(Path::new("uc_no_such_plugin.dll")).is_err());
}
