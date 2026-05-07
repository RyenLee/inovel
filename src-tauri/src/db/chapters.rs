use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, Result as SqliteResult};

use crate::models::{Chapter, Volume, VolumeWithChapters};

/// 创建卷（分卷）
/// 
/// 在指定项目下创建一个新的卷，自动计算排序顺序（追加到最后）。
/// 
/// # 参数
/// - `conn`: SQLite 数据库连接
/// - `project_id`: 所属项目 ID
/// - `name`: 卷名称（如 "第一卷"）
/// 
/// # 返回值
/// 创建成功返回 `Ok(Volume)`，失败返回 `rusqlite::Error`
pub fn create_volume(conn: &Connection, project_id: i64, name: &str) -> SqliteResult<Volume> {
    // 获取当前项目下最大的排序序号，新卷的序号为最大值 + 1
    let sort_order: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM volumes WHERE project_id = ?1",
            [project_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    conn.execute(
        "INSERT INTO volumes (project_id, name, sort_order) VALUES (?1, ?2, ?3)",
        (project_id, name, sort_order),
    )?;

    Ok(Volume {
        id: conn.last_insert_rowid(),
        project_id,
        name: name.to_string(),
        sort_order,
    })
}

/// 创建章节
/// 
/// 在指定卷下创建一个新的章节，自动计算排序顺序（追加到最后）。
/// 新建章节的状态默认为 `draft`（草稿）。
/// 
/// # 参数
/// - `conn`: SQLite 数据库连接
/// - `volume_id`: 所属卷 ID
/// - `title`: 章节标题
/// - `file_path`: 章节文件路径（相对于项目目录）
/// 
/// # 返回值
/// 创建成功返回 `Ok(Chapter)`，失败返回 `rusqlite::Error`
pub fn create_chapter(
    conn: &Connection,
    volume_id: i64,
    title: &str,
    file_path: &str,
) -> SqliteResult<Chapter> {
    // 获取当前卷下最大的排序序号，新章节的序号为最大值 + 1
    let sort_order: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM chapters WHERE volume_id = ?1",
            [volume_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO chapters (volume_id, title, file_path, sort_order, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 'draft', ?5, ?6)",
        (volume_id, title, file_path, sort_order, &now, &now),
    )?;

    Ok(Chapter {
        id: conn.last_insert_rowid(),
        volume_id,
        title: title.to_string(),
        file_path: file_path.to_string(),
        sort_order,
        summary: String::new(),
        word_count_cache: 0,
        status: "draft".to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

/// 根据 ID 获取章节
/// 
/// 通过章节 ID 查询章节记录。
/// 
/// # 参数
/// - `conn`: SQLite 数据库连接
/// - `chapter_id`: 章节 ID
/// 
/// # 返回值
/// 找到返回 `Ok(Some(Chapter))`，未找到返回 `Ok(None)`，出错返回 `rusqlite::Error`
pub fn get_chapter_by_id(conn: &Connection, chapter_id: i64) -> SqliteResult<Option<Chapter>> {
    conn.query_row(
        "SELECT id, volume_id, title, file_path, sort_order, summary, word_count_cache, status, created_at, updated_at
         FROM chapters WHERE id = ?1",
        [chapter_id],
        |row| {
            Ok(Chapter {
                id: row.get(0)?,
                volume_id: row.get(1)?,
                title: row.get(2)?,
                file_path: row.get(3)?,
                sort_order: row.get(4)?,
                summary: row.get(5)?,
                word_count_cache: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        },
    ).optional()
}

/// 获取项目的章节树结构
/// 
/// 查询指定项目下所有卷及其包含的章节，返回树形结构。
/// 使用 LEFT JOIN 确保即使卷下没有章节也能返回卷信息。
/// 
/// # 参数
/// - `conn`: SQLite 数据库连接
/// - `project_id`: 项目 ID
/// 
/// # 返回值
/// 卷及其章节的树形结构列表 `Ok(Vec<VolumeWithChapters>)`
pub fn get_chapter_tree(conn: &Connection, project_id: i64) -> SqliteResult<Vec<VolumeWithChapters>> {
    let mut stmt = conn.prepare(
        "SELECT v.id, v.project_id, v.name, v.sort_order,
                c.id, c.volume_id, c.title, c.file_path, c.sort_order, c.word_count_cache, c.status, c.created_at, c.updated_at
         FROM volumes v
         LEFT JOIN chapters c ON c.volume_id = v.id
         WHERE v.project_id = ?1
         ORDER BY v.sort_order, c.sort_order",
    )?;

    let mut volumes_map: std::collections::HashMap<i64, VolumeWithChapters> = std::collections::HashMap::new();

    let rows = stmt.query_map([project_id], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i32>(3)?,
            row.get::<_, Option<i64>>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<String>>(6)?,
            row.get::<_, Option<i32>>(7)?,
            row.get::<_, Option<i32>>(8)?,
            row.get::<_, Option<String>>(9)?,
            row.get::<_, Option<String>>(10)?,
            row.get::<_, Option<String>>(11)?,
        ))
    })?;

    // 使用 HashMap 聚合卷和章节数据
    for row in rows.flatten() {
        let (vid, pid, name, order, cid, ctitle, cpath, corder, cwc, cstatus, cat, cup) = row;

        // 如果卷不存在于 map 中，先创建卷
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

        // 如果存在章节数据，添加到卷的章节列表中
        if let (Some(id), Some(title), Some(ch_order)) = (cid, ctitle, corder) {
            if let Some(vol) = volumes_map.get_mut(&vid) {
                vol.chapters.push(Chapter {
                    id,
                    volume_id: vid,
                    title,
                    file_path: cpath.unwrap_or_default(),
                    sort_order: ch_order,
                    summary: String::new(),
                    word_count_cache: cwc.unwrap_or(0),
                    status: cstatus.unwrap_or_else(|| "draft".to_string()),
                    created_at: cat.unwrap_or_default(),
                    updated_at: cup.unwrap_or_default(),
                });
            }
        }
    }

    // 将 HashMap 转换为排序后的 Vec
    let mut volumes: Vec<VolumeWithChapters> = volumes_map.into_values().collect();
    volumes.sort_by(|a, b| a.sort_order.cmp(&b.sort_order));

    Ok(volumes)
}

/// 更新章节内容（字数缓存）
/// 
/// 更新章节的字数缓存和最后更新时间。注意：此函数不更新实际的文件内容，
/// 只更新数据库中的字数统计缓存。
/// 
/// # 参数
/// - `conn`: SQLite 数据库连接
/// - `chapter_id`: 章节 ID
/// - `content`: 章节内容（用于计算字数）
/// 
/// # 返回值
/// 更新成功返回 `Ok(())`，失败返回 `rusqlite::Error`
pub fn update_chapter_content(conn: &Connection, chapter_id: i64, content: &str) -> SqliteResult<()> {
    let now = Utc::now().to_rfc3339();
    // 计算非空白字符数量作为字数统计
    let word_count = content.chars().filter(|c| !c.is_whitespace()).count() as i32;

    conn.execute(
        "UPDATE chapters SET word_count_cache = ?1, updated_at = ?2 WHERE id = ?3",
        (word_count, &now, chapter_id),
    )?;

    Ok(())
}

/// 删除章节
/// 
/// 从数据库中删除指定章节记录。注意：此操作会触发外键级联删除。
/// 
/// # 参数
/// - `conn`: SQLite 数据库连接
/// - `chapter_id`: 章节 ID
/// 
/// # 返回值
/// 返回删除的行数 `Ok(usize)`，失败返回 `rusqlite::Error`
pub fn delete_chapter(conn: &Connection, chapter_id: i64) -> SqliteResult<usize> {
    conn.execute("DELETE FROM chapters WHERE id = ?1", [chapter_id])
}

/// 删除卷
/// 
/// 从数据库中删除指定卷记录。注意：此操作会触发外键级联删除，
/// 该卷下的所有章节也会被删除。
/// 
/// # 参数
/// - `conn`: SQLite 数据库连接
/// - `volume_id`: 卷 ID
/// 
/// # 返回值
/// 返回删除的行数 `Ok(usize)`，失败返回 `rusqlite::Error`
pub fn delete_volume(conn: &Connection, volume_id: i64) -> SqliteResult<usize> {
    conn.execute("DELETE FROM volumes WHERE id = ?1", [volume_id])
}
