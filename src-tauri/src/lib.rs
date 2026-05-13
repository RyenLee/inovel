pub mod commands;
pub mod config;
pub mod config_manager {
    pub mod api;
    pub mod encryption;
    pub mod history;
    pub mod loader;
    pub mod model;

    pub use api::*;
    pub use model::*;
}
pub mod db;
pub mod error;
pub mod logging;
pub mod models;
pub mod optimization;
pub mod services;
pub mod settings;
pub mod state;
pub mod utils;

use tauri::Manager;
use tracing::info;

#[tauri::command(rename_all = "snake_case")]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn get_startup_window_size(default_config: &settings::WindowConfig) -> (f64, f64) {
    let db_path = config::get_data_dir_for_webview()
        .parent()
        .map(|p| p.join("inovel.db"))
        .unwrap_or_else(|| std::path::PathBuf::from("inovel.db"));

    if !db_path.exists() {
        return (default_config.default_width, default_config.default_height);
    }

    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(_) => return (default_config.default_width, default_config.default_height),
    };

    let result: Result<(String, i64), _> = conn.query_row(
        "SELECT path, id FROM projects ORDER BY last_opened_at DESC LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );

    let (project_path, _project_id) = match result {
        Ok(r) => r,
        Err(_) => {
            let fallback: Result<(String, i64), _> = conn.query_row(
                "SELECT path, id FROM projects ORDER BY created_at DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            );
            match fallback {
                Ok(r) => r,
                Err(_) => return (default_config.default_width, default_config.default_height),
            }
        }
    };

    let project_json_path = std::path::Path::new(&project_path).join("project.json");
    if !project_json_path.exists() {
        return (default_config.default_width, default_config.default_height);
    }

    let content = match std::fs::read_to_string(&project_json_path) {
        Ok(c) => c,
        Err(_) => return (default_config.default_width, default_config.default_height),
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(j) => j,
        Err(_) => return (default_config.default_width, default_config.default_height),
    };

    if let (Some(width), Some(height)) = (
        json.get("window_width").and_then(|v| v.as_f64()),
        json.get("window_height").and_then(|v| v.as_f64()),
    ) {
        info!("从 project.json 读取窗口大小: {}x{}", width, height);
        return (width, height);
    }

    (default_config.default_width, default_config.default_height)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::WebviewUrl;
    use tauri::WebviewWindowBuilder;
    use tauri_plugin_prevent_default::Flags;

    let webview_data_dir = config::get_data_dir_for_webview();
    std::fs::create_dir_all(&webview_data_dir).expect("failed to create webview data directory");

    let (shared_config, _config_watcher) = settings::init_config();
    let optimization = optimization::OptimizationEngine::new(&shared_config);
    let app_state = state::AppState::new(shared_config.clone(), optimization);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(
            tauri_plugin_prevent_default::Builder::new()
                .with_flags(Flags::keyboard() | Flags::RELOAD | Flags::DEV_TOOLS)
                .build(),
        )
        .manage(app_state)
        .manage(shared_config.clone())
        .setup(move |app| {
            app.state::<state::AppState>()
                .set_app_handle(app.handle().clone());

            logging::init_logging_with_app(app.handle())?;

            info!("application startup...");

            let window_config = {
                let config_guard = shared_config.read().unwrap();
                let window_cfg = &config_guard.window;
                settings::WindowConfig {
                    default_width: window_cfg.default_width,
                    default_height: window_cfg.default_height,
                    min_width: window_cfg.min_width,
                    min_height: window_cfg.min_height,
                    max_width: window_cfg.max_width,
                    max_height: window_cfg.max_height,
                    resizable: window_cfg.resizable,
                    portrait: window_cfg.portrait.clone(),
                }
            };

            let (final_width, final_height) = get_startup_window_size(&window_config);

            let mut builder =
                WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                    .title("iNovel")
                    .inner_size(final_width, final_height)
                    .min_inner_size(window_config.min_width, window_config.min_height)
                    .max_inner_size(window_config.max_width, window_config.max_height)
                    .data_directory(webview_data_dir.clone());

            if !window_config.resizable {
                builder = builder.resizable(false);
            }

            let _window = builder.build()?;

            info!("application startup completed");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::config::get_config,
            commands::config::get_config_value,
            commands::config::get_config_by_category,
            commands::config::set_config_value,
            commands::config::set_config_values,
            commands::config::update_app_version,
            commands::config::reload_config,
            commands::config::export_config,
            commands::config::import_config,
            commands::config::reset_config,
            commands::config::get_config_history,
            commands::config::rollback_config,
            commands::config::read_toml_config,
            commands::config::write_toml_config,
            commands::config::reset_to_default_config,
            commands::project::create_project,
            commands::project::get_recent_projects,
            commands::project::open_project,
            commands::project::remove_project_from_list,
            commands::project::update_project,
            commands::project::set_cover,
            commands::project::migrate_existing_projects,
            commands::project::check_migration_needed,
            commands::project::rollback_migration,
            commands::project::save_window_size,
            commands::project::set_window_size,
            commands::project::get_window_size,
            commands::chapter::get_chapter_content,
            commands::chapter::save_chapter_content,
            commands::chapter::create_volume,
            commands::chapter::create_chapter,
            commands::chapter::update_volume_name,
            commands::chapter::update_chapter_title,
            commands::chapter::delete_volume,
            commands::chapter::delete_chapter,
            commands::chapter::reorder_volumes,
            commands::chapter::reorder_chapters,
            commands::chapter::move_chapter_to_volume,
            commands::chapter::get_chapter_tree,
            commands::chapter::update_chapter_word_count,
            commands::chapter::update_chapter_summary,
            commands::chapter::update_chapter_status,
            commands::chapter::get_chapter_status_counts,
            commands::chapter::save_image,
            commands::writing::get_writing_goal,
            commands::writing::save_writing_goal,
            commands::writing::get_writing_stats,
            commands::writing::upsert_writing_record,
            commands::writing::get_today_words,
            commands::writing::record_focus_session,
            commands::writing::get_focus_sessions,
            commands::writing::get_focus_stats,
            commands::names::generate_names,
            commands::worldbuilding::create_character,
            commands::worldbuilding::update_character,
            commands::worldbuilding::delete_character,
            commands::worldbuilding::list_characters,
            commands::worldbuilding::create_location,
            commands::worldbuilding::update_location,
            commands::worldbuilding::delete_location,
            commands::worldbuilding::list_locations,
            commands::worldbuilding::create_organization,
            commands::worldbuilding::update_organization,
            commands::worldbuilding::delete_organization,
            commands::worldbuilding::list_organizations,
            commands::relationship::create_relationship,
            commands::relationship::update_relationship,
            commands::relationship::delete_relationship,
            commands::relationship::get_relationships,
            commands::timeline::create_event,
            commands::timeline::update_event,
            commands::timeline::delete_event,
            commands::timeline::list_events,
            commands::git_snapshot::init_project_git,
            commands::git_snapshot::create_snapshot,
            commands::git_snapshot::get_snapshots,
            commands::git_snapshot::restore_snapshot,
            commands::git_snapshot::get_snapshot_diff,
            commands::sensitive::add_sensitive_word,
            commands::sensitive::remove_sensitive_word,
            commands::sensitive::list_sensitive_words,
            commands::sensitive::import_sensitive_words,
            commands::sensitive::scan_sensitive_words,
            commands::export::export_txt,
            commands::export::export_markdown,
            commands::export::export_epub,
            commands::export::get_export_content,
            commands::export::export_html_for_print,
            commands::export::get_exports_dir,
            commands::export::open_folder_in_explorer,
            commands::backup::backup_project,
            commands::backup::create_incremental_backup,
            commands::backup::list_backups,
            commands::backup::restore_backup,
            commands::backup::delete_backup_record,
            commands::backup::get_backup_logs,
            commands::backup::get_backup_stats,
            commands::encryption::encrypt_project,
            commands::encryption::decrypt_project,
            commands::encryption::verify_project_password,
            commands::encryption::change_project_password,
            commands::encryption::reencrypt_project,
            commands::encryption::is_project_encrypted_command,
            commands::encryption::get_global_encryption_status,
            commands::encryption::enable_global_encryption,
            commands::encryption::disable_global_encryption,
            commands::encryption::change_global_password,
            commands::encryption::verify_global_password,
            commands::encryption::get_language_list,
            commands::encryption::get_locale,
            commands::encryption::set_locale,
            commands::inspiration::create_inspiration_item,
            commands::inspiration::update_inspiration_item,
            commands::inspiration::delete_inspiration_item,
            commands::inspiration::reorder_inspiration_items,
            commands::inspiration::get_inspiration_board,
            commands::inspiration::get_inspiration_items,
            commands::template::get_builtin_templates,
            commands::template::get_user_templates,
            commands::template::save_user_template,
            commands::template::update_user_template,
            commands::template::delete_user_template,
            commands::template::get_all_templates,
            commands::file::read_text_file,
            commands::file::check_file_exists,
            commands::file::get_file_size,
            commands::optimization::get_app_config,
            commands::optimization::update_app_config,
            commands::optimization::reset_app_config,
            commands::optimization::get_cache_stats,
            commands::optimization::clear_cache,
            commands::optimization::get_performance_report,
            commands::optimization::clear_performance_metrics,
            commands::optimization::test_gzip_compression,
            logging::commands::record_operation_log,
            logging::commands::query_operation_logs,
            logging::commands::get_operation_stats,
            logging::commands::get_error_logs,
            logging::commands::clear_error_logs,
            logging::commands::get_enum_dictionary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
