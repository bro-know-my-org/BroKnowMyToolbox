use toolbox_core::{CapabilityId, ConsentStore, FileConsentStore};

fn assert_rejected(root: &std::path::Path) {
    let store = FileConsentStore::new(root);
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(store.get("file-generator", CapabilityId::FilesystemWrite));
    });
    let result = receiver
        .recv_timeout(std::time::Duration::from_secs(2))
        .expect("invalid consent storage must not block");
    assert!(result.is_err(), "invalid storage must never grant consent");
}

#[test]
fn consent_directory_and_oversized_file_are_rejected() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("config/consents.json");
    std::fs::create_dir_all(&path).unwrap();
    assert_rejected(root.path());
    std::fs::remove_dir(&path).unwrap();
    std::fs::File::create(&path)
        .unwrap()
        .set_len(64 * 1024 + 1)
        .unwrap();
    assert_rejected(root.path());
}

#[cfg(unix)]
#[test]
fn consent_symlink_cannot_grant_permissions() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("config")).unwrap();
    let outside = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(
        outside.path(),
        r#"{"schema_version":1,"decisions":{"file-generator":{"filesystem:write":"allowed"}}}"#,
    )
    .unwrap();
    std::os::unix::fs::symlink(outside.path(), root.path().join("config/consents.json")).unwrap();
    assert_rejected(root.path());
}

#[cfg(unix)]
#[test]
fn consent_fifo_does_not_wait_for_a_writer() {
    use std::os::unix::ffi::OsStrExt;
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("config")).unwrap();
    let path = std::ffi::CString::new(
        root.path()
            .join("config/consents.json")
            .as_os_str()
            .as_bytes(),
    )
    .unwrap();
    // SAFETY: path is a valid NUL-terminated string during this call.
    assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
    assert_rejected(root.path());
}

#[cfg(windows)]
#[test]
fn consent_reparse_points_are_rejected_even_when_dangling() {
    use toolbox_core::ConsentDecision;
    let source = tempfile::tempdir().unwrap();
    let mut source_store = FileConsentStore::new(source.path());
    source_store
        .set(
            "file-generator",
            CapabilityId::FilesystemWrite,
            ConsentDecision::Allowed,
        )
        .unwrap();
    assert_eq!(
        source_store
            .get("file-generator", CapabilityId::FilesystemWrite)
            .unwrap(),
        Some(ConsentDecision::Allowed)
    );
    let source_path = source.path().join("config/consents.json");
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("config")).unwrap();
    std::os::windows::fs::symlink_file(&source_path, root.path().join("config/consents.json"))
        .expect("Windows test runner requires symlink privileges or Developer Mode");
    assert_rejected(root.path());
    std::fs::remove_file(source_path).unwrap();
    assert_rejected(root.path());
}
