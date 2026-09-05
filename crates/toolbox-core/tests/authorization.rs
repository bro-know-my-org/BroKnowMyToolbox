use toolbox_core::{
    AuthorizationError, CapabilityAuthorizer, CapabilityId, ConsentDecision, ConsentStore,
    FileConsentStore, MemoryConsentStore,
};

#[test]
fn authorization_requires_a_declared_and_persisted_consent() {
    let mut authorizer =
        CapabilityAuthorizer::new(MemoryConsentStore::default()).expect("catalog should load");

    assert_eq!(
        authorizer.check("file-generator", CapabilityId::FilesystemWrite),
        Err(AuthorizationError::ConsentRequired)
    );

    authorizer
        .record(
            "file-generator",
            CapabilityId::FilesystemWrite,
            ConsentDecision::Allowed,
        )
        .expect("consent should be recorded");
    assert_eq!(
        authorizer.check("file-generator", CapabilityId::FilesystemWrite),
        Ok(())
    );

    authorizer
        .record(
            "file-generator",
            CapabilityId::FilesystemWrite,
            ConsentDecision::Denied,
        )
        .expect("denial should be recorded");
    assert_eq!(
        authorizer.check("file-generator", CapabilityId::FilesystemWrite),
        Err(AuthorizationError::ConsentDenied)
    );

    assert_eq!(
        authorizer.check("file-generator", CapabilityId::NetworkSpark),
        Err(AuthorizationError::CapabilityNotDeclared {
            tool_id: "file-generator".to_string(),
            capability: CapabilityId::NetworkSpark,
        })
    );
}

#[test]
fn spark_export_declares_filesystem_write_before_consent_can_be_recorded() {
    let authorizer =
        CapabilityAuthorizer::new(MemoryConsentStore::default()).expect("catalog should load");

    assert_eq!(
        authorizer.check("spark-analyzer", CapabilityId::FilesystemWrite),
        Err(AuthorizationError::ConsentRequired)
    );
}

#[test]
fn file_consent_store_survives_reopening() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let mut first = FileConsentStore::new(data_root.path());
    first
        .set(
            "file-generator",
            CapabilityId::FilesystemWrite,
            ConsentDecision::Allowed,
        )
        .expect("consent should be persisted");

    let reopened = FileConsentStore::new(data_root.path());

    assert_eq!(
        reopened
            .get("file-generator", CapabilityId::FilesystemWrite)
            .expect("consent should be read"),
        Some(ConsentDecision::Allowed)
    );
}

#[test]
fn consent_store_reports_lock_timeout_instead_of_blocking_indefinitely() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let config_root = data_root.path().join("config");
    std::fs::create_dir_all(&config_root).expect("config directory should be created");
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(config_root.join("consents.lock"))
        .expect("consent lock should open");
    fs2::FileExt::lock_exclusive(&lock).expect("test should hold the consent lock");

    let store = FileConsentStore::new(data_root.path());
    let (sender, receiver) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        sender
            .send(
                store
                    .get("file-generator", CapabilityId::FilesystemWrite)
                    .map_err(|error| error.to_string()),
            )
            .expect("test receiver should remain available");
    });

    let result = receiver.recv_timeout(std::time::Duration::from_secs(1));
    fs2::FileExt::unlock(&lock).expect("test lock should unlock");
    worker.join().expect("consent worker should finish");

    let error = result
        .expect("lock contention should return before the one-second test deadline")
        .expect_err("contended consent access should return a diagnostic error");
    assert!(error.contains("timed out waiting for consent lock"));
}

#[test]
fn consent_store_rejects_unknown_schema_versions() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let config_root = data_root.path().join("config");
    std::fs::create_dir_all(&config_root).expect("config directory should be created");
    std::fs::write(
        config_root.join("consents.json"),
        r#"{"schema_version":99,"decisions":{}}"#,
    )
    .expect("future consent file should be written");

    let error = FileConsentStore::new(data_root.path())
        .get("file-generator", CapabilityId::FilesystemWrite)
        .expect_err("unknown consent schema must not be interpreted as version one");

    assert!(error.contains("unsupported consent schema version: 99"));
}
