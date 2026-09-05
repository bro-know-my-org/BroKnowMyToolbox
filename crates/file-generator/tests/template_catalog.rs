use bkmt_file_generator::templates::{
    MAX_USER_TEMPLATE_BYTES, TemplateSource, load_template_catalog, read_template_file,
};

fn source(id: &str) -> String {
    serde_json::json!({
        "schemaVersion": 1, "id": id, "title": id, "variables": [],
        "files": [{ "path": "safe.txt", "content": "safe" }]
    })
    .to_string()
}

#[test]
fn damaged_user_files_do_not_hide_valid_templates_and_are_never_deleted() {
    let root = tempfile::tempdir().unwrap();
    for id in ["zeta", "alpha"] {
        std::fs::write(root.path().join(format!("{id}.json")), source(id)).unwrap();
    }
    std::fs::write(root.path().join("broken.json"), "{broken").unwrap();
    std::fs::write(root.path().join("encoding.json"), [0xff]).unwrap();
    std::fs::write(root.path().join("mismatch.json"), source("other")).unwrap();
    std::fs::write(
        root.path().join("basic-readme.json"),
        source("basic-readme"),
    )
    .unwrap();
    let oversized = std::fs::File::create(root.path().join("large.json")).unwrap();
    oversized.set_len(MAX_USER_TEMPLATE_BYTES + 1).unwrap();
    std::fs::write(root.path().join("notes.txt"), "not a template").unwrap();

    let catalog = load_template_catalog(root.path()).unwrap();
    assert_eq!(
        catalog
            .templates
            .iter()
            .map(|entry| entry.definition.id.as_str())
            .collect::<Vec<_>>(),
        ["basic-readme", "alpha", "zeta"]
    );
    assert_eq!(catalog.templates[0].source, TemplateSource::BuiltIn);
    assert_eq!(catalog.templates[1].source, TemplateSource::User);
    assert_eq!(catalog.templates[1].json, source("alpha"));
    assert_eq!(
        catalog
            .warnings
            .iter()
            .map(|warning| (warning.file_name.as_str(), warning.code.as_str()))
            .collect::<Vec<_>>(),
        [
            ("basic-readme.json", "invalid_template_id"),
            ("broken.json", "invalid_template"),
            ("encoding.json", "invalid_template"),
            ("large.json", "user_template_too_large"),
            ("mismatch.json", "invalid_template_id"),
        ]
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("broken.json")).unwrap(),
        "{broken"
    );
    let plan = bkmt_file_generator::FileGenerator::new()
        .plan_template(
            &catalog.templates[1].definition,
            root.path(),
            Default::default(),
            false,
        )
        .unwrap();
    assert_eq!(
        plan.files()[0].action,
        bkmt_file_generator::FileAction::Create
    );
    assert!(!root.path().join("safe.txt").exists());
}

#[test]
fn missing_directory_is_empty_but_broken_storage_remains_an_error() {
    let root = tempfile::tempdir().unwrap();
    let directory = root.path().join("templates");
    let catalog = load_template_catalog(&directory).unwrap();
    assert_eq!(catalog.templates.len(), 1);
    assert!(catalog.warnings.is_empty());
    assert!(!directory.exists());
    std::fs::write(&directory, "not a directory").unwrap();
    assert_eq!(
        load_template_catalog(&directory).unwrap_err().code(),
        "template_unreadable"
    );
}

#[test]
fn user_template_reader_has_an_inclusive_one_mib_limit() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("boundary.json");
    let mut json = source("boundary");
    json.push_str(&" ".repeat(MAX_USER_TEMPLATE_BYTES as usize - json.len()));
    std::fs::write(&file, &json).unwrap();
    assert_eq!(read_template_file(&file).unwrap(), json);
    json.push(' ');
    std::fs::write(&file, json).unwrap();
    assert_eq!(
        read_template_file(&file).unwrap_err().code(),
        "user_template_too_large"
    );
}

#[cfg(any(unix, windows))]
#[test]
fn linked_templates_are_reported_without_following_live_or_dangling_targets() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("private.txt");
    let link = root.path().join("linked.json");
    std::fs::write(&target, source("linked")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&target, &link).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&target, &link)
        .expect("Windows test runner requires symlink privileges or Developer Mode");
    for dangling in [false, true] {
        if dangling {
            std::fs::remove_file(&target).unwrap();
        }
        assert_eq!(
            read_template_file(&link).unwrap_err().code(),
            "template_unreadable"
        );
        let catalog = load_template_catalog(root.path()).unwrap();
        assert_eq!(catalog.templates.len(), 1);
        assert_eq!(catalog.warnings.len(), 1);
        assert_eq!(catalog.warnings[0].file_name, "linked.json");
        assert_eq!(catalog.warnings[0].code, "template_unreadable");
    }
}

#[cfg(any(unix, windows))]
#[test]
fn dangling_directory_links_are_storage_errors_not_empty_catalogs() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("missing");
    let link = root.path().join("linked-directory");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&target, &link).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(&target, &link)
        .expect("Windows test runner requires symlink privileges or Developer Mode");
    for directory in [link.clone(), link.join("nested/templates")] {
        assert_eq!(
            load_template_catalog(&directory).unwrap_err().code(),
            "template_unreadable"
        );
    }
    std::fs::create_dir(target).unwrap();
    let catalog = load_template_catalog(&link.join("nested/templates")).unwrap();
    assert_eq!(catalog.templates.len(), 1);
    assert!(catalog.warnings.is_empty());
}
