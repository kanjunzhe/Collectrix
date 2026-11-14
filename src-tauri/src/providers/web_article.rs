use super::{VideoMeta, VideoProvider};
use async_trait::async_trait;

/// 通用博客/网页解析器，放在 provider 列表最后。
pub struct WebArticleProvider;

#[async_trait]
impl VideoProvider for WebArticleProvider {
    fn name(&self) -> &str {
        "blog"
    }

    fn matches(&self, url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }

    async fn fetch_meta(&self, url: &str) -> Result<VideoMeta, String> {
        let page = crate::web::fetch_page(url).await?;
        Ok(VideoMeta {
            platform: "blog".into(),
            platform_id: url.to_string(),
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
