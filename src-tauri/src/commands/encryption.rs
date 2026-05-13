use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use argon2::{Argon2, Params};
use base64::{Engine as _, engine::general_purpose};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

// ==================== 数据结构定义 ====================

/// 加密进度事件负载
#[derive(Debug, Serialize, Clone)]
pub struct EncryptionProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
}

/// Vault 文件结构（加密后保存为 vault.json.enc）
#[derive(Debug, Serialize, Deserialize)]
pub struct Vault {
    pub salt: String,               // Base64 编码的盐
    pub files: Vec<VaultFileEntry>, // 文件索引
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultFileEntry {
    pub original_path: String,  // 原文件路径（相对项目根目录）
    pub encrypted_path: String, // .enc 文件路径（相对项目根目录）
    pub nonce: String,          // Base64 编码的 nonce
}

/// 加密参数
pub const KEY_SIZE: usize = 32; // 256 位密钥
pub const NONCE_SIZE: usize = 12; // 96 位 nonce（AES-GCM 推荐）
pub const SALT_SIZE: usize = 16; // 128 位盐
pub const VAULT_FILENAME: &str = "vault.json.enc";
pub const DECRYPTED_DIR: &str = ".decrypted";

// ==================== 密钥派生 ====================

/// 使用 Argon2 从密码和盐派生密钥
pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; KEY_SIZE], String> {
    let mut key = [0u8; KEY_SIZE];

    // 使用 Argon2id 变体（推荐），参数设置为中等安全级别
    let params =
        Params::new(65536, 3, 4, Some(KEY_SIZE)).map_err(|e| format!("Argon2 参数错误: {}", e))?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| format!("密钥派生失败: {}", e))?;

    Ok(key)
}

/// 生成随机盐
pub fn generate_salt() -> [u8; SALT_SIZE] {
    let mut salt = [0u8; SALT_SIZE];
    rand::rng().fill_bytes(&mut salt);
    salt
}

/// 生成随机 nonce
pub fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    rand::rng().fill_bytes(&mut nonce);
    nonce
}

// ==================== 加密/解密单文件 ====================

/// 加密单个文件
///
/// # 参数
/// - `input_path`: 输入文件路径
/// - `output_path`: 输出文件路径（.enc 文件）
/// - `key`: 256 位密钥
/// - `nonce`: 96 位 nonce
///
/// # 返回
/// 成功返回 (), 失败返回错误信息
pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    key: &[u8; KEY_SIZE],
    nonce: &[u8; NONCE_SIZE],
) -> Result<(), String> {
    // 读取原文件
    let mut input_data = Vec::new();
    fs::File::open(input_path)
        .map_err(|e| format!("无法打开文件 {}: {}", input_path.display(), e))?
        .read_to_end(&mut input_data)
        .map_err(|e| format!("读取文件失败 {}: {}", input_path.display(), e))?;

    // 初始化 AES-256-GCM
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("初始化加密器失败: {}", e))?;

    let nonce = Nonce::from_slice(nonce);

    // 加密数据
    let ciphertext = Aes256Gcm::encrypt(&cipher, nonce, input_data.as_ref())
        .map_err(|e| format!("加密失败 {}: {}", input_path.display(), e))?;

    // 确保输出目录存在
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建输出目录失败: {}", e))?;
    }

    // 写入加密后的文件
    fs::File::create(output_path)
        .map_err(|e| format!("无法创建输出文件 {}: {}", output_path.display(), e))?
        .write_all(&ciphertext)
        .map_err(|e| format!("写入加密文件失败 {}: {}", output_path.display(), e))?;

    Ok(())
}

/// 解密单个文件
///
/// # 参数
/// - `input_path`: 输入文件路径（.enc 文件）
/// - `output_path`: 输出文件路径（解密后的原文件）
/// - `key`: 256 位密钥
/// - `nonce`: 96 位 nonce
///
/// # 返回
/// 成功返回 (), 失败返回错误信息
pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    key: &[u8; KEY_SIZE],
    nonce: &[u8; NONCE_SIZE],
) -> Result<(), String> {
    // 读取加密文件
    let mut ciphertext = Vec::new();
    fs::File::open(input_path)
        .map_err(|e| format!("无法打开加密文件 {}: {}", input_path.display(), e))?
        .read_to_end(&mut ciphertext)
        .map_err(|e| format!("读取加密文件失败 {}: {}", input_path.display(), e))?;

    // 初始化 AES-256-GCM
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("初始化解密器失败: {}", e))?;

    let nonce = Nonce::from_slice(nonce);

    // 解密数据
    let plaintext = Aes256Gcm::decrypt(&cipher, nonce, ciphertext.as_ref())
        .map_err(|e| format!("解密失败 {}: {}", input_path.display(), e))?;

    // 确保输出目录存在
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建输出目录失败: {}", e))?;
    }

    // 写入解密后的文件
    fs::File::create(output_path)
        .map_err(|e| format!("无法创建输出文件 {}: {}", output_path.display(), e))?
        .write_all(&plaintext)
        .map_err(|e| format!("写入解密文件失败 {}: {}", output_path.display(), e))?;

    Ok(())
}

// ==================== 项目级加密/解密 ====================

/// 加密整个项目（核心函数）
///
/// # 参数
/// - `project_path`: 项目根目录路径
/// - `password`: 用户密码
/// - `app_handle`: Tauri app handle，用于发送进度事件
///
/// # 返回
/// 成功返回 (), 失败返回错误信息
pub fn encrypt_project_inner(
    project_path: &Path,
    password: &str,
    app_handle: &AppHandle,
) -> Result<(), String> {
    // 1. 生成盐
    let salt = generate_salt();

    // 2. 派生密钥
    let key = derive_key(password, &salt)?;

    // 3. 遍历项目文件夹，收集所有文件
    let mut files_to_encrypt: Vec<PathBuf> = Vec::new();
    collect_files(project_path, &mut files_to_encrypt)?;

    let total_files = files_to_encrypt.len();
    if total_files == 0 {
        return Err("项目中没有找到可加密的文件".to_string());
    }

    // 4. 加密每个文件
    let mut vault_entries: Vec<VaultFileEntry> = Vec::new();

    for (index, file_path) in files_to_encrypt.iter().enumerate() {
        // 跳过 vault.json.enc 和 .decrypted 目录
        if file_path
            .file_name()
            .map_or(false, |name| name == VAULT_FILENAME)
        {
            continue;
        }
        if file_path.to_string_lossy().contains(DECRYPTED_DIR) {
            continue;
        }

        // 计算相对路径
        let relative_path = file_path
            .strip_prefix(project_path)
            .map_err(|e| format!("路径计算失败: {}", e))?
            .to_string_lossy()
            .to_string();

        // 生成输出路径（.enc 文件）
        let encrypted_path = file_path.with_extension("enc");
        let encrypted_relative_path = encrypted_path
            .strip_prefix(project_path)
            .map_err(|e| format!("路径计算失败: {}", e))?
            .to_string_lossy()
            .to_string();

        // 生成 nonce
        let nonce = generate_nonce();

        // 加密文件
        encrypt_file(file_path, &encrypted_path, &key, &nonce)?;

        // 删除原文件
        fs::remove_file(file_path)
            .map_err(|e| format!("删除原文件失败 {}: {}", file_path.display(), e))?;

        // 记录到 vault
        vault_entries.push(VaultFileEntry {
            original_path: relative_path,
            encrypted_path: encrypted_relative_path,
            nonce: general_purpose::STANDARD.encode(nonce),
        });

        // 发送进度事件
        let progress = EncryptionProgress {
            current: index + 1,
            total: total_files,
            current_file: file_path.file_name().unwrap().to_string_lossy().to_string(),
        };

        let _ = app_handle.emit("encryption-progress", progress);
    }

    // 5. 创建 vault 并加密保存
    let vault = Vault {
        salt: general_purpose::STANDARD.encode(salt),
        files: vault_entries,
    };

    let vault_json =
        serde_json::to_string_pretty(&vault).map_err(|e| format!("序列化 vault 失败: {}", e))?;

    // 使用密钥加密 vault
    let vault_nonce = generate_nonce();
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("初始化加密器失败: {}", e))?;
    let vault_nonce_slice = Nonce::from_slice(&vault_nonce);
    let encrypted_vault = Aes256Gcm::encrypt(&cipher, vault_nonce_slice, vault_json.as_bytes())
        .map_err(|e| format!("加密 vault 失败: {}", e))?;

    // 保存加密后的 vault（包含 nonce）
    let mut vault_data = vault_nonce.to_vec();
    vault_data.extend_from_slice(&encrypted_vault);

    let vault_path = project_path.join(VAULT_FILENAME);
    fs::write(&vault_path, vault_data).map_err(|e| format!("写入 vault 文件失败: {}", e))?;

    // 6. 更新 project.json，设置 encrypted: true
    update_project_encrypted_status(project_path, true)?;

    Ok(())
}

/// 解密整个项目（核心函数）
///
/// # 参数
/// - `project_path`: 项目根目录路径
/// - `password`: 用户密码
/// - `app_handle`: Tauri app handle，用于发送进度事件
///
/// # 返回
/// 成功返回解密目录路径, 失败返回错误信息
pub fn decrypt_project_inner(
    project_path: &Path,
    password: &str,
    app_handle: &AppHandle,
) -> Result<String, String> {
    // 1. 读取 vault 文件
    let vault_path = project_path.join(VAULT_FILENAME);
    if !vault_path.exists() {
        return Err("项目未加密或 vault 文件丢失".to_string());
    }

    let vault_data = fs::read(&vault_path).map_err(|e| format!("读取 vault 文件失败: {}", e))?;

    if vault_data.len() < NONCE_SIZE {
        return Err("vault 文件损坏".to_string());
    }

    // 2. 提取 salt、nonce 和加密的 vault 数据
    // vault 文件格式 = salt(16字节) + nonce(12字节) + encrypted_data
    let salt = &vault_data[..SALT_SIZE];
    let vault_nonce = &vault_data[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
    let encrypted_vault = &vault_data[SALT_SIZE + NONCE_SIZE..];

    // 4. 派生密钥
    let key = derive_key(password, salt)?;

    // 5. 解密 vault
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("初始化解密器失败: {}", e))?;
    let vault_nonce_slice = Nonce::from_slice(vault_nonce);
    let vault_json = Aes256Gcm::decrypt(&cipher, vault_nonce_slice, encrypted_vault)
        .map_err(|_| "密码错误，无法解密项目".to_string())?;

    let vault: Vault =
        serde_json::from_slice(&vault_json).map_err(|e| format!("解析 vault 失败: {}", e))?;

    // 6. 创建解密目录
    let decrypted_dir = project_path.join(DECRYPTED_DIR);
    if decrypted_dir.exists() {
        fs::remove_dir_all(&decrypted_dir).map_err(|e| format!("清理旧的解密目录失败: {}", e))?;
    }
    fs::create_dir_all(&decrypted_dir).map_err(|e| format!("创建解密目录失败: {}", e))?;

    // 7. 解密每个文件
    let total_files = vault.files.len();

    for (index, entry) in vault.files.iter().enumerate() {
        let encrypted_path = project_path.join(&entry.encrypted_path);
        let nonce = general_purpose::STANDARD
            .decode(&entry.nonce)
            .map_err(|e| format!("解码 nonce 失败: {}", e))?;

        // 确保 nonce 长度正确
        if nonce.len() != NONCE_SIZE {
            return Err(format!("nonce 长度错误: {}", entry.original_path));
        }
        let nonce_array: [u8; NONCE_SIZE] = nonce.try_into().unwrap();

        // 解密到 .decrypted 目录
        let output_path = decrypted_dir.join(&entry.original_path);

        decrypt_file(&encrypted_path, &output_path, &key, &nonce_array)?;

        // 发送进度事件
        let progress = EncryptionProgress {
            current: index + 1,
            total: total_files,
            current_file: entry.original_path.clone(),
        };

        let _ = app_handle.emit("decryption-progress", progress);
    }

    Ok(decrypted_dir.to_string_lossy().to_string())
}

/// 验证密码是否正确（核心函数）
///
/// # 参数
/// - `project_path`: 项目根目录路径
/// - `password`: 用户密码
///
/// # 返回
/// 成功返回 true/false, 失败返回错误信息
pub fn verify_password_inner(project_path: &Path, password: &str) -> Result<bool, String> {
    let vault_path = project_path.join(VAULT_FILENAME);
    if !vault_path.exists() {
        return Ok(false);
    }

    let vault_data = fs::read(&vault_path).map_err(|e| format!("读取 vault 文件失败: {}", e))?;

    if vault_data.len() < SALT_SIZE + NONCE_SIZE {
        return Err("vault 文件损坏".to_string());
    }

    // 提取 salt
    let salt = &vault_data[..SALT_SIZE];
    let vault_nonce = &vault_data[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
    let encrypted_vault = &vault_data[SALT_SIZE + NONCE_SIZE..];

    // 派生密钥
    let key = derive_key(password, salt)?;

    // 尝试解密 vault
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("初始化解密器失败: {}", e))?;
    let vault_nonce_slice = Nonce::from_slice(vault_nonce);

    match Aes256Gcm::decrypt(&cipher, vault_nonce_slice, encrypted_vault) {
        Ok(_) => Ok(true),   // 解密成功，密码正确
        Err(_) => Ok(false), // 解密失败，密码错误
    }
}

/// 修改密码（核心函数）
///
/// # 参数
/// - `project_path`: 项目根目录路径
/// - `old_password`: 旧密码
/// - `new_password`: 新密码
///
/// # 返回
/// 成功返回 (), 失败返回错误信息
pub fn change_password_inner(
    project_path: &Path,
    old_password: &str,
    new_password: &str,
) -> Result<(), String> {
    // 1. 验证旧密码
    if !verify_password_inner(project_path, old_password)? {
        return Err("旧密码错误".to_string());
    }

    // 2. 读取 vault 文件
    let vault_path = project_path.join(VAULT_FILENAME);
    let vault_data = fs::read(&vault_path).map_err(|e| format!("读取 vault 文件失败: {}", e))?;

    let salt = &vault_data[..SALT_SIZE];
    let vault_nonce = &vault_data[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
    let encrypted_vault = &vault_data[SALT_SIZE + NONCE_SIZE..];

    // 3. 使用旧密码解密 vault
    let old_key = derive_key(old_password, salt)?;
    let cipher =
        Aes256Gcm::new_from_slice(&old_key).map_err(|e| format!("初始化解密器失败: {}", e))?;
    let vault_nonce_slice = Nonce::from_slice(vault_nonce);
    let vault_json = Aes256Gcm::decrypt(&cipher, vault_nonce_slice, encrypted_vault)
        .map_err(|_| "旧密码错误，无法解密项目".to_string())?;

    let mut vault: Vault =
        serde_json::from_slice(&vault_json).map_err(|e| format!("解析 vault 失败: {}", e))?;

    // 4. 生成新的盐
    let new_salt = generate_salt();

    // 5. 使用新密码派生新密钥
    let new_key = derive_key(new_password, &new_salt)?;

    // 6. 使用新密钥重新加密 vault
    let new_vault_nonce = generate_nonce();
    let new_cipher =
        Aes256Gcm::new_from_slice(&new_key).map_err(|e| format!("初始化加密器失败: {}", e))?;
    let new_vault_nonce_slice = Nonce::from_slice(&new_vault_nonce);
    let re_encrypted_vault =
        Aes256Gcm::encrypt(&new_cipher, new_vault_nonce_slice, vault_json.as_ref())
            .map_err(|e| format!("重新加密 vault 失败: {}", e))?;

    // 7. 保存新的 vault 文件（格式：new_salt + new_nonce + encrypted_data）
    let mut new_vault_data = new_salt.to_vec();
    new_vault_data.extend_from_slice(&new_vault_nonce);
    new_vault_data.extend_from_slice(&re_encrypted_vault);

    fs::write(&vault_path, new_vault_data).map_err(|e| format!("写入新 vault 文件失败: {}", e))?;

    // 8. 更新 vault 中的 salt（虽然 salt 现在在文件头部，但保留这个字段以兼容）
    vault.salt = general_purpose::STANDARD.encode(new_salt);

    Ok(())
}

// ==================== 辅助函数 ====================

/// 收集项目文件夹中所有需要加密的文件
fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries =
        fs::read_dir(dir).map_err(|e| format!("读取目录失败 {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();

        // 跳过 .decrypted 目录和 vault.json.enc
        if path.file_name().map_or(false, |name| name == DECRYPTED_DIR) {
            continue;
        }
        if path
            .file_name()
            .map_or(false, |name| name == VAULT_FILENAME)
        {
            continue;
        }

        if path.is_dir() {
            collect_files(&path, files)?;
        } else {
            files.push(path);
        }
    }

    Ok(())
}

/// 更新 project.json 中的 encrypted 状态
fn update_project_encrypted_status(project_path: &Path, encrypted: bool) -> Result<(), String> {
    let project_json_path = project_path.join("project.json");

    let json: serde_json::Value = if project_json_path.exists() {
        let content = fs::read_to_string(&project_json_path)
            .map_err(|e| format!("读取 project.json 失败: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("解析 project.json 失败: {}", e))?
    } else {
        // 如果文件不存在，创建一个基本的 JSON 对象
        serde_json::json!({
            "name": "Untitled Project",
            "encrypted": encrypted
        })
    };

    let mut json_obj = if let Some(obj) = json.as_object().cloned() {
        obj
    } else {
        serde_json::json!({}).as_object().cloned().unwrap()
    };

    if encrypted {
        json_obj.insert("encrypted".to_string(), serde_json::Value::Bool(true));
    } else {
        json_obj.remove("encrypted");
    }

    let updated_content = serde_json::to_string_pretty(&serde_json::Value::Object(json_obj))
        .map_err(|e| format!("序列化 project.json 失败: {}", e))?;

    fs::write(&project_json_path, updated_content)
        .map_err(|e| format!("写入 project.json 失败: {}", e))?;

    Ok(())
}

/// 清理解密目录
pub fn cleanup_decrypted_dir(project_path: &Path) -> Result<(), String> {
    let decrypted_dir = project_path.join(DECRYPTED_DIR);

    if decrypted_dir.exists() {
        fs::remove_dir_all(&decrypted_dir).map_err(|e| format!("清理解密目录失败: {}", e))?;
    }

    Ok(())
}

/// 检查项目是否已加密
pub fn is_project_encrypted(project_path: &Path) -> bool {
    let project_json_path = project_path.join("project.json");

    if !project_json_path.exists() {
        return false;
    }

    let content = fs::read_to_string(&project_json_path).ok();
    content
        .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
        .and_then(|json| json.get("encrypted").and_then(|v| v.as_bool()))
        .unwrap_or(false)
}

/// 重新加密项目（将 .decrypted 目录中的文件加密回原位置）（核心函数）
///
/// # 参数
/// - `project_path`: 项目根目录路径
/// - `password`: 用户密码
///
/// # 返回
/// 成功返回 (), 失败返回错误信息
pub fn reencrypt_project_inner(project_path: &Path, password: &str) -> Result<(), String> {
    // 1. 验证密码
    if !verify_password_inner(project_path, password)? {
        return Err("密码错误".to_string());
    }

    // 2. 读取 vault 文件
    let vault_path = project_path.join(VAULT_FILENAME);
    let vault_data = fs::read(&vault_path).map_err(|e| format!("读取 vault 文件失败: {}", e))?;

    let salt = &vault_data[..SALT_SIZE];
    let vault_nonce = &vault_data[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
    let encrypted_vault = &vault_data[SALT_SIZE + NONCE_SIZE..];

    // 3. 解密 vault 获取文件列表
    let key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("初始化解密器失败: {}", e))?;
    let vault_nonce_slice = Nonce::from_slice(vault_nonce);
    let vault_json = Aes256Gcm::decrypt(&cipher, vault_nonce_slice, encrypted_vault)
        .map_err(|_| "密码错误，无法解密项目".to_string())?;

    let vault: Vault =
        serde_json::from_slice(&vault_json).map_err(|e| format!("解析 vault 失败: {}", e))?;

    // 4. 读取 .decrypted 目录中的文件，重新加密
    let decrypted_dir = project_path.join(DECRYPTED_DIR);
    if !decrypted_dir.exists() {
        return Ok(()); // 没有解密目录，无需重新加密
    }

    for entry in &vault.files {
        let decrypted_file = decrypted_dir.join(&entry.original_path);

        if !decrypted_file.exists() {
            continue; // 跳过不存在的文件
        }

        let nonce = general_purpose::STANDARD
            .decode(&entry.nonce)
            .map_err(|e| format!("解码 nonce 失败: {}", e))?;

        if nonce.len() != NONCE_SIZE {
            return Err(format!("nonce 长度错误: {}", entry.original_path));
        }
        let nonce_array: [u8; NONCE_SIZE] = nonce.try_into().unwrap();

        // 加密文件到原位置（.enc 文件）
        let encrypted_path = project_path.join(&entry.encrypted_path);

        encrypt_file(&decrypted_file, &encrypted_path, &key, &nonce_array)?;
    }

    // 5. 清理 .decrypted 目录
    cleanup_decrypted_dir(project_path)?;

    Ok(())
}

// ==================== Tauri 命令 ====================

use crate::models::ChangePasswordParams;
use crate::models::DecryptProjectParams;
use crate::models::EncryptProjectParams;
use crate::models::{
    ChangeGlobalPasswordParams, DisableGlobalEncryptionParams, EnableGlobalEncryptionParams,
    VerifyGlobalPasswordParams,
};

/// Tauri 命令：加密项目
#[tauri::command(rename_all = "snake_case")]
pub async fn encrypt_project(
    app_handle: AppHandle,
    params: EncryptProjectParams,
) -> Result<(), String> {
    // 验证密码
    if params.password.len() < 8 {
        return Err("密码长度至少 8 位".to_string());
    }
    if params.password != params.confirm_password {
        return Err("两次输入的密码不一致".to_string());
    }

    let path = PathBuf::from(&params.project_path);

    // 检查项目是否已经加密
    if is_project_encrypted(&path) {
        return Err("项目已经加密".to_string());
    }

    // 执行加密
    encrypt_project_inner(&path, &params.password, &app_handle)
}

/// Tauri 命令：解密项目
#[tauri::command(rename_all = "snake_case")]
pub async fn decrypt_project(
    app_handle: AppHandle,
    params: DecryptProjectParams,
) -> Result<String, String> {
    let path = PathBuf::from(&params.project_path);

    // 检查项目是否加密
    if !is_project_encrypted(&path) {
        return Err("项目未加密".to_string());
    }

    // 执行解密
    decrypt_project_inner(&path, &params.password, &app_handle)
}

/// Tauri 命令：验证密码
#[tauri::command(rename_all = "snake_case")]
pub async fn verify_project_password(
    project_path: String,
    password: String,
) -> Result<bool, String> {
    let path = PathBuf::from(&project_path);

    // 验证密码
    verify_password_inner(&path, &password)
}

/// Tauri 命令：修改密码
#[tauri::command(rename_all = "snake_case")]
pub async fn change_project_password(params: ChangePasswordParams) -> Result<(), String> {
    // 验证新密码
    if params.new_password.len() < 8 {
        return Err("新密码长度至少 8 位".to_string());
    }
    if params.new_password != params.confirm_password {
        return Err("两次输入的新密码不一致".to_string());
    }

    let path = PathBuf::from(&params.project_path);

    // 修改密码
    change_password_inner(&path, &params.old_password, &params.new_password)
}

/// Tauri 命令：重新加密项目（关闭项目时调用）
#[tauri::command(rename_all = "snake_case")]
pub async fn reencrypt_project(project_path: String, password: String) -> Result<(), String> {
    let path = PathBuf::from(&project_path);

    // 重新加密
    reencrypt_project_inner(&path, &password)
}

/// Tauri 命令：检查项目是否已加密
#[tauri::command(rename_all = "snake_case")]
pub async fn is_project_encrypted_command(project_path: String) -> Result<bool, String> {
    let path = PathBuf::from(&project_path);

    // 检查加密状态
    Ok(is_project_encrypted(&path))
}

use crate::services::global_config;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_global_encryption_status(app_handle: AppHandle) -> Result<bool, String> {
    global_config::ensure_global_config(&app_handle)?;
    global_config::get_encryption_status(&app_handle)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn enable_global_encryption(
    app_handle: AppHandle,
    params: EnableGlobalEncryptionParams,
) -> Result<(), String> {
    if params.password.len() < 8 {
        return Err("密码长度至少 8 位".to_string());
    }
    if params.password != params.confirm_password {
        return Err("两次输入的密码不一致".to_string());
    }

    global_config::ensure_global_config(&app_handle)?;
    global_config::enable_encryption(&app_handle, &params.password)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn disable_global_encryption(
    app_handle: AppHandle,
    params: DisableGlobalEncryptionParams,
) -> Result<(), String> {
    global_config::disable_encryption(&app_handle, &params.password)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn change_global_password(
    app_handle: AppHandle,
    params: ChangeGlobalPasswordParams,
) -> Result<(), String> {
    if params.new_password.len() < 8 {
        return Err("新密码长度至少 8 位".to_string());
    }
    if params.new_password != params.confirm_password {
        return Err("两次输入的新密码不一致".to_string());
    }

    global_config::change_password(&app_handle, &params.old_password, &params.new_password)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn verify_global_password(
    app_handle: AppHandle,
    params: VerifyGlobalPasswordParams,
) -> Result<bool, String> {
    let config = global_config::read_global_config(&app_handle)?;

    if !config.encrypted {
        return Ok(false);
    }

    global_config::verify_password(&params.password, &config.password_hash)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_language_list(
    app_handle: AppHandle,
) -> Result<Vec<global_config::LanguageOption>, String> {
    global_config::get_language_list(&app_handle)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_locale(app_handle: AppHandle) -> Result<String, String> {
    global_config::get_locale(&app_handle)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_locale(app_handle: AppHandle, locale: String) -> Result<(), String> {
    global_config::set_locale(&app_handle, &locale)
}
