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
