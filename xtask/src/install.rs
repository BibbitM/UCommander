//! The `install` task: replaces the plugin file that TC has registered.

use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{CONTENT_PLUGIN_FILE, build_content_plugin};

const TC_REGISTRY_KEYS: [&str; 2] = [
    r"HKCU\Software\Ghisler\Total Commander",
    r"HKLM\Software\Ghisler\Total Commander",
];

/// Returns the path of `file_name` from the `[ContentPlugins]` section of a
/// `wincmd.ini` text.
pub fn plugin_path_in_ini(ini: &str, file_name: &str) -> Option<String> {
    let mut in_section = false;
    for line in ini.lines().map(str::trim) {
        if line.starts_with('[') {
            in_section = line.eq_ignore_ascii_case("[ContentPlugins]");
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        // The "0_detect" lines have the detect string, not a path.
        let is_plugin_line = !key.is_empty() && key.chars().all(|c| c.is_ascii_digit());
        let file = value.rsplit('\\').next().unwrap_or(value);
        if in_section && is_plugin_line && file.eq_ignore_ascii_case(file_name) {
            return Some(value.to_string());
        }
    }
    None
}

/// Returns the value `name` from the output of `reg.exe query`.
pub fn reg_value(output: &str, name: &str) -> Option<String> {
    for line in output.lines().map(str::trim_start) {
        let Some(rest) = line.strip_prefix(name) else {
            continue;
        };
        let rest = rest.trim_start().strip_prefix("REG_")?;
        let (_type, value) = rest.split_once(char::is_whitespace)?;
        return Some(value.trim().to_string());
    }
    None
}

/// Replaces each `%NAME%` that `lookup` knows.
pub fn expand_variables(text: &str, lookup: impl Fn(&str) -> Option<String>) -> String {
    let mut result = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('%') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('%') else {
            break;
        };
        result.push_str(&rest[..start]);
        match lookup(&after[..end]) {
            Some(value) => result.push_str(&value),
            None => result.push_str(&rest[start..start + end + 2]),
        }
        rest = &after[end + 1..];
    }
    result.push_str(rest);
    result
}

/// Replaces `installed` with `new`, also when `installed` is a loaded DLL.
///
/// Windows does not allow an overwrite of a loaded DLL, but it allows a rename.
/// So the current file gets the extension `.old`, or `.old1`, `.old2`, … when
/// an earlier `.old` file is still loaded.
pub fn replace_plugin(new: &Path, installed: &Path) -> io::Result<()> {
    let mut old_file = None;
    if installed.exists() {
        let old = free_old_path(installed);
        std::fs::rename(installed, &old)?;
        old_file = Some(old);
    }
    if let Err(error) = std::fs::copy(new, installed) {
        if let Some(old) = old_file
            && let Err(rollback) = std::fs::rename(&old, installed)
        {
            return Err(io::Error::new(
                error.kind(),
                format!(
                    "{error}. The old plugin could not go back ({rollback}), so {} is \
                     missing. The old plugin is {}.",
                    installed.display(),
                    old.display()
                ),
            ));
        }
        return Err(error);
    }
    Ok(())
}

/// Puts `text` in a PowerShell single-quoted string. PowerShell ends such a
/// string at `'` and also at the quotes U+2018, U+2019, U+201A and U+201B, so
/// each of them is doubled.
pub fn powershell_single_quoted(text: &str) -> String {
    let mut quoted = String::from("'");
    for c in text.chars() {
        if matches!(c, '\'' | '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}') {
            quoted.push(c);
        }
        quoted.push(c);
    }
    quoted.push('\'');
    quoted
}

/// Deletes the earlier `.old` files that are not loaded, and returns the first
/// free path for the current file.
fn free_old_path(installed: &Path) -> PathBuf {
    let path_with_suffix = |number: u32| {
        let mut path = installed.as_os_str().to_os_string();
        path.push(".old");
        if number > 0 {
            path.push(number.to_string());
        }
        PathBuf::from(path)
    };
    let mut number = 0;
    loop {
        let path = path_with_suffix(number);
        if !path.exists() || std::fs::remove_file(&path).is_ok() {
            return path;
        }
        number += 1;
    }
}

/// Reads one value of the TC registry keys with `reg.exe`.
fn tc_registry_value(name: &str) -> Option<String> {
    TC_REGISTRY_KEYS.iter().find_map(|key| {
        let output = Command::new("reg.exe")
            .args(["query", key, "/v", name])
            .output()
            .ok()?;
        reg_value(&String::from_utf8_lossy(&output.stdout), name)
    })
}

/// Expands `%COMMANDER_PATH%` and environment variables in a TC path.
fn expand_tc_path(path: &str) -> String {
    expand_variables(path, |name| {
        if name.eq_ignore_ascii_case("COMMANDER_PATH") {
            tc_registry_value("InstallDir")
        } else {
            std::env::var(name).ok()
        }
    })
}

/// Decodes a `wincmd.ini` file: UTF-16 with a BOM, or UTF-8 with or without a
/// BOM. Other bytes become U+FFFD, so a non-ASCII ANSI path does not match.
fn decode_ini(bytes: &[u8]) -> String {
    if let Some(utf16) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        let units: Vec<u16> = utf16
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u16::from_le_bytes(*pair))
            .collect();
        return String::from_utf16_lossy(&units);
    }
    let utf8 = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    String::from_utf8_lossy(utf8).into_owned()
}

/// Finds the plugin file that TC has registered in the `wincmd.ini` file at
/// `ini_path`. `expand` expands the variables in the path from the file.
pub fn installed_plugin_from_ini(
    ini_path: &Path,
    expand: impl Fn(&str) -> String,
) -> Result<PathBuf, String> {
    let bytes =
        std::fs::read(ini_path).map_err(|e| format!("cannot read {}: {e}", ini_path.display()))?;
    let ini = decode_ini(&bytes);
    let path = plugin_path_in_ini(&ini, CONTENT_PLUGIN_FILE).ok_or(format!(
        "Total Commander has not registered {CONTENT_PLUGIN_FILE} in {}. \
         Run `cargo xtask package`, and open the ZIP in Total Commander to install it.",
        ini_path.display()
    ))?;
    Ok(PathBuf::from(expand(&path)))
}

/// Finds the plugin file that TC has registered in its `wincmd.ini`.
fn find_installed_plugin() -> Result<PathBuf, String> {
    let ini_path =
        tc_registry_value("IniFileName").ok_or("cannot find Total Commander in the registry")?;
    let ini_path = PathBuf::from(expand_tc_path(&ini_path));
    installed_plugin_from_ini(&ini_path, expand_tc_path)
}

/// Runs `xtask replace-plugin` with administrator rights. This shows a UAC
/// prompt.
///
/// The window of the elevated process is hidden, so it writes its error to a
/// temporary file, and this function shows that error.
fn replace_plugin_elevated(new: &Path, installed: &Path) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let error_file =
        std::env::temp_dir().join(format!("uc-xtask-replace-{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&error_file);
    // Each argument is in double quotes for the process, because a path can
    // contain spaces.
    let argument = |path: &Path| powershell_single_quoted(&format!("\"{}\"", path.display()));
    let script = format!(
        "$p = Start-Process -FilePath {} \
         -ArgumentList 'replace-plugin',{},{},{} \
         -Verb RunAs -Wait -PassThru -WindowStyle Hidden -ErrorAction Stop; \
         exit $p.ExitCode",
        powershell_single_quoted(&exe.display().to_string()),
        argument(new),
        argument(installed),
        argument(&error_file),
    );
    let status = Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", &script])
        .status()
        .map_err(|e| format!("cannot start powershell.exe: {e}"))?;
    let child_error = std::fs::read_to_string(&error_file).ok();
    let _ = std::fs::remove_file(&error_file);
    if status.success() {
        return Ok(());
    }
    Err(match child_error {
        Some(error) => format!("cannot replace {}: {error}", installed.display()),
        None => "the plugin was not replaced with administrator rights. \
                 Accept the UAC prompt, or run the command in an administrator terminal."
            .to_string(),
    })
}

/// Builds the content plugin, and replaces the file that TC has registered.
pub fn install(root: &Path) -> Result<PathBuf, String> {
    let installed = find_installed_plugin()?;
    let new = build_content_plugin(root)?;
    match replace_plugin(&new, &installed) {
        Ok(()) => Ok(installed),
        Err(error) if error.kind() == ErrorKind::PermissionDenied => {
            replace_plugin_elevated(&new, &installed)?;
            Ok(installed)
        }
        Err(error) => Err(format!("cannot replace {}: {error}", installed.display())),
    }
}
