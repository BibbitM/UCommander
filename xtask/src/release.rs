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

/// Returns the text of the `## [<version>]` section of `changelog`, without
/// the heading. The section stops at the next `## ` heading, or at the link
/// definitions at the end of the file.
pub fn release_notes(changelog: &str, version: &str) -> Result<String, String> {
    let heading = format!("## [{version}]");
    let mut lines = changelog.lines();
    lines
        .by_ref()
        .find(|line| *line == heading || line.starts_with(&format!("{heading} ")))
        .ok_or(format!("CHANGELOG.md has no section {heading}"))?;
    let section: Vec<&str> = lines
        .take_while(|line| !line.starts_with("## ") && !is_link_definition(line))
        .collect();
    let text = section.join("\n").trim().to_string();
    if text.is_empty() {
        Err(format!("the section {heading} in CHANGELOG.md is empty"))
    } else {
        Ok(text + "\n")
    }
}

/// Returns true for a Markdown link definition, such as `[0.1.0]: https://…`.
fn is_link_definition(line: &str) -> bool {
    line.starts_with('[') && line.contains("]: ")
}
