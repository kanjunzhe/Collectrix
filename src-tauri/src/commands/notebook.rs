use crate::commands::video::{fill_tags_many, read_video_row};
use crate::models::{SELECT_FIELDS, VideoRow};
use crate::state::DbState;
use rusqlite::params;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct NotebookInfo {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub video_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct NotebookDetail {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub videos: Vec<VideoRow>,
}

#[tauri::command]
pub fn list_notebooks(state: State<'_, DbState>) -> Result<Vec<NotebookInfo>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT n.id, n.title, n.content,
                    (SELECT COUNT(*) FROM notebook_videos nv WHERE nv.notebook_id=n.id),
                    n.created_at, n.updated_at
             FROM notebooks n ORDER BY n.updated_at DESC",
        )
        .map_err(|e| e.to_string())?;
    stmt.query_map([], |row| {
        Ok(NotebookInfo {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            video_count: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_notebook(id: i64, state: State<'_, DbState>) -> Result<NotebookDetail, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let (title, content): (String, String) = conn
        .query_row(
            "SELECT title, content FROM notebooks WHERE id=?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(&format!(
            "SELECT {SELECT_FIELDS} FROM videos v
             JOIN notebook_videos nv ON nv.video_id = v.id
             WHERE nv.notebook_id = ?1
             ORDER BY nv.created_at DESC"
        ))
        .map_err(|e| e.to_string())?;
    let mut videos: Vec<VideoRow> = stmt
        .query_map(params![id], read_video_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    fill_tags_many(&conn, &mut videos)?;

    Ok(NotebookDetail {
        id,
        title,
        content,
        videos,
    })
}

#[tauri::command]
pub fn save_notebook(
    id: Option<i64>,
    title: String,
    content: String,
    state: State<'_, DbState>,
) -> Result<i64, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("记事本标题不能为空".into());
    }
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(id) = id {
        conn.execute(
            "UPDATE notebooks SET title=?1, content=?2, updated_at=datetime('now') WHERE id=?3",
            params![title, content, id],
        )
        .map_err(|e| e.to_string())?;
        Ok(id)
    } else {
        conn.execute(
            "INSERT INTO notebooks (title, content) VALUES (?1, ?2)",
            params![title, content],
        )
        .map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }
}

#[tauri::command]
pub fn delete_notebook(id: i64, state: State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM notebooks WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn add_video_to_notebook(
    notebook_id: i64,
    video_id: i64,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO notebook_videos (notebook_id, video_id) VALUES (?1, ?2)",
        params![notebook_id, video_id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE notebooks SET updated_at=datetime('now') WHERE id=?1",
        params![notebook_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn remove_video_from_notebook(
    notebook_id: i64,
    video_id: i64,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM notebook_videos WHERE notebook_id=?1 AND video_id=?2",
        params![notebook_id, video_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_notebooks_for_video(
    video_id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<NotebookInfo>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT n.id, n.title, n.content,
                    (SELECT COUNT(*) FROM notebook_videos nv2 WHERE nv2.notebook_id=n.id),
                    n.created_at, n.updated_at
             FROM notebooks n
             JOIN notebook_videos nv ON nv.notebook_id=n.id
             WHERE nv.video_id=?1
             ORDER BY n.updated_at DESC",
        )
        .map_err(|e| e.to_string())?;
    stmt.query_map(params![video_id], |row| {
        Ok(NotebookInfo {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            video_count: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())
}
