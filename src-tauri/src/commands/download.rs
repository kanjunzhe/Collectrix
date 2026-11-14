// src-tauri/src/commands/download.rs
use crate::download::{self, DownloadResult};
use crate::settings;
use crate::state::DbState;
use rusqlite::params;
use std::path::Path;
use tauri::{Manager, State};

fn default_out_dir(app: &tauri::AppHandle, configured: &str) -> Result<std::path::PathBuf, String> {
    if !configured.is_empty() {
        return Ok(Path::new(configured).to_path_buf());
    }
    Ok(app
        .path()
        .download_dir()
        .map_err(|e| e.to_string())?
        .join("Collectrix")
        .join("videos"))
}

#[tauri::command]
pub async fn download_video(
    video_id: i64,
    state: State<'_, DbState>,
    app: tauri::AppHandle,
) -> Result<DownloadResult, String> {
    let s = settings::load_settings(&app);
    let out_dir = default_out_dir(&app, &s.video_dir)?;

    let (bvid, cid, cover_url): (String, i64, Option<String>) = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT platform_id, COALESCE(cid, 0), cover_url FROM videos WHERE id=?1",
            params![video_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map_err(|e| e.to_string())?
    };

    if cid == 0 {
        return Err("该视频缺少 cid，无法下载。请先刷新视频信息。".into());
    }

    let result = download::download_media(
        &bvid,
        cid,
        cover_url.as_deref(),
        &out_dir,
        &app,
        false,
    )
    .await?;

    {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        if let Some(path) = &result.video_path {
            conn.execute(
                "UPDATE videos SET local_video_path=?1, updated_at=datetime('now') WHERE id=?2",
                params![path, video_id],
            )
            .map_err(|e| e.to_string())?;
        }
        if let Some(path) = &result.audio_path {
            conn.execute(
                "UPDATE videos SET local_audio_path=?1, updated_at=datetime('now') WHERE id=?2",
                params![path, video_id],
            )
            .map_err(|e| e.to_string())?;
        }
        if let Some(path) = &result.cover_path {
            conn.execute(
                "UPDATE videos SET local_cover_path=?1, updated_at=datetime('now') WHERE id=?2",
                params![path, video_id],
            )
            .map_err(|e| e.to_string())?;
        }
        if !result.subtitle_paths.is_empty() {
            let subtitle_json = serde_json::to_string(&result.subtitle_paths)
                .map_err(|e| e.to_string())?;
            conn.execute(
                "UPDATE videos SET local_subtitle_paths=?1, updated_at=datetime('now') WHERE id=?2",
                params![subtitle_json, video_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(result)
}

#[tauri::command]
pub async fn download_audio_only(
    video_id: i64,
    state: State<'_, DbState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let s = settings::load_settings(&app);
    let out_dir = if s.audio_dir.is_empty() {
        default_out_dir(&app, &s.video_dir)?
    } else {
        Path::new(&s.audio_dir).to_path_buf()
    };

    let (bvid, cid, cover_url): (String, i64, Option<String>) = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT platform_id, COALESCE(cid, 0), cover_url FROM videos WHERE id=?1",
            params![video_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map_err(|e| e.to_string())?
    };

    if cid == 0 {
        return Err("该视频缺少 cid，无法下载。请先刷新视频信息。".into());
    }

    let result = download::download_media(
        &bvid,
        cid,
        cover_url.as_deref(),
        &out_dir,
        &app,
        true,
    )
    .await?;

    let path = result
        .audio_path
        .ok_or_else(|| "未生成音频文件".to_string())?;
    {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE videos SET local_audio_path=?1, updated_at=datetime('now') WHERE id=?2",
            params![path, video_id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(path)
}
