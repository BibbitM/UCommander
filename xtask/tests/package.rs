use xtask::render_pluginst;

const TEMPLATE: &str = "[plugininstall]\ntype=wdx\nversion={version}\n";

#[test]
fn pluginst_gets_the_version() {
    let text = render_pluginst(TEMPLATE, "1.2.3");

    assert!(text.contains("version=1.2.3"));
    assert!(!text.contains("{version}"));
}

#[test]
fn pluginst_has_crlf_line_endings() {
    let text = render_pluginst(TEMPLATE, "1.2.3");

    assert_eq!(text.matches("\r\n").count(), text.matches('\n').count());
}

#[test]
fn the_template_also_works_with_crlf_input() {
    let text = render_pluginst("a\r\nb\n", "1.2.3");

    assert_eq!(text, "a\r\nb\r\n");
}

#[test]
fn the_content_plugin_template_names_the_plugin_file() {
    let text = render_pluginst(xtask::CONTENT_PLUGINST, "1.2.3");

    for line in [
        "[plugininstall]",
        "type=wdx",
        "file=UCommanderContent.wdx64",
        "defaultdir=UCommanderContent",
        "version=1.2.3",
    ] {
        assert!(text.contains(line), "{line}");
    }
}

fn repository_file(name: &str) -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    std::fs::read_to_string(root.join(name)).unwrap()
}

/// The README goes in the ZIP, where a relative link does not work.
#[test]
fn the_readme_has_only_absolute_links() {
    let readme = repository_file("README.md");

    for (start, _) in readme.match_indices("](") {
        let target = &readme[start + 2..];
        assert!(
            target.starts_with("https://"),
            "relative link: {target:.40}"
        );
    }
}

#[test]
fn the_readme_links_to_the_source() {
    assert!(
        repository_file("README.md").contains("Source code: https://github.com/BibbitM/UCommander")
    );
}
