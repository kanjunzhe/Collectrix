use crate::commands::video::{fill_tags_many, read_video_row};
use crate::models::{CategoryNode, SELECT_FIELDS, VideoRow};
use crate::state::DbState;
use rusqlite::{params, OptionalExtension};
use std::collections::HashMap;
use tauri::State;

// ---------- 分类 ----------
#[tauri::command]
pub fn get_category_tree(state: State<'_, DbState>) -> Result<Vec<CategoryNode>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, parent_id FROM categories ORDER BY position, id")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(CategoryNode {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                children: vec![],
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(build_tree(rows))
}

fn build_tree(rows: Vec<CategoryNode>) -> Vec<CategoryNode> {
    let mut groups: HashMap<Option<i64>, Vec<CategoryNode>> = HashMap::new();
    for node in rows {
        groups.entry(node.parent_id).or_default().push(node);
    }
    fn attach(
        parent_id: Option<i64>,
        groups: &mut HashMap<Option<i64>, Vec<CategoryNode>>,
    ) -> Vec<CategoryNode> {
        let mut nodes = groups.remove(&parent_id).unwrap_or_default();
        for node in &mut nodes {
            node.children = attach(Some(node.id), groups);
        }
        nodes
    }
    attach(None, &mut groups)
}

#[tauri::command]
pub fn add_category(
    name: String,
    parent_id: Option<i64>,
    state: State<'_, DbState>,
) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO categories (name, parent_id) VALUES (?1, ?2)",
        params![name, parent_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn delete_category(id: i64, state: State<'_, DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM categories WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 列出分类及其所有子分类下的视频，用于删除前确认。
#[tauri::command]
pub fn list_videos_in_category_tree(
    id: i64,
    state: State<'_, DbState>,
) -> Result<Vec<VideoRow>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let sql = format!(
        "WITH RECURSIVE tree(id) AS (
            SELECT id FROM categories WHERE id=?1
            UNION ALL
            SELECT c.id FROM categories c JOIN tree t ON c.parent_id=t.id
         )
         SELECT {SELECT_FIELDS} FROM videos
         WHERE category_id IN (SELECT id FROM tree)
         ORDER BY updated_at DESC"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut videos: Vec<VideoRow> = stmt
        .query_map(params![id], read_video_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    fill_tags_many(&conn, &mut videos)?;
    Ok(videos)
}

/// 移动分类到新的父分类；parent_id = None 表示移到顶级。
#[tauri::command]
pub fn move_category(
    id: i64,
    parent_id: Option<i64>,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let mut conn = state.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let exists: Option<i64> = tx
        .query_row("SELECT id FROM categories WHERE id=?1", params![id], |r| {
            r.get(0)
        })
        .optional()
        .map_err(|e| e.to_string())?;
    if exists.is_none() {
        return Err("分类不存在".into());
    }

    if let Some(pid) = parent_id {
        if pid == id {
            return Err("不能把分类移动到自身".into());
        }
        let parent_exists: Option<i64> = tx
            .query_row(
                "SELECT id FROM categories WHERE id=?1",
                params![pid],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if parent_exists.is_none() {
            return Err("目标父分类不存在".into());
        }

        let is_descendant: Option<i64> = tx
            .query_row(
                "WITH RECURSIVE descendants(id) AS (
                    SELECT id FROM categories WHERE parent_id=?1
                    UNION ALL
                    SELECT c.id FROM categories c
                    JOIN descendants d ON c.parent_id=d.id
                 )
                 SELECT id FROM descendants WHERE id=?2",
                params![id, pid],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if is_descendant.is_some() {
            return Err("不能把分类移动到自己的子分类下".into());
        }
    }

    tx.execute(
        "UPDATE categories SET parent_id=?1 WHERE id=?2",
        params![parent_id, id],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

/// 删除分类树。delete_videos=false 时保留视频并置为未分类；true 时一并删除视频。
#[tauri::command]
pub fn delete_category_tree(
    id: i64,
    delete_videos: bool,
    state: State<'_, DbState>,
) -> Result<(), String> {
    let mut conn = state.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let exists: Option<i64> = tx
        .query_row("SELECT id FROM categories WHERE id=?1", params![id], |r| {
            r.get(0)
        })
        .optional()
        .map_err(|e| e.to_string())?;
    if exists.is_none() {
        return Err("分类不存在".into());
    }

    if delete_videos {
        tx.execute(
            "WITH RECURSIVE tree(id) AS (
                SELECT id FROM categories WHERE id=?1
                UNION ALL
                SELECT c.id FROM categories c JOIN tree t ON c.parent_id=t.id
             )
             DELETE FROM videos WHERE category_id IN (SELECT id FROM tree)",
            params![id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        let fallback = crate::commands::video::uncategorized_id(&tx);
        tx.execute(
            "WITH RECURSIVE tree(id) AS (
                SELECT id FROM categories WHERE id=?1
                UNION ALL
                SELECT c.id FROM categories c JOIN tree t ON c.parent_id=t.id
             )
             UPDATE videos SET category_id=?2, updated_at=datetime('now')
             WHERE category_id IN (SELECT id FROM tree)",
            params![id, fallback],
        )
        .map_err(|e| e.to_string())?;
    }

    tx.execute("DELETE FROM categories WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}
