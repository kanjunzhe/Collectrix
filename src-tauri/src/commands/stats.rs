use crate::commands::video::{fill_tags_many, read_video_row};
use crate::models::{SELECT_FIELDS, Stats, VideoRow};
use crate::state::DbState;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct CountItem {
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct MonthlyCount {
    pub month: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct DetailedStats {
    pub videos_count: i64,
    pub categories_count: i64,
    pub tags_count: i64,
    pub topics_count: i64,
    pub ai_sorted_count: i64,
    pub local_video_count: i64,
    pub local_audio_count: i64,
    pub transcript_count: i64,
    pub platforms: Vec<CountItem>,
    pub categories: Vec<CountItem>,
    pub tags: Vec<CountItem>,
    pub monthly: Vec<MonthlyCount>,
}

fn count(conn: &rusqlite::Connection, sql: &str) -> Result<i64, String> {
    conn.query_row(sql, [], |row| row.get(0))
        .map_err(|e| e.to_string())
}

fn counts(conn: &rusqlite::Connection, sql: &str) -> Result<Vec<CountItem>, String> {
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    stmt.query_map([], |row| {
        Ok(CountItem {
            name: row.get(0)?,
            count: row.get(1)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_detailed_stats(state: State<'_, DbState>) -> Result<DetailedStats, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let platforms = counts(
        &conn,
        "SELECT platform, COUNT(*) FROM videos GROUP BY platform ORDER BY COUNT(*) DESC",
    )?;
    let categories = counts(
        &conn,
        "SELECT COALESCE(c.name, '未分类'), COUNT(*)
         FROM videos v LEFT JOIN categories c ON c.id = v.category_id
         GROUP BY c.id, c.name ORDER BY COUNT(*) DESC LIMIT 12",
    )?;
    let tags = counts(
        &conn,
        "SELECT t.name, COUNT(*) FROM tags t
         JOIN video_tags vt ON vt.tag_id = t.id
         GROUP BY t.id, t.name ORDER BY COUNT(*) DESC LIMIT 12",
    )?;
    let mut stmt = conn
        .prepare(
            "SELECT strftime('%Y-%m', created_at), COUNT(*)
             FROM videos
             WHERE created_at >= datetime('now', '-12 months')
             GROUP BY strftime('%Y-%m', created_at)
             ORDER BY strftime('%Y-%m', created_at)",
        )
        .map_err(|e| e.to_string())?;
    let monthly = stmt
        .query_map([], |row| {
            Ok(MonthlyCount {
                month: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(DetailedStats {
        videos_count: count(&conn, "SELECT COUNT(*) FROM videos")?,
        categories_count: count(&conn, "SELECT COUNT(*) FROM categories")?,
        tags_count: count(&conn, "SELECT COUNT(*) FROM tags")?,
        topics_count: count(&conn, "SELECT COUNT(*) FROM topics")?,
        ai_sorted_count: count(&conn, "SELECT COUNT(*) FROM videos WHERE is_ai_sorted=1")?,
        local_video_count: count(
            &conn,
            "SELECT COUNT(*) FROM videos WHERE local_video_path IS NOT NULL AND local_video_path != ''",
        )?,
        local_audio_count: count(
            &conn,
            "SELECT COUNT(*) FROM videos WHERE local_audio_path IS NOT NULL AND local_audio_path != ''",
        )?,
        transcript_count: count(
            &conn,
            "SELECT COUNT(*) FROM videos WHERE transcript_path IS NOT NULL AND transcript_path != ''",
        )?,
        platforms,
        categories,
        tags,
        monthly,
    })
}

#[tauri::command]
pub fn get_stats(state: State<'_, DbState>) -> Result<Stats, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let videos_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM videos", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let categories_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let tags_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM tags", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(&format!(
            "SELECT {SELECT_FIELDS} FROM videos ORDER BY created_at DESC LIMIT 8"
        ))
        .map_err(|e| e.to_string())?;
    let mut recent: Vec<VideoRow> = stmt
        .query_map([], read_video_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    fill_tags_many(&conn, &mut recent)?;

    Ok(Stats {
        videos_count,
        categories_count,
        tags_count,
        recent_videos: recent,
    })
}
