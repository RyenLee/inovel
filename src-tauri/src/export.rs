use crate::db::{get_db_path, init_db};
use crate::git_snapshot::get_project_folder_path;
use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

fn get_export_dir(app_handle: &AppHandle, project_id: i64) -> Result<PathBuf, String> {
    let proj_path = get_project_folder_path(app_handle, project_id)?;
    let export_dir = proj_path.join("exports");
    fs::create_dir_all(&export_dir).map_err(|e| format!("创建导出目录失败: {}", e))?;
    Ok(export_dir)
}

/// 获取项目的所有章节（按卷→章节排序）
fn get_all_chapters(app_handle: &AppHandle, project_id: i64) -> Result<Vec<(String, String)>, String> {
    let db_path = get_db_path(app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    let _project_path = get_project_folder_path(app_handle, project_id)?;

    // 先查卷
    let mut vol_stmt = conn
        .prepare("SELECT id, name FROM volumes WHERE project_id = ?1 ORDER BY sort_order")
        .map_err(|e| format!("查询卷失败: {}", e))?;
    let volumes: Vec<(i64, String)> = vol_stmt
        .query_map([project_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| format!("查询失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    let mut chapters: Vec<(String, String)> = Vec::new();
    for (vid, _vname) in &volumes {
        let mut ch_stmt = conn
            .prepare("SELECT id, title, file_path, sort_order FROM chapters WHERE volume_id = ?1 ORDER BY sort_order")
            .map_err(|e| format!("查询章节失败: {}", e))?;
        let vol_chapters: Vec<(i64, String, String, i32)> = ch_stmt
            .query_map([vid], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))
            .map_err(|e| format!("查询失败: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        for (_, title, file_path, _) in &vol_chapters {
            let full_path = if file_path.is_empty() {
                continue;
            } else {
                Path::new(file_path)
            };
            let content = if full_path.exists() {
                fs::read_to_string(full_path).unwrap_or_default()
            } else {
                String::new()
            };
            chapters.push((title.clone(), content));
        }
    }
    Ok(chapters)
}

/// 导出为纯文本
#[tauri::command]
pub async fn export_txt(app_handle: AppHandle, project_id: i64) -> Result<String, String> {
    let export_dir = get_export_dir(&app_handle, project_id)?;
    let chapters = get_all_chapters(&app_handle, project_id)?;

    let mut text = String::new();
    for (i, (title, content)) in chapters.iter().enumerate() {
        if i > 0 {
            text.push_str("\n\n\n==========\n\n\n");
        }
        text.push_str(&format!("# {}\n\n", title));
        text.push_str(content);
    }

    let output_path = export_dir.join("export.txt");
    fs::write(&output_path, &text).map_err(|e| format!("写入文件失败: {}", e))?;
    Ok(output_path.to_string_lossy().to_string())
}

/// 导出为 Markdown（含 TOC）
#[tauri::command]
pub async fn export_markdown(app_handle: AppHandle, project_id: i64) -> Result<String, String> {
    let export_dir = get_export_dir(&app_handle, project_id)?;
    let chapters = get_all_chapters(&app_handle, project_id)?;

    let mut md = String::new();
    md.push_str("# 目录\n\n[TOC]\n\n---\n\n");

    for (i, (title, content)) in chapters.iter().enumerate() {
        if i > 0 {
            md.push_str("\n\n---\n\n");
        }
        md.push_str(&format!("## {}\n\n{}", title, content));
    }

    let output_path = export_dir.join("export.md");
    fs::write(&output_path, &md).map_err(|e| format!("写入文件失败: {}", e))?;
    Ok(output_path.to_string_lossy().to_string())
}

/// 获取导出用的 Markdown 内容（供前端 PDF 导出使用）
#[tauri::command]
pub async fn get_export_content(app_handle: AppHandle, project_id: i64) -> Result<String, String> {
    let chapters = get_all_chapters(&app_handle, project_id)?;
    let mut md = String::new();

    for (i, (title, content)) in chapters.iter().enumerate() {
        if i > 0 {
            md.push_str("\n\n---\n\n");
        }
        md.push_str(&format!("## {}\n\n{}", title, content));
    }
    Ok(md)
}

/// 导出为 HTML（用于后续打印为 PDF）
#[tauri::command]
pub async fn export_html_for_print(app_handle: AppHandle, project_id: i64) -> Result<String, String> {
    let export_dir = get_export_dir(&app_handle, project_id)?;
    let md = get_export_content(app_handle, project_id).await?;

    // 简单 Markdown → HTML 转换（与前端保持一致）
    let body = markdown_to_html_simple(&md);

    // 提取标题
    let title = md
        .lines()
        .find(|l| l.starts_with("## "))
        .map(|l| l.trim_start_matches("## ").to_string())
        .unwrap_or_else(|| "导出文档".to_string());

    let css = r#"
        @page { size: A4; margin: 2cm; }
        body { font-family: "Noto Serif SC", "Source Han Serif SC", "SimSun", serif; line-height: 1.8; max-width: 800px; margin: 0 auto; padding: 2em; color: #333; }
        h1, h2, h3 { font-weight: bold; margin-top: 1.5em; }
        p { text-indent: 2em; margin: 0.5em 0; }
        hr { border: none; border-top: 1px solid #ccc; margin: 2em 0; }
        blockquote { border-left: 3px solid #ccc; padding-left: 1em; margin-left: 0; color: #666; }
        pre { background: #f5f5f5; padding: 1em; border-radius: 4px; overflow-x: auto; white-space: pre-wrap; }
        code { font-family: "Source Code Pro", monospace; }
        li { margin: 0.3em 0; }
    "#;

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head><meta charset="utf-8"><title>{}</title>
<style>{}</style>
</head>
<body>{}</body>
</html>"#,
        title, css, body
    );

    let output_path = export_dir.join("export_print.html");
    fs::write(&output_path, &html).map_err(|e| format!("写入 HTML 文件失败: {}", e))?;
    Ok(output_path.to_string_lossy().to_string())
}

/// 导出为 EPUB
#[tauri::command]
pub async fn export_epub(app_handle: AppHandle, project_id: i64) -> Result<String, String> {
    let export_dir = get_export_dir(&app_handle, project_id)?;
    let project_path = get_project_folder_path(&app_handle, project_id)?;
    let chapters = get_all_chapters(&app_handle, project_id)?;

    // 获取项目元数据
    let (project_name, author): (String, String) = {
        let db_path = get_db_path(&app_handle);
        let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
        conn.query_row(
            "SELECT name, author FROM projects WHERE id = ?1",
            [project_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("查询项目信息失败: {}", e))?
    };

    let zip_lib = ZipLibrary::new().map_err(|e| format!("创建 ZIP 库失败: {}", e))?;
    let mut builder =
        EpubBuilder::new(zip_lib).map_err(|e| format!("创建 EPUB 构建器失败: {}", e))?;

    builder.metadata("title", &project_name).map_err(|e| format!("设置标题失败: {}", e))?;
    builder.metadata("author", &author).map_err(|e| format!("设置作者失败: {}", e))?;

    // 添加封面图片
    let cover_path = project_path.join("cover.jpg");
    if cover_path.exists() {
        let cover_data = fs::read(&cover_path).map_err(|e| format!("读取封面失败: {}", e))?;
        builder
            .add_cover_image("cover.jpg", cover_data.as_slice(), "image/jpeg")
            .map_err(|e| format!("添加封面失败: {}", e))?;
    }

    // 添加 CSS 样式
    let css = r#"
        body { font-family: "Noto Serif SC", "Source Han Serif SC", serif; line-height: 1.8; padding: 1em; }
        h1, h2, h3, h4 { font-weight: bold; margin-top: 1.5em; margin-bottom: 0.5em; }
        p { text-indent: 2em; margin: 0.5em 0; }
        blockquote { border-left: 3px solid #ccc; padding-left: 1em; margin-left: 0; color: #666; }
        pre { background: #f5f5f5; padding: 1em; border-radius: 4px; overflow-x: auto; }
        code { font-family: "Source Code Pro", monospace; }
    "#;
    builder
        .stylesheet(css.as_bytes())
        .map_err(|e| format!("添加 CSS 失败: {}", e))?;

    // 添加各章节
    for (i, (title, content)) in chapters.iter().enumerate() {
        let xhtml = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>{}</title></head>
<body><h1>{}</h1>{}</body>
</html>"#,
            title,
            title,
            markdown_to_html_simple(content)
        );

        builder
            .add_content(
                EpubContent::new(
                    format!("chapter_{}.xhtml", i),
                    xhtml.as_bytes(),
                )
                .title(title)
                .reftype(ReferenceType::Text),
            )
            .map_err(|e| format!("添加章节失败: {}", e))?;
    }

    let output_path = export_dir.join("export.epub");
    let output_file =
        fs::File::create(&output_path).map_err(|e| format!("创建文件失败: {}", e))?;
    builder
        .generate(output_file)
        .map_err(|e| format!("生成 EPUB 失败: {}", e))?;

    Ok(output_path.to_string_lossy().to_string())
}

/// 简单将 Markdown 转为 XHTML（用于 EPUB）
fn markdown_to_html_simple(md: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;

    for line in md.lines() {
        if line.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>\n");
            } else {
                html.push_str("<pre><code>\n");
            }
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            html.push_str(&escape_html(line));
            html.push('\n');
            continue;
        }
        if line.is_empty() {
            html.push_str("<p></p>\n");
            continue;
        }
        if line.starts_with("### ") {
            html.push_str(&format!("<h3>{}</h3>\n", &line[4..]));
        } else if line.starts_with("## ") {
            html.push_str(&format!("<h2>{}</h2>\n", &line[3..]));
        } else if line.starts_with("# ") {
            html.push_str(&format!("<h1>{}</h1>\n", &line[2..]));
        } else if line.starts_with("> ") {
            html.push_str(&format!("<blockquote><p>{}</p></blockquote>\n", &line[2..]));
        } else if line.starts_with("- ") || line.starts_with("* ") {
            html.push_str(&format!("<li>{}</li>\n", &line[2..]));
        } else {
            html.push_str(&format!("<p>{}</p>\n", line));
        }
    }
    html
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[tauri::command]
pub fn get_exports_dir(app_handle: AppHandle, project_id: i64) -> Result<String, String> {
    let dir = get_export_dir(&app_handle, project_id)?;
    dir.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "路径包含无效字符".to_string())
}

#[tauri::command]
pub fn open_folder_in_explorer(app_handle: AppHandle, project_id: i64) -> Result<(), String> {
    let dir = get_export_dir(&app_handle, project_id)?;
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&dir)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = app_handle;
        let _ = dir;
        return Err("当前仅支持 Windows 系统".to_string());
    }
    Ok(())
}
