pub mod backups;
pub mod chapters;
pub mod common;
pub mod init;
pub mod inspiration;
pub mod projects;
pub mod relationships;
pub mod sensitive;
pub mod templates;
pub mod timeline;
pub mod worldbuilding;
pub mod writing;

pub use backups::{add_backup_log, create_backup, delete_backup_record, list_backups};
pub use chapters::{
    create_chapter, create_volume, delete_chapter, delete_volume, get_chapter_by_id,
    get_chapter_tree, update_chapter_content,
};
pub use common::{begin_transaction, commit_transaction, rollback_transaction};
pub use init::{get_db_path, init_db};
pub use inspiration::{
    create_inspiration_item, delete_inspiration_item, get_inspiration_items,
    update_inspiration_item,
};
pub use projects::{
    check_project_exists, create_project, delete_project, get_all_projects, get_project_by_id,
    get_project_by_project_id, update_project, update_project_last_opened,
};
pub use relationships::{
    create_relationship, delete_relationship, get_relationships, update_relationship,
};
pub use sensitive::{
    add_sensitive_word, import_sensitive_words, list_sensitive_words, remove_sensitive_word,
};
pub use templates::{
    delete_user_template, get_user_templates, save_user_template, update_user_template,
};
pub use timeline::{create_event, delete_event, list_events, update_event};
pub use worldbuilding::{
    create_character, create_location, create_organization, delete_character, list_characters,
    list_locations, list_organizations, update_character,
};
pub use writing::{
    get_writing_goal, get_writing_records, save_writing_goal, upsert_writing_record,
};
