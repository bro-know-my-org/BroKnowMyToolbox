use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn version_reports_the_workspace_release() {
    let mut command = Command::cargo_bin("bkmt").expect("bkmt binary should build");

    command
        .arg("--version")
        .assert()
        .success()
        .stdout(contains(format!("bkmt {}", env!("CARGO_PKG_VERSION"))));
}
