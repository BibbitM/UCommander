//! Tasks of the release workflow.

/// Checks that `tag` is `v` + `version`.
pub fn check_tag_version(tag: &str, version: &str) -> Result<(), String> {
    if tag.strip_prefix('v') == Some(version) {
        Ok(())
    } else {
        Err(format!(
            "the tag {tag} does not match the workspace version {version}. \
             Use the tag v{version}."
        ))
    }
}
