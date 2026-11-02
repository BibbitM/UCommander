//! Tasks of `cargo xtask`.

pub mod install;
pub mod plan;

use std::path::{Path, PathBuf};
use std::process::Command;

/// Name of the content plugin.
const CONTENT_PLUGIN: &str = "UCommanderContent";

/// File name of the content plugin in TC.
pub const CONTENT_PLUGIN_FILE: &str = "UCommanderContent.wdx64";

/// Repository files that go in each plugin ZIP.
pub const DOCUMENTS: [&str; 6] = [
    "README.md",
    "CHANGELOG.md",
    "LICENSE-MIT",
    "LICENSE-APACHE",
    "docs/FEATURES.md",
    "docs/PLAN.md",
];

/// Runs one command in `root`. Returns an error if it fails.
fn run(root: &Path, program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .map_err(|e| format!("cannot start {program}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("failed: {program} {}", args.join(" ")))
    }
}

/// Runs all checks. Stops at the first failure.
pub fn check(root: &Path) -> Result<(), String> {
    let steps: [&[&str]; 4] = [
        &["fmt", "--all", "--check"],
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
        &["test", "--workspace"],
        &["deny", "check"],
    ];
    for args in steps {
        run(root, "cargo", args)?;
    }
    Ok(())
}

/// Template of the TC plugin installer file for the content plugin.
pub const CONTENT_PLUGINST: &str = include_str!("../../packaging/content/pluginst.inf");

/// Fills in the version, and gives the text CRLF line endings for TC.
pub fn render_pluginst(template: &str, version: &str) -> String {
    template
        .replace("{version}", version)
        .replace("\r\n", "\n")
        .replace('\n', "\r\n")
}

/// The workspace version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// `tar.exe` of Windows. A `tar` from the PATH can be GNU tar, which cannot
/// make ZIP files.
pub fn tar_path() -> PathBuf {
    let system_root = std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:\Windows".into());
    Path::new(&system_root).join("System32").join("tar.exe")
}

/// The target folder of Cargo.
fn target_dir(root: &Path) -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"))
}

/// Builds the release DLL of the content plugin, and returns its path.
pub fn build_content_plugin(root: &Path) -> Result<PathBuf, String> {
    run(root, "cargo", &["build", "-p", "uc-content", "--release"])?;
    Ok(target_dir(root).join("release").join("uc_content.dll"))
}

/// Builds the content plugin, and makes its ZIP in `target/dist/`.
pub fn package(root: &Path) -> Result<PathBuf, String> {
    let dll = build_content_plugin(root)?;
    let dist = target_dir(root).join("dist");
    make_content_zip(&dll, root, &dist, VERSION)
}

/// Makes `UCommanderContent-<version>.zip` in `dist` from the file `dll`.
pub fn make_content_zip(
    dll: &Path,
    root: &Path,
    dist: &Path,
    version: &str,
) -> Result<PathBuf, String> {
    let describe = |e: std::io::Error| format!("cannot make the ZIP: {e}");
    let bytes = std::fs::read(dll).map_err(|e| format!("cannot read {}: {e}", dll.display()))?;
    if !bytes.starts_with(b"MZ") {
        return Err(format!("{} is not a DLL", dll.display()));
    }
    // A `RUSTFLAGS` variable replaces the static runtime setting in
    // `.cargo/config.toml`. The import table stores DLL names as ASCII.
    let vc_runtime = b"vcruntime140.dll";
    if bytes
        .windows(vc_runtime.len())
        .any(|window| window.eq_ignore_ascii_case(vc_runtime))
    {
        return Err(format!(
            "{} needs the VC++ runtime. Do not set RUSTFLAGS, or add \
             `-C target-feature=+crt-static` to it.",
            dll.display()
        ));
    }

    let staging = dist.join(format!("{CONTENT_PLUGIN}-{version}"));
    if staging.exists() {
        std::fs::remove_dir_all(&staging).map_err(describe)?;
    }
    std::fs::create_dir_all(staging.join("docs")).map_err(describe)?;
    std::fs::write(staging.join(CONTENT_PLUGIN_FILE), bytes).map_err(describe)?;
    let pluginst = render_pluginst(CONTENT_PLUGINST, version);
    std::fs::write(staging.join("pluginst.inf"), pluginst).map_err(describe)?;
    for name in DOCUMENTS {
        std::fs::copy(root.join(name), staging.join(name)).map_err(describe)?;
    }

    let zip = dist.join(format!("{CONTENT_PLUGIN}-{version}.zip"));
    if zip.exists() {
        std::fs::remove_file(&zip).map_err(describe)?;
    }
    let tar = tar_path();
    let zip_arg = zip.to_string_lossy();
    let mut args = vec![
        "-a",
        "-c",
        "-f",
        &zip_arg,
        CONTENT_PLUGIN_FILE,
        "pluginst.inf",
    ];
    args.extend(DOCUMENTS);
    run(&staging, &tar.to_string_lossy(), &args)?;
    std::fs::remove_dir_all(&staging).map_err(describe)?;
    Ok(zip)
}
