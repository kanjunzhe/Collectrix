use super::{VideoMeta, VideoProvider};
use async_trait::async_trait;
use innertube_rs::Innertube;

pub struct YouTubeProvider;

#[async_trait]
impl VideoProvider for YouTubeProvider {
    fn name(&self) -> &str {
        "youtube"
    }

    fn matches(&self, url: &str) -> bool {
        url.contains("youtube.com/watch") || url.contains("youtu.be/")
    }

    async fn fetch_meta(&self, url: &str) -> Result<VideoMeta, String> {
        let video_id = extract_youtube_id(url).ok_or("无法解析 YouTube video id")?;
        let yt = Innertube::new().await.map_err(|e| e.to_string())?;
        let info = yt
            .get_video_info(&video_id)
            .await
            .map_err(|e| e.to_string())?;

        let details = info.video_details.ok_or("YouTube 未返回视频详情")?;
        let duration = details.length_seconds.parse::<i64>().ok();
        let cover_url = details.thumbnail.and_then(|container| {
            container
                .thumbnails
                .into_iter()
                .max_by_key(|thumb| thumb.width)
                .map(|thumb| thumb.url)
        });

        Ok(VideoMeta {
            platform: "youtube".into(),
            platform_id: video_id,
            title: details.title,
            author: Some(details.author),
            cover_url,
            description: details.short_description,
            duration,
            original_url: url.to_string(),
            aid: None,
            cid: None,
        })
    }
}

fn extract_youtube_id(url: &str) -> Option<String> {
    if let Some(pos) = url.find("v=") {
        let rest = &url[pos + 2..];
        return Some(rest.split('&').next()?.to_string());
    }
    if let Some(pos) = url.find("youtu.be/") {
        let rest = &url[pos + 9..];
        return Some(rest.split('?').next()?.to_string());
    }
    None
}
