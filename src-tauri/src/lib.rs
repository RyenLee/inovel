pub mod models;
pub mod db;
pub mod config;
pub mod names;
pub mod project;
pub mod chapter;
pub mod writing;
pub mod worldbuilding;
pub mod relationship;
pub mod timeline;
pub mod git_snapshot;
pub mod sensitive;
pub mod export;
pub mod backup;
pub mod encryption;
pub mod inspiration;
pub mod template;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // 初始化配置系统（用户数据目录）
            if let Err(e) = config::init_config(app.handle()) {
                eprintln!("配置初始化失败: {}", e);
            }
            
            // 首次启动时在安装目录创建配置文件
            match config::init_install_config(app.handle()) {
                Ok(config_path) => {
                    println!("[Setup] 安装配置文件路径: {}", config_path.display());
                }
                Err(e) => {
                    eprintln!("[Setup] 安装配置初始化失败: {}", e);
                }
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            // 配置管理命令
            config::get_app_config,
            config::update_app_config,
            config::reset_app_config,
            config::get_config_file_path,
            config::get_install_config_info,
            config::validate_config_paths,
            project::create_project,
            project::get_recent_projects,
            project::open_project,
            project::remove_project_from_list,
            project::update_project,
            project::set_cover,
            project::migrate_existing_projects,
            project::check_migration_needed,
            project::rollback_migration,
            chapter::get_chapter_content,
            chapter::save_chapter_content,
            chapter::create_volume,
            chapter::create_chapter,
            chapter::update_volume_name,
            chapter::update_chapter_title,
            chapter::delete_volume,
            chapter::delete_chapter,
            chapter::reorder_volumes,
            chapter::reorder_chapters,
            chapter::get_chapter_tree,
            chapter::update_chapter_word_count,
            chapter::update_chapter_summary,
            chapter::update_chapter_status,
            chapter::get_chapter_status_counts,
            writing::get_writing_goal,
            writing::save_writing_goal,
            writing::get_writing_stats,
            writing::upsert_writing_record,
            writing::get_today_words,
            // 番茄钟专注会话命令
            writing::record_focus_session,
            writing::get_focus_sessions,
            writing::get_focus_stats,
            names::generate_names,
            worldbuilding::create_character,
            worldbuilding::update_character,
            worldbuilding::delete_character,
            worldbuilding::list_characters,
            worldbuilding::create_location,
            worldbuilding::update_location,
            worldbuilding::delete_location,
            worldbuilding::list_locations,
            worldbuilding::create_organization,
            worldbuilding::update_organization,
            worldbuilding::delete_organization,
            worldbuilding::list_organizations,
            relationship::create_relationship,
            relationship::update_relationship,
            relationship::delete_relationship,
            relationship::get_relationships,
            timeline::create_event,
            timeline::update_event,
            timeline::delete_event,
            timeline::list_events,
            git_snapshot::init_project_git,
            git_snapshot::create_snapshot,
            git_snapshot::get_snapshots,
            git_snapshot::restore_snapshot,
            git_snapshot::get_snapshot_diff,
            sensitive::add_sensitive_word,
            sensitive::remove_sensitive_word,
            sensitive::list_sensitive_words,
            sensitive::import_sensitive_words,
            sensitive::scan_sensitive_words,
            export::export_txt,
            export::export_markdown,
            export::export_epub,
            export::get_export_content,
            export::export_html_for_print,
            export::get_exports_dir,
            export::open_folder_in_explorer,
            backup::backup_project,
            backup::create_incremental_backup,
            backup::list_backups,
            backup::restore_backup,
            backup::delete_backup_record,
            backup::get_backup_logs,
            backup::get_backup_stats,
            // 加密相关命令
            encryption::encrypt_project,
            encryption::decrypt_project,
            encryption::verify_project_password,
            encryption::change_project_password,
            encryption::reencrypt_project,
            encryption::is_project_encrypted_command,
            // 灵感看板命令
            inspiration::create_inspiration_item,
            inspiration::update_inspiration_item,
            inspiration::delete_inspiration_item,
            inspiration::reorder_inspiration_items,
            inspiration::get_inspiration_board,
            inspiration::get_inspiration_items,
            // 模板系统命令
            template::get_builtin_templates,
            template::get_user_templates,
            template::save_user_template,
            template::update_user_template,
            template::delete_user_template,
            template::get_all_templates,
            // 图片处理命令
            chapter::save_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
