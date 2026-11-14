use crate::settings::{self, AppSettings};
use sha2::{Digest, Sha256};
use tauri::Manager;

fn hash_lock_pin(pin: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"collectrix-app-lock:");
    hasher.update(pin.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[tauri::command]
pub fn set_app_lock(
    enabled: bool,
    pin: Option<String>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut s = settings::load_settings(&app);
    if enabled {
        if let Some(pin) = pin.filter(|value| !value.trim().is_empty()) {
            let pin = pin.trim();
            if !(4..=12).contains(&pin.len()) || !pin.chars().all(|c| c.is_ascii_digit()) {
                return Err("数字密钥需为 4-12 位数字".into());
            }
            s.lock_pin_hash = hash_lock_pin(pin);
        } else if s.lock_pin_hash.is_empty() {
            return Err("请设置数字密钥".into());
        }
        s.lock_enabled = true;
    } else {
        s.lock_enabled = false;
        s.lock_pin_hash.clear();
    }
    settings::save_settings(&app, &s)
}

#[tauri::command]
pub fn verify_app_lock(pin: String, app: tauri::AppHandle) -> Result<bool, String> {
    let s = settings::load_settings(&app);
    if !s.lock_enabled {
        return Ok(true);
    }
    if s.lock_pin_hash.is_empty() {
        return Ok(false);
    }
    Ok(hash_lock_pin(pin.trim()) == s.lock_pin_hash)
}

#[tauri::command]
pub fn get_settings(app: tauri::AppHandle) -> AppSettings {
    settings::load_settings(&app)
}

#[tauri::command]
pub fn set_settings(settings: AppSettings, app: tauri::AppHandle) -> Result<(), String> {
    settings::save_settings(&app, &settings)
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
pub async fn pick_directory(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog().file().pick_folder(move |path| {
        let _ = tx.send(path);
    });
    let path = rx.recv().map_err(|e| e.to_string())?;
    Ok(path.map(|p| p.to_string()))
}

#[cfg(target_os = "android")]
#[tauri::command]
pub async fn pick_directory(_app: tauri::AppHandle) -> Result<Option<String>, String> {
    Err("Android 暂不支持选择目录，请使用系统默认目录".into())
}

#[tauri::command]
pub async fn pick_whisper_model(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter("Whisper GGML", &["bin"])
        .pick_file(move |path| {
            let _ = tx.send(path);
        });
    let path = rx.recv().map_err(|e| e.to_string())?;
    Ok(path.map(|p| p.to_string()))
}

#[tauri::command]
pub fn set_theme(theme: String, app: tauri::AppHandle) -> Result<(), String> {
    if !matches!(
        theme.as_str(),
        "light"
            | "dark"
            | "system"
            | "light-blue"
            | "light-green"
            | "light-red"
            | "light-purple"
            | "light-amber"
            | "dark-blue"
            | "dark-green"
            | "dark-red"
            | "dark-purple"
    ) {
        return Err(format!("非法主题: {theme}"));
    }
    let mut settings = settings::load_settings(&app);
    settings.theme = theme;
    settings::save_settings(&app, &settings)
}

#[tauri::command]
pub async fn pick_import_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter("Collectrix JSON", &["json"])
        .pick_file(move |path| {
            let _ = tx.send(path);
        });
    let path = rx.recv().map_err(|e| e.to_string())?;
    Ok(path.map(|p| p.to_string()))
}

#[tauri::command]
pub async fn pick_executable(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter("可执行文件", &["exe", ""])
        .pick_file(move |path| {
            let _ = tx.send(path);
        });
    let path = rx.recv().map_err(|e| e.to_string())?;
    Ok(path.map(|p| p.to_string()))
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
pub fn open_data_dir(app: tauri::AppHandle) -> Result<(), String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    #[cfg(target_os = "windows")]
    let opener = "explorer";
    #[cfg(target_os = "macos")]
    let opener = "open";
    #[cfg(target_os = "linux")]
    let opener = "xdg-open";
    std::process::Command::new(opener)
        .arg(dir)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(target_os = "android")]
#[tauri::command]
pub fn open_data_dir(_app: tauri::AppHandle) -> Result<(), String> {
    Err("Android 暂不支持打开数据目录".into())
}
