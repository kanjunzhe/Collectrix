use super::{VideoMeta, VideoProvider};
use async_trait::async_trait;

pub struct GitHubProvider;

fn repo_id(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
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
    Some(format!("{}/{}", parts[0], parts[1]))
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
