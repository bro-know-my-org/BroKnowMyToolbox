use bkmt_desktop::{DesktopState, load_app_config, save_app_config};
use toolbox_core::ThemeMode;

#[test]
fn desktop_config_commands_round_trip_through_the_shared_data_root() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let state = DesktopState::new(data_root.path().to_path_buf());
    let mut config = load_app_config(&state).expect("default config should load");
    config.locale = "en-US".to_string();
    config.theme = ThemeMode::Dark;
    config.favorite_tool_ids = vec!["file-generator".to_string()];

    save_app_config(&state, config).expect("config should save");
    let reopened = load_app_config(&state).expect("saved config should load");

    assert_eq!(reopened.locale, "en-US");
    assert_eq!(reopened.theme, ThemeMode::Dark);
    assert_eq!(reopened.favorite_tool_ids, ["file-generator"]);
}
