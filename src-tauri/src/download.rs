use crate::settings::{self, AppSettings};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use tauri::Manager;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Serialize)]
pub struct DownloadResult {
    pub video_path: Option<String>,
    pub audio_path: Option<String>,
    pub subtitle_paths: Vec<String>,
    pub danmaku_path: Option<String>,
    pub cover_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PlayUrlResponse {
    code: i64,
    message: String,
    data: Option<PlayUrlData>,
}

#[derive(Debug, Deserialize)]
struct PlayUrlData {
    dash: Option<Dash>,
}

#[derive(Debug, Deserialize)]
struct Dash {
    video: Vec<DashTrack>,
    audio: Vec<DashTrack>,
}

#[derive(Debug, Deserialize)]
struct DashTrack {
    id: i64,
    base_url: String,
    bandwidth: Option<i64>,
    #[serde(default)]
    codecs: Option<String>,
}

/// 下载视频/音频/字幕/弹幕/封面，具体项目由 settings 决定。
pub async fn download_media(
    bvid: &str,
    cid: i64,
    cover_url: Option<&str>,
    out_dir: &Path,
    app: &tauri::AppHandle,
    audio_only: bool,
) -> Result<DownloadResult, String> {
    let mut settings = settings::load_settings(app);
    if audio_only {
        settings.download_video = false;
        settings.download_audio = true;
        settings.download_subtitles = false;
        settings.download_danmaku = false;
        settings.download_cover = false;
    }

    if !settings.download_video
        && !settings.download_audio
        && !settings.download_subtitles
        && !settings.download_danmaku
        && !settings.download_cover
    {
        return Err("未选择任何下载内容".into());
    }

    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;

    let client = Client::builder()
        .user_agent("Mozilla/5.0")
        .build()
        .map_err(|e| e.to_string())?;

    let mut result = DownloadResult {
        video_path: None,
        audio_path: None,
        subtitle_paths: Vec::new(),
        danmaku_path: None,
        cover_path: None,
    };

    if settings.download_video || settings.download_audio {
        let dash = fetch_dash(&client, bvid, cid).await?;
        let ffmpeg = resolve_ffmpeg(app, &settings);

        let tmp_video = out_dir.join(format!("{bvid}_video.m4s"));
        let tmp_audio = out_dir.join(format!("{bvid}_audio.m4s"));

        if settings.download_video {
            let track = pick_video_track(&dash.video, &settings.download_video_quality)
                .ok_or_else(|| "没有可用的视频流".to_string())?;
            download_to_file(&client, &track.base_url, &tmp_video).await?;
        }

        if settings.download_audio {
            let track = pick_audio_track(&dash.audio, &settings.download_audio_quality)
                .ok_or_else(|| "没有可用的音频流".to_string())?;
            download_to_file(&client, &track.base_url, &tmp_audio).await?;
        }

        if settings.download_video && settings.download_audio {
            let video_out = out_dir.join(format!("{bvid}.mp4"));
            run_ffmpeg(
                &ffmpeg,
                &[
                    "-y",
                    "-i",
                    tmp_video.to_str().unwrap(),
                    "-i",
                    tmp_audio.to_str().unwrap(),
                    "-c",
                    "copy",
                    "-map",
                    "0:v:0",
                    "-map",
                    "1:a:0",
                    video_out.to_str().unwrap(),
                ],
            )?;
            result.video_path = Some(video_out.to_string_lossy().into_owned());

            let audio_out = out_dir.join(format!("{bvid}.m4a"));
            run_ffmpeg(
                &ffmpeg,
                &[
                    "-y",
                    "-i",
                    tmp_audio.to_str().unwrap(),
                    "-c",
                    "copy",
                    audio_out.to_str().unwrap(),
                ],
            )?;
            result.audio_path = Some(audio_out.to_string_lossy().into_owned());
        } else if settings.download_video {
            let video_out = out_dir.join(format!("{bvid}.mp4"));
            run_ffmpeg(
                &ffmpeg,
                &[
                    "-y",
                    "-i",
                    tmp_video.to_str().unwrap(),
                    "-c",
                    "copy",
                    video_out.to_str().unwrap(),
                ],
            )?;
            result.video_path = Some(video_out.to_string_lossy().into_owned());
        } else if settings.download_audio {
            let audio_out = out_dir.join(format!("{bvid}.m4a"));
            run_ffmpeg(
                &ffmpeg,
                &[
                    "-y",
                    "-i",
                    tmp_audio.to_str().unwrap(),
                    "-c",
                    "copy",
                    audio_out.to_str().unwrap(),
                ],
            )?;
            result.audio_path = Some(audio_out.to_string_lossy().into_owned());
        }

        let _ = std::fs::remove_file(&tmp_video);
        let _ = std::fs::remove_file(&tmp_audio);
    }

    if settings.download_subtitles {
        let subtitles = crate::bilibili::fetch_subtitle_list(
            bvid,
            cid,
            &settings.bilibili_cookie,
        )
        .await?;
        for subtitle in subtitles {
            let cues = crate::bilibili::fetch_subtitle_content(&subtitle.url).await?;
            let filename = format!("{bvid}_{}.srt", safe_filename(&subtitle.lan));
            let path = out_dir.join(filename);
            std::fs::write(&path, cues_to_srt(&cues)).map_err(|e| e.to_string())?;
            result.subtitle_paths.push(path.to_string_lossy().into_owned());
        }
    }

    if settings.download_danmaku {
        let items = crate::bilibili::fetch_danmaku_xml(cid).await?;
        let path = out_dir.join(format!("{bvid}_danmaku.json"));
        let text = serde_json::to_string_pretty(&items).map_err(|e| e.to_string())?;
        std::fs::write(&path, text).map_err(|e| e.to_string())?;
        result.danmaku_path = Some(path.to_string_lossy().into_owned());
    }

    if settings.download_cover {
        if let Some(url) = cover_url.filter(|url| !url.is_empty()) {
            let path = out_dir.join(format!("{bvid}_cover.jpg"));
            download_to_file(&client, url, &path).await?;
            result.cover_path = Some(path.to_string_lossy().into_owned());
        }
    }

    Ok(result)
}

async fn fetch_dash(client: &Client, bvid: &str, cid: i64) -> Result<Dash, String> {
    let mut params: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    params.insert("bvid".into(), bvid.to_string());
    params.insert("cid".into(), cid.to_string());
    params.insert("fnval".into(), "4048".into());
    params.insert("fnver".into(), "0".into());
    params.insert("fourk".into(), "1".into());

    let (img_key, sub_key) = crate::bilibili::fetch_wbi_keys(client).await?;
    let mixin_key = crate::bilibili::get_mixin_key(&img_key, &sub_key);
    crate::bilibili::sign_params(&mut params, &mixin_key);

    let resp: PlayUrlResponse = client
        .get("https://api.bilibili.com/x/player/wbi/playurl")
        .query(&params)
        .header("Referer", "https://www.bilibili.com/")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if resp.code != 0 {
        return Err(format!("playurl 错误: {} - {}", resp.code, resp.message));
    }
    resp.data
        .and_then(|d| d.dash)
        .ok_or_else(|| "未返回 DASH 流（可能需要登录或视频受限）".to_string())
}

fn is_webview_friendly_video(track: &DashTrack) -> bool {
    track
        .codecs
        .as_deref()
        .map(|codecs| {
            let value = codecs.to_ascii_lowercase();
            value.starts_with("avc1") || value.starts_with("avc3")
        })
        .unwrap_or(true)
}

fn pick_video_track<'a>(tracks: &'a [DashTrack], quality: &str) -> Option<&'a DashTrack> {
    let quality_id = match quality {
        "4k" => Some(120),
        "1080p" => Some(80),
        "720p" => Some(64),
        "480p" => Some(32),
        "360p" => Some(16),
        _ => None,
    };

    // WebView2 对 HEVC/AV1 支持不稳定，优先挑选 H.264/AVC 轨。
    let friendly: Vec<&DashTrack> = tracks
        .iter()
        .filter(|track| is_webview_friendly_video(track))
        .collect();
    let pool = if friendly.is_empty() {
        tracks.iter().collect::<Vec<_>>()
    } else {
        friendly
    };

    if let Some(wanted) = quality_id {
        if let Some(track) = pool
            .iter()
            .copied()
            .filter(|track| track.id == wanted)
            .max_by_key(|track| track.bandwidth.unwrap_or(0))
        {
            return Some(track);
        }
    }

    pool.into_iter()
        .max_by_key(|track| (track.id, track.bandwidth.unwrap_or(0)))
}

fn pick_audio_track<'a>(tracks: &'a [DashTrack], quality: &str) -> Option<&'a DashTrack> {
    let mut sorted: Vec<&DashTrack> = tracks.iter().collect();
    sorted.sort_by_key(|track| track.bandwidth.unwrap_or(0));
    match quality {
        "low" => sorted.first().copied(),
        "medium" => sorted.get(sorted.len() / 2).copied(),
        _ => sorted.last().copied(),
    }
}

fn run_ffmpeg(ffmpeg: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(ffmpeg)
        .args(args)
        .status()
        .map_err(|e| {
            format!("启动 ffmpeg 失败（路径: {ffmpeg}）: {e}。请在设置页指定 ffmpeg 路径。")
        })?;
    if !status.success() {
        return Err(format!(
            "ffmpeg 处理失败（退出码 {:?}）。请检查 ffmpeg 是否可执行。",
            status.code()
        ));
    }
    Ok(())
}

fn safe_filename(value: &str) -> String {
    value
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn srt_time(seconds: f64) -> String {
    let total_ms = (seconds.max(0.0) * 1000.0).round() as u64;
    let hours = total_ms / 3_600_000;
    let minutes = (total_ms / 60_000) % 60;
    let secs = (total_ms / 1000) % 60;
    let millis = total_ms % 1000;
    format!("{hours:02}:{minutes:02}:{secs:02},{millis:03}")
}

fn cues_to_srt(cues: &[crate::bilibili::SubtitleCue]) -> String {
    let mut out = String::new();
    for (index, cue) in cues.iter().enumerate() {
        out.push_str(&format!(
            "{}\n{} --> {}\n{}\n\n",
            index + 1,
            srt_time(cue.from),
            srt_time(cue.to),
            cue.content
        ));
    }
    out
}

async fn download_to_file(client: &Client, url: &str, path: &Path) -> Result<(), String> {
    let mut resp = client
        .get(url)
        .header("Referer", "https://www.bilibili.com/")
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("下载失败 HTTP {}: {url}", resp.status()));
    }

    let mut file = tokio::fs::File::create(path)
        .await
        .map_err(|e| e.to_string())?;

    while let Some(chunk) = resp.chunk().await.map_err(|e| e.to_string())? {
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
    }
    file.flush().await.map_err(|e| e.to_string())?;
    Ok(())
}

fn resolve_ffmpeg(app: &tauri::AppHandle, settings: &AppSettings) -> String {
    if !settings.ffmpeg_path.is_empty() {
        let path = Path::new(&settings.ffmpeg_path);
        if path.exists() {
            return settings.ffmpeg_path.clone();
        }
    }

    if let Ok(resource) = app.path().resource_dir() {
        let candidates = if cfg!(windows) {
            vec![resource.join("ffmpeg.exe"), resource.join("ffmpeg")]
        } else {
            vec![resource.join("ffmpeg")]
        };
        for candidate in candidates {
            if candidate.exists() {
                return candidate.to_string_lossy().into_owned();
            }
        }
    }

    "ffmpeg".into()
}
