use std::path::PathBuf;

use toolbox_core::{
    DataRootCandidates, DataRootDiscovery, DataRootSource, discover_data_root, resolve_data_root,
};

fn path(value: &str) -> PathBuf {
    PathBuf::from(value)
}

#[test]
fn portable_marker_selects_a_data_directory_beside_the_executable() {
    let package = tempfile::tempdir().expect("portable package should be created");
    let executable = package.path().join("bkmt");
    std::fs::write(package.path().join("portable.bkmt"), b"")
        .expect("portable marker should be written");

    let resolved = discover_data_root(DataRootDiscovery {
        explicit: None,
        environment: None,
        executable,
        portable_requested: false,
        system: path("/system"),
    })
    .expect("portable data root should resolve");

    assert_eq!(resolved.path, package.path().join("data"));
    assert_eq!(resolved.source, DataRootSource::Portable);
}

#[test]
fn unwritable_portable_location_falls_back_to_the_system_data_root() {
    let package = tempfile::tempdir().expect("portable package should be created");
    let blocked_root = package.path().join("blocked");
    std::fs::write(&blocked_root, b"not a directory").expect("blocking file should be created");

    let resolved = discover_data_root(DataRootDiscovery {
        explicit: None,
        environment: None,
        executable: blocked_root.join("bkmt"),
        portable_requested: true,
        system: package.path().join("system-data"),
    })
    .expect("system fallback should resolve");

    assert_eq!(resolved.path, package.path().join("system-data"));
    assert_eq!(resolved.source, DataRootSource::System);
}

#[test]
fn data_root_uses_the_highest_priority_available_candidate() {
    let cases = [
        (
            DataRootCandidates {
                explicit: Some(path("/explicit")),
                environment: Some(path("/environment")),
                portable: Some(path("/portable/data")),
                system: path("/system"),
            },
            path("/explicit"),
            DataRootSource::Explicit,
        ),
        (
            DataRootCandidates {
                explicit: None,
                environment: Some(path("/environment")),
                portable: Some(path("/portable/data")),
                system: path("/system"),
            },
            path("/environment"),
            DataRootSource::Environment,
        ),
        (
            DataRootCandidates {
                explicit: None,
                environment: None,
                portable: Some(path("/portable/data")),
                system: path("/system"),
            },
            path("/portable/data"),
            DataRootSource::Portable,
        ),
        (
            DataRootCandidates {
                explicit: None,
                environment: None,
                portable: None,
                system: path("/system"),
            },
            path("/system"),
            DataRootSource::System,
        ),
    ];

    for (candidates, expected_path, expected_source) in cases {
        let resolved = resolve_data_root(candidates).expect("candidate should resolve");
        assert_eq!(resolved.path, expected_path);
        assert_eq!(resolved.source, expected_source);
    }
}

#[test]
fn an_override_does_not_create_a_portable_data_directory() {
    let package = tempfile::tempdir().expect("portable package should be created");
    let explicit = tempfile::tempdir().expect("explicit root should be created");

    let resolved = discover_data_root(DataRootDiscovery {
        explicit: Some(explicit.path().to_path_buf()),
        environment: None,
        executable: package.path().join("bkmt"),
        portable_requested: true,
        system: package.path().join("system"),
    })
    .expect("explicit root should win without portable probing");

    assert_eq!(resolved.path, explicit.path());
    assert!(!package.path().join("data").exists());
}

#[test]
fn relative_executables_cannot_enable_portable_mode() {
    let system = tempfile::tempdir().expect("system root should be created");
    let resolved = discover_data_root(DataRootDiscovery {
        explicit: None,
        environment: None,
        executable: PathBuf::from("bkmt"),
        portable_requested: true,
        system: system.path().to_path_buf(),
    })
    .expect("relative executable should fall back to system data");

    assert_eq!(resolved.source, DataRootSource::System);
}

#[test]
fn relative_environment_roots_are_rejected() {
    let system = tempfile::tempdir().expect("system root should be created");
    let error = discover_data_root(DataRootDiscovery {
        explicit: None,
        environment: Some(PathBuf::from("relative-data")),
        executable: system.path().join("bkmt"),
        portable_requested: false,
        system: system.path().to_path_buf(),
    })
    .expect_err("relative environment root should be ambiguous");

    assert!(error.to_string().contains("must be absolute"));
}
