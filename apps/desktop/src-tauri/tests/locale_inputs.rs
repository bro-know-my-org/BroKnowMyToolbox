use bkmt_desktop::{DesktopState, load_locale_override};

fn assert_rejected(root: &std::path::Path, expected_code: &str) {
    let state = DesktopState::new(root.to_path_buf());
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(load_locale_override(&state, "en-US"));
    });
    let result = receiver
        .recv_timeout(std::time::Duration::from_secs(2))
        .expect("invalid locale file must not block");
    assert_eq!(result.unwrap_err().code, expected_code);
}

#[test]
fn missing_locale_pack_falls_back_without_creating_directories() {
    let root = tempfile::tempdir().unwrap();
    let state = DesktopState::new(root.path().join("absent"));
    assert!(load_locale_override(&state, "en-US").unwrap().is_empty());
    assert!(!root.path().join("absent").exists());
}

#[test]
fn locale_directory_and_oversized_pack_are_rejected() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("locales/en-US.json");
    std::fs::create_dir_all(&path).unwrap();
    assert_rejected(root.path(), "locale_storage_failed");
    std::fs::remove_dir(&path).unwrap();
    std::fs::File::create(path)
        .unwrap()
        .set_len(1024 * 1024 + 1)
        .unwrap();
    assert_rejected(root.path(), "locale_pack_too_large");
}

#[cfg(any(unix, windows))]
#[test]
fn locale_file_symlink_cannot_read_outside_the_data_root() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("locales")).unwrap();
    let outside = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(outside.path(), r#"{"secret":"outside"}"#).unwrap();
    let link = root.path().join("locales/en-US.json");
    #[cfg(unix)]
    std::os::unix::fs::symlink(outside.path(), &link).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(outside.path(), &link).unwrap();
    assert_rejected(root.path(), "locale_storage_failed");
}

#[cfg(any(unix, windows))]
#[test]
fn locale_directory_symlink_cannot_read_outside_the_data_root() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("en-US.json"), r#"{"secret":"outside"}"#).unwrap();
    let link = root.path().join("locales");
    #[cfg(unix)]
    std::os::unix::fs::symlink(outside.path(), &link).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(outside.path(), &link).unwrap();
    assert_rejected(root.path(), "locale_storage_failed");
}

#[cfg(unix)]
#[test]
fn locale_fifo_does_not_wait_for_a_writer() {
    use std::os::unix::ffi::OsStrExt;
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("locales")).unwrap();
    let path = std::ffi::CString::new(
        root.path()
            .join("locales/en-US.json")
            .as_os_str()
            .as_bytes(),
    )
    .unwrap();
    // SAFETY: path is a valid NUL-terminated string during mkfifo.
    assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
    assert_rejected(root.path(), "locale_storage_failed");
}
