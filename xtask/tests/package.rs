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
