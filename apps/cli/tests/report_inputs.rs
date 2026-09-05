use assert_cmd::Command;
use std::path::Path;
use std::time::Duration;

fn assert_rejected(source: &Path, expected: &str) {
    let data = tempfile::tempdir().unwrap();
    Command::cargo_bin("bkmt")
        .unwrap()
        .arg("--data-dir")
        .arg(data.path())
        .args(["consent", "allow", "spark-analyzer", "filesystem:read"])
        .assert()
        .success();
    let output = Command::cargo_bin("bkmt")
        .unwrap()
        .timeout(Duration::from_secs(5))
        .arg("--data-dir")
        .arg(data.path())
        .args(["spark", "inspect"])
        .arg(source)
        .args(["--text", "--json"])
        .assert()
        .code(2)
        .get_output()
        .clone();
    assert!(output.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["code"], expected);
}

#[test]
fn oversized_report_is_rejected_before_reading() {
    let report = tempfile::NamedTempFile::new().unwrap();
    report.as_file().set_len(64 * 1024 * 1024 + 1).unwrap();
    assert_rejected(report.path(), "report_too_large");
}

#[test]
fn directory_is_not_a_report() {
    let directory = tempfile::tempdir().unwrap();
    // Windows can reject the directory at open; Unix permits opening it.
    #[cfg(unix)]
    assert_rejected(directory.path(), "invalid_report_source");
    #[cfg(windows)]
    assert_rejected(directory.path(), "report_read_failed");
}

#[cfg(unix)]
#[test]
fn symlink_report_is_rejected() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("report");
    std::fs::write(&source, "private report").unwrap();
    let link = root.path().join("link");
    std::os::unix::fs::symlink(source, &link).unwrap();
    assert_rejected(&link, "report_read_failed");
}

#[cfg(unix)]
#[test]
fn fifo_report_is_rejected_without_waiting_for_a_writer() {
    use std::os::unix::ffi::OsStrExt;
    let root = tempfile::tempdir().unwrap();
    let fifo = root.path().join("fifo");
    let path = std::ffi::CString::new(fifo.as_os_str().as_bytes()).unwrap();
    // SAFETY: path is a valid NUL-terminated string for the duration of mkfifo.
    assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
    assert_rejected(&fifo, "invalid_report_source");
}
