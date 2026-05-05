use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct NamesDatabase {
    pub chinese_name: NameData,
    pub western_name: NameData,
    pub chinese_place: Vec<String>,
    pub western_place: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct NameData {
    pub male: Vec<String>,
    pub female: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectMeta {
    pub id: i64,
    /// 项目唯一标识符（字母+数字组合，如 "P7K3M9"）
    pub project_id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub path: String,
    pub created_at: String,
    #[serde(default)]
    pub last_opened_at: Option<String>,
    #[serde(default)]
    pub is_valid: bool,
    #[serde(default)]
    pub cover_path: Option<String>,
    #[serde(default)]
    pub encrypted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProjectParams {
    pub name: String,
    pub author: String,
    pub description: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProjectParams {
    pub name: String,
    pub author: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Volume {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chapter {
    pub id: i64,
    pub volume_id: i64,
    pub title: String,
    pub file_path: String,
    pub sort_order: i32,
    pub summary: String,
    pub word_count_cache: i32,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChapterStatusCount {
    pub status: String,
    pub count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VolumeWithChapters {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub sort_order: i32,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WritingGoal {
    pub id: i64,
    pub project_id: i64,
    pub daily_goal: i32,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WritingRecord {
    pub date: String,
    pub total_words: i32,
    pub duration: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Character {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub gender: String,
    pub age: Option<i32>,
    pub appearance: String,
    pub personality: String,
    pub background: String,
    pub custom_fields: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCharacterParams {
    pub project_id: i64,
    pub name: String,
    pub gender: String,
    pub age: Option<i32>,
    pub appearance: String,
    pub personality: String,
    pub background: String,
    pub custom_fields: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCharacterParams {
    pub name: String,
    pub gender: String,
    pub age: Option<i32>,
    pub appearance: String,
    pub personality: String,
    pub background: String,
    pub custom_fields: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub location_type: String,
    pub description: String,
    pub climate: String,
    pub population: Option<i32>,
    pub notable_features: String,
    pub custom_fields: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLocationParams {
    pub project_id: i64,
    pub name: String,
    pub location_type: String,
    pub description: String,
    pub climate: String,
    pub population: Option<i32>,
    pub notable_features: String,
    pub custom_fields: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateLocationParams {
    pub name: String,
    pub location_type: String,
    pub description: String,
    pub climate: String,
    pub population: Option<i32>,
    pub notable_features: String,
    pub custom_fields: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Organization {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub org_type: String,
    pub description: String,
    pub leader: String,
    pub headquarters: String,
    pub member_count: Option<i32>,
    pub custom_fields: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrganizationParams {
    pub project_id: i64,
    pub name: String,
    pub org_type: String,
    pub description: String,
    pub leader: String,
    pub headquarters: String,
    pub member_count: Option<i32>,
    pub custom_fields: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOrganizationParams {
    pub name: String,
    pub org_type: String,
    pub description: String,
    pub leader: String,
    pub headquarters: String,
    pub member_count: Option<i32>,
    pub custom_fields: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Relationship {
    pub id: i64,
    pub project_id: i64,
    pub source_id: i64,
    pub target_id: i64,
    pub relation_type: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRelationshipParams {
    pub project_id: i64,
    pub source_id: i64,
    pub target_id: i64,
    pub relation_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRelationshipParams {
    pub relation_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub id: i64,
    pub project_id: i64,
    pub title: String,
    pub story_time: String,
    pub description: String,
    pub chapter_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateEventParams {
    pub project_id: i64,
    pub title: String,
    pub story_time: String,
    pub description: String,
    pub chapter_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateEventParams {
    pub title: String,
    pub story_time: String,
    pub description: String,
    pub chapter_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Snapshot {
    pub hash: String,
    pub message: String,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SensitiveWord {
    pub id: i64,
    pub project_id: i64,
    pub word: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SensitiveWordMatch {
    pub word: String,
    pub start: usize,
    pub end: usize,
}

/// 单个项目迁移详情
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MigrationDetail {
    /// 数据库 id (INTEGER PK)
    pub project_db_id: i64,
    /// 旧项目名称（书名）
    pub old_name: String,
    /// 旧文件夹路径
    pub old_path: String,
    /// 新文件夹路径
    pub new_path: String,
    /// 生成的项目ID
    pub project_id: String,
    /// 迁移状态: "success" | "skipped" | "failed"
    pub status: String,
    /// 错误信息（仅失败时）
    pub error: Option<String>,
}

/// 迁移命令返回结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MigrateResult {
    /// 总待迁移数
    pub total: i32,
    /// 成功数
    pub success: i32,
    /// 失败数
    pub failed: i32,
    /// 跳过的项目数（因已迁移）
    pub skipped: i32,
    /// 备份文件路径
    pub backup_path: String,
    /// 详细列表
    pub details: Vec<MigrationDetail>,
}

/// 回滚参数
#[derive(Debug, Deserialize)]
pub struct RollbackParams {
    /// 要回滚的项目 DB ID 列表。为空则回滚全部。
    pub project_ids: Option<Vec<i64>>,
}

// ==================== 加密相关数据结构 ====================

/// 加密进度（用于前端显示）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncryptionProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
}

/// 加密项目参数
#[derive(Debug, Deserialize)]
pub struct EncryptProjectParams {
    pub project_path: String,
    pub password: String,
    pub confirm_password: String,
}

/// 解密项目参数
#[derive(Debug, Deserialize)]
pub struct DecryptProjectParams {
    pub project_path: String,
    pub password: String,
}

/// 修改密码参数
#[derive(Debug, Deserialize)]
pub struct ChangePasswordParams {
    pub project_path: String,
    pub old_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

// ==================== 番茄钟相关数据结构 ====================

/// 专注会话记录
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FocusSession {
    pub id: i64,
    pub project_id: i64,
    pub session_type: String,
    pub duration_minutes: i32,
    pub started_at: String,
    pub completed: bool,
    pub created_at: String,
}

/// 专注统计数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FocusStats {
    pub total_sessions: i32,
    pub total_minutes: i32,
    pub completed_sessions: i32,
    pub work_sessions: i32,
    pub short_break_sessions: i32,
    pub long_break_sessions: i32,
}

// ==================== 灵感看板相关数据结构 ====================

/// 灵感条目
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InspirationItem {
    pub id: i64,
    pub project_id: i64,
    pub column_name: String,
    pub content: String,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// 创建灵感条目参数
#[derive(Debug, Deserialize)]
pub struct CreateInspirationItemParams {
    pub project_id: i64,
    pub column_name: String,
    pub content: String,
}

/// 更新灵感条目参数
#[derive(Debug, Deserialize)]
pub struct UpdateInspirationItemParams {
    pub content: String,
}
