use xtask::release::{check_tag_version, release_notes};

const CHANGELOG: &str = "\
# Changelog

Intro text.

## [Unreleased]

## [0.2.0] - 2026-12-01

### Added

- Asset class field.

## [0.1.0] - 2026-11-04

### Added

- Is package field.

## [0.0.9] - 2026-10-01

[Unreleased]: https://example.com/compare/v0.2.0...HEAD
[0.2.0]: https://example.com/compare/v0.1.0...v0.2.0
";

const NOTES_0_1_0: &str = "### Added\n\n- Is package field.\n";

#[test]
fn a_tag_with_the_workspace_version_passes() {
    assert_eq!(check_tag_version("v0.1.0", "0.1.0"), Ok(()));
}

#[test]
fn a_tag_with_a_different_version_fails() {
    let error = check_tag_version("v0.1.1", "0.1.0").unwrap_err();

    assert!(error.contains("v0.1.1"));
    assert!(error.contains("0.1.0"));
}

#[test]
fn a_tag_without_the_v_prefix_fails() {
    assert!(check_tag_version("0.1.0", "0.1.0").is_err());
}

#[test]
fn a_tag_with_a_suffix_fails() {
    assert!(check_tag_version("v0.1.0-rc1", "0.1.0").is_err());
}

#[test]
fn release_notes_are_the_text_of_the_version_section() {
    assert_eq!(
        release_notes(CHANGELOG, "0.1.0"),
        Ok(NOTES_0_1_0.to_string())
    );
}

#[test]
fn release_notes_of_the_last_section_do_not_include_the_links() {
    let changelog = CHANGELOG.replace("## [0.0.9] - 2026-10-01\n\n", "");

    assert_eq!(
        release_notes(&changelog, "0.1.0"),
        Ok(NOTES_0_1_0.to_string())
    );
}

#[test]
fn release_notes_do_not_match_a_shorter_version() {
    assert!(release_notes(CHANGELOG, "0.2").is_err());
}

#[test]
fn release_notes_of_a_missing_version_fail() {
    let error = release_notes(CHANGELOG, "0.3.0").unwrap_err();

    assert!(error.contains("0.3.0"));
}

#[test]
fn release_notes_of_an_empty_section_fail() {
    let error = release_notes(CHANGELOG, "0.0.9").unwrap_err();

    assert!(error.contains("0.0.9"));
}

#[test]
fn release_notes_accept_crlf_line_endings() {
    let changelog = CHANGELOG.replace('\n', "\r\n");

    assert_eq!(
        release_notes(&changelog, "0.1.0"),
        Ok(NOTES_0_1_0.to_string())
    );
}

#[test]
fn the_real_changelog_has_notes_for_the_workspace_version() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../CHANGELOG.md");
    let changelog = std::fs::read_to_string(path).unwrap();

    assert!(release_notes(&changelog, xtask::VERSION).is_ok());
}
