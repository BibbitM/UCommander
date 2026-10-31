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
