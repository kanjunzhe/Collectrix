use super::{VideoMeta, VideoProvider};
use async_trait::async_trait;

pub struct GitHubProvider;

/// 读取 GitHub Star 列表。Token 为空时读取指定用户的公开 Star。
pub async fn fetch_starred_repos(username: &str, token: &str) -> Result<Vec<VideoMeta>, String> {
    let username = username
        .trim()
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("")
        .trim_start_matches('@');
    let token = token.trim();
    if username.is_empty() && token.is_empty() {
        return Err("请先在设置中填写 GitHub 用户名或 Token".into());
    }

    let client = reqwest::Client::builder()
        .user_agent("Collectrix")
        .connect_timeout(std::time::Duration::from_secs(12))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 GitHub 请求失败：{e}"))?;

    let mut result = Vec::new();
    for page in 1..=20 {
        let url = if !token.is_empty() {
            "https://api.github.com/user/starred".to_string()
        } else {
            format!(
                "https://api.github.com/users/{}/starred",
                username.trim_start_matches('@')
            )
        };
        let mut request = client
            .get(&url)
            .query(&[
                ("per_page", "100".to_string()),
                ("page", page.to_string()),
                ("sort", "created".to_string()),
                ("direction", "desc".to_string()),
            ])
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28");
        if !token.is_empty() {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("连接 GitHub 失败：{e}。请检查网络、代理或 DNS 设置"))?;
        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err("GitHub Token 无效或已过期".into());
        }
        if status == reqwest::StatusCode::FORBIDDEN {
            return Err("GitHub API 访问被拒绝，可能是 Token 权限不足或触发频率限制".into());
        }
        if !status.is_success() {
            return Err(format!("GitHub 返回 HTTP {status}"));
        }

        let items: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| format!("解析 GitHub Star 列表失败：{e}"))?;
        if items.is_empty() {
            break;
        }

        for item in &items {
            let Some(full_name) = item["full_name"].as_str() else {
                continue;
            };
            let html_url = item["html_url"]
                .as_str()
                .map(String::from)
                .unwrap_or_else(|| format!("https://github.com/{full_name}"));
            result.push(VideoMeta {
                platform: "github".into(),
                platform_id: full_name.to_string(),
                title: full_name.to_string(),
                author: item["owner"]["login"].as_str().map(String::from),
                cover_url: item["owner"]["avatar_url"].as_str().map(String::from),
                description: item["description"].as_str().map(String::from),
                duration: None,
                original_url: html_url,
                aid: None,
                cid: None,
            });
        }

        if items.len() < 100 {
            break;
        }
    }

    Ok(result)
}

fn repo_id(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url)
        .ok()
        .or_else(|| url::Url::parse(&format!("https://{url}")).ok())?;
    let host = parsed.host_str()?;
    if host != "github.com" && host != "www.github.com" {
        return None;
    }
    let parts: Vec<&str> = parsed
        .path_segments()?
        .filter(|part| !part.is_empty())
        .collect();
    if parts.len() < 2 {
        return None;
    }
    Some(format!("{}/{}", parts[0], parts[1].trim_end_matches(".git")))
}

#[async_trait]
impl VideoProvider for GitHubProvider {
    fn name(&self) -> &str {
        "github"
    }

    fn matches(&self, url: &str) -> bool {
        repo_id(url).is_some()
    }

    async fn fetch_meta(&self, url: &str) -> Result<VideoMeta, String> {
        let page = crate::web::fetch_page(url).await?;
        Ok(VideoMeta {
            platform: "github".into(),
            platform_id: repo_id(url).unwrap_or_else(|| url.to_string()),
            title: page.title,
            author: page.author,
            cover_url: page.cover_url,
            description: page.description,
            duration: None,
            original_url: url.to_string(),
            aid: None,
            cid: None,
        })
    }
}
