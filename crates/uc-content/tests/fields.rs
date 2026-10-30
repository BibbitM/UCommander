mod common;

use std::ffi::c_char;

use common::{GetDetectString, GetSupportedField, ansi_text, export, load};

const FT_NOMOREFIELDS: i32 = 0;
const FT_BOOLEAN: i32 = 6;
const CANARY: c_char = 0x55;

fn supported_field(index: i32, maxlen: usize) -> (i32, String, String) {
    let plugin = load();
    let get: GetSupportedField = export(&plugin, "ContentGetSupportedField");
    // One byte more than `maxlen`: the plugin must not write it.
    let mut name = vec![CANARY; maxlen + 1];
    let mut units = vec![CANARY; maxlen + 1];

    // SAFETY: both buffers have more than `maxlen` bytes.
    let field_type = unsafe { get(index, name.as_mut_ptr(), units.as_mut_ptr(), maxlen as i32) };

    assert_eq!(name[maxlen], CANARY, "name buffer overflow");
    assert_eq!(units[maxlen], CANARY, "units buffer overflow");
    (field_type, ansi_text(&name), ansi_text(&units))
}

#[test]
fn field_0_is_is_package_of_type_boolean_without_units() {
    assert_eq!(
        supported_field(0, 256),
        (FT_BOOLEAN, "Is package".to_string(), String::new())
    );
}

#[test]
fn there_are_no_more_fields_after_field_0() {
    assert_eq!(supported_field(1, 256).0, FT_NOMOREFIELDS);
}

#[test]
fn a_negative_field_index_gives_no_more_fields() {
    assert_eq!(supported_field(-1, 256).0, FT_NOMOREFIELDS);
}

#[test]
fn a_small_buffer_gets_a_truncated_name_and_a_nul() {
    let (field_type, name, units) = supported_field(0, 4);

    assert_eq!(field_type, FT_BOOLEAN);
    assert_eq!(name, "Is ");
    assert_eq!(units, "");
}

#[test]
fn the_detect_string_selects_uasset_and_umap_files() {
    let plugin = load();
    let get: GetDetectString = export(&plugin, "ContentGetDetectString");
    let mut detect = vec![0 as c_char; 2048];

    // SAFETY: the buffer has 2048 bytes.
    unsafe { get(detect.as_mut_ptr(), detect.len() as i32) };

    assert_eq!(ansi_text(&detect), r#"EXT="UASSET" | EXT="UMAP""#);
}

#[test]
fn the_plugin_exports_the_load_and_unload_functions() {
    let plugin = load();

    // SAFETY: the exports are only looked up, not called.
    unsafe {
        assert!(
            plugin
                .export::<unsafe extern "system" fn(*const u8)>("ContentSetDefaultParams")
                .is_some()
        );
        assert!(
            plugin
                .export::<unsafe extern "system" fn()>("ContentPluginUnloading")
                .is_some()
        );
    }
}
