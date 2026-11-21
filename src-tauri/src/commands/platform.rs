use crate::bilibili::{self, BiliLoginInfo, BiliVideoData, QrCodeInfo, QrLoginStatus};
use crate::providers::VideoMeta;
use crate::settings::{self};
use crate::state::DbState;
use rusqlite::params;
use tauri::State;

fn parse_up_mid(cookie: &str) -> Option<i64> {
    cookie
        .split(';')
        .find_map(|s| s.trim().strip_prefix("DedeUserID="))
        .and_then(|v| v.trim().parse().ok())
}

fn bulk_to_meta(video: bilibili::BulkVideo) -> VideoMeta {
    VideoMeta {
        platform: video.platform,
        platform_id: video.platform_id,
        title: video.title,
        author: video.author,
        cover_url: video.cover_url,
        description: video.description,
        duration: video.duration,
        original_url: video.original_url,
        aid: video.aid,
        cid: video.cid,
    }
}

#[tauri::command]
pub async fn fetch_bili_collection(url: String) -> Result<Vec<VideoMeta>, String> {
    let items = bilibili::fetch_collection(&url).await?;
    Ok(items.into_iter().map(bulk_to_meta).collect())
}

#[tauri::command]
pub async fn list_bili_watch_later(
    app: tauri::AppHandle,
) -> Result<Vec<VideoMeta>, String> {
    let settings = settings::load_settings(&app);
    let items = bilibili::fetch_watch_later(&settings.bilibili_cookie).await?;
    Ok(items.into_iter().map(bulk_to_meta).collect())
}

#[tauri::command]
pub async fn check_bili_login(app: tauri::AppHandle) -> Result<BiliLoginInfo, String> {
    let settings = settings::load_settings(&app);
    bilibili::check_login(&settings.bilibili_cookie).await
}

#[tauri::command]
pub async fn generate_bili_qr() -> Result<QrCodeInfo, String> {
    bilibili::generate_qr_code().await
}

#[tauri::command]
pub async fn poll_bili_qr(
    qrcode_key: String,
    app: tauri::AppHandle,
) -> Result<QrLoginStatus, String> {
    let (status, cookie) = bilibili::poll_qr_code(&qrcode_key).await?;
    if status.logged_in {
        let cookie = cookie.ok_or_else(|| "登录成功但未获取到 Cookie".to_string())?;
        let mut s = settings::load_settings(&app);
        s.bilibili_cookie = cookie;
        settings::save_settings(&app, &s)?;
    }
    Ok(status)
}

#[tauri::command]
pub async fn search_bili_videos(
    keyword: String,
    app: tauri::AppHandle,
) -> Result<Vec<crate::bilibili::BiliSearchItem>, String> {
    let s = settings::load_settings(&app);
    crate::bilibili::search_videos(&keyword, &s.bilibili_cookie).await
}

#[tauri::command]
pub async fn fetch_bili_info(url: String) -> Result<BiliVideoData, String> {
    let bvid = bilibili::extract_bvid(&url)
        .ok_or_else(|| "无法从 URL 中识别 Bilibili BV 号".to_string())?;
    bilibili::fetch_video_info(&bvid).await
}

#[tauri::command]
pub async fn list_fav_folders(
    app: tauri::AppHandle,
) -> Result<Vec<crate::bilibili::FavFolder>, String> {
    let s = settings::load_settings(&app);
    let mid = parse_up_mid(&s.bilibili_cookie).ok_or("Cookie 中未找到 DedeUserID")?;
    crate::bilibili::fetch_fav_folders(&s.bilibili_cookie, mid).await
}

#[tauri::command]
pub async fn import_fav_folder(
    media_id: i64,
    category_id: Option<i64>,
    state: State<'_, DbState>,
    app: tauri::AppHandle,
) -> Result<usize, String> {
    let s = settings::load_settings(&app);
    let mut page = 1;
    let mut total = 0;
    loop {
        let (items, has_more) =
            crate::bilibili::fetch_fav_videos(&s.bilibili_cookie, media_id, page).await?;
        for item in &items {
            if item.attr != 0 {
                continue;
            } // 失效视频跳过
            let conn = state.0.lock().map_err(|e| e.to_string())?;
            let target_category =
                category_id.or_else(|| crate::commands::video::uncategorized_id(&conn));
            let _ = conn.execute(
                "INSERT OR IGNORE INTO videos
                    (category_id, platform, platform_id, title, author, cover_url, duration, description, original_url)
                 VALUES (?1, 'bilibili', ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    target_category,
                    item.bvid,
                    item.title,
                    item.upper.name,
                    item.cover,
                    item.duration,
                    item.intro,
                    format!("https://www.bilibili.com/video/{}", item.bvid),
                ],
            );
            total += 1;
        }
        if !has_more || items.is_empty() {
            break;
        }
        page += 1;
    }
    Ok(total)
}

#[tauri::command]
pub async fn get_danmaku(cid: i64) -> Result<Vec<crate::bilibili::DanmakuItem>, String> {
    crate::bilibili::fetch_danmaku_xml(cid).await
}

#[tauri::command]
pub async fn get_subtitle_list(
    bvid: String,
    cid: i64,
    app: tauri::AppHandle,
) -> Result<Vec<crate::bilibili::SubtitleMeta>, String> {
    let s = crate::settings::load_settings(&app);
    crate::bilibili::fetch_subtitle_list(&bvid, cid, &s.bilibili_cookie).await
}

#[tauri::command]
pub async fn get_subtitle_content(
    url: String,
) -> Result<Vec<crate::bilibili::SubtitleCue>, String> {
    crate::bilibili::fetch_subtitle_content(&url).await
}

#[tauri::command]
pub async fn fetch_video_meta(url: String) -> Result<VideoMeta, String> {
    let providers = crate::providers::all_providers();
    for p in providers {
        if p.matches(&url) {
            return p.fetch_meta(&url).await;
        }
    }
    Err(format!("不支持的链接：{url}"))
}
#[tauri::command]
pub async fn fetch_web_article(
    url: String,
    app: tauri::AppHandle,
) -> Result<crate::web::WebPage, String> {
    let token = settings::load_settings(&app).github_token;
    let token = if token.trim().is_empty() {
        None
    } else {
        Some(token.as_str())
    };
    crate::web::fetch_page_with_token(&url, token).await
}

#[tauri::command]
pub async fn list_github_stars(
    app: tauri::AppHandle,
    username: Option<String>,
    token: Option<String>,
) -> Result<Vec<VideoMeta>, String> {
    let settings = settings::load_settings(&app);
    let username = username.unwrap_or(settings.github_username);
    let token = token.unwrap_or(settings.github_token);
    crate::providers::github::fetch_starred_repos(&username, &token).await
}

#[tauri::command]
pub async fn fetch_bili_article_content(cv_id: String) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .build()
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = client
        .get("https://api.bilibili.com/x/article/view")
        .query(&[("id", cv_id.as_str())])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if json["code"].as_i64() != Some(0) {
        return Err(format!("B站专栏接口错误: {}", json["message"]));
    }

    let data = &json["data"];
    Ok(serde_json::json!({
        "title": data["title"],
        "author": data["author"]["name"],
        "cover_url": data["banner_url"],
        "content_html": data["content"],
        "publish_time": data["publish_time"],
    }))
}
