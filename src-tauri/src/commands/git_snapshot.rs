use crate::db::get_db_path;
use crate::models::Snapshot;
use git2::{Oid, Repository, Signature};
use std::fs;
use std::path::Path;
use tauri::AppHandle;

/// 获取项目文件夹路径
///
/// 根据项目 ID 从数据库查询项目的根目录路径。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回项目路径，失败返回错误信息
pub fn get_project_folder_path(
    app_handle: &AppHandle,
    project_id: i64,
) -> Result<std::path::PathBuf, String> {
    let db_path = get_db_path(app_handle);
    let conn =
        rusqlite::Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    let path: String = conn
        .query_row(
            "SELECT path FROM projects WHERE id = ?1",
            [project_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询项目路径失败: {}", e))?;
    Ok(std::path::PathBuf::from(path))
}

/// 初始化 Git 仓库（使用默认的 .gitignore）
///
/// 在项目目录下初始化 Git 仓库，创建初始提交。
/// 默认忽略：metadata.db、exports/、cover.jpg、node_modules/
///
/// # 参数
/// - `project_path`: 项目根目录路径
///
/// # 返回值
/// 成功返回 Ok(())，失败返回错误信息
pub fn init_git_repo(project_path: &Path) -> Result<(), String> {
    init_git_repo_with_ignore(
        project_path,
        "metadata.db\nexports/\ncover.jpg\nnode_modules/\n",
    )
}

/// 初始化 Git 仓库（使用自定义的 .gitignore）
///
/// 在项目目录下初始化 Git 仓库，创建 .gitignore 文件和初始提交。
///
/// # 参数
/// - `project_path`: 项目根目录路径
/// - `gitignore_content`: .gitignore 文件内容
///
/// # 返回值
/// 成功返回 Ok(())，失败返回错误信息
pub fn init_git_repo_with_ignore(
    project_path: &Path,
    gitignore_content: &str,
) -> Result<(), String> {
    let repo = Repository::init(project_path).map_err(|e| format!("Git 初始化失败: {}", e))?;
    let gitignore_path = project_path.join(".gitignore");
    fs::write(&gitignore_path, gitignore_content)
        .map_err(|e| format!("创建 .gitignore 失败: {}", e))?;

    let mut index = repo
        .index()
        .map_err(|e| format!("获取 index 失败: {}", e))?;
    index
        .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .map_err(|e| format!("添加文件到暂存区失败: {}", e))?;
    let tree_oid = index
        .write_tree()
        .map_err(|e| format!("写入 tree 失败: {}", e))?;
    let tree = repo
        .find_tree(tree_oid)
        .map_err(|e| format!("查找 tree 失败: {}", e))?;
    let sig =
        Signature::now("inovel", "inovel@local").map_err(|e| format!("创建签名失败: {}", e))?;
    repo.commit(Some("HEAD"), &sig, &sig, "初始提交", &tree, &[])
        .map_err(|e| format!("初始提交失败: {}", e))?;
    Ok(())
}

/// 为项目初始化 Git 仓库
///
/// 懒加载初始化：如果仓库已存在则跳过，不存在则创建新仓库。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
/// - `gitignore`: 自定义的 .gitignore 内容（可选）
///
/// # 返回值
/// 成功返回 Ok(())，失败返回错误信息
#[tauri::command(rename_all = "snake_case")]
pub async fn init_project_git(
    app_handle: AppHandle,
    project_id: i64,
    gitignore: Option<String>,
) -> Result<(), String> {
    let path = get_project_folder_path(&app_handle, project_id)?;
    match gitignore {
        Some(content) => init_git_repo_with_ignore(&path, &content)?,
        None => init_git_repo(&path)?,
    }
    Ok(())
}

/// 打开或初始化 Git 仓库（懒加载）
///
/// 尝试打开现有仓库，如果不存在则初始化新仓库。
/// 用于确保操作时仓库已存在。
///
/// # 参数
/// - `project_path`: 项目根目录路径
///
/// # 返回值
/// 成功返回 Repository 实例，失败返回错误信息
pub(crate) fn open_or_init_repo(project_path: &std::path::Path) -> Result<Repository, String> {
    // 首先检查目录是否存在
    if !project_path.exists() {
        return Err(format!("项目目录不存在: {}", project_path.display()));
    }

    if !project_path.is_dir() {
        return Err(format!("路径不是目录: {}", project_path.display()));
    }

    match Repository::open(project_path) {
        Ok(repo) => Ok(repo),
        Err(e) => {
            tracing::info!("现有仓库打开失败，尝试初始化新仓库: {}", e);
            // No repo yet, initialize one
            init_git_repo(project_path)?;
            Repository::open(project_path).map_err(|e| format!("初始化后打开仓库失败: {}", e))
        }
    }
}

/// 创建快照（Git 提交）
///
/// 将项目当前状态作为新的 Git 提交保存。
/// 使用 "inovel" 作为提交者签名。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
/// - `message`: 提交消息
///
/// # 返回值
/// 成功返回快照信息（包含哈希、消息、日期），失败返回错误信息
#[tauri::command(rename_all = "snake_case")]
pub async fn create_snapshot(
    app_handle: AppHandle,
    project_id: i64,
    message: String,
) -> Result<Snapshot, String> {
    let project_path = get_project_folder_path(&app_handle, project_id)?;
    let repo = open_or_init_repo(&project_path)?;

    let mut index = repo
        .index()
        .map_err(|e| format!("获取 index 失败: {}", e))?;
    index
        .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .map_err(|e| format!("添加文件失败: {}", e))?;
    let tree_oid = index
        .write_tree()
        .map_err(|e| format!("写入 tree 失败: {}", e))?;
    let tree = repo
        .find_tree(tree_oid)
        .map_err(|e| format!("查找 tree 失败: {}", e))?;
    let sig =
        Signature::now("inovel", "inovel@local").map_err(|e| format!("创建签名失败: {}", e))?;

    let parent = match repo.head() {
        Ok(h) => h.target().and_then(|o| repo.find_commit(o).ok()),
        Err(_) => None,
    };
    let parents: Vec<&git2::Commit> = parent.iter().collect();

    let oid = repo
        .commit(Some("HEAD"), &sig, &sig, &message, &tree, &parents)
        .map_err(|e| format!("提交失败: {}", e))?;
    let commit = repo
        .find_commit(oid)
        .map_err(|e| format!("查找新提交失败: {}", e))?;
    let time = commit.time();

    Ok(Snapshot {
        hash: oid.to_string(),
        message,
        date: chrono::DateTime::from_timestamp(time.seconds(), 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
    })
}

/// 获取项目的所有快照（Git 提交历史）
///
/// 按时间倒序返回项目中所有 Git 提交记录。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回快照列表，失败返回错误信息
#[tauri::command(rename_all = "snake_case")]
pub async fn get_snapshots(
    app_handle: AppHandle,
    project_id: i64,
) -> Result<Vec<Snapshot>, String> {
    let project_path = get_project_folder_path(&app_handle, project_id)?;
    let repo = open_or_init_repo(&project_path)?;

    let mut revwalk = repo
        .revwalk()
        .map_err(|e| format!("创建 revwalk 失败: {}", e))?;
    revwalk
        .push_head()
        .map_err(|_| "没有提交记录".to_string())?;

    let mut snapshots = Vec::new();
    for oid_result in revwalk {
        let oid = oid_result.map_err(|e| format!("遍历提交失败: {}", e))?;
        let commit = repo
            .find_commit(oid)
            .map_err(|e| format!("查找提交失败: {}", e))?;
        let time = commit.time();
        snapshots.push(Snapshot {
            hash: oid.to_string(),
            message: commit.message().unwrap_or("").to_string(),
            date: chrono::DateTime::from_timestamp(time.seconds(), 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
        });
    }
    Ok(snapshots)
}

/// 恢复快照（将项目重置到指定版本）
///
/// 执行 Git 硬重置到指定提交，然后创建一个新的提交记录恢复操作。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
/// - `commit_hash`: 要恢复到的提交哈希
///
/// # 返回值
/// 成功返回 Ok(())，失败返回错误信息
#[tauri::command(rename_all = "snake_case")]
pub async fn restore_snapshot(
    app_handle: AppHandle,
    project_id: i64,
    commit_hash: String,
) -> Result<(), String> {
    let project_path = get_project_folder_path(&app_handle, project_id)?;
    let repo = open_or_init_repo(&project_path)?;

    let oid = commit_hash
        .parse::<Oid>()
        .map_err(|e| format!("解析哈希失败: {}", e))?;

    let obj = repo
        .find_object(oid, Some(git2::ObjectType::Commit))
        .map_err(|e| format!("查找对象失败: {}", e))?;
    repo.reset(&obj, git2::ResetType::Hard, None)
        .map_err(|e| format!("重置失败: {}", e))?;

    let mut index = repo
        .index()
        .map_err(|e| format!("获取 index 失败: {}", e))?;
    let tree_oid = index
        .write_tree()
        .map_err(|e| format!("写入 tree 失败: {}", e))?;
    let tree = repo
        .find_tree(tree_oid)
        .map_err(|e| format!("查找 tree 失败: {}", e))?;

    let sig =
        Signature::now("inovel", "inovel@local").map_err(|e| format!("创建签名失败: {}", e))?;
    let restore_msg = format!("恢复至版本 {}", &commit_hash[..8]);
    let parent = repo.find_commit(oid).ok();
    let parents: Vec<&git2::Commit> = parent.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, &restore_msg, &tree, &parents)
        .map_err(|e| format!("创建恢复提交失败: {}", e))?;
    Ok(())
}

/// 获取两个快照之间的差异
///
/// 计算并返回两个 Git 提交之间的文件差异，以标准 diff 格式输出。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
/// - `from_hash`: 起始提交哈希
/// - `to_hash`: 目标提交哈希
///
/// # 返回值
/// 成功返回差异内容（标准 diff 格式），失败返回错误信息
#[tauri::command(rename_all = "snake_case")]
pub async fn get_snapshot_diff(
    app_handle: AppHandle,
    project_id: i64,
    from_hash: String,
    to_hash: String,
) -> Result<String, String> {
    let project_path = get_project_folder_path(&app_handle, project_id)?;
    let repo = open_or_init_repo(&project_path)?;

    let from_oid = from_hash
        .parse::<Oid>()
        .map_err(|e| format!("解析哈希失败: {}", e))?;
    let to_oid = to_hash
        .parse::<Oid>()
        .map_err(|e| format!("解析哈希失败: {}", e))?;

    let from_tree = repo
        .find_commit(from_oid)
        .and_then(|c| c.tree())
        .map_err(|e| format!("获取树失败: {}", e))?;
    let to_tree = repo
        .find_commit(to_oid)
        .and_then(|c| c.tree())
        .map_err(|e| format!("获取树失败: {}", e))?;

    let diff = repo
        .diff_tree_to_tree(Some(&from_tree), Some(&to_tree), None)
        .map_err(|e| format!("对比失败: {}", e))?;

    let mut lines: Vec<String> = Vec::new();
    diff.print(git2::DiffFormat::Patch, |delta, _hunk, line| {
        let origin = line.origin();
        if origin == 'F' {
            let file = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            lines.push(format!("--- a/{}", file));
            lines.push(format!("+++ b/{}", file));
        } else if origin == 'H' {
            // Hunk header — already starts with @@, output as-is
            let content = std::str::from_utf8(line.content()).unwrap_or("");
            lines.push(content.trim_end_matches('\n').to_string());
        } else if origin == '+' || origin == '-' {
            // Added/removed lines — standard diff prefix
            let content = std::str::from_utf8(line.content()).unwrap_or("");
            lines.push(format!("{}{}", origin, content.trim_end_matches('\n')));
        } else if origin == ' ' {
            // Context lines — prefix with space
            let content = std::str::from_utf8(line.content()).unwrap_or("");
            lines.push(format!(" {}", content.trim_end_matches('\n')));
        }
        true
    })
    .map_err(|e| format!("生成 diff 失败: {}", e))?;

    Ok(lines.join("\n"))
}
