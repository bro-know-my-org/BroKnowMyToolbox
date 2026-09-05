use bkmsa_tauri::{HostAuthorizer, HostCapability};
use bkmt_desktop::{DesktopState, SparkHostAuthorizer, set_spark_consent};
use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder};

#[test]
fn spark_host_authorizer_uses_the_shared_persisted_consent_store() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let host = SparkHostAuthorizer::new(data_root.path());

    assert!(
        host.authorize(HostCapability::Network)
            .expect_err("network access should require consent")
            .contains("consent_required")
    );

    let mut authorizer = toolbox_core::CapabilityAuthorizer::new(
        toolbox_core::FileConsentStore::new(data_root.path()),
    )
    .expect("catalog should load");
    authorizer
        .record(
            "spark-analyzer",
            toolbox_core::CapabilityId::NetworkSpark,
            toolbox_core::ConsentDecision::Allowed,
        )
        .expect("consent should be persisted");

    assert_eq!(host.authorize(HostCapability::Network), Ok(()));
}

#[test]
fn desktop_can_persist_a_spark_consent_decision_for_the_plugin_gate() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let state =
        DesktopState::new(data_root.path().to_path_buf()).expect("test data root must be absolute");

    set_spark_consent(&state, "credentials:ai", true).expect("desktop consent should be persisted");

    let host = SparkHostAuthorizer::new(data_root.path());
    assert_eq!(host.authorize(HostCapability::Credentials), Ok(()));
}

#[test]
fn toolbox_spark_plugin_ipc_is_gated_by_the_shared_rust_consent_store() {
    let data_root = tempfile::tempdir().expect("data root should be created");
    let context = tauri::generate_context!();
    let app = mock_builder()
        .plugin(bkmsa_tauri::init_with_authorizer(SparkHostAuthorizer::new(
            data_root.path(),
        )))
        .build(context)
        .expect("Toolbox test app should register the Spark plugin");
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("test webview should build");
    let invoke = || {
        get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "plugin:bkmsa-tauri|analyzer_fetch_report".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "tauri://localhost".parse().expect("URL should parse"),
                body: serde_json::json!({ "input": "not-a-valid-report-url" }).into(),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .expect_err("the deliberately invalid fetch should return an error")
        .as_str()
        .expect("plugin errors should be strings")
        .to_string()
    };

    let denied = invoke();
    assert!(denied.contains("consent_required"));
    assert!(denied.contains("network:spark"));

    let state =
        DesktopState::new(data_root.path().to_path_buf()).expect("test data root must be absolute");
    set_spark_consent(&state, "network:spark", true)
        .expect("desktop should persist network consent");
    let after_consent = invoke();
    assert!(!after_consent.contains("consent_required"));
}
