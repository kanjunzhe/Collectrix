use crate::commands::video::{fill_tags_many, read_video_row};
use crate::models::{SELECT_FIELDS, VideoRow};
use crate::state::DbState;
use rusqlite::{params, params_from_iter, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

const TRANSFER_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct TransferData {
    version: u32,
    exported_at: i64,
    categories: Vec<TransferCategory>,
    tags: Vec<TransferTag>,
    videos: Vec<TransferVideo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TransferCategory {
    path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TransferTag {
    name: String,
    color: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TransferVideo {
    platform: String,
    platform_id: String,
    title: String,
    author: Option<String>,
    cover_url: Option<String>,
    duration: Option<i64>,
    description: Option<String>,
    original_url: Option<String>,
    aid: Option<i64>,
    cid: Option<i64>,
    category_path: Option<Vec<String>>,
    tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ImportSummary {
    pub videos: usize,
    pub categories: usize,
    pub tags: usize,
}

fn placeholders(count: usize) -> String {
    std::iter::repeat("?")
        .take(count)
        .collect::<Vec<_>>()
        .join(",")
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn category_path(
    id: i64,
    categories: &HashMap<i64, (String, Option<i64>)>,
) -> Option<Vec<String>> {
    let mut path = Vec::new();
    let mut current = Some(id);
    let mut guard = 0;
    while let Some(category_id) = current {
        guard += 1;
        if guard > 100 {
            break;
        }
        let (name, parent_id) = categories.get(&category_id)?;
        path.push(name.clone());
        current = *parent_id;
    }
    path.reverse();
    Some(path)
}

/// 导出全部或部分收藏为 JSON。directory 是目标文件夹。
#[tauri::command]
pub fn export_data(
    directory: String,
    include_all: bool,
    category_ids: Vec<i64>,
    tag_ids: Vec<i64>,
    state: State<'_, DbState>,
) -> Result<String, String> {
    if !include_all && category_ids.is_empty() && tag_ids.is_empty() {
        return Err("请选择要导出的分类或标签".into());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let categories: HashMap<i64, (String, Option<i64>)> = {
        let mut stmt = conn
            .prepare("SELECT id, name, parent_id FROM categories")
            .map_err(|e| e.to_string())?;
        stmt.query_map([], |r| Ok((r.get(0)?, (r.get(1)?, r.get(2)?))))
            .map_err(|e| e.to_string())?
            .collect::<Result<HashMap<_, _>, _>>()
            .map_err(|e| e.to_string())?
    };

    let durations: HashMap<i64, Option<i64>> = {
        let mut stmt = conn
            .prepare("SELECT id, duration FROM videos")
            .map_err(|e| e.to_string())?;
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<HashMap<_, _>, _>>()
            .map_err(|e| e.to_string())?
    };

    let mut videos: Vec<VideoRow> = if include_all {
        let mut stmt = conn
            .prepare(&format!(
                "SELECT {SELECT_FIELDS} FROM videos ORDER BY updated_at DESC"
            ))
            .map_err(|e| e.to_string())?;
        stmt.query_map([], read_video_row)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    } else {
        let cat_clause = if category_ids.is_empty() {
            "0".to_string()
        } else {
            format!(
                "category_id IN (SELECT id FROM tree)"
            )
        };
        let tag_clause = if tag_ids.is_empty() {
            "0".to_string()
        } else {
            format!(
                "id IN (SELECT video_id FROM video_tags WHERE tag_id IN ({}))",
                placeholders(tag_ids.len())
            )
        };
        let cte = if category_ids.is_empty() {
            String::new()
        } else {
            format!(
                "WITH RECURSIVE tree(id) AS (
                    SELECT id FROM categories WHERE id IN ({})
                    UNION ALL
                    SELECT c.id FROM categories c JOIN tree t ON c.parent_id=t.id
                 ) ",
                placeholders(category_ids.len())
            )
        };
        let sql = format!(
            "{cte}SELECT {SELECT_FIELDS} FROM videos
             WHERE {cat_clause} OR {tag_clause}
             ORDER BY updated_at DESC"
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        stmt.query_map(
            params_from_iter(category_ids.iter().chain(tag_ids.iter())),
            read_video_row,
        )
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    };

    fill_tags_many(&conn, &mut videos)?;

    let mut used_category_ids: HashSet<i64> = HashSet::new();
    let mut used_tags: HashMap<String, String> = HashMap::new();

    if include_all {
        used_category_ids.extend(categories.keys().copied());
        let mut stmt = conn
            .prepare("SELECT name, color FROM tags")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (name, color) = row.map_err(|e| e.to_string())?;
            used_tags.insert(name, color);
        }
    } else {
        for video in &videos {
            let mut current = video.category_id;
            while let Some(id) = current {
                if !used_category_ids.insert(id) {
                    break;
                }
                current = categories.get(&id).and_then(|(_, parent)| *parent);
            }
            for tag in &video.tags {
                used_tags
                    .entry(tag.name.clone())
                    .or_insert_with(|| tag.color.clone());
            }
        }
    }

    let mut transfer_categories: Vec<TransferCategory> = used_category_ids
        .into_iter()
        .filter_map(|id| category_path(id, &categories))
        .map(|path| TransferCategory { path })
        .collect();
    transfer_categories.sort_by(|a, b| a.path.cmp(&b.path));

    let mut transfer_tags: Vec<TransferTag> = used_tags
        .into_iter()
        .map(|(name, color)| TransferTag { name, color })
        .collect();
    transfer_tags.sort_by(|a, b| a.name.cmp(&b.name));

    let transfer_videos: Vec<TransferVideo> = videos
        .into_iter()
        .map(|video| TransferVideo {
            platform: video.platform,
            platform_id: video.platform_id,
            title: video.title,
            author: video.author,
            cover_url: video.cover_url,
            duration: durations.get(&video.id).copied().flatten(),
            description: video.description,
            original_url: video.original_url,
            aid: video.aid,
            cid: video.cid,
            category_path: video
                .category_id
                .and_then(|id| category_path(id, &categories)),
            tags: video.tags.into_iter().map(|tag| tag.name).collect(),
        })
        .collect();

    let data = TransferData {
        version: TRANSFER_VERSION,
        exported_at: now_ts(),
        categories: transfer_categories,
        tags: transfer_tags,
        videos: transfer_videos,
    };

    let dir = Path::new(&directory);
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let file = dir.join(format!("collectrix-export-{}.json", now_ts()));
    let text = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    std::fs::write(&file, text).map_err(|e| e.to_string())?;
    Ok(file.to_string_lossy().into_owned())
}

/// 导入 JSON，分类按路径匹配/创建，标签和视频按名称/平台 ID 合并。
#[tauri::command]
pub fn import_data(path: String, state: State<'_, DbState>) -> Result<ImportSummary, String> {
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let data: TransferData = serde_json::from_str(&text).map_err(|e| format!("JSON 解析失败: {e}"))?;
    if data.version != TRANSFER_VERSION {
        return Err(format!("不支持的导出文件版本: {}", data.version));
    }

    let mut conn = state.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let mut categories_created = 0usize;
    let mut category_map: HashMap<Vec<String>, i64> = HashMap::new();
    let mut paths: Vec<Vec<String>> = data.categories.into_iter().map(|c| c.path).collect();
    paths.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
    paths.dedup();

    for path in paths {
        if path.is_empty() {
            continue;
        }
        for end in 1..=path.len() {
            let prefix = path[..end].to_vec();
            if category_map.contains_key(&prefix) {
                continue;
            }
            let parent_id = if end == 1 {
                None
            } else {
                category_map.get(&path[..end - 1]).copied()
            };
            let name = &path[end - 1];
            let existing: Option<i64> = if let Some(pid) = parent_id {
                tx.query_row(
                    "SELECT id FROM categories WHERE name=?1 AND parent_id=?2",
                    params![name, pid],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| e.to_string())?
            } else {
                tx.query_row(
                    "SELECT id FROM categories WHERE name=?1 AND parent_id IS NULL",
                    params![name],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| e.to_string())?
            };
            let id = match existing {
                Some(id) => id,
                None => {
                    tx.execute(
                        "INSERT INTO categories (name, parent_id) VALUES (?1, ?2)",
                        params![name, parent_id],
                    )
                    .map_err(|e| e.to_string())?;
                    categories_created += 1;
                    tx.last_insert_rowid()
                }
            };
            category_map.insert(prefix, id);
        }
    }

    let mut tags_created = 0usize;
    let mut tag_map: HashMap<String, i64> = HashMap::new();
    for tag in data.tags {
        let existing: Option<i64> = tx
            .query_row(
                "SELECT id FROM tags WHERE name=?1",
                params![tag.name],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let id = match existing {
            Some(id) => id,
            None => {
                tx.execute(
                    "INSERT INTO tags (name, color) VALUES (?1, ?2)",
                    params![tag.name, tag.color],
                )
                .map_err(|e| e.to_string())?;
                tags_created += 1;
                tx.last_insert_rowid()
            }
        };
        tag_map.insert(tag.name, id);
    }

    let mut videos_imported = 0usize;
    for video in data.videos {
        if video.platform.trim().is_empty() || video.platform_id.trim().is_empty() {
            continue;
        }
        let category_id = video
            .category_path
            .as_ref()
            .and_then(|path| category_map.get(path).copied())
            .or_else(|| {
                tx.query_row(
                    "SELECT id FROM categories WHERE name='未分类' AND parent_id IS NULL LIMIT 1",
                    [],
                    |r| r.get(0),
                )
                .optional()
                .ok()
                .flatten()
            });

        tx.execute(
            "INSERT INTO videos
                (category_id, platform, platform_id, title, author, cover_url,
                 duration, description, original_url, aid, cid)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(platform, platform_id) DO UPDATE SET
                category_id = COALESCE(excluded.category_id, videos.category_id),
                title = excluded.title,
                author = excluded.author,
                cover_url = excluded.cover_url,
                description = excluded.description,
                original_url = excluded.original_url,
                aid = COALESCE(excluded.aid, videos.aid),
                cid = COALESCE(excluded.cid, videos.cid),
                updated_at = datetime('now')",
            params![
                category_id,
                video.platform,
                video.platform_id,
                video.title,
                video.author,
                video.cover_url,
                video.duration,
                video.description,
                video.original_url,
                video.aid,
                video.cid,
            ],
        )
        .map_err(|e| e.to_string())?;

        let video_id: i64 = tx
            .query_row(
                "SELECT id FROM videos WHERE platform=?1 AND platform_id=?2",
                params![video.platform, video.platform_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;

        for tag_name in video.tags {
            let name = tag_name.trim();
            if name.is_empty() {
                continue;
            }
            let tag_id = if let Some(id) = tag_map.get(name) {
                *id
            } else {
                let existing: Option<i64> = tx
                    .query_row("SELECT id FROM tags WHERE name=?1", params![name], |r| {
                        r.get(0)
                    })
                    .optional()
                    .map_err(|e| e.to_string())?;
                let id = match existing {
                    Some(id) => id,
                    None => {
                        tx.execute(
                            "INSERT INTO tags (name, color) VALUES (?1, ?2)",
                            params![name, "#1976D2"],
                        )
                        .map_err(|e| e.to_string())?;
                        tags_created += 1;
                        tx.last_insert_rowid()
                    }
                };
                tag_map.insert(name.to_string(), id);
                id
            };
            tx.execute(
                "INSERT OR IGNORE INTO video_tags (video_id, tag_id) VALUES (?1, ?2)",
                params![video_id, tag_id],
            )
            .map_err(|e| e.to_string())?;
        }

        videos_imported += 1;
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok(ImportSummary {
        videos: videos_imported,
        categories: categories_created,
        tags: tags_created,
    })
}
