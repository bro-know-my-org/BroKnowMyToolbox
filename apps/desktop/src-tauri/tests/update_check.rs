use std::io::{Read, Write};
use std::net::TcpListener;

#[test]
fn opener_scope_only_allows_canonical_toolbox_release_tags() {
    let capability: serde_json::Value =
        serde_json::from_str(include_str!("../capabilities/default.json")).unwrap();
    let opener = capability["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|permission| permission["identifier"] == "opener:allow-open-url")
        .expect("opener must have an explicit URL scope");
    assert_eq!(
        opener["allow"],
        serde_json::json!([
            { "url": "https://github.com/bro-know-my-org/BroKnowMyToolbox/releases/tag/*" }
        ])
    );
}

fn mock_release(body: &'static str) -> (String, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("mock server should bind");
    let address = listener.local_addr().expect("mock address should resolve");
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("request should arrive");
        let mut request = [0_u8; 2048];
        let _ = stream
            .read(&mut request)
            .expect("request should be readable");
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .expect("response should be written");
    });
    (format!("http://{address}/releases/latest"), server)
}

#[tokio::test]
async fn update_check_reports_a_newer_release_and_its_download_page() {
    let (endpoint, server) = mock_release(
        r#"{"tag_name":"v0.2.0","html_url":"https://github.com/bro-know-my-org/BroKnowMyToolbox/releases/tag/v0.2.0"}"#,
    );

    let result = bkmt_desktop::check_for_update(&endpoint, "0.1.0")
        .await
        .expect("release response should be accepted");

    server.join().expect("mock server should finish");
    assert!(result.available);
    assert_eq!(result.latest_version, "0.2.0");
    assert_eq!(
        result.release_url,
        "https://github.com/bro-know-my-org/BroKnowMyToolbox/releases/tag/v0.2.0"
    );
}

#[tokio::test]
async fn update_check_rejects_untrusted_release_urls() {
    for url in [
        "http://github.com/example/release",
        "https://example.com/release",
        "https://github.com/other-owner/other-repo/releases/tag/v0.2.0",
        "https://github.com/bro-know-my-org/bro-know-my-toolbox/releases/tag/v0.2.0",
        "https://github.com/bro-know-my-org/BroKnowMyToolbox/releases/tag/v0.3.0",
        "https://github.com/bro-know-my-org/BroKnowMyToolbox/issues/1",
        "https://user@github.com/bro-know-my-org/BroKnowMyToolbox/releases/tag/v0.2.0",
        "https://github.com/bro-know-my-org/BroKnowMyToolbox/releases/tag/v0.2.0?redirect=elsewhere",
        "https://github.com/bro-know-my-org/BroKnowMyToolbox/releases/tag/v0.2.0#other",
    ] {
        let body =
            Box::leak(format!(r#"{{"tag_name":"v0.2.0","html_url":"{url}"}}"#).into_boxed_str());
        let (endpoint, server) = mock_release(body);
        let error = bkmt_desktop::check_for_update(&endpoint, "0.1.0")
            .await
            .expect_err("untrusted release URL should fail");
        server.join().expect("mock server should finish");
        assert!(error.contains("HTTPS on github.com"));
    }
}

#[tokio::test]
async fn update_check_handles_equal_older_and_malformed_versions() {
    for (tag, available) in [
        ("v0.1.0", Some(false)),
        ("v0.0.9", Some(false)),
        ("vbad", None),
    ] {
        let body = Box::leak(
            format!(r#"{{"tag_name":"{tag}","html_url":"https://github.com/bro-know-my-org/BroKnowMyToolbox/releases/tag/{tag}"}}"#)
                .into_boxed_str(),
        );
        let (endpoint, server) = mock_release(body);
        let result = bkmt_desktop::check_for_update(&endpoint, "0.1.0").await;
        server.join().expect("mock server should finish");
        match available {
            Some(expected) => assert_eq!(result.expect("version should parse").available, expected),
            None => assert!(
                result
                    .expect_err("malformed version should fail")
                    .contains("version")
            ),
        }
    }
}
