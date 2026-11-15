use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub theme: String,
    pub language: String,
    pub video_dir: String,
    pub audio_dir: String,
    pub transcript_dir: String,
    pub ffmpeg_path: String,

    // 下载选项
    pub download_video_quality: String,
    pub download_audio_quality: String,
    pub download_video: bool,
    pub download_audio: bool,
    pub download_subtitles: bool,
    pub download_danmaku: bool,
    pub download_cover: bool,

    // 本地转录
    pub whisper_model_path: String,
    pub whisper_language: String,

    // 在线转录
    pub transcribe_provider: String, // "local" | "openai" | "bilibili"
    pub transcribe_auto_polish: bool,
    pub openai_api_key: String,
    pub openai_base_url: String,
    pub openai_transcribe_model: String,

    // AI 分类
    pub ai_enabled: bool,
    pub ai_model: String,

    // B 站登录态
    pub bilibili_cookie: String,

    // UI
    pub recent_days: i64,

    // 应用锁
    pub lock_enabled: bool,
    pub lock_pin_hash: String,

    pub ai_provider: String, // "openai" | "ollama" | "custom"
    pub ollama_path: String, // Ollama 可执行文件路径（可选）
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "light".into(),
            language: "zh".into(),
            video_dir: String::new(),
            audio_dir: String::new(),
            transcript_dir: String::new(),
            ffmpeg_path: String::new(),

            download_video_quality: "best".into(),
            download_audio_quality: "best".into(),
            download_video: true,
            download_audio: true,
            download_subtitles: false,
            download_danmaku: false,
            download_cover: false,

            whisper_model_path: String::new(),
            whisper_language: "auto".into(),

            transcribe_provider: "local".into(),
            transcribe_auto_polish: false,
            openai_api_key: String::new(),
            openai_base_url: "https://api.openai.com/v1".into(),
            openai_transcribe_model: "whisper-1".into(),

            ai_enabled: true,
            ai_model: "gpt-4o-mini".into(),

            bilibili_cookie: String::new(),

            recent_days: 7,

            lock_enabled: false,
            lock_pin_hash: String::new(),

            ai_provider: "openai".into(),
            ollama_path: String::new(),
        }
    }
}

pub fn settings_path(app: &tauri::AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("app data dir")
        .join("settings.json")
}

pub fn load_settings(app: &tauri::AppHandle) -> AppSettings {
    let path = settings_path(app);
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(mut s) = serde_json::from_str::<AppSettings>(&text) {
            // 兜底：旧版本可能缺少新字段，合并默认值
            let def = AppSettings::default();
            if s.language.is_empty() {
                s.language = def.language;
            }
            // 防止损坏/旧版设置导致永久锁死。
            if s.lock_enabled && s.lock_pin_hash.is_empty() {
                s.lock_enabled = false;
            }
            if s.download_video_quality.is_empty() {
                s.download_video_quality = def.download_video_quality;
            }
            if s.download_audio_quality.is_empty() {
                s.download_audio_quality = def.download_audio_quality;
            }
            if s.openai_base_url.is_empty() {
                s.openai_base_url = def.openai_base_url;
            }
            if s.openai_transcribe_model.is_empty() {
                s.openai_transcribe_model = def.openai_transcribe_model;
            }
            if s.ai_model.is_empty() {
                s.ai_model = def.ai_model;
            }
            if s.transcribe_provider.is_empty() {
                s.transcribe_provider = def.transcribe_provider;
            }
            if s.recent_days == 0 {
                s.recent_days = def.recent_days;
            }
            if s.ai_provider.is_empty() {
                s.ai_provider = def.ai_provider;
            }
            return s;
        }
    }
    let defaults = AppSettings::default();
    let _ = save_settings(app, &defaults);
    defaults
}

pub fn save_settings(app: &tauri::AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = settings_path(app);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| e.to_string())
}
