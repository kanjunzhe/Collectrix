// src-tauri/src/models.rs
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct TagInfo {
    pub id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Serialize, Clone)]
pub struct CategoryNode {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub children: Vec<CategoryNode>,
}

#[derive(Serialize, Clone)]
pub struct VideoRow {
    pub id: i64,
    pub platform: String,
    pub platform_id: String,
    pub title: String,
    pub author: Option<String>,
    pub cover_url: Option<String>,
    pub category_id: Option<i64>,
    pub original_url: Option<String>,
    pub aid: Option<i64>,
    pub cid: Option<i64>,
    pub local_video_path: Option<String>,
    pub local_audio_path: Option<String>,
    pub transcript_path: Option<String>,
    pub local_cover_path: Option<String>,
    pub local_subtitle_paths: Vec<String>,
    pub description: Option<String>,
    pub is_ai_sorted: bool,
    pub is_recent: bool,
    pub tags: Vec<TagInfo>,
}

#[derive(Serialize)]
pub struct Stats {
    pub videos_count: i64,
    pub categories_count: i64,
    pub tags_count: i64,
    pub recent_videos: Vec<VideoRow>,
}

/// 所有查询视频的语句都必须使用这个字段列表，且顺序与 read_video_row 一一对应。
/// is_recent 由 SQL 计算：创建时间在 N 天内（默认 7 天，与 settings.recent_days 解耦，
/// 保持查询简单；如需跟随设置，可把下面的 7 改为参数占位符）
pub const SELECT_FIELDS: &str = "\
    id, platform, platform_id, title, author, cover_url, category_id, original_url, \
    aid, cid, local_video_path, local_audio_path, transcript_path, \
    local_cover_path, local_subtitle_paths, \
    description, is_ai_sorted, \
    CASE WHEN julianday('now') - julianday(created_at) < 7 THEN 1 ELSE 0 END AS is_recent";
