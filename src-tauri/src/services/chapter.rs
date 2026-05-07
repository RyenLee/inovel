use rusqlite::Connection;
use tauri::AppHandle;
use tracing::info;

use crate::config;
use crate::error::{AppError, Result};
use crate::models::{Chapter, Volume, VolumeWithChapters};

/// 章节服务层
///
/// 提供章节相关的业务逻辑，包括创建、读取、保存、删除章节等操作。
/// 该服务封装了数据库访问和文件操作，向上层提供简洁的 API。
pub struct ChapterService;

impl ChapterService {
    /// 获取章节内容
    ///
    /// 从文件系统中读取指定章节的内容。
    ///
    /// # 参数
    /// - `app_handle`: Tauri 应用句柄
    /// - `chapter_id`: 章节 ID
    ///
    /// # 返回值
    /// 章节内容字符串 `Ok(String)`，章节不存在返回 `AppError::NotFound`
    pub fn get_chapter_content(app_handle: &AppHandle, chapter_id: i64) -> Result<String> {
        let db_path = config::get_db_path(app_handle);
        let conn = Connection::open(&db_path)?;

        // 查询章节的文件路径和字数缓存
        let (file_path, _): (String, i32) = conn
            .query_row(
                "SELECT file_path, word_count_cache FROM chapters WHERE id = ?1",
                [chapter_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|_| AppError::not_found(format!("章节 {} 不存在", chapter_id)))?;

        // 如果文件存在，读取内容；否则返回空字符串
        if std::path::Path::new(&file_path).exists() {
            std::fs::read_to_string(&file_path).map_err(AppError::from)
        } else {
            Ok(String::new())
        }
    }

    /// 保存章节内容
    ///
    /// 将章节内容写入文件，并更新数据库中的字数缓存和更新时间。
    ///
    /// # 参数
    /// - `app_handle`: Tauri 应用句柄
    /// - `chapter_id`: 章节 ID
    /// - `content`: 章节内容
    ///
    /// # 返回值
    /// 保存成功返回 `Ok(())`，章节不存在返回 `AppError::NotFound`
    pub fn save_chapter_content(
        app_handle: &AppHandle,
        chapter_id: i64,
        content: &str,
    ) -> Result<()> {
        let db_path = config::get_db_path(app_handle);
        let conn = Connection::open(&db_path)?;
        let now = chrono::Utc::now().to_rfc3339();

        // 查询章节的文件路径
        let file_path: String = conn
            .query_row(
                "SELECT file_path FROM chapters WHERE id = ?1",
                [chapter_id],
                |row| row.get(0),
            )
            .map_err(|_| AppError::not_found(format!("章节 {} 不存在", chapter_id)))?;

        // 将内容写入文件
        std::fs::write(&file_path, content)?;

        // 计算字数（排除空白字符）
        let word_count = content.chars().filter(|c| !c.is_whitespace()).count() as i32;

        // 更新数据库中的字数缓存和更新时间
        conn.execute(
            "UPDATE chapters SET word_count_cache = ?1, updated_at = ?2 WHERE id = ?3",
            (word_count, &now, chapter_id),
        )?;

        info!(chapter_id = %chapter_id, word_count = %word_count, "章节内容保存成功");
        Ok(())
    }

    /// 创建章节
    ///
    /// 创建一个新的章节，包括：
    /// 1. 在数据库中记录章节信息
    /// 2. 创建对应的章节文件
    ///
    /// # 参数
    /// - `app_handle`: Tauri 应用句柄
    /// - `volume_id`: 所属卷 ID
    /// - `title`: 章节标题
    ///
    /// # 返回值
    /// 创建成功返回 `Ok(Chapter)`，失败返回 `AppError`
    pub fn create_chapter(app_handle: &AppHandle, volume_id: i64, title: &str) -> Result<Chapter> {
        let db_path = config::get_db_path(app_handle);
        let conn = Connection::open(&db_path)?;
        let now = chrono::Utc::now().to_rfc3339();
        let id = chrono::Utc::now().timestamp_millis();

        // 获取当前卷下的最大排序序号
        let sort_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM chapters WHERE volume_id = ?1",
                [volume_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // 构建章节文件路径
        let project_dir = config::get_data_dir(app_handle);
        let file_name = format!("chapter_{}.txt", id);
        let file_path = project_dir.join("chapters").join(file_name);

        // 确保目录存在
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 创建空文件
        std::fs::File::create(&file_path)?;

        // 插入数据库记录
        conn.execute(
            "INSERT INTO chapters (id, volume_id, title, file_path, sort_order, status, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, 'draft', ?6, ?7)",
            (id, volume_id, title, file_path.to_string_lossy().to_string(), sort_order, &now, &now),
        )?;

        info!(chapter_id = %id, title = %title, "章节创建成功");

        Ok(Chapter {
            id,
            volume_id,
            title: title.to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            sort_order,
            summary: String::new(),
            word_count_cache: 0,
            status: "draft".to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// 删除章节
    ///
    /// 删除指定章节，包括：
    /// 1. 从数据库中删除章节记录
    /// 2. 删除对应的章节文件
    ///
    /// # 参数
    /// - `app_handle`: Tauri 应用句柄
    /// - `chapter_id`: 章节 ID
    ///
    /// # 返回值
    /// 删除成功返回 `Ok(())`，章节不存在返回 `AppError::NotFound`
    pub fn delete_chapter(app_handle: &AppHandle, chapter_id: i64) -> Result<()> {
        let db_path = config::get_db_path(app_handle);
        let conn = Connection::open(&db_path)?;

        // 先获取章节的文件路径（用于后续删除文件）
        let file_path: Option<String> = conn
            .query_row(
                "SELECT file_path FROM chapters WHERE id = ?1",
                [chapter_id],
                |row| row.get(0),
            )
            .ok();

        // 从数据库中删除章节记录
        let rows_affected = conn.execute("DELETE FROM chapters WHERE id = ?1", [chapter_id])?;

        if rows_affected == 0 {
            return Err(AppError::not_found(format!("章节 {} 不存在", chapter_id)));
        }

        // 删除对应的文件（忽略删除失败，因为文件可能不存在）
        if let Some(fp) = file_path {
            let _ = std::fs::remove_file(&fp);
        }

        info!(chapter_id = %chapter_id, "章节删除成功");
        Ok(())
    }

    /// 创建卷（分卷）
    ///
    /// 在指定项目下创建一个新的卷。
    ///
    /// # 参数
    /// - `app_handle`: Tauri 应用句柄
    /// - `project_id`: 所属项目 ID
    /// - `name`: 卷名称
    ///
    /// # 返回值
    /// 创建成功返回 `Ok(Volume)`，失败返回 `AppError`
    pub fn create_volume(app_handle: &AppHandle, project_id: i64, name: &str) -> Result<Volume> {
        let db_path = config::get_db_path(app_handle);
        let conn = Connection::open(&db_path)?;
        let id = chrono::Utc::now().timestamp_millis();

        // 获取当前项目下的最大排序序号
        let sort_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM volumes WHERE project_id = ?1",
                [project_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // 插入数据库记录
        conn.execute(
            "INSERT INTO volumes (id, project_id, name, sort_order) 
             VALUES (?1, ?2, ?3, ?4)",
            (id, project_id, name, sort_order),
        )?;

        info!(volume_id = %id, name = %name, "卷创建成功");

        Ok(Volume {
            id,
            project_id,
            name: name.to_string(),
            sort_order,
        })
    }

    /// 获取项目的章节树结构
    ///
    /// 查询指定项目下所有卷及其包含的章节，返回树形结构。
    ///
    /// # 参数
    /// - `app_handle`: Tauri 应用句柄
    /// - `project_id`: 项目 ID
    ///
    /// # 返回值
    /// 卷及其章节的树形结构列表 `Ok(Vec<VolumeWithChapters>)`
    pub fn get_chapter_tree(
        app_handle: &AppHandle,
        project_id: i64,
    ) -> Result<Vec<VolumeWithChapters>> {
        let db_path = config::get_db_path(app_handle);
        let conn = Connection::open(&db_path)?;

        let mut stmt = conn.prepare(
            "SELECT v.id, v.project_id, v.name, v.sort_order,
                    c.id, c.volume_id, c.title, c.file_path, c.sort_order, c.word_count_cache, c.status
             FROM volumes v
             LEFT JOIN chapters c ON c.volume_id = v.id
             WHERE v.project_id = ?1
             ORDER BY v.sort_order, c.sort_order"
        )?;

        let mut volumes_map: std::collections::HashMap<i64, VolumeWithChapters> =
            std::collections::HashMap::new();

        let rows = stmt.query_map([project_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, Option<i64>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<i32>>(7)?,
                row.get::<_, Option<i32>>(8)?,
            ))
        })?;

        // 聚合卷和章节数据
        for row in rows.flatten() {
            let (vid, pid, name, order, cid, cname, corder, cwc) = row;

            if !volumes_map.contains_key(&vid) {
                volumes_map.insert(
                    vid,
                    VolumeWithChapters {
                        id: vid,
                        project_id: pid,
                        name,
                        sort_order: order,
                        chapters: Vec::new(),
                    },
                );
            }

            if let (Some(id), Some(title), Some(ch_order)) = (cid, cname, corder) {
                if let Some(vol) = volumes_map.get_mut(&vid) {
                    vol.chapters.push(Chapter {
                        id,
                        volume_id: vid,
                        title,
                        file_path: String::new(),
                        sort_order: ch_order,
                        summary: String::new(),
                        word_count_cache: cwc.unwrap_or(0),
                        status: "draft".to_string(),
                        created_at: String::new(),
                        updated_at: String::new(),
                    });
                }
            }
        }

        // 转换为排序后的 Vec
        let mut volumes: Vec<VolumeWithChapters> = volumes_map.into_values().collect();
        volumes.sort_by(|a, b| a.sort_order.cmp(&b.sort_order));

        Ok(volumes)
    }
}
