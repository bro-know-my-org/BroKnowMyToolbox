use bkmt_desktop::{DesktopState, runtime_diagnostics};

#[test]
fn desktop_diagnostics_exposes_the_actual_data_root() {
    let root = tempfile::tempdir().expect("data root should be created");
    let state =
        DesktopState::new(root.path().to_path_buf()).expect("test data root must be absolute");

    let diagnostics = runtime_diagnostics(&state);

    assert_eq!(diagnostics.data_root, root.path().to_string_lossy());
    assert_eq!(diagnostics.data_root_source, "explicit");
}
