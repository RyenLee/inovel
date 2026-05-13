use serde::{Deserialize, Serialize};

// ==================== 名称数据库相关 ====================

/// 名称数据库结构（从 names.json 加载）
///
/// 用于生成角色名称和地名
#[derive(Debug, Deserialize)]
pub struct NamesDatabase {
    /// 中文姓名数据
    pub chinese_name: NameData,
    /// 西方姓名数据
    pub western_name: NameData,
    /// 中文地名列表
    pub chinese_place: Vec<String>,
    /// 西方地名列表
    pub western_place: Vec<String>,
}

/// 姓名数据结构
#[derive(Debug, Deserialize)]
pub struct NameData {
    /// 男性名字列表
    pub male: Vec<String>,
    /// 女性名字列表
    pub female: Vec<String>,
}

// ==================== 项目相关 ====================

/// 项目元数据结构
///
/// 包含项目的基本信息，用于在欢迎页展示和项目管理
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectMeta {
    /// 数据库主键 ID
    pub id: i64,
    /// 项目唯一标识符（字母P开头 + 5位字母数字组合，如 "P7K3M9"）
    pub project_id: String,
    /// 项目名称（书名）
    pub name: String,
    /// 作者名
    pub author: String,
    /// 项目描述
    pub description: String,
    /// 项目文件夹绝对路径
    pub path: String,
    /// 创建时间（RFC3339 格式）
    pub created_at: String,
    /// 最后打开时间（RFC3339 格式）
    #[serde(default)]
    pub last_opened_at: Option<String>,
    /// 项目是否有效（文件夹是否存在）
    #[serde(default)]
    pub is_valid: bool,
    /// 封面图片路径（绝对路径）
    #[serde(default)]
    pub cover_path: Option<String>,
    /// 项目是否加密
    #[serde(default)]
    pub encrypted: bool,
}

/// 创建项目参数
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProjectParams {
    /// 项目名称（书名）
    pub name: String,
    /// 作者名
    pub author: String,
    /// 项目描述
    pub description: String,
    /// 项目存储目录（父目录）
    pub path: String,
}

/// 更新项目参数
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProjectParams {
    /// 项目名称（书名）
    pub name: String,
    /// 作者名
    pub author: String,
    /// 项目描述
    pub description: String,
}

/// 分页项目列表响应
///
/// 包含分页信息和项目列表
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedProjects {
    /// 项目列表
    pub items: Vec<ProjectMeta>,
    /// 总项目数
    pub total: i64,
    /// 当前页码（从1开始）
    pub page: i32,
    /// 每页项目数
    pub page_size: i32,
    /// 总页数
    pub total_pages: i32,
}

// ==================== 章节结构相关 ====================

/// 卷（书籍的分卷）结构
///
/// 用于组织章节，一个项目可以包含多个卷
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Volume {
    /// 卷的唯一标识符（数据库主键）
    pub id: i64,
    /// 所属项目 ID
    pub project_id: i64,
    /// 卷名称（如 "第一卷"、"上部"）
    pub name: String,
    /// 排序顺序（数字越小越靠前）
    pub sort_order: i32,
}

/// 章节结构
///
/// 小说的基本组成单元，每个章节对应一个 Markdown 文件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chapter {
    /// 章节唯一标识符（数据库主键）
    pub id: i64,
    /// 所属卷 ID（可为 NULL，表示不属于任何卷）
    pub volume_id: i64,
    /// 章节标题
    pub title: String,
    /// 章节文件路径（相对于项目目录的路径）
    pub file_path: String,
    /// 排序顺序（同卷内排序）
    pub sort_order: i32,
    /// 章节摘要/简介
    pub summary: String,
    /// 字数缓存（避免每次打开都重新计算）
    pub word_count_cache: i32,
    /// 章节状态（draft: 草稿, completed: 已完成, published: 已发布）
    pub status: String,
    /// 创建时间（RFC3339 格式）
    pub created_at: String,
    /// 最后更新时间（RFC3339 格式）
    pub updated_at: String,
}

/// 章节状态统计结构
///
/// 用于统计各状态章节的数量
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChapterStatusCount {
    /// 状态名称
    pub status: String,
    /// 该状态的章节数量
    pub count: i32,
}

/// 包含章节列表的卷结构
///
/// 用于一次性获取卷及其包含的所有章节
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VolumeWithChapters {
    /// 卷的唯一标识符
    pub id: i64,
    /// 所属项目 ID
    pub project_id: i64,
    /// 卷名称
    pub name: String,
    /// 排序顺序
    pub sort_order: i32,
    /// 卷下的章节列表
    pub chapters: Vec<Chapter>,
}

// ==================== 写作目标相关 ====================

/// 写作目标结构
///
/// 存储项目的每日写作目标设置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WritingGoal {
    /// 目标唯一标识符
    pub id: i64,
    /// 所属项目 ID
    pub project_id: i64,
    /// 每日写作目标字数
    pub daily_goal: i32,
    /// 最后更新时间
    pub updated_at: String,
}

/// 写作记录结构
///
/// 存储每日写作统计数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WritingRecord {
    /// 日期（格式：YYYY-MM-DD）
    pub date: String,
    /// 当日写作总字数
    pub total_words: i32,
    /// 当日写作时长（分钟）
    pub duration: i32,
}

// ==================== 角色相关 ====================

/// 角色结构
///
/// 存储小说角色的详细信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Character {
    /// 角色唯一标识符
    pub id: i64,
    /// 所属项目 ID
    pub project_id: i64,
    /// 角色名称
    pub name: String,
    /// 角色性别（male/female/other）
    pub gender: String,
    /// 角色年龄（可选）
    pub age: Option<i32>,
    /// 外貌描述
    pub appearance: String,
    /// 性格描述
    pub personality: String,
    /// 背景故事
    pub background: String,
    /// 自定义字段（JSON 格式）
    pub custom_fields: String,
    /// 创建时间
    pub created_at: String,
    /// 最后更新时间
    pub updated_at: String,
}

/// 创建角色参数
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCharacterParams {
    /// 所属项目 ID
    pub project_id: i64,
    /// 角色名称
    pub name: String,
    /// 角色性别
    pub gender: String,
    /// 角色年龄（可选）
    pub age: Option<i32>,
    /// 外貌描述
    pub appearance: String,
    /// 性格描述
    pub personality: String,
    /// 背景故事
    pub background: String,
    /// 自定义字段（JSON 格式）
    pub custom_fields: String,
}

/// 更新角色参数
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCharacterParams {
    /// 角色名称
    pub name: String,
    /// 角色性别
    pub gender: String,
    /// 角色年龄（可选）
    pub age: Option<i32>,
    /// 外貌描述
    pub appearance: String,
    /// 性格描述
    pub personality: String,
    /// 背景故事
    pub background: String,
    /// 自定义字段（JSON 格式）
    pub custom_fields: String,
}

// ==================== 地点相关 ====================

/// 地点结构
///
/// 存储小说中出现的地点信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    /// 地点唯一标识符
    pub id: i64,
    /// 所属项目 ID
    pub project_id: i64,
    /// 地点名称
    pub name: String,
    /// 地点类型（city/village/forest/castle 等）
    pub location_type: String,
    /// 地点描述
    pub description: String,
    /// 气候特征
    pub climate: String,
    /// 人口数量（可选）
    pub population: Option<i32>,
    /// 显著特征/景点
    pub notable_features: String,
    /// 自定义字段（JSON 格式）
    pub custom_fields: String,
    /// 创建时间
    pub created_at: String,
    /// 最后更新时间
    pub updated_at: String,
}

/// 创建地点参数
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLocationParams {
    /// 所属项目 ID
    pub project_id: i64,
    /// 地点名称
    pub name: String,
    /// 地点类型
    pub location_type: String,
    /// 地点描述
    pub description: String,
    /// 气候特征
    pub climate: String,
    /// 人口数量（可选）
    pub population: Option<i32>,
    /// 显著特征/景点
    pub notable_features: String,
    /// 自定义字段（JSON 格式）
    pub custom_fields: String,
}

/// 更新地点参数
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateLocationParams {
    /// 地点名称
    pub name: String,
    /// 地点类型
    pub location_type: String,
    /// 地点描述
    pub description: String,
    /// 气候特征
    pub climate: String,
    /// 人口数量（可选）
    pub population: Option<i32>,
    /// 显著特征/景点
    pub notable_features: String,
    /// 自定义字段（JSON 格式）
    pub custom_fields: String,
}

// ==================== 组织相关 ====================

/// 组织/势力结构
///
/// 存储小说中的组织、门派、势力等信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Organization {
    /// 组织唯一标识符
    pub id: i64,
    /// 所属项目 ID
    pub project_id: i64,
    /// 组织名称
    pub name: String,
    /// 组织类型（clan/guild/kingdom/company 等）
    pub org_type: String,
    /// 组织描述
    pub description: String,
    /// 领袖/负责人
    pub leader: String,
    /// 总部位置
    pub headquarters: String,
    /// 成员数量（可选）
    pub member_count: Option<i32>,
    /// 自定义字段（JSON 格式）
    pub custom_fields: String,
    /// 创建时间
    pub created_at: String,
    /// 最后更新时间
    pub updated_at: String,
}

/// 创建组织参数
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrganizationParams {
    /// 所属项目 ID
    pub project_id: i64,
    /// 组织名称
    pub name: String,
    /// 组织类型
    pub org_type: String,
    /// 组织描述
    pub description: String,
    /// 领袖/负责人
    pub leader: String,
    /// 总部位置
    pub headquarters: String,
    /// 成员数量（可选）
    pub member_count: Option<i32>,
    /// 自定义字段（JSON 格式）
    pub custom_fields: String,
}

/// 更新组织参数
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOrganizationParams {
    /// 组织名称
    pub name: String,
    /// 组织类型
    pub org_type: String,
    /// 组织描述
    pub description: String,
    /// 领袖/负责人
    pub leader: String,
    /// 总部位置
    pub headquarters: String,
    /// 成员数量（可选）
    pub member_count: Option<i32>,
    /// 自定义字段（JSON 格式）
    pub custom_fields: String,
}

// ==================== 关系与事件相关 ====================

/// 角色关系结构
///
/// 存储角色之间的关系（如父子、师徒、朋友、敌人等）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Relationship {
    /// 关系唯一标识符
    pub id: i64,
    /// 所属项目 ID
    pub project_id: i64,
    /// 源角色 ID（关系发起方）
    pub source_id: i64,
    /// 目标角色 ID（关系接收方）
    pub target_id: i64,
    /// 关系类型（father/teacher/friend/enemy 等）
    pub relation_type: String,
    /// 创建时间
    pub created_at: String,
}

/// 创建角色关系参数
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRelationshipParams {
    /// 所属项目 ID
    pub project_id: i64,
    /// 源角色 ID
    pub source_id: i64,
    /// 目标角色 ID
    pub target_id: i64,
    /// 关系类型
    pub relation_type: String,
}

/// 更新角色关系参数
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRelationshipParams {
    /// 关系类型
    pub relation_type: String,
}

/// 故事事件结构
///
/// 存储小说中的关键事件节点，用于时间线管理
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    /// 事件唯一标识符
    pub id: i64,
    /// 所属项目 ID
    pub project_id: i64,
    /// 事件标题
    pub title: String,
    /// 故事时间（虚构时间线中的时间点）
    pub story_time: String,
    /// 事件描述
    pub description: String,
    /// 关联的章节 ID（可选）
    pub chapter_id: Option<i64>,
    /// 创建时间
    pub created_at: String,
    /// 最后更新时间
    pub updated_at: String,
}

/// 创建事件参数
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateEventParams {
    /// 所属项目 ID
    pub project_id: i64,
    /// 事件标题
    pub title: String,
    /// 故事时间
    pub story_time: String,
    /// 事件描述
    pub description: String,
    /// 关联的章节 ID（可选）
    pub chapter_id: Option<i64>,
}

/// 更新事件参数
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateEventParams {
    /// 事件标题
    pub title: String,
    /// 故事时间
    pub story_time: String,
    /// 事件描述
    pub description: String,
    /// 关联的章节 ID（可选）
    pub chapter_id: Option<i64>,
}

// ==================== 版本快照与敏感词相关 ====================

/// 版本快照结构
///
/// 用于项目版本控制，记录每次修改的快照信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Snapshot {
    /// 快照哈希值（唯一标识）
    pub hash: String,
    /// 快照描述信息
    pub message: String,
    /// 创建时间
    pub date: String,
}

/// 敏感词结构
///
/// 存储项目自定义的敏感词列表，用于内容审查
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SensitiveWord {
    /// 敏感词唯一标识符
    pub id: i64,
    /// 所属项目 ID
    pub project_id: i64,
    /// 敏感词内容
    pub word: String,
    /// 创建时间
    pub created_at: String,
}

/// 敏感词匹配结果结构
///
/// 存储敏感词检测时的匹配结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SensitiveWordMatch {
    /// 匹配到的敏感词
    pub word: String,
    /// 在文本中的起始位置
    pub start: usize,
    /// 在文本中的结束位置
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

/// 启用全局加密参数
#[derive(Debug, Deserialize)]
pub struct EnableGlobalEncryptionParams {
    pub password: String,
    pub confirm_password: String,
}

/// 禁用全局加密参数
#[derive(Debug, Deserialize)]
pub struct DisableGlobalEncryptionParams {
    pub password: String,
}

/// 修改全局密码参数
#[derive(Debug, Deserialize)]
pub struct ChangeGlobalPasswordParams {
    pub old_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

/// 验证全局密码参数
#[derive(Debug, Deserialize)]
pub struct VerifyGlobalPasswordParams {
    pub password: String,
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
    pub completed_work_sessions: i32,
    pub work_duration_minutes: i32,
}

// ==================== 灵感看板相关数据结构 ====================

/// 灵感条目
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InspirationItem {
    pub id: i64,
    pub project_id: i64,
    pub column_key: String,
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
    pub column_key: String,
    pub content: String,
}

/// 更新灵感条目参数
#[derive(Debug, Deserialize)]
pub struct UpdateInspirationItemParams {
    pub content: String,
}

// ==================== 模板系统相关数据结构 ====================

/// 内置模板结构体（从 builtin_templates.json 加载）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WritingTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub content: String,
    pub is_builtin: bool,
}

/// 模板分组（用于新格式的 JSON 解析）
#[derive(Debug, Deserialize)]
pub struct TemplateGroup {
    pub category: String,
    pub objects: Vec<WritingTemplate>,
}

/// 用户自定义模板（存储在数据库中）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserTemplate {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub description: String,
    pub category: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

/// 创建用户模板参数
#[derive(Debug, Deserialize)]
pub struct CreateUserTemplateParams {
    pub project_id: i64,
    pub name: String,
    pub description: String,
    pub category: String,
    pub content: String,
}

/// 更新用户模板参数
#[derive(Debug, Deserialize)]
pub struct UpdateUserTemplateParams {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub content: Option<String>,
}
