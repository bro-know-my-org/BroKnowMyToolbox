#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    pub available: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
}

#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
}

pub async fn check_for_update(
    endpoint: &str,
    current_version: &str,
) -> Result<UpdateCheckResult, String> {
    let current = semver::Version::parse(current_version)
        .map_err(|error| format!("invalid current version: {error}"))?;
    let response = reqwest::Client::builder()
        .user_agent(format!("bkmt/{current_version}"))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|error| format!("failed to create update client: {error}"))?
        .get(endpoint)
        .send()
        .await
        .map_err(|error| format!("failed to check for updates: {error}"))?
        .error_for_status()
        .map_err(|error| format!("update service returned an error: {error}"))?;
    let release = response
        .json::<GithubRelease>()
        .await
        .map_err(|error| format!("invalid update response: {error}"))?;
    let latest_text = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);
    let latest = semver::Version::parse(latest_text)
        .map_err(|error| format!("invalid release version: {error}"))?;
    let release_url = url::Url::parse(&release.html_url)
        .map_err(|error| format!("invalid release URL: {error}"))?;
    let expected_url = format!(
        "https://github.com/bro-know-my-org/BroKnowMyToolbox/releases/tag/{}",
        release.tag_name,
    );
    if release_url.as_str() != expected_url {
        return Err("release URL must use HTTPS on github.com for the canonical Toolbox repository and release tag".to_string());
    }

    Ok(UpdateCheckResult {
        available: latest > current,
        current_version: current.to_string(),
        latest_version: latest.to_string(),
        release_url: release_url.to_string(),
    })
}

#[tauri::command]
pub async fn check_for_update_command() -> Result<UpdateCheckResult, String> {
    check_for_update(
        "https://api.github.com/repos/bro-know-my-org/BroKnowMyToolbox/releases/latest",
        toolbox_core::WORKSPACE_VERSION,
    )
    .await
}
