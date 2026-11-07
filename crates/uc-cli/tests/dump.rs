//! `uc dump <file>`.

#[path = "../../uc-format/tests/common/mod.rs"]
mod common;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use common::{Export, Import, Package, Str};

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

/// A package with 6 names, 2 imports (`/Script/Engine` and its class
/// `Actor`), and 2 exports (`BP_Test`, and `BP_Test:Root_0`).
fn package() -> Package {
    Package {
        imports: vec![
            Import {
                class_package: (0, 0),
                class_name: (3, 0),
                outer_index: 0,
                object_name: (0, 0),
                package_name: (5, 0),
                import_optional: 0,
            },
            Import {
                class_package: (0, 0),
                class_name: (2, 0),
                outer_index: -1,
                object_name: (1, 0),
                package_name: (5, 0),
                import_optional: 1,
            },
        ],
        exports: vec![
            Export {
                class_index: -2,
                object_name: (4, 0),
                object_flags: 0x0000_000B,
                serial_size: 733,
                serial_offset: 18857,
                not_for_client: 1,
                is_asset: 1,
                dependencies: [-1, 0, 0, 0, 0],
                script_serialization_end_offset: 725,
                ..Export::default()
            },
            Export {
                class_index: -2,
                super_index: -2,
                template_index: -2,
                outer_index: 1,
                object_name: (6, 1),
                serial_size: 13,
                serial_offset: 19590,
                package_flags: 0x0000_0400,
                ..Export::default()
            },
        ],
        ..Package::new(&[
            "/Script/Engine",
            "Actor",
            "Class",
            "Package",
            "BP_Test",
            "None",
            "Root",
        ])
    }
}

#[test]
fn dump_prints_the_summary_and_the_tables() {
    let path = temp_file("synthetic.uasset", &package().to_bytes());

    let output = dump(&path);

    assert!(output.status.success(), "{}", text(&output.stderr));
    assert_snapshot(&text(&output.stdout), "dump_synthetic.txt");
}

#[test]
fn each_line_is_a_name_and_a_value() {
    let path = temp_file("lines.uasset", &package().to_bytes());

    let output = dump(&path);

    assert!(output.status.success(), "{}", text(&output.stderr));
    for line in text(&output.stdout).lines() {
        let (name, value) = line.split_once(": ").expect(line);
        assert!(!name.trim().is_empty() && !value.is_empty(), "{line}");
    }
}

#[test]
fn dump_shows_package_indexes_with_their_paths() {
    let path = temp_file("paths.uasset", &package().to_bytes());

    let output = dump(&path);

    let stdout = text(&output.stdout);
    assert!(stdout.contains("Name 6: Root\n"), "{stdout}");
    assert!(
        stdout.contains("Import 1 outer: -1 (/Script/Engine)\n"),
        "{stdout}"
    );
    assert!(
        stdout.contains("Export 1 path: /Game/Fixtures/BP_Test.BP_Test:Root_0\n"),
        "{stdout}"
    );
    assert!(stdout.contains("Export 0 outer: 0 (null)\n"), "{stdout}");
}

#[test]
fn dump_reads_a_header_that_is_larger_than_64_kib() {
    let mut input = package();
    input.summary.custom_versions = vec![([1, 2, 3, 4], 5); 4000];
    let path = temp_file("large.uasset", &input.to_bytes());

    let output = dump(&path);

    assert!(output.status.success(), "{}", text(&output.stderr));
    let stdout = text(&output.stdout);
    assert!(stdout.contains("Custom version count: 4000\n"));
    assert!(stdout.contains("Export 1 object name: Root_0\n"));
}

#[test]
fn dump_prints_a_warning_for_a_newer_version() {
    let mut input = package();
    input.summary.file_version_ue5 = 1019;
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
    let bytes = package().to_bytes();
    let path = temp_file("truncated.uasset", &bytes[..bytes.len() - 1]);

    let output = dump(&path);

    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("truncated"));
}

#[test]
fn a_damaged_index_gives_an_error_and_exit_code_1() {
    let mut input = package();
    input.exports[1].outer_index = 3;
    let path = temp_file("bad-index.uasset", &input.to_bytes());

    let output = dump(&path);

    assert_eq!(output.status.code(), Some(1));
    assert!(
        text(&output.stderr).contains("bad value of export outer index"),
        "{}",
        text(&output.stderr)
    );
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
    let mut input = package();
    input.summary.package_name = Str::Utf16("/Game/Żółw");
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
    let mut input = package();
    input.summary.custom_version_count = Some(i32::MAX);
    let mut bytes = input.to_bytes();
    // Larger than the read limit of 16 MiB.
    bytes.resize(17 * 1024 * 1024, 0);
    let path = temp_file("damaged-large.uasset", &bytes);

    let output = dump(&path);

    assert_eq!(output.status.code(), Some(1));
    assert!(
        text(&output.stderr).contains("header is larger than 16 MiB"),
        "{}",
        text(&output.stderr)
    );
}

#[test]
fn control_characters_in_a_string_are_escaped() {
    let mut input = package();
    input.summary.package_name = Str::Ansi("/Game/\u{1b}[31mRed\n");
    input.names[6] = common::Name::ansi("Ro\rot");
    let path = temp_file("control.uasset", &input.to_bytes());

    let output = dump(&path);

    assert!(output.status.success(), "{}", text(&output.stderr));
    let stdout = text(&output.stdout);
    assert!(
        stdout.contains(r"Package name: /Game/\u{1b}[31mRed\u{a}"),
        "{stdout}"
    );
    assert!(stdout.contains(r"Name 6: Ro\u{d}ot"), "{stdout}");
}
