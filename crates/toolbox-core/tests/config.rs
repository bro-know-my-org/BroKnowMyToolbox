use toolbox_core::{AppConfig, DensityMode, FileConfigStore, ThemeMode};

#[test]
fn app_config_uses_product_defaults() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let store = FileConfigStore::new(data_root.path());

    let config = store.load().expect("default config should load");

    assert_eq!(config.schema_version, 1);
    assert_eq!(config.locale, "system");
    assert_eq!(config.theme, ThemeMode::System);
    assert_eq!(config.density, DensityMode::Comfortable);
    assert_eq!(config.font_scale_percent, 100);
    assert!(config.favorite_tool_ids.is_empty());
    assert!(config.recent_tool_ids.is_empty());
}

#[test]
fn app_config_survives_reopening_from_the_shared_data_root() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let store = FileConfigStore::new(data_root.path());
    let mut config = AppConfig {
        locale: "en-US".to_string(),
        theme: ThemeMode::Dark,
        density: DensityMode::Compact,
        font_scale_percent: 110,
        accent_color: "#ff7a45".to_string(),
        reduced_motion: true,
        favorite_tool_ids: vec!["file-generator".to_string()],
        recent_tool_ids: vec!["spark-analyzer".to_string()],
        ..AppConfig::default()
    };
    store.save(&config).expect("config should be persisted");

    config = FileConfigStore::new(data_root.path())
        .load()
        .expect("persisted config should reopen");

    assert_eq!(config.locale, "en-US");
    assert_eq!(config.theme, ThemeMode::Dark);
    assert_eq!(config.density, DensityMode::Compact);
    assert_eq!(config.font_scale_percent, 110);
    assert_eq!(config.accent_color, "#ff7a45");
    assert!(config.reduced_motion);
    assert_eq!(config.favorite_tool_ids, ["file-generator"]);
    assert_eq!(config.recent_tool_ids, ["spark-analyzer"]);
    let source = std::fs::read_to_string(data_root.path().join("config.toml"))
        .expect("config TOML should be readable");
    assert!(source.contains("schema_version = 1"));
}

#[test]
fn config_store_rejects_a_font_scale_outside_the_desktop_range() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let store = FileConfigStore::new(data_root.path());
    let config = AppConfig {
        font_scale_percent: 131,
        ..AppConfig::default()
    };

    let error = store
        .save(&config)
        .expect_err("invalid font scale should not be persisted");

    assert_eq!(error.code(), "invalid_config");
}

#[test]
fn config_store_rejects_an_accent_that_is_not_an_rgb_hex_color() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let store = FileConfigStore::new(data_root.path());
    let config = AppConfig {
        accent_color: "red".to_string(),
        ..AppConfig::default()
    };

    let error = store
        .save(&config)
        .expect_err("invalid accent should not be persisted");

    assert_eq!(error.code(), "invalid_config");
}

#[test]
fn config_store_filters_stale_tool_ids_when_reopening() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let store = FileConfigStore::new(data_root.path());
    let config = AppConfig {
        favorite_tool_ids: vec!["runtime-plugin".to_string()],
        ..AppConfig::default()
    };

    store
        .save(&config)
        .expect("stale tool ids should be ignored");
    let loaded = store.load().expect("config should remain readable");

    assert!(loaded.favorite_tool_ids.is_empty());
}

#[test]
fn config_store_deduplicates_and_caps_recent_entries() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let store = FileConfigStore::new(data_root.path());
    let config = AppConfig {
        recent_tool_ids: vec!["spark-analyzer".to_string(); 11],
        ..AppConfig::default()
    };

    store.save(&config).expect("recents should be normalized");
    let loaded = store.load().expect("normalized config should load");

    assert_eq!(loaded.recent_tool_ids, ["spark-analyzer"]);
}

#[test]
fn config_store_rejects_an_unknown_locale() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let store = FileConfigStore::new(data_root.path());
    let config = AppConfig {
        locale: "../../custom".to_string(),
        ..AppConfig::default()
    };

    let error = store.save(&config).expect_err("unknown locale should fail");

    assert_eq!(error.code(), "invalid_config");
}

#[test]
fn unsupported_schema_is_reported_before_new_fields_are_deserialized() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    std::fs::write(
        data_root.path().join("config.toml"),
        "schema_version = 2\nfuture_field = true\n",
    )
    .expect("future config should be written");

    let error = FileConfigStore::new(data_root.path())
        .load()
        .expect_err("future schema should be rejected explicitly");

    assert_eq!(error.code(), "unsupported_config_schema");
}

#[test]
fn config_store_reports_lock_timeout_instead_of_blocking_indefinitely() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let lock_path = data_root.path().join("config.lock");
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(lock_path)
        .expect("lock file should open");
    fs2::FileExt::lock_exclusive(&lock).expect("test should hold the config lock");

    let store = FileConfigStore::new(data_root.path());
    let (sender, receiver) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        sender
            .send(store.load().map_err(|error| error.to_string()))
            .expect("test receiver should remain available");
    });

    let result = receiver.recv_timeout(std::time::Duration::from_secs(1));
    fs2::FileExt::unlock(&lock).expect("test lock should unlock");
    worker.join().expect("config worker should finish");

    let error = result
        .expect("lock contention should return before the one-second test deadline")
        .expect_err("contended config access should return a diagnostic error");
    assert!(error.contains("timed out waiting for config lock"));
}

#[test]
#[ignore = "helper invoked by the cross-process contention test"]
fn config_child_writer() {
    let Some(data_root) = std::env::var_os("BKMT_TEST_CONFIG_ROOT") else {
        return;
    };
    let locale =
        std::env::var("BKMT_TEST_CONFIG_LOCALE").expect("parent should provide a child locale");
    let config = AppConfig {
        locale,
        ..AppConfig::default()
    };
    FileConfigStore::new(data_root)
        .save(&config)
        .expect("child process should save a complete config");
}

#[test]
fn concurrent_processes_leave_a_complete_readable_config() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let test_binary = std::env::current_exe().expect("test binary path should resolve");
    let mut children = (0..4)
        .map(|index| {
            std::process::Command::new(&test_binary)
                .args(["--ignored", "--exact", "config_child_writer"])
                .env("BKMT_TEST_CONFIG_ROOT", data_root.path())
                .env(
                    "BKMT_TEST_CONFIG_LOCALE",
                    if index % 2 == 0 { "zh-CN" } else { "en-US" },
                )
                .spawn()
                .expect("config writer process should start")
        })
        .collect::<Vec<_>>();

    for child in &mut children {
        assert!(
            child.wait().expect("config writer should exit").success(),
            "every config writer must complete successfully"
        );
    }

    let config = FileConfigStore::new(data_root.path())
        .load()
        .expect("final config should remain readable");
    assert!(matches!(config.locale.as_str(), "zh-CN" | "en-US"));
}
