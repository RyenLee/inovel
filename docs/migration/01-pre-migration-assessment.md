# 数据迁移前期评估报告

> 项目：inovel v0.1.0 | 生成日期：2026-05-05

---

## 1. 环境信息

| 项目 | 值 |
|------|-----|
| 操作系统 | Windows 11 (win32) |
| 应用版本 | 0.1.0 |
| Rust 编译器 | edition 2024 |
| 数据库引擎 | SQLite (rusqlite 0.39.0, bundled) |
| 前端框架 | Vue 3 + TypeScript + Naive UI |
| 桌面框架 | Tauri 2.11.0 |
| Git 库 | git2 0.19 |

---

## 2. 数据库现状分析

### 2.1 表结构清单

数据库中共有 **12 张业务表**：

| # | 表名 | 记录行(预估) | 外键引用目标 | 迁移影响 |
|---|------|------------|------------|---------|
| 1 | projects | 若干 | — (主表) | **直接修改** (project_id, path) |
| 2 | volumes | 若干 | projects.id | 无影响 |
| 3 | chapters | 若干 | volumes.id (间接) | 无影响 |
| 4 | writing_goals | 若干 | projects.id | 无影响 |
| 5 | writing_records | 若干 | projects.id | 无影响 |
| 6 | characters | 若干 | projects.id | 无影响 |
| 7 | locations | 若干 | projects.id | 无影响 |
| 8 | organizations | 若干 | projects.id | 无影响 |
| 9 | relationships | 若干 | projects.id | 无影响 |
| 10 | events | 若干 | projects.id | 无影响 |
| 11 | sensitive_words | 若干 | projects.id | 无影响 |
| 12 | backups | 若干 | projects.id | 无影响 |
| 13 | backup_logs | 若干 | projects.id | 无影响 |

**关键结论**：所有外键引用 **`projects.id`（整数 PK）**，而非 `projects.project_id`（文本 UID）。文件夹重命名 **不会破坏任何外键关系**。

### 2.2 待迁移项目识别

```sql
-- 迁移目标：无 project_id 或 project_id 为空的记录
SELECT id, name, path FROM projects WHERE project_id IS NULL OR project_id = '';
```

### 2.3 数据库文件位置

```rust
// 数据库路径逻辑 (db.rs:7-14)
// {app_data_dir}/inovel.db
// Windows 上通常位于: C:\Users\{user}\AppData\Roaming\inovel\inovel.db
```

---

## 3. 文件系统现状分析

### 3.1 当前文件夹命名规则

- **旧规则**：以书名作为文件夹名（如 `D:/novels/三体/`）
- **新规则**：以项目ID作为文件夹名（如 `D:/novels/P7K3M9/`）

### 3.2 项目目录结构（迁移前）

```
{parent_path}/
  └── {book_name}/
      ├── project.json         # 项目元数据（无 project_id 字段）
      ├── chapters/            # 章节文件目录
      │   ├── v1_c1.md
      │   └── v1_c2.md
      ├── covers/              # 封面目录
      │   └── {book_name}_cover.jpg
      └── .git/                # Git 仓库（可能已初始化）
```

### 3.3 项目目录结构（迁移后）

```
{parent_path}/
  └── {project_id}/            # 如 P7K3M9
      ├── project.json         # 新增 project_id 字段
      ├── chapters/            # 不变
      │   ├── v1_c1.md
      │   └── v1_c2.md
      ├── covers/              # 不变
      │   └── {book_name}_cover.jpg
      └── .git/                # 需要重建引用
```

---

## 4. 当前迁移代码状态评估

### 4.1 已实现的功能

`project.rs:374-436` — `migrate_existing_projects`：

- ✅ 查询无 `project_id` 的项目
- ✅ 生成字母数字组合 ID（如 `P7K3M9`）
- ✅ 使用 `fs::rename` 重命名文件夹
- ✅ 更新 DB 中 `project_id` 和 `path` 字段
- ✅ 更新 `project.json` 中的 `project_id`

### 4.2 需增强的缺失环节

| 缺失项 | 严重程度 | 说明 |
|--------|---------|------|
| ❌ 无迁移前备份 | **高** | 没有自动备份数据库 |
| ❌ 无事务保护 | **高** | 循环中未使用事务，单个项目失败可能造成不一致 |
| ❌ 无目标路径冲突检测 | **中** | 目标路径已存在时直接报错 |
| ❌ 无进度反馈 | **中** | 仅返回计数，前端无法显示进度 |
| ❌ 无迁移日志 | **中** | 没有记录迁移操作历史 |
| ❌ 无 Git 仓库完整性检查 | **低** | 重命名后 Git 引用可能断裂 |
| ❌ 无回滚能力 | **高** | 没有提供回滚机制 |
| ❌ 无幂等校验 | **中** | 重复执行需保证安全 |

---

## 5. 风险评估矩阵

| 风险项 | 概率 | 影响 | 等级 | 触发条件 |
|--------|------|------|------|---------|
| 同名书名不同目录 | 低 | 低 | **低** | 两个项目同名但目录不同，旧名不冲突，但迁移后不涉及 |
| 特殊字符路径 | 中 | 中 | **中** | 书名含 `/ \ : * ? " < > |` 等 Windows 非法字符 |
| Git 引用断裂 | 高 | 中 | **中** | `fs::rename` 后 Git 内部引用 `.git` 相对路径可能不匹配 |
| 权限不足无法重命名 | 低 | 高 | **中** | 文件被占用或只读 |
| 磁盘空间不足 | 低 | 中 | **低** | VACUUM INTO 备份需要额外空间 |
| 迁移中断 | 低 | 高 | **中** | 应用崩溃或断电 |

---

## 6. 迁移前检查清单

### □ 数据库备份
- [ ] 确认数据库文件存在
- [ ] 验证数据库文件完整性（`PRAGMA integrity_check`）
- [ ] 确认目标备份目录有足够空间

### □ 数据统计
- [ ] 统计 projects 表总记录数
- [ ] 统计待迁移记录数（`project_id IS NULL`）
- [ ] 统计每个项目的子表数据量

### □ 环境确认
- [ ] 确认应用没有在运行
- [ ] 确认目标盘剩余空间 > 1GB
- [ ] 确认对项目目录有读写权限
- [ ] 确认无其他进程占用项目文件

---

## 7. 建议迁移策略

迁移采用 **零拷贝原地重命名** 策略：

```
迁移前                    操作                      迁移后
----------------------------------------------------------
{path}/{book_name}/  ──>  fs::rename()  ──>  {path}/{P7K3M9}/
  project.json       ──>  更新字段        ──>  project.json (+ project_id)
  chapters/          ──>  不变            ──>  chapters/  (不变)
  covers/            ──>  不变            ──>  covers/    (不变)
```

- **零拷贝**：`fs::rename` 在同一文件系统内是 O(1) 操作
- **事务保护**：每个项目一个独立数据库事务
- **可回滚**：记录旧路径 ↔ 新路径映射表

---

## 8. 验收标准

- [ ] 所有待迁移项目都获得唯一的字母数字 `project_id`
- [ ] 数据库 `project_id` 列正确填充，无重复
- [ ] 所有项目文件夹已从 `{书名}` 重命名为 `{project_id}`
- [ ] 所有 `project.json` 文件包含新的 `project_id` 字段
- [ ] 所有子表数据（volumes, chapters 等）可正常查询
- [ ] 所有封面图片路径有效
- [ ] 所有 Git 仓库在重命名后可正常操作
- [ ] `get_recent_projects`、`open_project` 等接口返回正确数据
