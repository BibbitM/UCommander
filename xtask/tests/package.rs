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

fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

/// A new, empty folder for one test.
fn test_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("uc-xtask-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn tar(args: &[&str]) -> String {
    let output = std::process::Command::new(xtask::tar_path())
        .args(args)
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}

fn fake_dll(dir: &std::path::Path) -> std::path::PathBuf {
    let path = dir.join("uc_content.dll");
    std::fs::write(&path, b"MZ fake dll").unwrap();
    path
}

#[test]
fn the_zip_has_exactly_the_plugin_files() {
    let dir = test_dir("files");
    let dll = fake_dll(&dir);

    let zip = xtask::make_content_zip(&dll, &root(), &dir.join("dist"), "1.2.3").unwrap();

    assert!(zip.ends_with("UCommanderContent-1.2.3.zip"));
    let mut names: Vec<String> = tar(&["-tf", zip.to_str().unwrap()])
        .lines()
        .map(str::to_string)
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "CHANGELOG.md",
            "LICENSE-APACHE",
            "LICENSE-MIT",
            "README.md",
            "UCommanderContent.wdx64",
            "pluginst.inf",
        ]
    );
}

#[test]
fn the_zip_has_the_dll_as_plugin_file_and_the_version_in_pluginst() {
    let dir = test_dir("content");
    let dll = fake_dll(&dir);

    let zip = xtask::make_content_zip(&dll, &root(), &dir.join("dist"), "1.2.3").unwrap();

    let zip = zip.to_str().unwrap();
    assert_eq!(
        tar(&["-xOf", zip, "UCommanderContent.wdx64"]),
        "MZ fake dll"
    );
    assert!(tar(&["-xOf", zip, "pluginst.inf"]).contains("version=1.2.3\r\n"));
}

#[test]
fn a_new_zip_replaces_the_old_zip() {
    let dir = test_dir("replace");
    let dll = fake_dll(&dir);
    let dist = dir.join("dist");

    xtask::make_content_zip(&dll, &root(), &dist, "1.2.3").unwrap();
    let zip = xtask::make_content_zip(&dll, &root(), &dist, "1.2.3").unwrap();

    assert_eq!(tar(&["-tf", zip.to_str().unwrap()]).lines().count(), 6);
}

/// A `.pdb` file with a wrong name must not go in a ZIP.
#[test]
fn a_file_that_is_not_a_dll_is_refused() {
    let dir = test_dir("not-dll");
    let pdb = dir.join("uc_content.dll");
    std::fs::write(&pdb, b"Microsoft C/C++ MSF 7.00").unwrap();

    let result = xtask::make_content_zip(&pdb, &root(), &dir.join("dist"), "1.2.3");

    assert!(result.unwrap_err().contains("not a DLL"));
}
