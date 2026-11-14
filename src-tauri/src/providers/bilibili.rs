use super::{VideoMeta, VideoProvider};
use async_trait::async_trait;

pub struct BilibiliProvider;

#[async_trait]
impl VideoProvider for BilibiliProvider {
    fn name(&self) -> &str {
        "bilibili"
    }

    fn matches(&self, url: &str) -> bool {
        crate::bilibili::extract_bvid(url).is_some()
    }

    async fn fetch_meta(&self, url: &str) -> Result<VideoMeta, String> {
        let bvid = crate::bilibili::extract_bvid(url)
            .ok_or_else(|| "无法从 URL 中识别 Bilibili BV 号".to_string())?;
        let info = crate::bilibili::fetch_video_info(&bvid).await?;

        Ok(VideoMeta {
            platform: "bilibili".into(),
            platform_id: info.bvid,
            title: info.title,
            author: Some(info.owner.name),
            cover_url: Some(info.pic),
            description: Some(info.desc),
            duration: Some(info.duration),
            original_url: url.to_string(),
            aid: Some(info.aid),
            cid: Some(info.cid),
        })
    }
}
