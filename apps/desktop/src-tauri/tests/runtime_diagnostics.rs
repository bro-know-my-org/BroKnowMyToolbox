use bkmt_desktop::{DesktopState, runtime_diagnostics};

#[test]
fn desktop_diagnostics_exposes_the_actual_data_root() {
    let root = tempfile::tempdir().expect("data root should be created");
    let state =
        DesktopState::new(root.path().to_path_buf()).expect("test data root must be absolute");

    let diagnostics = runtime_diagnostics(&state).unwrap();

    assert_eq!(diagnostics.data_root, root.path().to_string_lossy());
    assert_eq!(
        diagnostics.data_root_source,
        toolbox_core::DataRootSource::Explicit
    );
    let json = serde_json::to_value(diagnostics).unwrap();
    assert_eq!(json["dataRootSource"], "explicit");
}

#[cfg(unix)]
#[test]
fn diagnostics_never_silently_replaces_non_utf8_path_bytes() {
    use std::os::unix::ffi::OsStringExt;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join(std::ffi::OsString::from_vec(vec![0xff]));
    let state = DesktopState::new(path).unwrap();
    assert_eq!(
        runtime_diagnostics(&state).unwrap_err().code,
        "data_root_not_unicode"
    );
}
