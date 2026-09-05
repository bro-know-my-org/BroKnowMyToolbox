use assert_cmd::Command;

#[test]
fn tools_json_reports_every_registered_tool() {
    let mut command = Command::cargo_bin("bkmt").expect("bkmt binary should build");

    command
        .args(["tools", "--json"])
        .assert()
        .success()
        .stdout(
            "[{\"id\":\"file-generator\",\"cliNamespace\":\"file\"},{\"id\":\"spark-analyzer\",\"cliNamespace\":\"spark\"}]\n",
        );
}
