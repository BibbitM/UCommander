use std::path::PathBuf;

use uc_test_host::Plugin;
use xtask::install::{
    expand_variables, installed_plugin_from_ini, plugin_path_in_ini, reg_value, replace_plugin,
};

const FILE: &str = "UCommanderContent.wdx64";

/// A new, empty folder for one test.
fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("uc-xtask-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn the_plugin_path_comes_from_the_content_plugins_section() {
    let ini = "[Configuration]\r\n\
               0=C:\\Wrong\\UCommanderContent.wdx64\r\n\
               [ContentPlugins]\r\n\
               0=C:\\Other\\other.wdx64\r\n\
               0_detect=EXT=\"X\"\r\n\
               1=%COMMANDER_PATH%\\plugins\\wdx\\UCommanderContent\\UCommanderContent.wdx64\r\n\
               1_detect=EXT=\"UASSET\"\r\n\
               [ContentPlugins64]\r\n";

    assert_eq!(
        plugin_path_in_ini(ini, FILE).as_deref(),
        Some("%COMMANDER_PATH%\\plugins\\wdx\\UCommanderContent\\UCommanderContent.wdx64")
    );
}

#[test]
fn the_ini_search_ignores_upper_and_lower_case() {
    let ini = "[contentplugins]\n0=E:\\Temp\\UCOMMANDERCONTENT.WDX64\n";

    assert_eq!(
        plugin_path_in_ini(ini, FILE).as_deref(),
        Some("E:\\Temp\\UCOMMANDERCONTENT.WDX64")
    );
}

#[test]
fn a_plugin_that_tc_has_not_registered_is_none() {
    let ini = "[ContentPlugins]\n0=C:\\Other\\other.wdx64\n";

    assert_eq!(plugin_path_in_ini(ini, FILE), None);
}

#[test]
fn a_detect_line_is_not_a_plugin_path() {
    let ini = "[ContentPlugins]\n0_detect=UCommanderContent.wdx64\n";

    assert_eq!(plugin_path_in_ini(ini, FILE), None);
}

#[test]
fn the_registry_value_comes_from_the_reg_output() {
    let output = "\r\nHKEY_CURRENT_USER\\Software\\Ghisler\\Total Commander\r\n    \
                  IniFileName    REG_SZ    %APPDATA%\\GHISLER\\wincmd.ini\r\n\r\n";

    assert_eq!(
        reg_value(output, "IniFileName").as_deref(),
        Some("%APPDATA%\\GHISLER\\wincmd.ini")
    );
}

#[test]
fn a_registry_value_can_contain_spaces() {
    let output = "    InstallDir    REG_EXPAND_SZ    C:\\Program Files\\Total Commander\r\n";

    assert_eq!(
        reg_value(output, "InstallDir").as_deref(),
        Some("C:\\Program Files\\Total Commander")
    );
}

#[test]
fn a_missing_registry_value_is_none() {
    assert_eq!(
        reg_value("    Other    REG_SZ    x\r\n", "InstallDir"),
        None
    );
}

#[test]
fn variables_are_expanded() {
    let lookup = |name: &str| (name == "COMMANDER_PATH").then(|| "C:\\totalcmd".to_string());

    assert_eq!(
        expand_variables("%COMMANDER_PATH%\\plugins\\a.wdx64", lookup),
        "C:\\totalcmd\\plugins\\a.wdx64"
    );
}

#[test]
fn an_unknown_variable_stays_in_the_text() {
    assert_eq!(expand_variables("%NOPE%\\a", |_| None), "%NOPE%\\a");
}

#[test]
fn a_loaded_dll_is_replaced() {
    let dir = test_dir("loaded");
    let installed = dir.join(format!("plugin-{}.dll", std::process::id()));
    let system_dll = PathBuf::from(std::env::var_os("SystemRoot").unwrap())
        .join("System32")
        .join("version.dll");
    std::fs::copy(system_dll, &installed).unwrap();
    let loaded = Plugin::load(&installed).unwrap();
    assert!(std::fs::write(&installed, b"overwrite").is_err());
    let new = dir.join("new.dll");
    std::fs::write(&new, b"new plugin").unwrap();

    replace_plugin(&new, &installed).unwrap();

    assert_eq!(std::fs::read(&installed).unwrap(), b"new plugin");
    drop(loaded);
}

#[test]
fn an_old_file_from_an_earlier_install_is_removed() {
    let dir = test_dir("old");
    let installed = dir.join("plugin.dll");
    std::fs::write(&installed, b"current").unwrap();
    std::fs::write(dir.join("plugin.dll.old"), b"older").unwrap();
    let new = dir.join("new.dll");
    std::fs::write(&new, b"new plugin").unwrap();

    replace_plugin(&new, &installed).unwrap();

    assert_eq!(std::fs::read(&installed).unwrap(), b"new plugin");
    assert_eq!(
        std::fs::read(dir.join("plugin.dll.old")).unwrap(),
        b"current"
    );
}

#[test]
fn a_second_install_works_while_the_old_file_is_still_loaded() {
    let dir = test_dir("twice");
    let installed = dir.join(format!("plugin2-{}.dll", std::process::id()));
    let system_dll = PathBuf::from(std::env::var_os("SystemRoot").unwrap())
        .join("System32")
        .join("version.dll");
    std::fs::copy(system_dll, &installed).unwrap();
    let loaded = Plugin::load(&installed).unwrap();
    let new = dir.join("new.dll");
    std::fs::write(&new, b"new plugin").unwrap();

    replace_plugin(&new, &installed).unwrap();
    replace_plugin(&new, &installed).unwrap();

    assert_eq!(std::fs::read(&installed).unwrap(), b"new plugin");
    drop(loaded);
}

const INI: &str = "[ContentPlugins]\r\n\
                   0=%COMMANDER_PATH%\\plugins\\wdx\\UCommanderContent\\UCommanderContent.wdx64\r\n";

fn expand(path: &str) -> String {
    path.replace("%COMMANDER_PATH%", r"C:\totalcmd")
}

const EXPECTED: &str = r"C:\totalcmd\plugins\wdx\UCommanderContent\UCommanderContent.wdx64";

#[test]
fn the_installed_plugin_comes_from_a_wincmd_ini_file() {
    let dir = test_dir("ini-file");
    let ini = dir.join("wincmd.ini");
    std::fs::write(&ini, INI).unwrap();

    assert_eq!(
        installed_plugin_from_ini(&ini, expand),
        Ok(PathBuf::from(EXPECTED))
    );
}

#[test]
fn a_utf8_ini_with_a_bom_is_read() {
    let dir = test_dir("ini-utf8-bom");
    let ini = dir.join("wincmd.ini");
    std::fs::write(&ini, [b"\xEF\xBB\xBF".as_slice(), INI.as_bytes()].concat()).unwrap();

    assert_eq!(
        installed_plugin_from_ini(&ini, expand),
        Ok(PathBuf::from(EXPECTED))
    );
}

#[test]
fn a_utf16_ini_is_read() {
    let dir = test_dir("ini-utf16");
    let ini = dir.join("wincmd.ini");
    let mut bytes = vec![0xFF, 0xFE];
    bytes.extend(INI.encode_utf16().flat_map(u16::to_le_bytes));
    std::fs::write(&ini, bytes).unwrap();

    assert_eq!(
        installed_plugin_from_ini(&ini, expand),
        Ok(PathBuf::from(EXPECTED))
    );
}

#[test]
fn a_plugin_that_tc_has_not_registered_gives_the_install_instruction() {
    let dir = test_dir("ini-missing-line");
    let ini = dir.join("wincmd.ini");
    std::fs::write(&ini, "[ContentPlugins]\r\n0=C:\\Other\\other.wdx64\r\n").unwrap();

    let error = installed_plugin_from_ini(&ini, expand).unwrap_err();

    assert!(error.contains("has not registered"), "{error}");
    assert!(error.contains("cargo xtask package"), "{error}");
    assert!(error.contains(&ini.display().to_string()), "{error}");
}

#[test]
fn a_missing_ini_gives_its_path() {
    let dir = test_dir("ini-no-file");
    let ini = dir.join("wincmd.ini");

    let error = installed_plugin_from_ini(&ini, expand).unwrap_err();

    assert!(error.contains(&ini.display().to_string()), "{error}");
}

#[test]
fn a_line_with_an_empty_key_is_not_a_plugin_path() {
    assert_eq!(
        plugin_path_in_ini("[ContentPlugins]\n=UCommanderContent.wdx64\n", FILE),
        None
    );
}
