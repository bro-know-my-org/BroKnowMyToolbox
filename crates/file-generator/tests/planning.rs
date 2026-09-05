use std::collections::BTreeMap;

use bkmt_file_generator::{
    FileAction, FileGenerator, GenerationError, GenerationRequest, TemplateFile,
};

#[test]
fn planning_renders_variables_without_writing_files() {
    let destination = tempfile::tempdir().expect("destination should be created");
    let generator = FileGenerator::new();
    let request = GenerationRequest {
        destination: destination.path().to_path_buf(),
        files: vec![TemplateFile {
            path: "config/{{name}}.toml".to_string(),
            content: "server = \"{{server}}\"\n".to_string(),
        }],
        variables: BTreeMap::from([
            ("name".to_string(), "production".to_string()),
            ("server".to_string(), "example.test".to_string()),
        ]),
        overwrite: false,
    };

    let plan = generator
        .plan(request)
        .expect("generation should be planned");

    assert_eq!(plan.files().len(), 1);
    assert_eq!(
        plan.files()[0].relative_path,
        std::path::PathBuf::from("config/production.toml")
    );
    assert_eq!(plan.files()[0].content, "server = \"example.test\"\n");
    assert!(!destination.path().join("config/production.toml").exists());
}

#[test]
fn rendered_paths_cannot_escape_the_destination() {
    let destination = tempfile::tempdir().expect("destination should be created");
    let generator = FileGenerator::new();
    let request = GenerationRequest {
        destination: destination.path().to_path_buf(),
        files: vec![TemplateFile {
            path: "{{folder}}/settings.toml".to_string(),
            content: String::new(),
        }],
        variables: BTreeMap::from([("folder".to_string(), "../outside".to_string())]),
        overwrite: false,
    };

    assert_eq!(
        generator.plan(request),
        Err(GenerationError::UnsafePath(std::path::PathBuf::from(
            "../outside/settings.toml"
        )))
    );
}

#[test]
fn windows_style_paths_are_validated_consistently_on_every_platform() {
    let destination = tempfile::tempdir().expect("destination should be created");
    let generator = FileGenerator::new();

    for unsafe_path in [
        r"..\outside.txt",
        r"C:\outside.txt",
        "C:/outside.txt",
        r"\\server\share.txt",
        "output.txt:payload",
        "dir/C:payload",
        ".",
        "./file.txt",
        "dir/./file.txt",
        "dir//file.txt",
        "dir/",
        "CON",
        "con.txt",
        "CON .txt",
        "dir/Lpt9.log",
        "COM¹.txt",
        "LPT²",
        "CONIN$",
        "CONOUT$.txt",
        "foo.",
        "foo ",
        "dir./file.txt",
        "bad?.txt",
        "bad*.txt",
        "bad<name>.txt",
        "bad|name.txt",
        "bad\"name.txt",
        "bad\u{0}name.txt",
        "bad\u{1f}name.txt",
    ] {
        let result = generator.plan(GenerationRequest {
            destination: destination.path().to_path_buf(),
            files: vec![TemplateFile {
                path: unsafe_path.to_string(),
                content: String::new(),
            }],
            variables: BTreeMap::new(),
            overwrite: false,
        });

        assert_eq!(
            result,
            Err(GenerationError::UnsafePath(std::path::PathBuf::from(
                unsafe_path
            )))
        );
    }
}

#[test]
fn planning_marks_existing_files_as_conflicts_by_default() {
    let destination = tempfile::tempdir().expect("destination should be created");
    std::fs::write(destination.path().join("settings.toml"), "existing")
        .expect("existing file should be written");
    let generator = FileGenerator::new();

    let plan = generator
        .plan(GenerationRequest {
            destination: destination.path().to_path_buf(),
            files: vec![TemplateFile {
                path: "settings.toml".to_string(),
                content: "replacement".to_string(),
            }],
            variables: BTreeMap::new(),
            overwrite: false,
        })
        .expect("conflict should be represented in the plan");

    assert_eq!(plan.files()[0].action, FileAction::Conflict);
    assert_eq!(
        std::fs::read_to_string(destination.path().join("settings.toml"))
            .expect("existing file should remain readable"),
        "existing"
    );
}

#[test]
fn planning_rejects_duplicate_rendered_paths() {
    let destination = tempfile::tempdir().expect("destination should be created");
    let generator = FileGenerator::new();
    let request = GenerationRequest {
        destination: destination.path().to_path_buf(),
        files: vec![
            TemplateFile {
                path: "{{name}}.txt".to_string(),
                content: "first".to_string(),
            },
            TemplateFile {
                path: "same.txt".to_string(),
                content: "second".to_string(),
            },
        ],
        variables: BTreeMap::from([("name".to_string(), "same".to_string())]),
        overwrite: false,
    };

    assert_eq!(
        generator.plan(request),
        Err(GenerationError::DuplicatePath(std::path::PathBuf::from(
            "same.txt"
        )))
    );
}

#[test]
fn planning_rejects_file_directory_prefix_conflicts() {
    let destination = tempfile::tempdir().expect("destination should be created");
    let request = GenerationRequest {
        destination: destination.path().to_path_buf(),
        files: vec![
            TemplateFile {
                path: "entry".to_string(),
                content: "file".to_string(),
            },
            TemplateFile {
                path: "entry/child.txt".to_string(),
                content: "child".to_string(),
            },
        ],
        variables: BTreeMap::new(),
        overwrite: false,
    };

    assert_eq!(
        FileGenerator::new().plan(request),
        Err(GenerationError::PathConflict(std::path::PathBuf::from(
            "entry/child.txt"
        )))
    );
}

#[test]
fn planning_bounds_rendered_output_amplification() {
    let destination = tempfile::tempdir().expect("destination should be created");
    let request = GenerationRequest {
        destination: destination.path().to_path_buf(),
        files: vec![TemplateFile {
            path: "large.txt".to_string(),
            content: "{{value}}".repeat(65),
        }],
        variables: BTreeMap::from([("value".to_string(), "x".repeat(1024 * 1024))]),
        overwrite: false,
    };

    assert_eq!(
        FileGenerator::new().plan(request),
        Err(GenerationError::RenderedOutputTooLarge)
    );
}

#[test]
fn path_conflicts_are_case_insensitive_and_independent_of_input_order() {
    let destination = tempfile::tempdir().unwrap();
    for paths in [["Entry", "entry/child.txt"], ["entry/child.txt", "ENTRY"]] {
        let result = FileGenerator::new().plan(GenerationRequest {
            destination: destination.path().to_path_buf(),
            files: paths
                .iter()
                .map(|path| TemplateFile {
                    path: (*path).into(),
                    content: String::new(),
                })
                .collect(),
            variables: BTreeMap::new(),
            overwrite: false,
        });
        assert_eq!(result, Err(GenerationError::PathConflict(paths[1].into())));
    }
}

#[test]
fn large_flat_catalogs_and_shared_directory_prefixes_preserve_input_order() {
    let destination = tempfile::tempdir().unwrap();
    let paths: Vec<String> = (0..10_000)
        .map(|index| format!("shared/entry-{index}.txt"))
        .collect();
    let plan = FileGenerator::new()
        .plan(GenerationRequest {
            destination: destination.path().to_path_buf(),
            files: paths
                .iter()
                .map(|path| TemplateFile {
                    path: path.clone(),
                    content: String::new(),
                })
                .collect(),
            variables: BTreeMap::new(),
            overwrite: false,
        })
        .unwrap();
    assert_eq!(plan.files().len(), paths.len());
    for (file, path) in plan.files().iter().zip(paths) {
        assert_eq!(file.relative_path, std::path::PathBuf::from(path));
        assert_eq!(file.action, FileAction::Create);
    }
}

#[test]
fn directories_remain_conflicts_even_with_overwrite_enabled() {
    let destination = tempfile::tempdir().unwrap();
    std::fs::create_dir(destination.path().join("existing")).unwrap();
    std::fs::write(destination.path().join("existing/keep.txt"), "keep").unwrap();
    let plan = FileGenerator::new()
        .plan(GenerationRequest {
            destination: destination.path().to_path_buf(),
            files: vec![TemplateFile {
                path: "existing".into(),
                content: "replacement".into(),
            }],
            variables: BTreeMap::new(),
            overwrite: true,
        })
        .unwrap();
    assert_eq!(plan.files()[0].action, FileAction::Conflict);
    let report = FileGenerator::new().execute(&plan);
    assert_eq!(
        report.status(),
        bkmt_file_generator::ExecutionStatus::Conflict
    );
    assert_eq!(
        std::fs::read_to_string(destination.path().join("existing/keep.txt")).unwrap(),
        "keep"
    );
}

#[test]
fn deeply_nested_paths_reach_filesystem_validation_without_expanding_all_prefixes() {
    let destination = tempfile::tempdir().unwrap();
    // 100 KB input must not allocate 2.5 GB of duplicated parent prefixes.
    let path = format!("{}file.txt", "a/".repeat(50_000));
    let result = FileGenerator::new().plan(GenerationRequest {
        destination: destination.path().to_path_buf(),
        files: vec![TemplateFile {
            path,
            content: String::new(),
        }],
        variables: BTreeMap::new(),
        overwrite: false,
    });
    assert!(matches!(result, Err(GenerationError::PathInspection(_, _))));
}

#[cfg(unix)]
#[test]
fn overwrite_does_not_replace_a_symbolic_link() {
    let destination = tempfile::tempdir().unwrap();
    std::fs::write(destination.path().join("keep.txt"), "keep").unwrap();
    std::os::unix::fs::symlink("keep.txt", destination.path().join("link.txt")).unwrap();
    let plan = FileGenerator::new()
        .plan(GenerationRequest {
            destination: destination.path().to_path_buf(),
            files: vec![TemplateFile {
                path: "link.txt".into(),
                content: "replacement".into(),
            }],
            variables: BTreeMap::new(),
            overwrite: true,
        })
        .unwrap();
    assert_eq!(plan.files()[0].action, FileAction::Conflict);
    assert_eq!(
        FileGenerator::new().execute(&plan).status(),
        bkmt_file_generator::ExecutionStatus::Conflict
    );
    assert!(
        std::fs::symlink_metadata(destination.path().join("link.txt"))
            .unwrap()
            .is_symlink()
    );
    assert_eq!(
        std::fs::read_to_string(destination.path().join("keep.txt")).unwrap(),
        "keep"
    );
}

#[test]
fn ordinary_unicode_hidden_and_nonreserved_device_like_names_remain_valid() {
    let destination = tempfile::tempdir().unwrap();
    let plan = FileGenerator::new()
        .plan(GenerationRequest {
            destination: destination.path().to_path_buf(),
            files: [
                ".gitignore",
                "中文/配置.json",
                "COM10.txt",
                "auxiliary.txt",
                "file..txt",
                "directory/item.txt",
                "directory-name.txt",
            ]
            .into_iter()
            .map(|path| TemplateFile {
                path: path.into(),
                content: String::new(),
            })
            .collect(),
            variables: BTreeMap::new(),
            overwrite: false,
        })
        .unwrap();
    assert_eq!(plan.files().len(), 7);
    assert!(
        plan.files()
            .iter()
            .all(|file| file.action == FileAction::Create)
    );
}

#[test]
fn generation_errors_expose_stable_machine_codes() {
    assert_eq!(
        GenerationError::MissingVariable("name".to_string()).code(),
        "missing_variable"
    );
    assert_eq!(
        GenerationError::UnclosedVariable.code(),
        "unclosed_variable"
    );
    assert_eq!(
        GenerationError::UnsafePath(std::path::PathBuf::from("../outside")).code(),
        "unsafe_path"
    );
    assert_eq!(
        GenerationError::DuplicatePath(std::path::PathBuf::from("same.txt")).code(),
        "duplicate_path"
    );
}
