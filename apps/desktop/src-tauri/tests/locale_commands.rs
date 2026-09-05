use bkmt_desktop::{DesktopState, load_locale_override};

#[test]
fn desktop_loads_flat_minecraft_style_locale_overrides() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    std::fs::create_dir_all(data_root.path().join("locales"))
        .expect("locale directory should be created");
    std::fs::write(
        data_root.path().join("locales/zh-CN.json"),
        r#"{
          "tool.file_generator.name": "我的文件工具",
          "custom.greeting": "你好"
        }"#,
    )
    .expect("locale override should be written");
    let state = DesktopState::new(data_root.path().to_path_buf());

    let messages = load_locale_override(&state, "zh-CN").expect("flat locale override should load");

    assert_eq!(messages["tool.file_generator.name"], "我的文件工具");
    assert_eq!(messages["custom.greeting"], "你好");
}

#[test]
fn locale_names_cannot_escape_the_locale_directory() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let state = DesktopState::new(data_root.path().to_path_buf());

    let error = load_locale_override(&state, "../secrets")
        .expect_err("unsafe locale name should be rejected");

    assert_eq!(error.code, "invalid_locale");
}
