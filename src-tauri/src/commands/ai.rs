//! AI 相关 Tauri 命令：连接检测、Ollama 启动、模型列表、AI 整理预览、
//! 接受 / 拒绝建议、相似度筛选，以及遗留的 ai_categorize_pending。

use crate::ai::{self, AiSuggestion, AiVideoInput};
use crate::ai_task::AiTaskManager;
use crate::commands::video::{fill_tags, read_video_row};
use crate::models::{SELECT_FIELDS, VideoRow};
use crate::settings;
use crate::state::DbState;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tauri::State;

// ============ 连接 / 环境检测 ============

/// 检测 AI 服务是否可达：请求 {base_url}/models
#[tauri::command]
pub async fn ping_ai_provider(app: tauri::AppHandle) -> Result<String, String> {
    let s = settings::load_settings(&app);
    if s.openai_base_url.is_empty() {
        return Err("未配置 API Base URL".into());
    }
    let url = format!("{}/models", s.openai_base_url.trim_end_matches('/'));

    let mut req = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?
        .get(&url);

    if !s.openai_api_key.is_empty() {
        req = req.bearer_auth(&s.openai_api_key);
    }

    let resp = req.send().await.map_err(|e| format!("连接失败: {e}"))?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!(
            "HTTP {}: {}",
            status,
            body.chars().take(200).collect::<String>()
        ));
    }
    Ok(format!("连接成功（{}）", url))
}

/// 请求 {base_url}/models，返回服务端支持的模型 id 列表
#[tauri::command]
pub async fn list_ai_models(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let s = settings::load_settings(&app);
    if s.openai_base_url.is_empty() {
        return Err("未配置 API Base URL".into());
    }
    let url = format!("{}/models", s.openai_base_url.trim_end_matches('/'));

    let mut req = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?
        .get(&url);

    if !s.openai_api_key.is_empty() {
        req = req.bearer_auth(&s.openai_api_key);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("请求 {} 失败: {e}", url))?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!("HTTP {}: {}", status, body));
    }

    #[derive(serde::Deserialize)]
    struct Resp {
        data: Vec<ModelEntry>,
    }
    #[derive(serde::Deserialize)]
    struct ModelEntry {
        id: String,
    }

    let parsed: Resp =
        serde_json::from_str(&body).map_err(|e| format!("解析响应失败: {e}，原文: {body}"))?;

    Ok(parsed.data.into_iter().map(|m| m.id).collect())
}

/// 启动 Ollama：先探测是否已在运行，未运行则 spawn `ollama serve`
#[tauri::command]
pub async fn launch_ollama(app: tauri::AppHandle) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(1))
        .build()
        .map_err(|e| e.to_string())?;

    if probe_ollama(&client).await {
        return Ok("Ollama 已在运行".into());
    }

    let s = settings::load_settings(&app);
    let path = if s.ollama_path.is_empty() {
        "ollama".to_string()
    } else {
        s.ollama_path.clone()
    };

    let mut cmd = std::process::Command::new(&path);
    cmd.arg("serve")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }

    match cmd.spawn() {
        Ok(_) => {}
        Err(e) => {
            if probe_ollama(&client).await {
                return Ok("Ollama 已在运行".into());
            }
            return Err(format!("启动 Ollama 失败（路径: {path}）: {e}"));
        }
    }

    for _ in 0..30 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if probe_ollama(&client).await {
            return Ok("Ollama 已就绪".into());
        }
    }

    if probe_port_in_use().await {
        return Err("端口 11434 已被其他程序占用，但 Ollama 未响应。\
             请检查是否有其他服务占用该端口，或手动启动 Ollama 桌面版。"
            .into());
    }

    Err("Ollama 已启动但 15 秒内未就绪，请检查模型是否已下载".into())
}

async fn probe_ollama(client: &reqwest::Client) -> bool {
    client
        .get("http://127.0.0.1:11434/api/tags")
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

async fn probe_port_in_use() -> bool {
    tokio::net::TcpStream::connect("127.0.0.1:11434")
        .await
        .is_ok()
}

// ============ AI 整理预览 ============

/// 读取待处理视频 + 已有分类/标签，调用 AI 返回建议（不落库）
#[tauri::command]
pub async fn ai_organize_preview(
    limit: Option<i64>,
    state: State<'_, DbState>,
    app: tauri::AppHandle,
) -> Result<Vec<AiSuggestion>, String> {
    eprintln!("[Organize] 命令开始，limit={:?}", limit);

    let s = settings::load_settings(&app);
    let limit = limit.unwrap_or(15);

    eprintln!("[Organize] 读取数据库…");

    let (inputs, existing_categories, existing_tags) = {
        let conn = state.0.lock().map_err(|e| {
            eprintln!("[Organize] ✗ 获取数据库锁失败: {e}");
            e.to_string()
        })?;

        let mut stmt = conn
            .prepare(&format!(
                "SELECT id, title, COALESCE(description,''), COALESCE(author,''), cover_url
                 FROM videos
                 WHERE is_ai_sorted = 0 OR ai_confirmed = 0
                 ORDER BY updated_at DESC LIMIT {limit}"
            ))
            .map_err(|e| {
                eprintln!("[Organize] ✗ 准备 SQL 失败: {e}");
                e.to_string()
            })?;

        let inputs: Vec<AiVideoInput> = stmt
            .query_map([], |r| {
                Ok(AiVideoInput {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    description: r.get(2)?,
                    author: r.get(3)?,
                    cover_url: r.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let cats: Vec<String> = conn
            .prepare("SELECT name FROM categories")
            .and_then(|mut s| s.query_map([], |r| r.get(0))?.collect::<Result<_, _>>())
            .unwrap_or_default();

        let tags: Vec<String> = conn
            .prepare("SELECT name FROM tags")
            .and_then(|mut s| s.query_map([], |r| r.get(0))?.collect::<Result<_, _>>())
            .unwrap_or_default();

        (inputs, cats, tags)
    };

    eprintln!(
        "[Organize] 数据库读取完成：{} 个待处理视频，{} 分类，{} 标签",
        inputs.len(),
        existing_categories.len(),
        existing_tags.len()
    );

    if inputs.is_empty() {
        eprintln!("[Organize] 没有待处理视频，直接返回");
        return Ok(vec![]);
    }

    eprintln!("[Organize] 开始调用 AI…");
    let start = std::time::Instant::now();

    let suggestions = ai::categorize_batch(&s, &existing_categories, &existing_tags, &inputs)
        .await
        .map_err(|e| {
            eprintln!(
                "[Organize] ✗ AI 调用失败 (耗时 {:.1}s): {e}",
                start.elapsed().as_secs_f32()
            );
            e
        })?;

    eprintln!(
        "[Organize] ✓ AI 调用成功 (耗时 {:.1}s)，返回 {} 条建议",
        start.elapsed().as_secs_f32(),
        suggestions.len()
    );

    let info_map: std::collections::HashMap<i64, (String, Option<String>)> = inputs
        .iter()
        .map(|v| (v.id, (v.title.clone(), v.cover_url.clone())))
        .collect();

    Ok(suggestions
        .into_iter()
        .map(|(id, raw)| {
            let (title, cover) = info_map.get(&id).cloned().unwrap_or((String::new(), None));
            AiSuggestion {
                video_id: id,
                title,
                cover_url: cover,
                suggested_category: raw.category,
                suggested_category_id: None,
                parent_category_id: None,
                category_path: None,
                suggested_tags: raw.tags,
                summary: raw.summary,
                confidence: raw.confidence,
            }
        })
        .collect())
}

// ============ 应用 / 拒绝建议 ============

fn ensure_category_path(
    tx: &rusqlite::Transaction<'_>,
    path: &[String],
) -> Result<i64, String> {
    let mut parent_id: Option<i64> = None;
    let mut result: Option<i64> = None;
    for raw in path {
        let name = raw.trim();
        if name.is_empty() {
            continue;
        }
        let existing: Option<i64> = if let Some(pid) = parent_id {
            tx.query_row(
                "SELECT id FROM categories WHERE name=?1 AND parent_id=?2",
                params![name, pid],
                |r| r.get(0),
            )
            .ok()
        } else {
            tx.query_row(
                "SELECT id FROM categories WHERE name=?1 AND parent_id IS NULL",
                params![name],
                |r| r.get(0),
            )
            .ok()
        };
        let id = match existing {
            Some(id) => id,
            None => {
                tx.execute(
                    "INSERT INTO categories (name, parent_id) VALUES (?1, ?2)",
                    params![name, parent_id],
                )
                .map_err(|e| e.to_string())?;
                tx.last_insert_rowid()
            }
        };
        parent_id = Some(id);
        result = Some(id);
    }
    result.ok_or_else(|| "分类路径不能为空".to_string())
}

/// 接受一条建议：分类（按需新建，支持指定父级/多级路径）→ 标签（按需新建）→ 更新视频
#[tauri::command]
pub fn ai_apply_suggestion(
    suggestion: AiSuggestion,
    state: State<'_, DbState>,
    task_state: State<'_, AiTaskManager>,
) -> Result<(), String> {
    let mut conn = state.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // ---------- 1. 确定分类 id ----------
    let cat_id: i64 = if let Some(cid) = suggestion.suggested_category_id {
        // 用户直接选中了已有分类：校验存在性
        let exists: bool = tx
            .query_row("SELECT 1 FROM categories WHERE id=?1", params![cid], |_| {
                Ok(true)
            })
            .unwrap_or(false);
        if !exists {
            return Err(format!("指定的分类 id={cid} 不存在"));
        }
        cid
    } else if let Some(path) = suggestion
        .category_path
        .as_deref()
        .filter(|path| !path.is_empty())
    {
        ensure_category_path(&tx, path)?
    } else {
        // 按 (parent_id, name) 查找/新建

        // 校验父分类存在性，不存在则视为顶级
        let parent_id: Option<i64> = match suggestion.parent_category_id {
            Some(pid) => {
                let exists: bool = tx
                    .query_row("SELECT 1 FROM categories WHERE id=?1", params![pid], |_| {
                        Ok(true)
                    })
                    .unwrap_or(false);
                if exists { Some(pid) } else { None }
            }
            None => None,
        };

        let cat_name = suggestion.suggested_category.trim();
        if cat_name.is_empty() {
            return Err("分类名不能为空".into());
        }

        let existing: Option<i64> = if let Some(pid) = parent_id {
            tx.query_row(
                "SELECT id FROM categories WHERE name=?1 AND parent_id=?2",
                params![cat_name, pid],
                |r| r.get(0),
            )
            .ok()
        } else {
            tx.query_row(
                "SELECT id FROM categories WHERE name=?1 AND parent_id IS NULL",
                params![cat_name],
                |r| r.get(0),
            )
            .ok()
        };

        match existing {
            Some(id) => id,
            None => {
                tx.execute(
                    "INSERT INTO categories (name, parent_id) VALUES (?1, ?2)",
                    params![cat_name, parent_id],
                )
                .map_err(|e| e.to_string())?;
                let new_id = tx.last_insert_rowid();
                eprintln!(
                    "[AI] 新建分类「{}」(parent_id={:?}) id={}",
                    cat_name, parent_id, new_id
                );
                new_id
            }
        }
    };

    // ---------- 2. 更新视频 ----------
    tx.execute(
        "UPDATE videos SET category_id=?1, is_ai_sorted=1, ai_confirmed=1, \
         updated_at=datetime('now') WHERE id=?2",
        params![cat_id, suggestion.video_id],
    )
    .map_err(|e| e.to_string())?;

    // ---------- 3. 标签：有则复用，无则新建 ----------
    for tag in &suggestion.suggested_tags {
        let tag_name = tag.trim();
        if tag_name.is_empty() {
            continue;
        }

        tx.execute(
            "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
            params![tag_name],
        )
        .map_err(|e| e.to_string())?;

        let tid: i64 = tx
            .query_row(
                "SELECT id FROM tags WHERE name=?1",
                params![tag_name],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;

        tx.execute(
            "INSERT OR IGNORE INTO video_tags (video_id, tag_id) VALUES (?1, ?2)",
            params![suggestion.video_id, tid],
        )
        .map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    drop(conn);

    // 已处理的建议要从后台任务状态里移除，避免离开页面再回来时重新出现。
    if let Ok(mut ts) = task_state.0.lock() {
        ts.removed_ids.insert(suggestion.video_id);
        ts.suggestions
            .retain(|item| item.video_id != suggestion.video_id);
        if ts.status == "done" {
            ts.progress = format!("还剩 {} 条建议", ts.suggestions.len());
        }
    }

    Ok(())
}

/// 拒绝一条建议：只置 ai_confirmed=1，保留 is_ai_sorted=0 以便重试
#[tauri::command]
pub fn ai_reject_suggestion(
    video_id: i64,
    state: State<'_, DbState>,
    task_state: State<'_, AiTaskManager>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE videos SET ai_confirmed=1, is_ai_sorted=0 WHERE id=?1",
        params![video_id],
    )
    .map_err(|e| e.to_string())?;
    drop(conn);

    if let Ok(mut ts) = task_state.0.lock() {
        ts.removed_ids.insert(video_id);
        ts.suggestions.retain(|item| item.video_id != video_id);
        if ts.status == "done" {
            ts.progress = format!("还剩 {} 条建议", ts.suggestions.len());
        }
    }

    Ok(())
}

// ============ 收藏夹 AI 筛选 ============

#[derive(Debug, Serialize)]
pub struct AiFavCandidate {
    pub bvid: String,
    pub title: String,
    pub author: Option<String>,
    pub cover_url: Option<String>,
    pub description: Option<String>,
    pub duration: Option<i64>,
    pub score: f64,
    pub in_library: bool,
}

/// 从 B 站收藏夹中筛选与指定话题相关的内容，返回给用户决定是否入库。
#[tauri::command]
pub async fn ai_filter_favorites(
    media_id: i64,
    topic: String,
    limit: Option<i64>,
    min_score: Option<f64>,
    state: State<'_, DbState>,
    app: tauri::AppHandle,
) -> Result<Vec<AiFavCandidate>, String> {
    let topic = topic.trim();
    if topic.is_empty() {
        return Err("请输入要筛选的话题".into());
    }
    let app_settings = settings::load_settings(&app);
    if app_settings.openai_base_url.is_empty() || app_settings.ai_model.is_empty() {
        return Err("请先在设置页配置 AI Base URL 和模型名".into());
    }

    let limit = limit.unwrap_or(120).clamp(1, 200) as usize;
    let mut favorites = Vec::new();
    let mut page = 1;
    loop {
        let (items, has_more) = crate::bilibili::fetch_fav_videos(
            &app_settings.bilibili_cookie,
            media_id,
            page,
        )
        .await?;
        for item in items {
            if item.attr == 0 {
                favorites.push(item);
                if favorites.len() >= limit {
                    break;
                }
            }
        }
        if favorites.len() >= limit || !has_more {
            break;
        }
        page += 1;
    }

    if favorites.is_empty() {
        return Ok(vec![]);
    }

    let existing: HashSet<String> = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT platform_id FROM videos WHERE platform='bilibili'")
            .map_err(|e| e.to_string())?;
        stmt.query_map([], |r| r.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<HashSet<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    let metas: Vec<FavoriteMeta> = favorites
        .into_iter()
        .map(|item| FavoriteMeta {
            bvid: item.bvid,
            title: item.title,
            author: Some(item.upper.name),
            cover_url: Some(item.cover),
            description: Some(item.intro),
            duration: Some(item.duration),
        })
        .collect();
    filter_favorite_meta(
        &app_settings,
        topic,
        metas,
        &existing,
        min_score.unwrap_or(0.4),
    )
    .await
}

struct FavoriteMeta {
    bvid: String,
    title: String,
    author: Option<String>,
    cover_url: Option<String>,
    description: Option<String>,
    duration: Option<i64>,
}

async fn filter_favorite_meta(
    app_settings: &settings::AppSettings,
    topic: &str,
    metas: Vec<FavoriteMeta>,
    existing: &HashSet<String>,
    min_score: f64,
) -> Result<Vec<AiFavCandidate>, String> {
    if metas.is_empty() {
        return Ok(vec![]);
    }
    let candidates: Vec<AiVideoInput> = metas
        .iter()
        .enumerate()
        .map(|(index, item)| AiVideoInput {
            id: index as i64,
            title: item.title.clone(),
            description: item.description.clone().unwrap_or_default(),
            author: item.author.clone().unwrap_or_default(),
            cover_url: item.cover_url.clone(),
        })
        .collect();
    let reference = AiVideoInput {
        id: -1,
        title: topic.to_string(),
        description: topic.to_string(),
        author: String::new(),
        cover_url: None,
    };
    let scores = ai::similarity_filter(app_settings, &reference, &candidates).await?;
    let score_map: std::collections::HashMap<i64, f64> = scores.into_iter().collect();
    let mut result: Vec<AiFavCandidate> = metas
        .into_iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let score = *score_map.get(&(index as i64))?;
            if score < min_score {
                return None;
            }
            let in_library = existing.contains(&item.bvid);
            Some(AiFavCandidate {
                bvid: item.bvid,
                title: item.title,
                author: item.author,
                cover_url: item.cover_url,
                description: item.description,
                duration: item.duration,
                score,
                in_library,
            })
        })
        .collect();
    result.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(result)
}

/// 从「稍后再看」中筛选与指定话题相关的内容。
#[tauri::command]
pub async fn ai_filter_watch_later(
    topic: String,
    limit: Option<i64>,
    min_score: Option<f64>,
    state: State<'_, DbState>,
    app: tauri::AppHandle,
) -> Result<Vec<AiFavCandidate>, String> {
    let topic = topic.trim();
    if topic.is_empty() {
        return Err("请输入要筛选的话题".into());
    }
    let app_settings = settings::load_settings(&app);
    if app_settings.openai_base_url.is_empty() || app_settings.ai_model.is_empty() {
        return Err("请先在设置页配置 AI Base URL 和模型名".into());
    }
    let limit = limit.unwrap_or(120).clamp(1, 200) as usize;
    let items = crate::bilibili::fetch_watch_later(&app_settings.bilibili_cookie).await?;
    let metas: Vec<FavoriteMeta> = items
        .into_iter()
        .take(limit)
        .map(|item| FavoriteMeta {
            bvid: item.platform_id,
            title: item.title,
            author: item.author,
            cover_url: item.cover_url,
            description: item.description,
            duration: item.duration,
        })
        .collect();
    let existing: HashSet<String> = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT platform_id FROM videos WHERE platform='bilibili'")
            .map_err(|e| e.to_string())?;
        stmt.query_map([], |r| r.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<HashSet<_>, _>>()
            .map_err(|e| e.to_string())?
    };
    filter_favorite_meta(
        &app_settings,
        topic,
        metas,
        &existing,
        min_score.unwrap_or(0.4),
    )
    .await
}

// ============ AI 对话 / 在视频库中查找 ============

#[derive(Debug, Deserialize)]
pub struct ChatTurn {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct AiChatReply {
    pub reply: String,
    pub videos: Vec<VideoRow>,
}

#[derive(Debug, Deserialize)]
struct ChatAnswer {
    #[serde(default)]
    reply: String,
    #[serde(default)]
    video_ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
struct ChatVideoContext {
    id: i64,
    title: String,
    author: String,
    description: String,
    category: String,
    tags: Vec<String>,
}

/// 让 AI 根据自然语言问题在本地视频库中查找内容。
#[tauri::command]
pub async fn ai_chat(
    message: String,
    history: Vec<ChatTurn>,
    state: State<'_, DbState>,
    app: tauri::AppHandle,
) -> Result<AiChatReply, String> {
    let question = message.trim();
    if question.is_empty() {
        return Err("请输入问题".into());
    }

    let app_settings = settings::load_settings(&app);
    if app_settings.openai_base_url.is_empty() || app_settings.ai_model.is_empty() {
        return Err("请先在设置页配置 AI Base URL 和模型名".into());
    }

    let contexts: Vec<ChatVideoContext> = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT v.id, v.title, COALESCE(v.author,''), COALESCE(v.description,''),
                        COALESCE(c.name,'')
                 FROM videos v
                 LEFT JOIN categories c ON c.id = v.category_id
                 ORDER BY v.updated_at DESC
                 LIMIT 120",
            )
            .map_err(|e| e.to_string())?;
        let rows: Vec<(i64, String, String, String, String)> = stmt
            .query_map([], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut contexts = Vec::with_capacity(rows.len());
        for (id, title, author, description, category) in rows {
            let tags = fill_tags(&conn, id)?
                .into_iter()
                .map(|tag| tag.name)
                .collect();
            contexts.push(ChatVideoContext {
                id,
                title,
                author,
                description: description.chars().take(160).collect(),
                category,
                tags,
            });
        }
        contexts
    };

    let library_json = serde_json::to_string(&contexts).map_err(|e| e.to_string())?;
    let system = format!(
        "你是 Collectrix 的视频库助手，负责帮助用户在自己的视频收藏中查找内容。\n\
         只根据下面提供的视频数据回答，不要编造视频或 id。\n\
         如果找到相关视频，把它们的 id 放进 video_ids（最多 8 个）；没有匹配就返回空数组。\n\
         用用户使用的语言回答，回答要简洁。\n\
         严格输出 JSON，不要输出 markdown 代码块或额外说明：\n\
         {{\"reply\":\"给用户的回答\",\"video_ids\":[1,2]}}\n\n\
         视频库（最多 120 条）：\n{library_json}"
    );

    let mut user_prompt = String::new();
    let recent: Vec<&ChatTurn> = history.iter().rev().take(10).rev().collect();
    for turn in recent {
        if turn.role == "user" || turn.role == "assistant" {
            user_prompt.push_str(&format!("{}: {}\n", turn.role, turn.content));
        }
    }
    user_prompt.push_str(&format!("user: {question}"));

    let raw = ai::chat_completion(&app_settings, &system, &user_prompt).await?;
    let parsed = ai::parse_json_object(&raw).and_then(|json| {
        serde_json::from_str::<ChatAnswer>(&json).ok()
    });

    let (mut reply, ids) = match parsed {
        Some(answer) => (answer.reply, answer.video_ids),
        None => (raw.trim().to_string(), Vec::new()),
    };
    if reply.trim().is_empty() {
        reply = if ids.is_empty() {
            "没有找到匹配的视频。".into()
        } else {
            "为你找到以下相关视频：".into()
        };
    }

    let mut seen = HashSet::new();
    let ids: Vec<i64> = ids
        .into_iter()
        .filter(|id| seen.insert(*id))
        .take(8)
        .collect();

    let videos = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let mut videos = Vec::new();
        for id in ids {
            if let Ok(mut video) = conn.query_row(
                &format!("SELECT {SELECT_FIELDS} FROM videos WHERE id=?1"),
                params![id],
                read_video_row,
            ) {
                video.tags = fill_tags(&conn, id).unwrap_or_default();
                videos.push(video);
            }
        }
        videos
    };

    Ok(AiChatReply { reply, videos })
}

// ============ 相似度筛选 ============

/// 以 reference_video_id 为基准，返回相似度 ≥ threshold 的视频
#[tauri::command]
pub async fn ai_filter_similar(
    reference_video_id: i64,
    threshold: f64,
    state: State<'_, DbState>,
    app: tauri::AppHandle,
) -> Result<Vec<(i64, f64)>, String> {
    let s = settings::load_settings(&app);

    let (reference, candidates) = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;

        let reference: AiVideoInput = conn
            .query_row(
                "SELECT id, title, COALESCE(description,''), COALESCE(author,''), cover_url
                 FROM videos WHERE id=?1",
                params![reference_video_id],
                |r| {
                    Ok(AiVideoInput {
                        id: r.get(0)?,
                        title: r.get(1)?,
                        description: r.get(2)?,
                        author: r.get(3)?,
                        cover_url: r.get(4)?,
                    })
                },
            )
            .map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, title, COALESCE(description,''), COALESCE(author,''), cover_url
                 FROM videos WHERE id != ?1 LIMIT 50",
            )
            .map_err(|e| e.to_string())?;

        let candidates: Vec<AiVideoInput> = stmt
            .query_map(params![reference_video_id], |r| {
                Ok(AiVideoInput {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    description: r.get(2)?,
                    author: r.get(3)?,
                    cover_url: r.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        (reference, candidates)
    };

    let scores = ai::similarity_filter(&s, &reference, &candidates).await?;
    Ok(scores
        .into_iter()
        .filter(|(_, score)| *score >= threshold)
        .collect())
}

// ============ 遗留命令：ai_categorize_pending ============

/// 一次性处理「未分类且未 AI 处理」的视频，直接写库（无预览、无父级支持）。
/// 如果前端已经不再调用，可以从 main.rs 里移除注册并删除本函数。
#[tauri::command]
pub async fn ai_categorize_pending(
    state: State<'_, DbState>,
    app: tauri::AppHandle,
) -> Result<Vec<(i64, String, Vec<String>, String)>, String> {
    let s = settings::load_settings(&app);

    let (inputs, existing_categories, existing_tags) = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, title, COALESCE(description,''), COALESCE(author,''), cover_url
                 FROM videos
                 WHERE is_ai_sorted = 0 AND category_id IS NULL
                 LIMIT 30",
            )
            .map_err(|e| e.to_string())?;
        let inputs: Vec<AiVideoInput> = stmt
            .query_map([], |r| {
                Ok(AiVideoInput {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    description: r.get(2)?,
                    author: r.get(3)?,
                    cover_url: r.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let cats: Vec<String> = conn
            .prepare("SELECT name FROM categories")
            .and_then(|mut s| s.query_map([], |r| r.get(0))?.collect::<Result<_, _>>())
            .unwrap_or_default();

        let tags: Vec<String> = conn
            .prepare("SELECT name FROM tags")
            .and_then(|mut s| s.query_map([], |r| r.get(0))?.collect::<Result<_, _>>())
            .unwrap_or_default();

        (inputs, cats, tags)
    };

    if inputs.is_empty() {
        return Ok(vec![]);
    }

    let suggestions =
        ai::categorize_batch(&s, &existing_categories, &existing_tags, &inputs).await?;

    {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        for (id, sug) in &suggestions {
            // 分类：只按顶级查/建（此命令无父级信息）
            let cat_id: Option<i64> = conn
                .query_row(
                    "SELECT id FROM categories WHERE name=?1 AND parent_id IS NULL",
                    params![sug.category],
                    |r| r.get(0),
                )
                .ok();
            let cat_id = match cat_id {
                Some(id) => id,
                None => {
                    conn.execute(
                        "INSERT INTO categories (name, parent_id) VALUES (?1, NULL)",
                        params![sug.category],
                    )
                    .map_err(|e| e.to_string())?;
                    conn.last_insert_rowid()
                }
            };

            conn.execute(
                "UPDATE videos SET category_id=?1, is_ai_sorted=1, ai_suggestion=?2 WHERE id=?3",
                params![cat_id, serde_json::to_string(sug).unwrap_or_default(), id],
            )
            .map_err(|e| e.to_string())?;

            for tag in &sug.tags {
                let tag_name = tag.trim();
                if tag_name.is_empty() {
                    continue;
                }
                conn.execute(
                    "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
                    params![tag_name],
                )
                .map_err(|e| e.to_string())?;
                let tag_id: i64 = conn
                    .query_row(
                        "SELECT id FROM tags WHERE name=?1",
                        params![tag_name],
                        |r| r.get(0),
                    )
                    .map_err(|e| e.to_string())?;
                conn.execute(
                    "INSERT OR IGNORE INTO video_tags (video_id, tag_id) VALUES (?1, ?2)",
                    params![id, tag_id],
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(suggestions
        .into_iter()
        .map(|(id, s)| (id, s.category, s.tags, s.summary))
        .collect())
}
