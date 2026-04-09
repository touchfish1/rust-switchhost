use serde::{Deserialize, Serialize};

const GITHUB_LATEST_RELEASE_API: &str =
    "https://api.github.com/repos/touchfish1/rust-switchhost/releases/latest";

#[derive(Debug, Serialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub release_name: String,
    pub published_at: String,
    pub body: String,
    pub html_url: String,
    pub download_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    html_url: String,
    published_at: Option<String>,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("rust-switchhost-update-checker")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let release = client
        .get(GITHUB_LATEST_RELEASE_API)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch latest release: {}", e))?
        .error_for_status()
        .map_err(|e| format!("GitHub release API error: {}", e))?
        .json::<GithubRelease>()
        .await
        .map_err(|e| format!("Failed to parse latest release: {}", e))?;

    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let latest_version = normalize_version(&release.tag_name);
    let current_normalized = normalize_version(&current_version);
    let has_update = compare_versions(&latest_version, &current_normalized).is_gt();

    Ok(UpdateInfo {
        current_version,
        latest_version: latest_version.clone(),
        has_update,
        release_name: release
            .name
            .clone()
            .unwrap_or_else(|| release.tag_name.clone()),
        published_at: release.published_at.unwrap_or_default(),
        body: release.body.unwrap_or_default(),
        html_url: release.html_url.clone(),
        download_url: select_download_url(&release.assets),
    })
}

fn normalize_version(version: &str) -> String {
    version
        .trim()
        .trim_start_matches('v')
        .split(['-', '+'])
        .next()
        .unwrap_or("0.0.0")
        .to_string()
}

fn compare_versions(left: &str, right: &str) -> std::cmp::Ordering {
    let left_parts = parse_version_parts(left);
    let right_parts = parse_version_parts(right);

    for index in 0..left_parts.len().max(right_parts.len()) {
        let left_part = *left_parts.get(index).unwrap_or(&0);
        let right_part = *right_parts.get(index).unwrap_or(&0);

        match left_part.cmp(&right_part) {
            std::cmp::Ordering::Equal => continue,
            ordering => return ordering,
        }
    }

    std::cmp::Ordering::Equal
}

fn parse_version_parts(version: &str) -> Vec<u32> {
    version
        .split('.')
        .map(|part| part.parse::<u32>().unwrap_or(0))
        .collect()
}

fn select_download_url(assets: &[GithubAsset]) -> Option<String> {
    let current_os = std::env::consts::OS;
    let preferred_extensions: &[&str] = match current_os {
        "windows" => &[".msi", ".exe", ".zip"],
        "macos" => &[".dmg", ".app.tar.gz", ".tar.gz"],
        "linux" => &[".AppImage", ".deb", ".rpm", ".tar.gz"],
        _ => &[],
    };

    for extension in preferred_extensions {
        if let Some(asset) = assets.iter().find(|asset| asset.name.ends_with(extension)) {
            return Some(asset.browser_download_url.clone());
        }
    }

    assets
        .first()
        .map(|asset| asset.browser_download_url.clone())
}

#[cfg(test)]
mod tests {
    use super::{compare_versions, normalize_version};

    #[test]
    fn normalizes_tag_version() {
        assert_eq!(normalize_version("v1.2.3"), "1.2.3");
        assert_eq!(normalize_version("1.2.3-beta.1"), "1.2.3");
    }

    #[test]
    fn compares_versions_correctly() {
        assert!(compare_versions("0.0.3", "0.0.2").is_gt());
        assert!(compare_versions("1.0.0", "1.0.0").is_eq());
        assert!(compare_versions("1.2.0", "1.10.0").is_lt());
    }
}
