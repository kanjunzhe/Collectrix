use crate::commands::video::{fill_tags_many, read_video_row};
use crate::models::{SELECT_FIELDS, VideoRow};
use crate::state::DbState;
use rusqlite::params;
use serde::Serialize;
use tauri::State;

#[derive(Serialize, Clone)]
pub struct TopicInfo {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub video_count: i64,
}

#[tauri::command]
pub fn list_topics(state: State<'_, DbState>) -> Result<Vec<TopicInfo>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name, t.description, t.color,
                    (SELECT COUNT(*) FROM video_topics vt WHERE vt.topic_id = t.id)
             FROM topics t ORDER BY t.name",
        )
        .map_err(|e| e.to_string())?;
    let list = stmt
        .query_map([], |r| {
            Ok(TopicInfo {
                id: r.get(0)?,
                name: r.get(1)?,
                description: r.get(2)?,
                color: r.get(3)?,
                video_count: r.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(list)
}

#[tauri::command]
pub fn create_topic(
    name: String,
    description: Option<String>,
    color: String,
    state: State<'_, DbState>,
) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO topics (name, description, color) VALUES (?1, ?2, ?3)",
        params![name, description, color],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_topic(
    id: i64,
    name: String,
    description: Option<String>,
    color: String,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE topics SET name=?1, description=?2, color=?3 WHERE id=?4",
        params![name, description, color, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_topic(id: i64, state: State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM topics WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn set_video_topics(
    video_id: i64,
    topic_ids: Vec<i64>,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let mut conn = state.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "DELETE FROM video_topics WHERE video_id=?1",
        params![video_id],
    )
    .map_err(|e| e.to_string())?;
    for tid in topic_ids {
        tx.execute(
            "INSERT OR IGNORE INTO video_topics (video_id, topic_id) VALUES (?1, ?2)",
            params![video_id, tid],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_topics_for_video(
    video_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<TopicInfo>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name, t.description, t.color, 0
             FROM topics t JOIN video_topics vt ON vt.topic_id = t.id
             WHERE vt.video_id = ?1 ORDER BY t.name",
        )
        .map_err(|e| e.to_string())?;
    let list = stmt
        .query_map(params![video_id], |r| {
            Ok(TopicInfo {
                id: r.get(0)?,
                name: r.get(1)?,
                description: r.get(2)?,
                color: r.get(3)?,
                video_count: r.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(list)
}

#[tauri::command]
pub fn list_videos_by_topic(
    topic_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<VideoRow>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let sql = format!(
        "SELECT {SELECT_FIELDS} FROM videos v
         JOIN video_topics vt ON vt.video_id = v.id
         WHERE vt.topic_id = ?1 ORDER BY v.updated_at DESC"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut videos: Vec<VideoRow> = stmt
        .query_map(params![topic_id], read_video_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    fill_tags_many(&conn, &mut videos)?;
    Ok(videos)
}

#[tauri::command]
pub fn add_videos_to_topic(
    topic_id: i64,
    video_ids: Vec<i64>,
    state: State<'_, DbState>,
) -> Result<usize, String> {
    let mut conn = state.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut count = 0;
    for vid in video_ids {
        let changed = tx
            .execute(
                "INSERT OR IGNORE INTO video_topics (video_id, topic_id) VALUES (?1, ?2)",
                params![vid, topic_id],
            )
            .map_err(|e| e.to_string())?;
        count += changed;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(count)
}
