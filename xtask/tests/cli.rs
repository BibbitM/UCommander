use std::process::Command;

fn xtask() -> Command {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
}

#[test]
fn no_task_fails_with_usage() {
    let output = xtask().output().unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage:"));
}

#[test]
fn an_unknown_task_fails_with_usage() {
    let output = xtask().arg("no-such-task").output().unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage:"));
}

#[test]
fn tag_version_passes_for_the_workspace_version() {
    let tag = format!("v{}", env!("CARGO_PKG_VERSION"));
    let output = xtask().args(["tag-version", &tag]).output().unwrap();

    assert!(output.status.success());
}

#[test]
fn tag_version_fails_for_another_version() {
    let output = xtask().args(["tag-version", "v999.0.0"]).output().unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("v999.0.0"));
}
