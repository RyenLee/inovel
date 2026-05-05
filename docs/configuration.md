# 配置文件方案说明

## 概述

本文档描述 inovel 项目的后端配置管理方案，将数据库路径、日志路径等关键路径从硬编码改为通过独立的 JSON 配置文件进行管理。

## 配置文件格式

### 文件位置

配置文件位于应用的 **AppData** 目录下：

- **Windows**: `%APPDATA%\inovel\config.json`
- **macOS**: `~/Library/Application Support/inovel/config.json`
- **Linux**: `~/.config/inovel/config.json`

首次启动应用时，如果配置文件不存在，将自动创建默认配置文件。

### JSON 结构

```json
{
  "version": "1.0",
  "database": {
    "path": "${APP_DATA}/inovel.db",
    "backup_dir": "${APP_DATA}/backups",
    "auto_backup": true,
    "vacuum_on_close": false
  },
  "paths": {
    "projects_root": "${APP_DATA}/projects",
    "exports_dir": "${APP_DATA}/exports"
  },
  "logging": {
    "level": "info",
    "directory": "${APP_DATA}/logs",
    "max_file_size_mb": 50,
    "retention_days": 30,
    "backup_log_enabled": true
  },
  "backup": {
    "enabled": true,
    "backup_dir": "${APP_DATA}/backups",
    "max_backups_per_project": 10,
    "incremental_threshold_hours": 24
  }
}
```

## 配置项说明

### database

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `path` | string | `${APP_DATA}/inovel.db` | SQLite 数据库文件路径 |
| `backup_dir` | string | `${APP_DATA}/backups` | 数据库备份目录 |
| `auto_backup` | boolean | `true` | 启动时是否自动备份数据库 |
| `vacuum_on_close` | boolean | `false` | 关闭连接时是否执行 VACUUM |

### paths

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `projects_root` | string | `${APP_DATA}/projects` | 项目文件夹根目录 |
| `exports_dir` | string | `${APP_DATA}/exports` | 导出文件目录 |

### logging

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `level` | string | `info` | 日志级别: trace, debug, info, warn, error |
| `directory` | string | `${APP_DATA}/logs` | 日志文件目录 |
| `max_file_size_mb` | number | `50` | 单个日志文件最大大小（MB） |
| `retention_days` | number | `30` | 日志保留天数 |
| `backup_log_enabled` | boolean | `true` | 是否启用备份日志 |

### backup

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `enabled` | boolean | `true` | 是否启用备份功能 |
| `backup_dir` | string | `${APP_DATA}/backups` | 备份文件存储目录 |
| `max_backups_per_project` | number | `10` | 每个项目最大备份数量 |
| `incremental_threshold_hours` | number | `24` | 增量备份阈值（小时） |

## 环境变量支持

配置文件支持以下环境变量替换语法：

### 预定义变量

| 变量 | 说明 |
|------|------|
| `${APP_DATA}` | 应用数据目录（Tauri 自动解析） |
| `${HOME}` | 用户主目录 |
| `${TEMP}` | 系统临时目录 |
| `${USERNAME}` | 当前用户名 |

### 自定义变量

支持 `${VAR_NAME}` 格式的自定义环境变量引用：

```json
{
  "database": {
    "path": "${CUSTOM_DB_PATH}/inovel.db"
  }
}
```

### 语法说明

- **标准格式**: `${VAR_NAME}` - 推荐使用
- **简写格式**: `$VAR_NAME` - 仅支持大写字母、数字和下划线

## Rust API

### 核心函数

```rust
// 初始化配置系统（应用启动时调用）
pub fn init_config(app_handle: &AppHandle) -> Result<(), String>

// 获取配置（需先调用 init_config）
pub fn get_config() -> Result<AppConfig, String>

// 获取配置并解析所有路径中的环境变量
pub fn get_config_with_expanded_paths(app_handle: &AppHandle) -> Result<ExpandedConfig, String>

// 获取各路径的便捷函数
pub fn get_db_path(app_handle: &AppHandle) -> Result<PathBuf, String>
pub fn get_projects_root(app_handle: &AppHandle) -> Result<PathBuf, String>
pub fn get_exports_dir(app_handle: &AppHandle) -> Result<PathBuf, String>
pub fn get_log_dir(app_handle: &AppHandle) -> Result<PathBuf, String>
pub fn get_backup_dir(app_handle: &AppHandle) -> Result<PathBuf, String>
```

### Tauri 命令

| 命令 | 说明 | 返回值 |
|------|------|--------|
| `get_app_config` | 获取当前配置 | `ConfigInfo` |
| `update_app_config` | 更新配置（部分更新） | `ConfigInfo` |
| `reset_app_config` | 重置为默认配置 | `ConfigInfo` |
| `get_config_file_path` | 获取配置文件路径 | `string` |
| `validate_config_paths` | 验证所有配置路径是否可访问 | `PathValidationResult[]` |

## 使用示例

### 前端调用示例（TypeScript）

```typescript
import { invoke } from '@tauri-apps/api/core';

// 获取当前配置
const config = await invoke<ConfigInfo>('get_app_config');
console.log('数据库路径:', config.expanded_paths.database_path);

// 更新数据库路径
await invoke('update_app_config', {
  updates: {
    database: {
      path: 'D:/my-data/inovel.db'
    }
  }
});

// 验证路径
const results = await invoke<PathValidationResult[]>('validate_config_paths');
results.forEach(r => {
  console.log(`${r.key}: ${r.exists ? '存在' : '不存在'}, ${r.writable ? '可写' : '不可写'}`);
});
```

### 后端 Rust 使用示例

```rust
use crate::config::{get_db_path, get_projects_root, get_config_with_expanded_paths};

// 获取数据库路径
let db_path = get_db_path(&app_handle)?;

// 获取项目根目录
let projects_dir = get_projects_root(&app_handle)?;

// 获取展开后的完整配置
let config = get_config_with_expanded_paths(&app_handle)?;
println!("日志目录: {}", config.logging.directory.display());
```

## 部署与迁移

### 场景一：单机部署

默认配置即可满足，无需修改配置文件。

### 场景二：多数据目录

将数据库和备份存储到不同位置：

```json
{
  "database": {
    "path": "D:/app-data/inovel.db",
    "backup_dir": "D:/app-data/backups"
  },
  "paths": {
    "projects_root": "D:/novels/projects",
    "exports_dir": "D:/novels/exports"
  },
  "backup": {
    "backup_dir": "D:/novels/backups"
  }
}
```

### 场景三：项目迁移

迁移到新服务器时，只需：

1. 复制整个配置文件目录
2. 修改配置文件中各路径指向新的数据目录
3. 重启应用

示例迁移脚本：

```bash
# Windows PowerShell
$configPath = "$env:APPDATA\inovel\config.json"
$config = Get-Content $configPath | ConvertFrom-Json

# 修改路径
$config.database.path = "E:\new-data\inovel.db"
$config.paths.projects_root = "E:\new-novels\projects"

# 保存
$config | ConvertTo-Json | Set-Content $configPath
```

### 场景四：便携模式（USB 驱动器）

使用相对路径或可移动驱动器路径：

```json
{
  "database": {
    "path": "./data/inovel.db",
    "backup_dir": "./data/backups"
  },
  "paths": {
    "projects_root": "./projects",
    "exports_dir": "./exports"
  },
  "logging": {
    "directory": "./logs"
  }
}
```

## 故障排除

### 配置文件损坏

如果配置文件格式错误，应用将使用默认配置并重新生成配置文件。

### 路径不存在

`validate_config_paths` 命令会检查所有路径的可访问性，包括尝试创建不存在的目录。

### 权限问题

确保应用有权限写入配置文件和配置的目录。

## 版本历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-05-05 | 初始版本，支持数据库、日志、备份路径配置 |
