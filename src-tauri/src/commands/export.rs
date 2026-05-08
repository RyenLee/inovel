use crate::commands::git_snapshot::get_project_folder_path;
use crate::db::{get_db_path, init_db};
use crate::logging::operation::record_simple_operation;
use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};
use regex::Regex;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

/// 获取导出目录路径
///
/// 创建并返回项目的导出目录路径（项目根目录下的 exports 文件夹）。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回导出目录路径，失败返回错误信息
fn get_export_dir(app_handle: &AppHandle, project_id: i64) -> Result<PathBuf, String> {
    let proj_path = get_project_folder_path(app_handle, project_id)?;
    let export_dir = proj_path.join("exports");
    fs::create_dir_all(&export_dir).map_err(|e| format!("创建导出目录失败: {}", e))?;
    Ok(export_dir)
}

/// 获取项目的所有章节内容
///
/// 按卷和章节的排序顺序获取所有章节的标题和内容。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回章节列表（每项为标题-内容元组），失败返回错误信息
fn get_all_chapters(
    app_handle: &AppHandle,
    project_id: i64,
) -> Result<Vec<(String, String)>, String> {
    let db_path = get_db_path(app_handle);
    let conn = Connection::open(&db_path).map_err(|e| format!("数据库连接失败: {}", e))?;
    init_db(&conn).map_err(|e| format!("数据库初始化失败: {}", e))?;

    let _project_path = get_project_folder_path(app_handle, project_id)?;

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
            .query_map([vid], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
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

// ======================== HTML 清洗工具函数 ========================

fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if !in_tag {
            if chars[i] == '<' {
                in_tag = true;
            } else {
                result.push(chars[i]);
            }
        } else if chars[i] == '>' {
            in_tag = false;
        }
        i += 1;
    }

    result
}

fn normalize_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_newline = false;

    for line in s.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !prev_newline {
                result.push('\n');
                prev_newline = true;
            }
        } else {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(trimmed);
            prev_newline = false;
        }
    }

    result.trim().to_string()
}

/// 规范化空白并保留适当的换行符
///
/// 与 normalize_whitespace 不同，此函数保留段落之间的空行（多个连续换行符），
/// 同时去除多余的行尾空格和连续空格。
fn normalize_whitespace_with_newlines(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut consecutive_newlines = 0;

    for c in s.chars() {
        match c {
            '\n' => {
                consecutive_newlines += 1;
                // 保留最多两个连续换行符（空行）
                if consecutive_newlines <= 2 {
                    result.push(c);
                }
            }
            ' ' | '\t' => {
                // 只在行首或非空白字符后添加单个空格
                if !result.ends_with(|c: char| c.is_whitespace() && c != '\n') {
                    result.push(' ');
                }
                consecutive_newlines = 0;
            }
            _ => {
                result.push(c);
                consecutive_newlines = 0;
            }
        }
    }

    // 移除首尾空白，但保留开头的单个换行符
    let trimmed = result.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        trimmed.to_string()
    }
}

fn html_to_plain_text(html: &str) -> String {
    let mut text = html.to_string();

    // 处理换行标签：<br> 转换为换行符
    text = text
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n");

    // 处理段落标签：<p> 转换为空行分隔
    text = replace_tag_content(&text, "<p>", "</p>", |inner| {
        format!("\n{}\n", inner.trim())
    });

    // 处理列表项：添加换行和缩进
    text = replace_tag_content(&text, "<li>", "</li>", |inner| {
        format!("\n  - {}\n", inner.trim())
    });

    // 处理无序列表：添加空行分隔
    text = text.replace("<ul>", "\n").replace("</ul>", "\n");
    text = text.replace("<ol>", "\n").replace("</ol>", "\n");

    // 处理标题标签
    for level in 1..=6 {
        let open = format!("<h{}>", level);
        let close = format!("</h{}>", level);
        let prefix = "#".repeat(level);
        text = replace_tag_content(&text, &open, &close, |inner| {
            format!("\n\n{} {}\n\n", prefix, inner.trim())
        });
    }

    // 处理引用块
    text = replace_tag_content(&text, "<blockquote>", "</blockquote>", |inner| {
        format!("\n> {}\n", inner.trim())
    });

    // 处理代码块
    text = replace_tag_content(&text, "<pre>", "</pre>", |inner| {
        let code = strip_html_tags(inner);
        format!("\n{}\n", code)
    });

    // 处理分割线
    text = text
        .replace("<hr>", "\n---\n")
        .replace("<hr/>", "\n---\n")
        .replace("<hr />", "\n---\n");

    // 移除剩余的 HTML 标签
    text = strip_html_tags(&text);

    // 解码 HTML 实体
    text = decode_html_entities(&text);

    // 规范化空白，保留适当的换行
    normalize_whitespace_with_newlines(&text)
}

fn html_to_markdown(html: &str) -> String {
    let mut md = html.to_string();

    // 换行标签
    md = md
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n");

    // 分割线
    md = md
        .replace("<hr>", "\n---\n")
        .replace("<hr/>", "\n---\n")
        .replace("<hr />", "\n---\n");

    // 标题: <h1> → # , <h2> → ## , ...
    for level in (1..=6).rev() {
        let open = format!("<h{}>", level);
        let close = format!("</h{}>", level);
        let prefix = "#".repeat(level) + " ";
        md = replace_tag_content(&md, &open, &close, |inner| {
            format!("{}{}", prefix, inner.trim())
        });
    }

    // 粗体/斜体
    md = replace_tag_content(&md, "<strong>", "</strong>", |inner| {
        format!("**{}**", inner)
    });
    md = replace_tag_content(&md, "<b>", "</b>", |inner| format!("**{}**", inner));
    md = replace_tag_content(&md, "<em>", "</em>", |inner| format!("*{}*", inner));
    md = replace_tag_content(&md, "<i>", "</i>", |inner| format!("*{}*", inner));

    // 行内代码
    md = replace_tag_content(&md, "<code>", "</code>", |inner| format!("`{}`", inner));

    // 代码块
    md = replace_tag_content(&md, "<pre>", "</pre>", |inner| {
        let code = strip_html_tags(inner);
        format!("\n```\n{}\n```\n", code.trim())
    });

    // 引用块
    md = replace_tag_content(&md, "<blockquote>", "</blockquote>", |inner| {
        let stripped = html_to_markdown(inner);
        stripped
            .lines()
            .map(|l| format!("> {}", l))
            .collect::<Vec<_>>()
            .join("\n")
    });

    // 无序列表
    md = replace_tag_content(&md, "<ul>", "</ul>", |inner| process_list_items(inner, "-"));
    // 有序列表
    md = replace_tag_content(&md, "<ol>", "</ol>", |inner| {
        process_list_items(inner, "1.")
    });

    // 列表项（独立出现时）
    md = replace_tag_content(&md, "<li>", "</li>", |inner| format!("- {}", inner.trim()));

    // 链接
    let link_re = Regex::new(r#"<a\s+[^>]*href="([^"]*)"[^>]*>(.*?)</a>"#).unwrap();
    md = link_re
        .replace_all(&md, |caps: &regex::Captures| {
            format!("[{}]({})", &caps[2], &caps[1])
        })
        .to_string();

    // 图片
    let img_re = Regex::new(r#"<img\s+[^>]*src="([^"]*)"[^>]*alt="([^"]*)"[^>]*/?>"#).unwrap();
    md = img_re
        .replace_all(&md, |caps: &regex::Captures| {
            format!("![{}]({})", &caps[2], &caps[1])
        })
        .to_string();
    let img_re2 = Regex::new(r#"<img\s+[^>]*src="([^"]*)"[^>]*/?>"#).unwrap();
    md = img_re2
        .replace_all(&md, |caps: &regex::Captures| format!("![]({})", &caps[1]))
        .to_string();

    // 段落: <p>...</p> → 内容 + 双换行
    md = replace_tag_content(&md, "<p>", "</p>", |inner| format!("{}\n", inner.trim()));

    // 清除所有残留 HTML 标签
    md = strip_html_tags(&md);

    // 解码 HTML 实体
    md = decode_html_entities(&md);

    // 规范化空白
    normalize_whitespace(&md)
}

fn replace_tag_content<F>(html: &str, open: &str, close: &str, transform: F) -> String
where
    F: Fn(&str) -> String,
{
    let mut result = String::with_capacity(html.len());
    let mut remaining = html;

    loop {
        match remaining.find(open) {
            None => {
                result.push_str(remaining);
                break;
            }
            Some(start) => {
                result.push_str(&remaining[..start]);
                let after_open = &remaining[start + open.len()..];
                match after_open.find(close) {
                    None => {
                        result.push_str(remaining);
                        break;
                    }
                    Some(end) => {
                        let inner = &after_open[..end];
                        result.push_str(&transform(inner));
                        remaining = &after_open[end + close.len()..];
                    }
                }
            }
        }
    }

    result
}

fn process_list_items(inner: &str, prefix: &str) -> String {
    let mut result = String::new();
    let mut counter = 1;
    for item in inner.split("</li>") {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        let content = strip_html_tags(item).trim().to_string();
        if content.is_empty() {
            continue;
        }
        if prefix == "1." {
            result.push_str(&format!("{}. {}\n", counter, content));
            counter += 1;
        } else {
            result.push_str(&format!("{} {}\n", prefix, content));
        }
    }
    result
}

// ======================== 导出命令 ========================

/// 导出为 TXT 文本文件
///
/// 将所有章节内容合并导出为纯文本文件，每个章节之间用分隔符隔开。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回导出文件路径，失败返回错误信息
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
        text.push_str(&html_to_plain_text(content));
    }

    let output_path = export_dir.join("export.txt");
    fs::write(&output_path, &text).map_err(|e| format!("写入文件失败: {}", e))?;

    let _ = record_simple_operation(
        &app_handle,
        "export",
        "txt",
        "project",
        Some(project_id),
        Some("导出TXT格式"),
        Some(project_id),
    );

    Ok(output_path.to_string_lossy().to_string())
}

/// 导出为 Markdown 文件
///
/// 将所有章节内容转换为 Markdown 格式并导出，支持目录生成。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回导出文件路径，失败返回错误信息
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
        md.push_str(&format!("## {}\n\n{}", title, html_to_markdown(content)));
    }

    let output_path = export_dir.join("export.md");
    fs::write(&output_path, &md).map_err(|e| format!("写入文件失败: {}", e))?;
    Ok(output_path.to_string_lossy().to_string())
}

/// 获取导出内容（用于前端预览）
///
/// 返回所有章节的 Markdown 格式内容，不写入文件。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回 Markdown 内容，失败返回错误信息
#[tauri::command]
pub async fn get_export_content(app_handle: AppHandle, project_id: i64) -> Result<String, String> {
    let chapters = get_all_chapters(&app_handle, project_id)?;
    let mut md = String::new();

    for (i, (title, content)) in chapters.iter().enumerate() {
        if i > 0 {
            md.push_str("\n\n---\n\n");
        }
        md.push_str(&format!("## {}\n\n{}", title, html_to_markdown(content)));
    }
    Ok(md)
}

/// 导出为打印友好 HTML
///
/// 生成适合打印的 HTML 文件，包含分页样式（A4纸张）。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回导出文件路径，失败返回错误信息
#[tauri::command]
pub async fn export_html_for_print(
    app_handle: AppHandle,
    project_id: i64,
) -> Result<String, String> {
    let export_dir = get_export_dir(&app_handle, project_id)?;
    let chapters = get_all_chapters(&app_handle, project_id)?;

    let mut body = String::new();
    for (i, (title, content)) in chapters.iter().enumerate() {
        if i > 0 {
            body.push_str("<hr>\n");
        }
        body.push_str(&format!("<h2>{}</h2>\n", title));
        body.push_str(content);
    }

    let title = chapters
        .first()
        .map(|(t, _)| t.clone())
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

/// 导出为 EPUB 电子书格式
///
/// 生成符合标准的 EPUB 格式电子书，包含封面、目录和章节内容。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回导出文件路径，失败返回错误信息
#[tauri::command]
pub async fn export_epub(app_handle: AppHandle, project_id: i64) -> Result<String, String> {
    let export_dir = get_export_dir(&app_handle, project_id)?;
    let project_path = get_project_folder_path(&app_handle, project_id)?;
    let chapters = get_all_chapters(&app_handle, project_id)?;

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

    builder
        .metadata("title", &project_name)
        .map_err(|e| format!("设置标题失败: {}", e))?;
    builder
        .metadata("author", &author)
        .map_err(|e| format!("设置作者失败: {}", e))?;

    let cover_path = project_path.join("cover.jpg");
    if cover_path.exists() {
        let cover_data = fs::read(&cover_path).map_err(|e| format!("读取封面失败: {}", e))?;
        builder
            .add_cover_image("cover.jpg", cover_data.as_slice(), "image/jpeg")
            .map_err(|e| format!("添加封面失败: {}", e))?;
    }

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

    for (i, (title, content)) in chapters.iter().enumerate() {
        let xhtml = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>{}</title></head>
<body><h1>{}</h1>{}</body>
</html>"#,
            title, title, content
        );

        builder
            .add_content(
                EpubContent::new(format!("chapter_{}.xhtml", i), xhtml.as_bytes())
                    .title(title)
                    .reftype(ReferenceType::Text),
            )
            .map_err(|e| format!("添加章节失败: {}", e))?;
    }

    let output_path = export_dir.join("export.epub");
    let output_file = fs::File::create(&output_path).map_err(|e| format!("创建文件失败: {}", e))?;
    builder
        .generate(output_file)
        .map_err(|e| format!("生成 EPUB 失败: {}", e))?;

    Ok(output_path.to_string_lossy().to_string())
}

/// 获取导出目录路径
///
/// 返回项目导出目录的绝对路径。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回目录路径字符串，失败返回错误信息
#[tauri::command]
pub fn get_exports_dir(app_handle: AppHandle, project_id: i64) -> Result<String, String> {
    let dir = get_export_dir(&app_handle, project_id)?;
    dir.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "路径包含无效字符".to_string())
}

/// 在文件管理器中打开导出目录
///
/// 调用系统文件管理器打开项目的导出目录。
/// 仅支持 Windows 系统。
///
/// # 参数
/// - `app_handle`: Tauri 应用句柄
/// - `project_id`: 项目 ID
///
/// # 返回值
/// 成功返回 Ok(())，失败返回错误信息
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
