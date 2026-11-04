use xtask::release::check_tag_version;

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
