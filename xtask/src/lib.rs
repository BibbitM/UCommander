//! Tasks of `cargo xtask`.

use std::path::Path;
use std::process::Command;

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
