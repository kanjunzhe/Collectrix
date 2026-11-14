use crate::ai::{self, AiSuggestion, AiVideoInput};
use crate::settings;
use crate::state::DbState;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State};

/// 每批送给模型的视频数量。小批量可以更快看到结果，也更不容易超时。
const AI_CHUNK_SIZE: usize = 5;

#[derive(Debug, Clone, Serialize)]
pub struct AiTaskState {
    pub status: String,   // "idle" | "running" | "done" | "error"
    pub progress: String, // 实时进度描述
    pub suggestions: Vec<AiSuggestion>,
    pub error: Option<String>,
    pub started_at: Option<i64>,
    /// 已接受/忽略的 id，用于防止流式批次把旧建议重新加回来。
    #[serde(skip)]
    pub removed_ids: HashSet<i64>,
}

pub struct AiTaskManager(pub Mutex<AiTaskState>);

impl AiTaskState {
    pub fn idle() -> Self {
        Self {
            status: "idle".into(),
            progress: String::new(),
            suggestions: vec![],
            error: None,
            started_at: None,
            removed_ids: HashSet::new(),
        }
    }
}

#[tauri::command]
pub async fn start_ai_organize(
    limit: Option<i64>,
    task_state: State<'_, AiTaskManager>,
    app: AppHandle,
) -> Result<(), String> {
    // 防止重复启动
    {
        let ts = task_state.0.lock().map_err(|e| e.to_string())?;
        if ts.status == "running" {
            return Err("已有 AI 分析任务正在运行".into());
        }
    }

    let limit = limit.unwrap_or(15);
    let s = settings::load_settings(&app);
    if s.openai_base_url.is_empty() || s.ai_model.is_empty() {
        return Err("请先在设置页配置 AI Base URL 和模型名".into());
    }

    // 标记为 running
    {
        let mut ts = task_state.0.lock().map_err(|e| e.to_string())?;
        *ts = AiTaskState {
            status: "running".into(),
            progress: "正在读取待处理视频…".into(),
            suggestions: vec![],
            error: None,
            started_at: Some(now_ts()),
            removed_ids: HashSet::new(),
        };
    }

    let app_clone = app.clone();
    drop(task_state); // 释放 AppHandle 上的状态借用，后续在 spawn 里重新获取

    tokio::spawn(async move {
        let result = run_ai_organize(limit, &app_clone).await;

        let task_state = app_clone.state::<AiTaskManager>();
        let mut ts = task_state.0.lock().unwrap();

        match result {
            Ok(()) => {
                ts.status = "done".into();
                ts.progress = format!("分析完成，共 {} 条建议", ts.suggestions.len());
                ts.error = None;
                let _ = app_clone.emit("ai-task-update", ts.clone());
            }
            Err(e) => {
                ts.status = "error".into();
                ts.progress = "分析失败".into();
                ts.error = Some(e.clone());
                let _ = app_clone.emit("ai-task-update", ts.clone());
            }
        }
    });

    Ok(())
}

async fn run_ai_organize(limit: i64, app: &AppHandle) -> Result<(), String> {
    let s = settings::load_settings(app);

    let (inputs, existing_categories, existing_tags) = {
        let db = app.state::<DbState>();
        let conn = db.0.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(&format!(
                "SELECT id, title, COALESCE(description,''), COALESCE(author,''), cover_url
                 FROM videos
                 WHERE is_ai_sorted = 0 OR ai_confirmed = 0
                 ORDER BY updated_at DESC LIMIT {limit}"
            ))
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

        let cats = read_string_list(&conn, "SELECT name FROM categories")?;
        let tags = read_string_list(&conn, "SELECT name FROM tags")?;

        (inputs, cats, tags)
    };

    if inputs.is_empty() {
        return Ok(());
    }

    let info_map: HashMap<i64, (String, Option<String>)> = inputs
        .iter()
        .map(|v| (v.id, (v.title.clone(), v.cover_url.clone())))
        .collect();

    let total = inputs.len();
    let mut done = 0usize;

    // 少量多次：逐批请求并把结果实时推给前端。
    for chunk in inputs.chunks(AI_CHUNK_SIZE) {
        update_task_progress(
            app,
            format!("正在分析 {done}/{total}（本批 {} 个）…", chunk.len()),
        )?;

        let raw =
            ai::categorize_batch(&s, &existing_categories, &existing_tags, chunk).await?;

        // 用户在请求期间可能已经接受/忽略了一部分建议，这些不能再被加回来。
        let confirmed_ids = read_confirmed_ids(app)?;
        let new_items: Vec<AiSuggestion> = raw
            .into_iter()
            .filter(|(id, _)| !confirmed_ids.contains(id))
            .map(|(id, raw)| {
                let (title, cover) = info_map
                    .get(&id)
                    .cloned()
                    .unwrap_or((String::new(), None));
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
            .collect();

        done += chunk.len();

        let task_state = app.state::<AiTaskManager>();
        let mut ts = task_state.0.lock().map_err(|e| e.to_string())?;
        ts.suggestions
            .retain(|item| !confirmed_ids.contains(&item.video_id));
        for item in new_items {
            if ts.removed_ids.contains(&item.video_id) {
                continue;
            }
            if !ts
                .suggestions
                .iter()
                .any(|existing| existing.video_id == item.video_id)
            {
                ts.suggestions.push(item);
            }
        }
        ts.progress = format!(
            "已分析 {done}/{total}，当前共 {} 条建议",
            ts.suggestions.len()
        );
        let snapshot = ts.clone();
        drop(ts);
        let _ = app.emit("ai-task-update", snapshot);
    }

    Ok(())
}

fn update_task_progress(app: &AppHandle, progress: String) -> Result<(), String> {
    let task_state = app.state::<AiTaskManager>();
    let mut ts = task_state.0.lock().map_err(|e| e.to_string())?;
    ts.progress = progress;
    let snapshot = ts.clone();
    drop(ts);
    let _ = app.emit("ai-task-update", snapshot);
    Ok(())
}

fn read_confirmed_ids(app: &AppHandle) -> Result<HashSet<i64>, String> {
    let db = app.state::<DbState>();
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id FROM videos WHERE ai_confirmed = 1")
        .map_err(|e| e.to_string())?;
    let ids = stmt
        .query_map([], |r| r.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<HashSet<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(ids)
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn read_string_list(conn: &rusqlite::Connection, sql: &str) -> Result<Vec<String>, String> {
    conn.prepare(sql)
        .and_then(|mut s| s.query_map([], |r| r.get(0))?.collect::<Result<_, _>>())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_ai_task_state(task_state: State<'_, AiTaskManager>) -> AiTaskState {
    task_state
        .0
        .lock()
        .map(|s| s.clone())
        .unwrap_or_else(|_| AiTaskState::idle())
}

#[tauri::command]
pub fn cancel_ai_task(task_state: State<'_, AiTaskManager>) -> Result<(), String> {
    let mut ts = task_state.0.lock().map_err(|e| e.to_string())?;
    ts.status = "idle".into();
    ts.progress = String::new();
    Ok(())
}
