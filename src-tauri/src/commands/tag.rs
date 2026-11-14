use crate::commands::video::{fill_tags_many, read_video_row};
use crate::models::{SELECT_FIELDS, TagInfo, VideoRow};
use crate::state::DbState;
use rusqlite::{params, params_from_iter, OptionalExtension};
use tauri::State;

#[tauri::command]
pub fn list_tags(state: State<'_, DbState>) -> Result<Vec<TagInfo>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, color FROM tags ORDER BY name")
        .map_err(|e| e.to_string())?;
    let tags = stmt
        .query_map([], |row| {
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

#[tauri::command]
pub fn create_tag(name: String, color: String, state: State<'_, DbState>) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO tags (name, color) VALUES (?1, ?2)",
        params![name, color],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_tag(
    id: i64,
    name: String,
    color: String,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE tags SET name=?1, color=?2 WHERE id=?3",
        params![name, color, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_tag(id: i64, state: State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM tags WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 列出多个标签下的视频（去重），用于标签合并预览。
#[tauri::command]
pub fn list_videos_by_tags(
    tag_ids: Vec<i64>,
    state: State<'_, DbState>,
) -> Result<Vec<VideoRow>, String> {
    if tag_ids.is_empty() {
        return Ok(vec![]);
    }
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let placeholders = std::iter::repeat("?")
        .take(tag_ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT DISTINCT {SELECT_FIELDS} FROM videos v
         JOIN video_tags vt ON vt.video_id = v.id
         WHERE vt.tag_id IN ({placeholders})
         ORDER BY v.updated_at DESC"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut videos: Vec<VideoRow> = stmt
        .query_map(params_from_iter(tag_ids.iter()), read_video_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    fill_tags_many(&conn, &mut videos)?;
    Ok(videos)
}

/// 将 source 标签合并到 target 标签，可同时重命名 target。
#[tauri::command]
pub fn merge_tags(
    source_id: i64,
    target_id: i64,
    merged_name: String,
    state: State<'_, DbState>,
) -> Result<i64, String> {
    if source_id == target_id {
        return Err("不能合并同一个标签".into());
    }
    let name = merged_name.trim();
    if name.is_empty() {
        return Err("标签名不能为空".into());
    }

    let mut conn = state.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let source_exists: Option<i64> = tx
        .query_row("SELECT id FROM tags WHERE id=?1", params![source_id], |r| {
            r.get(0)
        })
        .optional()
        .map_err(|e| e.to_string())?;
    if source_exists.is_none() {
        return Err("源标签不存在".into());
    }

    let target_name: String = tx
        .query_row("SELECT name FROM tags WHERE id=?1", params![target_id], |r| {
            r.get(0)
        })
        .map_err(|_| "目标标签不存在".to_string())?;

    if name != target_name {
        let conflict: Option<i64> = tx
            .query_row(
                "SELECT id FROM tags WHERE name=?1 AND id NOT IN (?2, ?3)",
                params![name, source_id, target_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if conflict.is_some() {
            return Err(format!("标签「{name}」已存在"));
        }
    }

    tx.execute(
        "INSERT OR IGNORE INTO video_tags (video_id, tag_id)
         SELECT video_id, ?1 FROM video_tags WHERE tag_id=?2",
        params![target_id, source_id],
    )
    .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM video_tags WHERE tag_id=?1", params![source_id])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM tags WHERE id=?1", params![source_id])
        .map_err(|e| e.to_string())?;

    if name != target_name {
        tx.execute(
            "UPDATE tags SET name=?1 WHERE id=?2",
            params![name, target_id],
        )
        .map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(target_id)
}
