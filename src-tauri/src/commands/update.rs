use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

const UPDATE_API: &str = "https://api.github.com/repos/kandimord/collectrix/releases/latest";

#[derive(Debug, Serialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub release_url: Option<String>,
    pub download_url: Option<String>,
    pub notes: Option<String>,
    pub published_at: Option<String>,
}

fn parse_version(value: &str) -> Vec<u64> {
    value
        .trim()
        .trim_start_matches(['v', 'V'])
        .split(|ch: char| !ch.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<u64>().ok())
        .collect()
}

fn version_gt(latest: &str, current: &str) -> bool {
    let latest = parse_version(latest);
    let current = parse_version(current);
    for index in 0..latest.len().max(current.len()) {
        let left = latest.get(index).copied().unwrap_or(0);
        let right = current.get(index).copied().unwrap_or(0);
        if left != right {
            return left > right;
        }
    }
    false
}

#[tauri::command]
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<UpdateInfo, String> {
    let current_version = app.package_info().version.to_string();
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Collectrix-Update-Checker")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(UPDATE_API)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("检查更新失败: {e}"))?;

    let status = response.status();
    if status.as_u16() == 404 {
        return Err("没有找到 Collectrix 的发布仓库".into());
    }
    if !status.is_success() {
        return Err(format!("检查更新失败 HTTP {status}"));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析更新信息失败: {e}"))?;

    let latest_version = json["tag_name"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches(['v', 'V'])
        .to_string();
    if latest_version.is_empty() {
        return Err("发布信息中没有版本号".into());
    }

    let release_url = json["html_url"].as_str().map(String::from);
    let download_url = json["assets"]
        .as_array()
        .and_then(|assets| {
            assets.iter().find_map(|asset| {
                let name = asset["name"].as_str()?.to_lowercase();
                if name.ends_with(".apk") || name.ends_with(".exe") || name.ends_with(".msi") {
                    asset["browser_download_url"].as_str().map(String::from)
                } else {
                    None
                }
            })
        })
        .or_else(|| release_url.clone());

    Ok(UpdateInfo {
        has_update: version_gt(&latest_version, &current_version),
        current_version,
        latest_version,
        release_url,
        download_url,
        notes: json["body"].as_str().map(String::from),
        published_at: json["published_at"].as_str().map(String::from),
    })
}
