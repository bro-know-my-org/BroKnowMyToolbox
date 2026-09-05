use bkmt_desktop::DesktopState;
use toolbox_core::{DataRootError, DataRootSource};

#[test]
fn explicit_desktop_roots_reject_empty_and_relative_paths() {
    assert_eq!(
        DesktopState::new("".into()).unwrap_err(),
        DataRootError::EmptyPath(DataRootSource::Explicit)
    );
    assert_eq!(
        DesktopState::new("relative".into()).unwrap_err(),
        DataRootError::RelativePath(DataRootSource::Explicit)
    );
}

#[test]
fn explicit_desktop_roots_accept_absolute_paths_without_creating_them() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("data");
    DesktopState::new(path.clone()).unwrap();
    assert!(!path.exists());
}
