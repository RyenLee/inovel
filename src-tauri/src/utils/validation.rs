use crate::error::{AppError, Result};

pub fn validate_project_name(name: &str) -> Result<()> {
    let name = name.trim();
    
    if name.is_empty() {
        return Err(AppError::validation("项目名称不能为空"));
    }
    
    if name.len() > 100 {
        return Err(AppError::validation("项目名称不能超过100个字符"));
    }
    
    if !name.chars().all(|c| c.is_alphanumeric() || c.is_whitespace() || "._-".contains(c)) {
        return Err(AppError::validation("项目名称包含非法字符"));
    }
    
    Ok(())
}

pub fn validate_chapter_title(title: &str) -> Result<()> {
    let title = title.trim();
    
    if title.is_empty() {
        return Err(AppError::validation("章节标题不能为空"));
    }
    
    if title.len() > 200 {
        return Err(AppError::validation("章节标题不能超过200个字符"));
    }
    
    Ok(())
}

pub fn validate_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(AppError::validation("路径不能为空"));
    }
    
    for c in path.chars() {
        match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => {
                return Err(AppError::validation(format!("路径包含非法字符: {}", c)));
            }
            _ => {}
        }
    }
    
    Ok(())
}

pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' | ' ' => '_',
            _ => c,
        })
        .collect::<String>()
}
