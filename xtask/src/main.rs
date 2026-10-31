//! Build automation for UCommander. Run it with `cargo xtask <task>`.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
Usage: cargo xtask <task>

Tasks:
  check    Run the format check, lint, tests and dependency check.
  package  Build the release plugin and make its ZIP in target/dist/.
  install  Replace the plugin file that Total Commander has registered.
  plan-status <branch>  Check that the plan point of the branch is Done.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let task = args.first().map(String::as_str);
    let result = match task {
        Some("check") => xtask::check(&repository_root()),
        Some("package") => xtask::package(&repository_root()).map(|zip| {
            println!("{}", zip.display());
        }),
        Some("install") => xtask::install::install(&repository_root()).map(|path| {
            println!("Installed {}. Restart Total Commander.", path.display());
        }),
        Some("plan-status") if args.len() == 2 => check_plan_status(&args[1]),
        Some("replace-plugin") if args.len() == 3 => {
            xtask::install::replace_plugin(Path::new(&args[1]), Path::new(&args[2]))
                .map_err(|e| e.to_string())
        }
        _ => Err(USAGE.to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

/// The repository root: the parent folder of the `xtask` crate.
fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask is in the repository root")
        .to_path_buf()
}

fn check_plan_status(branch: &str) -> Result<(), String> {
    let path = repository_root().join("docs").join("PLAN.md");
    let plan = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    xtask::plan::check_plan_status(branch, &plan)
}
