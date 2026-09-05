use bkmt_desktop::{
    DesktopState, SparkPreferences, load_spark_preferences, save_spark_preferences,
};

#[test]
fn spark_preferences_are_stored_in_the_shared_tool_data_root() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let state = DesktopState::new(data_root.path().to_path_buf());
    assert_eq!(
        load_spark_preferences(&state).expect("missing preferences should load"),
        None
    );

    let preferences = SparkPreferences {
        provider_id: "openai".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        model: "gpt-4.1-mini".to_string(),
        temperature: 0.3,
    };
    save_spark_preferences(&state, preferences.clone()).expect("preferences should be persisted");

    assert_eq!(
        load_spark_preferences(&state).expect("preferences should reopen"),
        Some(preferences.clone())
    );
    let updated = SparkPreferences {
        model: "gpt-4.1".to_string(),
        temperature: 0.7,
        ..preferences
    };
    save_spark_preferences(&state, updated.clone()).expect("preferences should be replaced");
    assert_eq!(
        load_spark_preferences(&state).expect("updated preferences should reopen"),
        Some(updated)
    );
    assert!(
        data_root
            .path()
            .join("tools/spark-analyzer/preferences.json")
            .is_file()
    );
}

#[test]
fn spark_preferences_reject_invalid_values_and_future_schemas() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let state = DesktopState::new(data_root.path().to_path_buf());
    let invalid = SparkPreferences {
        provider_id: "openai".to_string(),
        base_url: "file:///tmp/not-a-provider".to_string(),
        model: "gpt-4.1-mini".to_string(),
        temperature: 0.3,
    };
    let error = save_spark_preferences(&state, invalid)
        .expect_err("non-HTTP provider URLs should be rejected");
    assert_eq!(error.code, "invalid_spark_preferences");

    let directory = data_root.path().join("tools/spark-analyzer");
    std::fs::create_dir_all(&directory).expect("tool data directory should be created");
    std::fs::write(
        directory.join("preferences.json"),
        r#"{
          "schemaVersion": 99,
          "providerId": "custom",
          "baseUrl": "",
          "model": "",
          "temperature": 0.2
        }"#,
    )
    .expect("future preferences should be written");

    let error = load_spark_preferences(&state)
        .expect_err("future preference schemas must not be interpreted as version one");
    assert_eq!(error.code, "unsupported_spark_preferences_schema");
}
