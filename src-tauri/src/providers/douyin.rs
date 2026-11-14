use super::{VideoMeta, VideoProvider};
use async_trait::async_trait;

pub struct DouyinProvider;

#[async_trait]
impl VideoProvider for DouyinProvider {
    fn name(&self) -> &str {
        "douyin"
    }
    fn matches(&self, url: &str) -> bool {
        url.contains("v.douyin.com") || url.contains("douyin.com/video")
    }

    async fn fetch_meta(&self, url: &str) -> Result<VideoMeta, String> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0")
            .build()
            .map_err(|e| e.to_string())?;

        // 短链重定向拿 video id
        let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
        let final_url = resp.url().as_str().to_string();
        let id = final_url
            .split("/video/")
            .nth(1)
            .and_then(|s| s.split('?').next())
            .ok_or("无法解析抖音 video id")?;

        // 请求 iteminfo API
        let api = format!("https://www.iesdouyin.com/web/api/v2/aweme/iteminfo/?item_ids={id}");
        let json: serde_json::Value = client
            .get(&api)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        let item = json["item_list"][0]
            .as_object()
            .ok_or("抖音 API 返回为空")?;
        let title = item["desc"].as_str().unwrap_or("").to_string();
        let author = item["author"]["nickname"].as_str().map(String::from);
        let cover = item["video"]["cover"]["url_list"][0]
            .as_str()
            .map(String::from);

        Ok(VideoMeta {
            platform: "douyin".into(),
            platform_id: id.to_string(),
            title,
            author,
            cover_url: cover,
            description: None,
            duration: None,
            original_url: url.to_string(),
            aid: None,
            cid: None,
        })
    }
}
