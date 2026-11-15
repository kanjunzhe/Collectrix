use crate::ai;
use crate::bilibili;
use crate::commands::local_import::get_local_subtitle;
use crate::state::DbState;
use crate::{settings, transcribe};
use rusqlite::params;
use std::path::{Path, PathBuf};
use tauri::Manager;
use tauri::State;

fn transcript_out_path(
    app: &tauri::AppHandle,
    settings: &settings::AppSettings,
    video_id: i64,
) -> Result<PathBuf, String> {
    let out_dir = if settings.transcript_dir.is_empty() {
        app.path()
            .download_dir()
            .map_err(|e| e.to_string())?
            .join("Collectrix")
            .join("transcripts")
    } else {
        Path::new(&settings.transcript_dir).to_path_buf()
    };
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    Ok(out_dir.join(format!("{video_id}.txt")))
}

fn save_transcript(
    app: &tauri::AppHandle,
    settings: &settings::AppSettings,
    state: &DbState,
    video_id: i64,
    text: &str,
) -> Result<(), String> {
    let out_path = transcript_out_path(app, settings, video_id)?;
    std::fs::write(&out_path, text).map_err(|e| e.to_string())?;
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE videos SET transcript_path=?1, updated_at=datetime('now') WHERE id=?2",
        params![out_path.to_string_lossy().into_owned(), video_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

async fn bilibili_subtitle_text(
    bvid: &str,
    cid: i64,
    cookie: &str,
) -> Result<Option<String>, String> {
    let list = bilibili::fetch_subtitle_list(bvid, cid, cookie).await?;
    if list.is_empty() {
        return Ok(None);
    }
    let subtitle = list
        .iter()
        .find(|item| item.lan.starts_with("zh"))
        .unwrap_or(&list[0]);
    let cues = bilibili::fetch_subtitle_content(&subtitle.url).await?;
    let text = cues
        .into_iter()
        .map(|cue| cue.content.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    Ok(if text.is_empty() { None } else { Some(text) })
}

fn local_subtitle_text(paths_json: Option<&str>) -> Result<Option<String>, String> {
    let Some(paths_json) = paths_json else {
        return Ok(None);
    };
    let paths: Vec<String> = serde_json::from_str(paths_json).unwrap_or_default();
    let mut lines = Vec::new();
    for path in paths {
        if let Ok(cues) = get_local_subtitle(path) {
            for cue in cues {
                let content = cue.content.trim();
                if !content.is_empty() {
                    lines.push(content.to_string());
                }
            }
        }
    }
    Ok(if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    })
}

#[tauri::command]
pub async fn transcribe_video(
    video_id: i64,
    state: State<'_, DbState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let settings = settings::load_settings(&app);

    let (platform, platform_id, audio_path, cid, local_subtitles) = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT platform, platform_id, local_audio_path, cid, local_subtitle_paths
             FROM videos WHERE id=?1",
            params![video_id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, Option<i64>>(3)?,
                    r.get::<_, Option<String>>(4)?,
                ))
            },
        )
        .map_err(|e| e.to_string())?
    };

    let audio_path = audio_path.filter(|path| !path.trim().is_empty());
    let mut text: Option<String> = None;
    let mut last_error: Option<String> = None;

    // 1. 优先使用音频文件：本地 Whisper 或 OpenAI 兼容转录接口。
    //    选择 Bilibili 字幕模式时跳过音频，直接使用字幕。
    if settings.transcribe_provider != "bilibili" {
        if let Some(audio_path) = audio_path {
            let result = if settings.transcribe_provider == "openai" {
                transcribe::transcribe_openai(Path::new(&audio_path), &settings).await
            } else {
                tokio::task::spawn_blocking({
                    let s = settings.clone();
                    let path = audio_path.clone();
                    move || transcribe::transcribe_local(Path::new(&path), &s)
                })
                .await
                .map_err(|e| e.to_string())?
            };
            match result {
                Ok(value) if !value.trim().is_empty() => text = Some(value),
                Ok(_) => last_error = Some("音频转录结果为空".into()),
                Err(error) => last_error = Some(error),
            }
        }
    }

    // 2. 没有音频或音频转录失败时，尝试本地 sidecar 字幕。
    if text.is_none() {
        if let Ok(Some(value)) = local_subtitle_text(local_subtitles.as_deref()) {
            text = Some(value);
        }
    }

    // 3. 再尝试 B 站字幕接口。
    if text.is_none() && platform == "bilibili" {
        if let Some(cid) = cid.filter(|cid| *cid > 0) {
            match bilibili_subtitle_text(&platform_id, cid, &settings.bilibili_cookie).await {
                Ok(Some(value)) => text = Some(value),
                Ok(None) => {}
                Err(error) => {
                    if last_error.is_none() {
                        last_error = Some(error);
                    }
                }
            }
        }
    }

    let mut text = text.ok_or_else(|| {
        last_error.unwrap_or_else(|| {
            "该视频没有可用的音频或字幕。请先下载音频，或在设置中配置在线转录服务。".into()
        })
    })?;

    // 可选：转录完成后直接交给 AI 加标点、纠错和分段。
    // AI 失败时保留原始转录，不让一次校对失败导致整个转录失败。
    if settings.ai_enabled && settings.transcribe_auto_polish {
        match ai::polish_text(&settings, &text).await {
            Ok(polished) if !polished.trim().is_empty() => text = polished,
            Ok(_) => {}
            Err(error) => eprintln!("[transcribe] AI polish skipped: {error}"),
        }
    }

    save_transcript(&app, &settings, &state, video_id, &text)?;
    Ok(text)
}

#[tauri::command]
pub fn get_transcript(
    video_id: i64,
    state: State<'_, DbState>,
) -> Result<Option<String>, String> {
    let path: Option<String> = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT transcript_path FROM videos WHERE id=?1",
            params![video_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?
    };
    let Some(path) = path.filter(|path| !path.is_empty()) else {
        return Ok(None);
    };
    Ok(std::fs::read_to_string(path).ok())
}

/// 对转录稿做 AI 加标点、纠错和分段，并写回文字稿文件。
#[tauri::command]
pub async fn polish_transcript(
    video_id: i64,
    text: String,
    state: State<'_, DbState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let settings = settings::load_settings(&app);
    if !settings.ai_enabled {
        return Err("请先在设置中开启 AI".into());
    }
    if text.trim().is_empty() {
        return Err("没有可校对的文字".into());
    }
    let polished = ai::polish_text(&settings, &text).await?;
    if polished.trim().is_empty() {
        return Err("AI 返回了空文本".into());
    }
    save_transcript(&app, &settings, &state, video_id, &polished)?;
    Ok(polished)
}
