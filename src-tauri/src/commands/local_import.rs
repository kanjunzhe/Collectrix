//! 本地视频扫描与导入。
//!
//! 典型场景：用户之前用别的方式下载了 B 站视频，文件周围还带有封面、音频或字幕。
//! 这里负责识别这些 sidecar 文件，并允许用户把本地文件关联到 B 站元数据。

use crate::commands::video::uncategorized_id;
use crate::providers::VideoMeta;
use crate::state::DbState;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use tauri::State;

const VIDEO_EXTS: &[&str] = &[
    "mp4", "mkv", "webm", "mov", "avi", "flv", "m4v", "wmv", "mpg", "mpeg", "ts",
];
const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "avif", "gif"];
const AUDIO_EXTS: &[&str] = &["m4a", "mp3", "aac", "wav", "flac", "ogg", "opus"];
const SUBTITLE_EXTS: &[&str] = &["srt", "ass", "ssa", "vtt", "sub"];
const TEXT_EXTS: &[&str] = &["txt"];

#[derive(Debug, Serialize, Clone)]
pub struct LocalVideoCandidate {
    pub video_path: String,
    pub file_name: String,
    pub title: String,
    pub folder: String,
    pub cover_path: Option<String>,
    pub audio_path: Option<String>,
    pub subtitle_paths: Vec<String>,
    pub transcript_path: Option<String>,
    pub already_imported: bool,
}

#[derive(Debug, Deserialize)]
pub struct LocalImportItem {
    pub video_path: String,
    pub title: String,
    #[serde(default)]
    pub cover_path: Option<String>,
    #[serde(default)]
    pub audio_path: Option<String>,
    #[serde(default)]
    pub subtitle_paths: Vec<String>,
    #[serde(default)]
    pub transcript_path: Option<String>,
    #[serde(default)]
    pub matched: Option<VideoMeta>,
}

fn ext_lower(path: &Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn has_ext(path: &Path, extensions: &[&str]) -> bool {
    extensions.contains(&ext_lower(path).as_str())
}

fn stem_lower(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>, depth: usize) -> Result<(), String> {
    if depth > 24 || out.len() >= 10_000 {
        return Ok(());
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name == "node_modules" || name == ".git" || name == "@eadir" {
                continue;
            }
            collect_files(&path, out, depth + 1)?;
        } else if file_type.is_file() {
            out.push(path);
        }
        if out.len() >= 10_000 {
            break;
        }
    }
    Ok(())
}

fn choose_cover(files: &[PathBuf], video_stem: &str, single_video: bool) -> Option<String> {
    let candidates: Vec<&PathBuf> = files.iter().filter(|p| has_ext(p, IMAGE_EXTS)).collect();
    if candidates.is_empty() {
        return None;
    }
    if let Some(path) = candidates
        .iter()
        .find(|path| stem_lower(path) == video_stem)
    {
        return Some(path_string(path));
    }
    if !single_video {
        return candidates
            .iter()
            .find(|path| {
                let stem = stem_lower(path);
                stem.starts_with(&format!("{video_stem}_"))
                    || stem.starts_with(&format!("{video_stem}."))
            })
            .map(|path| path_string(path));
    }
    if let Some(path) = candidates.iter().find(|path| {
        let name = stem_lower(path);
        name.contains("cover")
            || name.contains("poster")
            || name.contains("thumb")
            || name.contains("封面")
            || name.contains("缩略")
    }) {
        return Some(path_string(path));
    }
    if candidates.len() == 1 {
        return Some(path_string(candidates[0]));
    }
    None
}

fn choose_audio(files: &[PathBuf], video_stem: &str, single_video: bool) -> Option<String> {
    let candidates: Vec<&PathBuf> = files.iter().filter(|p| has_ext(p, AUDIO_EXTS)).collect();
    if candidates.is_empty() {
        return None;
    }
    if let Some(path) = candidates
        .iter()
        .find(|path| stem_lower(path) == video_stem)
    {
        return Some(path_string(path));
    }
    if !single_video {
        return candidates
            .iter()
            .find(|path| {
                let stem = stem_lower(path);
                stem.starts_with(&format!("{video_stem}_"))
                    || stem.starts_with(&format!("{video_stem}."))
            })
            .map(|path| path_string(path));
    }
    if let Some(path) = candidates.iter().find(|path| {
        let name = stem_lower(path);
        name.contains("audio") || name.contains("音频") || name.contains("声音")
    }) {
        return Some(path_string(path));
    }
    if candidates.len() == 1 {
        return Some(path_string(candidates[0]));
    }
    None
}

fn choose_subtitles(files: &[PathBuf], video_stem: &str, single_video: bool) -> Vec<String> {
    let mut candidates: Vec<&PathBuf> = files
        .iter()
        .filter(|path| has_ext(path, SUBTITLE_EXTS))
        .filter(|path| {
            if single_video {
                true
            } else {
                let stem = stem_lower(path);
                stem == video_stem
                    || stem.starts_with(&format!("{video_stem}."))
                    || stem.starts_with(&format!("{video_stem}_"))
            }
        })
        .collect();
    candidates.sort_by_key(|path| path_string(path));
    candidates.into_iter().map(|path| path_string(path)).collect()
}

fn choose_transcript(files: &[PathBuf], video_stem: &str, single_video: bool) -> Option<String> {
    let candidates: Vec<&PathBuf> = files.iter().filter(|p| has_ext(p, TEXT_EXTS)).collect();
    if candidates.is_empty() {
        return None;
    }
    if let Some(path) = candidates
        .iter()
        .find(|path| stem_lower(path) == video_stem)
    {
        return Some(path_string(path));
    }
    if !single_video {
        return candidates
            .iter()
            .find(|path| {
                let stem = stem_lower(path);
                stem.starts_with(&format!("{video_stem}_"))
                    || stem.starts_with(&format!("{video_stem}."))
            })
            .map(|path| path_string(path));
    }
    if let Some(path) = candidates.iter().find(|path| {
        let name = stem_lower(path);
        name.contains("transcript") || name.contains("subtitle") || name.contains("字幕")
    }) {
        return Some(path_string(path));
    }
    if candidates.len() == 1 {
        return Some(path_string(candidates[0]));
    }
    None
}

#[tauri::command]
pub fn scan_local_videos(
    dir: String,
    state: State<'_, DbState>,
) -> Result<Vec<LocalVideoCandidate>, String> {
    let root = Path::new(&dir);
    if !root.is_dir() {
        return Err(format!("目录不存在：{dir}"));
    }

    let mut files = Vec::new();
    collect_files(root, &mut files, 0)?;
    let mut by_folder: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
    for file in files {
        if let Some(parent) = file.parent() {
            by_folder.entry(parent.to_path_buf()).or_default().push(file);
        }
    }

    let existing: HashSet<String> = {
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT local_video_path FROM videos WHERE local_video_path IS NOT NULL")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<HashSet<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    let mut candidates = Vec::new();
    for (folder, folder_files) in by_folder {
        let videos: Vec<&PathBuf> = folder_files
            .iter()
            .filter(|path| has_ext(path, VIDEO_EXTS))
            .collect();
        if videos.is_empty() {
            continue;
        }
        let single_video = videos.len() == 1;
        for video in videos {
            let video_stem = stem_lower(video);
            let others: Vec<PathBuf> = folder_files
                .iter()
                .filter(|path| path.as_path() != video.as_path())
                .cloned()
                .collect();
            let title = video
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("未命名视频")
                .trim()
                .to_string();
            let video_path = path_string(video);
            candidates.push(LocalVideoCandidate {
                video_path: video_path.clone(),
                file_name: video
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                title,
                folder: path_string(&folder),
                cover_path: choose_cover(&others, &video_stem, single_video),
                audio_path: choose_audio(&others, &video_stem, single_video),
                subtitle_paths: choose_subtitles(&others, &video_stem, single_video),
                transcript_path: choose_transcript(&others, &video_stem, single_video),
                already_imported: existing.contains(&video_path),
            });
        }
    }
    candidates.sort_by(|a, b| a.video_path.cmp(&b.video_path));
    Ok(candidates)
}

fn ensure_tag(tx: &Connection, name: &str) -> Result<i64, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("标签名为空".into());
    }
    tx.execute("INSERT OR IGNORE INTO tags (name) VALUES (?1)", params![name])
        .map_err(|e| e.to_string())?;
    tx.query_row("SELECT id FROM tags WHERE name=?1", params![name], |row| {
        row.get(0)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_local_videos(
    items: Vec<LocalImportItem>,
    category_id: Option<i64>,
    tags: Vec<String>,
    state: State<'_, DbState>,
) -> Result<usize, String> {
    if items.is_empty() {
        return Err("没有选择要导入的视频".into());
    }
    let mut conn = state.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let category_id = category_id.or_else(|| uncategorized_id(&tx));
    let mut imported = 0;

    for item in items {
        let video_path = item.video_path.trim();
        if video_path.is_empty() {
            continue;
        }
        // 不要使用 canonicalize：Windows 会返回带 `\\?\` 前缀的路径，
        // 该路径既不方便展示，也可能无法被 asset protocol 直接读取。
        let video_file = PathBuf::from(video_path);
        let matched = item.matched.as_ref();
        let platform = matched
            .map(|m| m.platform.clone())
            .unwrap_or_else(|| "local".to_string());
        let platform_id = matched
            .map(|m| m.platform_id.clone())
            .unwrap_or_else(|| path_string(&video_file));
        let title = matched
            .map(|m| m.title.clone())
            .filter(|title| !title.trim().is_empty())
            .unwrap_or_else(|| item.title.trim().to_string());
        if title.is_empty() {
            continue;
        }
        let author = matched.and_then(|m| m.author.clone());
        let cover_url = matched.and_then(|m| m.cover_url.clone());
        let duration = matched.and_then(|m| m.duration);
        let description = matched.and_then(|m| m.description.clone());
        let original_url = matched
            .map(|m| m.original_url.clone())
            .unwrap_or_default();
        let aid = matched.and_then(|m| m.aid);
        let cid = matched.and_then(|m| m.cid);
        let subtitle_json =
            serde_json::to_string(&item.subtitle_paths).unwrap_or_else(|_| "[]".to_string());

        tx.execute(
            "INSERT INTO videos
                (category_id, platform, platform_id, title, author, cover_url, duration,
                 description, original_url, aid, cid, local_video_path, local_audio_path,
                 transcript_path, local_cover_path, local_subtitle_paths)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
             ON CONFLICT(platform, platform_id) DO UPDATE SET
                category_id = excluded.category_id,
                title = excluded.title,
                author = excluded.author,
                cover_url = excluded.cover_url,
                duration = excluded.duration,
                description = excluded.description,
                original_url = excluded.original_url,
                aid = excluded.aid,
                cid = excluded.cid,
                local_video_path = excluded.local_video_path,
                local_audio_path = COALESCE(excluded.local_audio_path, videos.local_audio_path),
                transcript_path = COALESCE(excluded.transcript_path, videos.transcript_path),
                local_cover_path = COALESCE(excluded.local_cover_path, videos.local_cover_path),
                local_subtitle_paths = CASE
                    WHEN excluded.local_subtitle_paths = '[]' THEN videos.local_subtitle_paths
                    ELSE excluded.local_subtitle_paths
                END,
                updated_at = datetime('now')",
            params![
                category_id,
                &platform,
                &platform_id,
                title,
                author,
                cover_url,
                duration,
                description,
                original_url,
                aid,
                cid,
                path_string(&video_file),
                item.audio_path,
                item.transcript_path,
                item.cover_path,
                subtitle_json,
            ],
        )
        .map_err(|e| e.to_string())?;

        let video_id: i64 = tx
            .query_row(
                "SELECT id FROM videos WHERE platform=?1 AND platform_id=?2",
                params![&platform, &platform_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        for tag in &tags {
            if tag.trim().is_empty() {
                continue;
            }
            let tag_id = ensure_tag(&tx, tag)?;
            tx.execute(
                "INSERT OR IGNORE INTO video_tags (video_id, tag_id) VALUES (?1, ?2)",
                params![video_id, tag_id],
            )
            .map_err(|e| e.to_string())?;
        }
        imported += 1;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(imported)
}

/// 读取本地 SRT/VTT/ASS 字幕，转换为播放器使用的字幕 cue。
#[tauri::command]
pub fn get_local_subtitle(path: String) -> Result<Vec<crate::bilibili::SubtitleCue>, String> {
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let ext = Path::new(&path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "ass" | "ssa" => Ok(parse_ass(&text)),
        "vtt" => Ok(parse_srt_like(&text, true)),
        _ => Ok(parse_srt_like(&text, false)),
    }
}

fn parse_timestamp(value: &str) -> Option<f64> {
    let value = value.trim().replace(',', ".");
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() == 3 {
        let hours: f64 = parts[0].parse().ok()?;
        let minutes: f64 = parts[1].parse().ok()?;
        let seconds: f64 = parts[2].parse().ok()?;
        Some(hours * 3600.0 + minutes * 60.0 + seconds)
    } else if parts.len() == 2 {
        let minutes: f64 = parts[0].parse().ok()?;
        let seconds: f64 = parts[1].parse().ok()?;
        Some(minutes * 60.0 + seconds)
    } else {
        value.parse().ok()
    }
}

fn parse_srt_like(text: &str, is_vtt: bool) -> Vec<crate::bilibili::SubtitleCue> {
    let mut cues = Vec::new();
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    for block in normalized.split("\n\n") {
        let lines: Vec<&str> = block.lines().filter(|line| !line.trim().is_empty()).collect();
        if lines.is_empty() || (is_vtt && lines[0].starts_with("WEBVTT")) {
            continue;
        }
        let timing_index = lines.iter().position(|line| line.contains("-->"));
        let Some(index) = timing_index else {
            continue;
        };
        let timing: Vec<&str> = lines[index].split("-->").collect();
        if timing.len() != 2 {
            continue;
        }
        let start = parse_timestamp(timing[0]);
        let end = parse_timestamp(timing[1].split_whitespace().next().unwrap_or(""));
        if let (Some(from), Some(to)) = (start, end) {
            cues.push(crate::bilibili::SubtitleCue {
                from,
                to,
                content: lines[index + 1..].join("\n"),
            });
        }
    }
    cues
}

fn parse_ass(text: &str) -> Vec<crate::bilibili::SubtitleCue> {
    let mut cues = Vec::new();
    for line in text.lines() {
        let Some(payload) = line.strip_prefix("Dialogue:") else {
            continue;
        };
        let parts: Vec<&str> = payload.splitn(10, ',').collect();
        if parts.len() < 10 {
            continue;
        }
        let Some(from) = parse_timestamp(parts[1]) else {
            continue;
        };
        let Some(to) = parse_timestamp(parts[2]) else {
            continue;
        };
        let content = parts[9]
            .replace("\\N", "\n")
            .replace("\\n", "\n")
            .replace("\\h", " ");
        cues.push(crate::bilibili::SubtitleCue { from, to, content });
    }
    cues
}
