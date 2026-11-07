//! `uc dump <file>`.

#[path = "../../uc-format/tests/common/mod.rs"]
mod common;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use common::{Str, Summary};

/// Compares `actual` with the stored file `tests/snapshots/<name>`. Set the
/// environment variable `UC_UPDATE_SNAPSHOTS=1` to write the file again.
fn assert_snapshot(actual: &str, name: &str) {
    let snapshot = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(name);
    if std::env::var_os("UC_UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(&snapshot, actual).unwrap();
    }
    assert_eq!(actual, std::fs::read_to_string(&snapshot).unwrap());
}

fn dump(path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_uc"))
        .arg("dump")
        .arg(path)
        .output()
        .unwrap()
}

/// Writes `bytes` to a new file in the temporary folder.
fn temp_file(name: &str, bytes: &[u8]) -> PathBuf {
    let path = std::env::temp_dir().join(format!("uc-dump-{}-{name}", std::process::id()));
    std::fs::write(&path, bytes).unwrap();
    path
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[test]
fn dump_prints_the_summary_fields() {
    let path = temp_file("synthetic.uasset", &Summary::default().to_bytes());

    let output = dump(&path);

    assert!(output.status.success(), "{}", text(&output.stderr));
    assert_snapshot(&text(&output.stdout), "dump_synthetic.txt");
}

#[test]
fn each_line_is_a_name_and_a_value() {
    let path = temp_file("lines.uasset", &Summary::default().to_bytes());

    let output = dump(&path);

    assert!(output.status.success(), "{}", text(&output.stderr));
    for line in text(&output.stdout).lines() {
        let (name, value) = line.split_once(": ").expect(line);
        assert!(!name.trim().is_empty() && !value.is_empty(), "{line}");
    }
}

#[test]
fn dump_reads_a_summary_that_is_larger_than_64_kib() {
    let input = Summary {
        custom_versions: vec![([1, 2, 3, 4], 5); 4000],
        ..Summary::default()
    };
    let path = temp_file("large.uasset", &input.to_bytes());

    let output = dump(&path);

    assert!(output.status.success(), "{}", text(&output.stderr));
    assert!(text(&output.stdout).contains("Custom version count: 4000\n"));
}

#[test]
fn dump_prints_a_warning_for_a_newer_version() {
    let input = Summary {
        file_version_ue5: 1019,
        ..Summary::default()
    };
    let path = temp_file("newer.uasset", &input.to_bytes());

    let output = dump(&path);

    assert!(output.status.success());
    assert!(
        text(&output.stdout).contains(
            "Warning: file version UE5 1019 is newer than the newest known version 1018\n"
        )
    );
}

#[test]
fn a_file_that_is_not_a_package_gives_an_error_and_exit_code_1() {
    let path = temp_file("text.txt", b"This is a text file, not a package.");

    let output = dump(&path);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(text(&output.stderr).contains("not a package"));
}

#[test]
fn a_truncated_package_gives_an_error_and_exit_code_1() {
    let bytes = Summary::default().to_bytes();
    let path = temp_file("truncated.uasset", &bytes[..bytes.len() - 1]);

    let output = dump(&path);

    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("truncated"));
}

#[test]
fn a_missing_file_gives_an_error_and_exit_code_1() {
    let output = dump(Path::new("this-file-does-not-exist.uasset"));

    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("this-file-does-not-exist.uasset"));
}

#[test]
fn dump_without_a_file_gives_the_usage() {
    let output = Command::new(env!("CARGO_BIN_EXE_uc"))
        .arg("dump")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("Usage:"));
}

#[test]
fn a_utf16_package_name_is_printed() {
    let input = Summary {
        package_name: Str::Utf16("/Game/Żółw"),
        ..Summary::default()
    };
    let path = temp_file("utf16.uasset", &input.to_bytes());

    let output = dump(&path);

    assert!(text(&output.stdout).contains("Package name: /Game/Żółw\n"));
}

#[test]
fn dump_of_the_blueprint_fixture_matches_the_snapshot() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/editor/BP_SummaryActor.uasset");

    let output = dump(&fixture);

    assert!(output.status.success(), "{}", text(&output.stderr));
    assert_snapshot(&text(&output.stdout), "dump_bp_summary_actor.txt");
}

#[test]
fn a_damaged_count_in_a_large_file_stops_at_the_read_limit() {
    let mut bytes = Summary {
        custom_version_count: Some(i32::MAX),
        ..Summary::default()
    }
    .to_bytes();
    // Larger than the read limit of 16 MiB.
    bytes.resize(17 * 1024 * 1024, 0);
    let path = temp_file("damaged-large.uasset", &bytes);

    let output = dump(&path);

    assert_eq!(output.status.code(), Some(1));
    assert!(
        text(&output.stderr).contains("summary is larger than 16 MiB"),
        "{}",
        text(&output.stderr)
    );
}

#[test]
fn control_characters_in_a_string_are_escaped() {
    let input = Summary {
        package_name: Str::Ansi("/Game/\u{1b}[31mRed\n"),
        ..Summary::default()
    };
    let path = temp_file("control.uasset", &input.to_bytes());

    let output = dump(&path);

    assert!(output.status.success(), "{}", text(&output.stderr));
    assert!(
        text(&output.stdout).contains(r"Package name: /Game/\u{1b}[31mRed\u{a}"),
        "{}",
        text(&output.stdout)
    );
}
