use std::process::Command;

fn uc() -> Command {
    Command::new(env!("CARGO_BIN_EXE_uc"))
}

#[test]
fn version_prints_name_and_version() {
    let output = uc().arg("--version").output().unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        format!("uc {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn unknown_arguments_fail_with_usage() {
    let output = uc().arg("--unknown").output().unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr).unwrap().contains("Usage:"));
}
