use super::{VideoMeta, VideoProvider};
use async_trait::async_trait;

pub struct BiliArticleProvider;

#[async_trait]
impl VideoProvider for BiliArticleProvider {
    fn name(&self) -> &str {
        "bilibili_article"
    }
    fn matches(&self, url: &str) -> bool {
        url.contains("bilibili.com/read/cv")
    }

    async fn fetch_meta(&self, url: &str) -> Result<VideoMeta, String> {
        let cv_id = url
            .split("/cv")
            .nth(1)
            .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
            .ok_or("无法解析专栏 id")?;

        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0")
            .build()
            .map_err(|e| e.to_string())?;

        let json: serde_json::Value = client
            .get("https://api.bilibili.com/x/article/view")
            .query(&[("id", cv_id)])
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        let data = &json["data"];
        Ok(VideoMeta {
            platform: "bilibili_article".into(),
            platform_id: cv_id.to_string(),
            title: data["title"].as_str().unwrap_or("").to_string(),
            author: data["author"]["name"].as_str().map(String::from),
            cover_url: data["banner_url"].as_str().map(String::from),
            description: data["summary"].as_str().map(String::from),
            duration: None,
            original_url: url.to_string(),
            aid: None,
            cid: None,
        })
    }
}
