use uc_test_host::plugin_dll_path;

/// A user without the VC++ runtime must be able to load the plugin.
#[test]
fn the_dll_does_not_need_the_vc_runtime() {
    let path = plugin_dll_path("uc_content");
    let bytes = std::fs::read(&path).unwrap();
    let text = String::from_utf8_lossy(&bytes).to_ascii_lowercase();

    assert!(!text.contains("vcruntime140.dll"));
}
