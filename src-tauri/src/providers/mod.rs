mod bili_article;
mod bilibili;
mod douyin;
pub(crate) mod github;
mod web_article;
#[cfg(not(target_os = "android"))]
mod youtube;

use async_trait::async_trait;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VideoMeta {
    pub platform: String,
    pub platform_id: String,
    pub title: String,
    pub author: Option<String>,
    pub cover_url: Option<String>,
    pub description: Option<String>,
    pub duration: Option<i64>,
    pub original_url: String,
    pub aid: Option<i64>,
    pub cid: Option<i64>,
}

#[async_trait]
pub trait VideoProvider: Send + Sync {
    fn name(&self) -> &str;
    fn matches(&self, url: &str) -> bool;
    async fn fetch_meta(&self, url: &str) -> Result<VideoMeta, String>;
}

/// 按优先级返回所有平台解析器。新增平台时同时在这里注册。
pub fn all_providers() -> Vec<Box<dyn VideoProvider>> {
    #[allow(unused_mut)]
    let mut providers: Vec<Box<dyn VideoProvider>> = vec![
        Box::new(bili_article::BiliArticleProvider),
        Box::new(douyin::DouyinProvider),
        Box::new(github::GitHubProvider),
        // Bilibili 的匹配较宽松（URL 中出现 BV 即命中），放在前面避免被通用网页解析器抢走。
        Box::new(bilibili::BilibiliProvider),
        // 通用博客/网页解析器必须放在最后。
        Box::new(web_article::WebArticleProvider),
    ];
    #[cfg(not(target_os = "android"))]
    providers.insert(1, Box::new(youtube::YouTubeProvider));
    providers
}
