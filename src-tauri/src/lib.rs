mod ai;
mod ai_task;
mod bilibili;
mod commands;
mod comment;
mod db;
mod download;
mod models;
mod providers;
mod settings;
mod state;
mod transcribe;
mod web;

use state::DbState;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let conn = db::init_db(app.handle());
            app.manage(DbState(Mutex::new(conn)));
            app.manage(ai_task::AiTaskManager(std::sync::Mutex::new(
                ai_task::AiTaskState::idle(),
            )));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // category
            commands::category::get_category_tree,
            commands::category::add_category,
            commands::category::delete_category,
            commands::category::move_category,
            commands::category::delete_category_tree,
            commands::category::list_videos_in_category_tree,
            // video
            commands::video::list_videos,
            commands::video::list_videos_by_tag,
            commands::video::get_video,
            commands::video::save_video,
            commands::video::update_video,
            commands::video::move_video_to_category,
            commands::video::open_video_window,
            commands::video::delete_video,
            commands::video::refresh_video_info,
            commands::video::list_videos_with_children,
            // tag
            commands::tag::list_tags,
            commands::tag::create_tag,
            commands::tag::update_tag,
            commands::tag::delete_tag,
            commands::tag::list_videos_by_tags,
            commands::tag::merge_tags,
            // stats
            commands::stats::get_stats,
            commands::stats::get_detailed_stats,
            // comment
            commands::comment::get_comments,
            commands::comment::get_sub_replies,
            // download
            commands::download::download_video,
            commands::download::download_audio_only,
            // transcribe
            commands::transcribe::transcribe_video,
            commands::transcribe::polish_transcript,
            commands::transcribe::get_transcript,
            // settings
            commands::settings::get_settings,
            commands::settings::set_settings,
            commands::settings::set_theme,
            commands::settings::pick_directory,
            commands::settings::pick_executable,
            commands::settings::pick_whisper_model,
            commands::settings::open_data_dir,
            commands::settings::set_app_lock,
            commands::settings::verify_app_lock,
            commands::settings::pick_import_file,
            commands::update::check_for_updates,
            // data transfer
            commands::transfer::export_data,
            commands::transfer::import_data,
            // platform (B站)
            commands::platform::check_bili_login,
            commands::platform::generate_bili_qr,
            commands::platform::poll_bili_qr,
            commands::platform::fetch_bili_collection,
            commands::platform::list_bili_watch_later,
            commands::platform::fetch_bili_info,
            commands::platform::fetch_video_meta,
            commands::platform::search_bili_videos,
            commands::platform::fetch_bili_article_content,
            commands::platform::fetch_web_article,
            commands::platform::list_fav_folders,
            commands::platform::import_fav_folder,
            commands::platform::get_danmaku,
            commands::platform::get_subtitle_list,
            commands::platform::get_subtitle_content,
            // ai
            commands::ai::ai_categorize_pending,
            commands::ai::ai_filter_similar,
            commands::ai::ai_chat,
            commands::ai::ai_filter_favorites,
            commands::ai::ai_filter_watch_later,
            commands::ai::ping_ai_provider,
            commands::ai::launch_ollama,
            commands::ai::ai_organize_preview,
            commands::ai::ai_apply_suggestion,
            commands::ai::ai_reject_suggestion,
            commands::ai::list_ai_models,
            // ai_task
            ai_task::start_ai_organize,
            ai_task::get_ai_task_state,
            ai_task::cancel_ai_task,
            // local import
            commands::local_import::scan_local_videos,
            commands::local_import::import_local_videos,
            commands::local_import::get_local_subtitle,
            // notebook
            commands::notebook::list_notebooks,
            commands::notebook::get_notebook,
            commands::notebook::save_notebook,
            commands::notebook::delete_notebook,
            commands::notebook::add_video_to_notebook,
            commands::notebook::remove_video_from_notebook,
            commands::notebook::list_notebooks_for_video,
            // topic
            commands::topic::list_topics,
            commands::topic::create_topic,
            commands::topic::update_topic,
            commands::topic::delete_topic,
            commands::topic::set_video_topics,
            commands::topic::list_topics_for_video,
            commands::topic::list_videos_by_topic,
            commands::topic::add_videos_to_topic,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
