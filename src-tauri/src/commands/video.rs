use crate::models::{SELECT_FIELDS, TagInfo, VideoRow};
use crate::providers::VideoMeta;
use crate::state::DbState;
use rusqlite::{Connection, params, params_from_iter};
use serde::Serialize;
use std::collections::HashMap;
#[cfg(not(target_os = "android"))]
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;
#[cfg(not(target_os = "android"))]
use tauri::{WebviewUrl, WebviewWindowBuilder};

#[derive(Serialize)]
pub struct CategoryVideoGroup {
    pub category_id: Option<i64>, // None = 当前分类本身
    pub category_name: String,
    pub videos: Vec<VideoRow>,
}

pub(crate) fn uncategorized_id(conn: &Connection) -> Option<i64> {
    conn.query_row(
        "SELECT id FROM categories WHERE name='未分类' AND parent_id IS NULL ORDER BY id LIMIT 1",
        [],
        |r| r.get(0),
    )
    .ok()
}

pub(crate) fn fill_tags(conn: &Connection, video_id: i64) -> Result<Vec<TagInfo>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name, t.color FROM tags t
             JOIN video_tags vt ON vt.tag_id = t.id
             WHERE vt.video_id = ?1 ORDER BY t.name",
        )
        .map_err(|e| e.to_string())?;
    let tags = stmt
        .query_map(params![video_id], |row| {
            Ok(TagInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(tags)
}

/// 批量填充标签，避免列表页 N 条视频产生 N 次查询。
pub(crate) fn fill_tags_many(
    conn: &Connection,
    videos: &mut [VideoRow],
) -> Result<(), String> {
    if videos.is_empty() {
        return Ok(());
    }
    let mut by_video: HashMap<i64, Vec<TagInfo>> = HashMap::new();
    // 分批查询，避免超过 SQLite 的参数数量上限。
    for chunk in videos.chunks(500) {
        let placeholders = std::iter::repeat("?")
            .take(chunk.len())
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT vt.video_id, t.id, t.name, t.color
             FROM video_tags vt JOIN tags t ON t.id = vt.tag_id
             WHERE vt.video_id IN ({placeholders})
             ORDER BY t.name"
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(
                params_from_iter(chunk.iter().map(|video| video.id)),
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        TagInfo {
                            id: row.get(1)?,
                            name: row.get(2)?,
                            color: row.get(3)?,
                        },
                    ))
                },
            )
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (video_id, tag) = row.map_err(|e| e.to_string())?;
            by_video.entry(video_id).or_default().push(tag);
        }
    }
    for video in videos {
        video.tags = by_video.remove(&video.id).unwrap_or_default();
    }
    Ok(())
}

pub(crate) fn read_video_row(row: &rusqlite::Row) -> rusqlite::Result<VideoRow> {
    Ok(VideoRow {
        id: row.get(0)?,
        platform: row.get(1)?,
        platform_id: row.get(2)?,
        title: row.get(3)?,
        author: row.get(4)?,
        cover_url: row.get(5)?,
        category_id: row.get(6)?,
        original_url: row.get(7)?,
        aid: row.get(8)?,
        cid: row.get(9)?,
        local_video_path: row.get(10)?,
        local_audio_path: row.get(11)?,
        transcript_path: row.get(12)?,
        local_cover_path: row.get(13)?,
        local_subtitle_paths: row
            .get::<_, Option<String>>(14)?
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default(),
        description: row.get(15)?,
        is_ai_sorted: row.get(16)?,
        is_recent: row.get(17)?,
        tags: vec![],
    })
}

#[tauri::command]
pub fn list_videos(
    category_id: Option<i64>,
    limit: Option<i64>,
    offset: Option<i64>,
    exclude_articles: Option<bool>,
    state: State<'_, DbState>,
) -> Result<Vec<VideoRow>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(-1);
    let offset = offset.unwrap_or(0);
    let filter = if exclude_articles.unwrap_or(false) {
        " AND platform NOT IN ('github','blog','bilibili_article')"
    } else {
        ""
    };
    let sql = if category_id.is_some() {
        format!(
            "SELECT {SELECT_FIELDS} FROM videos WHERE category_id = ?1{filter} ORDER BY updated_at DESC LIMIT ?2 OFFSET ?3"
        )
    } else {
        format!(
            "SELECT {SELECT_FIELDS} FROM videos WHERE 1=1{filter} ORDER BY updated_at DESC LIMIT ?1 OFFSET ?2"
        )
    };
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut videos: Vec<VideoRow> = if let Some(cid) = category_id {
        stmt.query_map(params![cid, limit, offset], read_video_row)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    } else {
        stmt.query_map(params![limit, offset], read_video_row)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };
    fill_tags_many(&conn, &mut videos)?;
    Ok(videos)
}

/// 网页/GitHub 归类列表：scope = github | web | all。
#[tauri::command]
pub fn list_web_videos(
    scope: String,
    state: State<'_, DbState>,
) -> Result<Vec<VideoRow>, String> {
    let platforms: Vec<&str> = match scope.as_str() {
        "github" => vec!["github"],
        "web" => vec!["blog", "bilibili_article"],
        _ => vec!["github", "blog", "bilibili_article"],
    };
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let placeholders = (1..=platforms.len())
        .map(|index| format!("?{index}"))
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT {SELECT_FIELDS} FROM videos WHERE platform IN ({placeholders}) ORDER BY updated_at DESC"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut videos: Vec<VideoRow> = stmt
        .query_map(params_from_iter(platforms.iter()), read_video_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    fill_tags_many(&conn, &mut videos)?;
    Ok(videos)
}

#[tauri::command]
pub fn list_videos_by_tag(
    tag_id: i64,
    limit: Option<i64>,
    offset: Option<i64>,
    exclude_articles: Option<bool>,
    state: State<'_, DbState>,
) -> Result<Vec<VideoRow>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(-1);
    let offset = offset.unwrap_or(0);
    let filter = if exclude_articles.unwrap_or(false) {
        " AND v.platform NOT IN ('github','blog','bilibili_article')"
    } else {
        ""
    };
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {SELECT_FIELDS} FROM videos v
             JOIN video_tags vt ON vt.video_id = v.id
             WHERE vt.tag_id = ?1{filter} ORDER BY v.updated_at DESC LIMIT ?2 OFFSET ?3"
        ))
        .map_err(|e| e.to_string())?;
    let mut videos: Vec<VideoRow> = stmt
        .query_map(params![tag_id, limit, offset], read_video_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    fill_tags_many(&conn, &mut videos)?;
    Ok(videos)
}

#[tauri::command]
pub fn get_video(id: i64, state: State<'_, DbState>) -> Result<VideoRow, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(&format!("SELECT {SELECT_FIELDS} FROM videos WHERE id = ?1"))
        .map_err(|e| e.to_string())?;
    let mut v = stmt
        .query_row(params![id], read_video_row)
        .map_err(|e| e.to_string())?;
    v.tags = fill_tags(&conn, id)?;
    Ok(v)
}

#[tauri::command]
pub fn save_video(
    data: VideoMeta,
    category_id: Option<i64>,
    original_url: String,
    tags: Vec<String>,
    preserve_tags: Option<bool>,
    state: State<'_, DbState>,
) -> Result<i64, String> {
    let mut conn = state.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let category_id = category_id.or_else(|| uncategorized_id(&tx));

    tx.execute(
        "INSERT INTO videos
        (category_id, platform, platform_id, title, author, cover_url,
         duration, description, original_url, aid, cid)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
     ON CONFLICT(platform, platform_id) DO UPDATE SET
       category_id   = excluded.category_id,
       title         = excluded.title,
       author        = excluded.author,
       cover_url     = excluded.cover_url,
       duration      = excluded.duration,
       description   = excluded.description,
       original_url  = excluded.original_url,
       aid           = excluded.aid,
       cid           = excluded.cid,
       updated_at    = datetime('now')",
        params![
            category_id,
            &data.platform,
            &data.platform_id,
            &data.title,
            &data.author,
            &data.cover_url,
            data.duration,
            &data.description,
            original_url,
            data.aid,
            data.cid,
        ],
    )
    .map_err(|e| e.to_string())?;

    let video_id: i64 = tx
        .query_row(
            "SELECT id FROM videos WHERE platform=?1 AND platform_id=?2",
            params![&data.platform, &data.platform_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if !preserve_tags.unwrap_or(false) {
        // 覆盖标签（防止更新时残留旧标签）
        tx.execute(
            "DELETE FROM video_tags WHERE video_id=?1",
            params![video_id],
        )
        .map_err(|e| e.to_string())?;

        for tag in &tags {
            let name = tag.trim();
            if name.is_empty() {
                continue;
            }
            tx.execute(
                "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
                params![name],
            )
            .map_err(|e| e.to_string())?;
            let tag_id: i64 = tx
                .query_row("SELECT id FROM tags WHERE name=?1", params![name], |r| {
                    r.get(0)
                })
                .map_err(|e| e.to_string())?;
            tx.execute(
                "INSERT OR IGNORE INTO video_tags (video_id, tag_id) VALUES (?1, ?2)",
                params![video_id, tag_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(video_id)
}

/// 在独立窗口打开视频，支持同时打开多个播放窗口。
#[tauri::command]
pub fn open_video_window(
    video_id: i64,
    app: tauri::AppHandle,
) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        let _ = (video_id, app);
        return Err("Android 使用内嵌播放".into());
    }

    #[cfg(not(target_os = "android"))]
    {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let label = format!("player-{video_id}-{stamp}");
        // 同时把视频 id 放进 URL query，避免某些 WebView 环境里
        // initialization_script 没有及时执行时播放窗口空白。
        let url = format!("index.html?player_id={video_id}");
        WebviewWindowBuilder::new(&app, label, WebviewUrl::App(url.into()))
            .title("Collectrix")
            .inner_size(1280.0, 800.0)
            .initialization_script(format!(
                "window.__COLLECTRIX_PLAYER_ID__ = {video_id};"
            ))
            .build()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[tauri::command]
pub fn move_video_to_category(
    video_id: i64,
    category_id: Option<i64>,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let category_id = category_id.or_else(|| uncategorized_id(&conn));
    if let Some(cid) = category_id {
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM categories WHERE id=?1",
                params![cid],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if !exists {
            return Err("目标分类不存在".into());
        }
    }
    let changed = conn
        .execute(
            "UPDATE videos SET category_id=?1, updated_at=datetime('now') WHERE id=?2",
            params![category_id, video_id],
        )
        .map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err("视频不存在".into());
    }
    Ok(())
}

#[tauri::command]
pub fn update_video(
    id: i64,
    category_id: Option<i64>,
    tags: Vec<String>,
    title: Option<String>,
    author: Option<String>,
    description: Option<String>,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let mut conn = state.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let category_id = category_id.or_else(|| uncategorized_id(&tx));
    tx.execute(
        "UPDATE videos SET
            category_id=?1,
            title=COALESCE(?2, title),
            author=COALESCE(?3, author),
            description=COALESCE(?4, description),
            updated_at=datetime('now')
         WHERE id=?5",
        params![category_id, title, author, description, id],
    )
    .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM video_tags WHERE video_id=?1", params![id])
        .map_err(|e| e.to_string())?;
    for tag in &tags {
        let name = tag.trim();
        if name.is_empty() {
            continue;
        }
        tx.execute(
            "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
            params![name],
        )
        .map_err(|e| e.to_string())?;
        let tag_id: i64 = tx
            .query_row("SELECT id FROM tags WHERE name=?1", params![name], |r| {
                r.get(0)
            })
            .map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT OR IGNORE INTO video_tags (video_id, tag_id) VALUES (?1, ?2)",
            params![id, tag_id],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_video(id: i64, state: State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM videos WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn refresh_video_info(video_id: i64, state: State<'_, DbState>) -> Result<(), String> {
    let (platform, platform_id): (String, String) = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT platform, platform_id FROM videos WHERE id=?1",
            params![video_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| e.to_string())?
    };

    if platform != "bilibili" {
        return Err("仅支持补全 Bilibili 视频信息".into());
    }

    let info = crate::bilibili::fetch_video_info(&platform_id).await?;

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE videos SET aid=?1, cid=?2, updated_at=datetime('now') WHERE id=?3",
        params![info.aid, info.cid, video_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_videos_with_children(
    category_id: i64,
    exclude_articles: Option<bool>,
    state: State<'_, DbState>,
) -> Result<Vec<CategoryVideoGroup>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut groups: Vec<CategoryVideoGroup> = Vec::new();
    let filter = if exclude_articles.unwrap_or(false) {
        " AND platform NOT IN ('github','blog','bilibili_article')"
    } else {
        ""
    };

    let current_name: String = conn
        .query_row(
            "SELECT name FROM categories WHERE id=?1",
            params![category_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    // 通用查询函数：给定分类 id 返回视频列表
    let query_videos = |cat_id: i64| -> Result<Vec<VideoRow>, String> {
        let sql = format!(
            "SELECT {SELECT_FIELDS} FROM videos WHERE category_id = ?1{filter} ORDER BY updated_at DESC"
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let mut videos: Vec<VideoRow> = stmt
            .query_map(params![cat_id], read_video_row)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        fill_tags_many(&conn, &mut videos)?;
        Ok(videos)
    };

    // 1) 当前分类自身的视频
    let own = query_videos(category_id)?;
    if !own.is_empty() {
        groups.push(CategoryVideoGroup {
            category_id: Some(category_id),
            category_name: format!("{current_name}（本分类）"),
            videos: own,
        });
    }

    // 2) 直接子分类
    let mut child_stmt = conn
        .prepare("SELECT id, name FROM categories WHERE parent_id=?1 ORDER BY position, id")
        .map_err(|e| e.to_string())?;
    let children: Vec<(i64, String)> = child_stmt
        .query_map(params![category_id], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (child_id, child_name) in children {
        let vids = query_videos(child_id)?;
        if !vids.is_empty() {
            groups.push(CategoryVideoGroup {
                category_id: Some(child_id),
                category_name: child_name,
                videos: vids,
            });
        }
    }

    Ok(groups)
}
