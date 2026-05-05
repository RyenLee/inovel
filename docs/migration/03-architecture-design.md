# 迁移架构与接口设计

> 项目：inovel v0.1.0 | 生成日期：2026-05-05

---

## 1. 迁移流程架构

### 1.1 总体流程

```
┌─────────────────────────────────────────────────────────────────┐
│                        迁移执行总流程                             │
└─────────────────────────────────────────────────────────────────┘

 (1) 用户点击"一键迁移"  ──>  (2) 迁移前预检
                                      │
                                      ├── 数据库 integrity_check
                                      ├── 磁盘空间检查 (> 1GB)
                                      ├── 统计待迁移项目数量
                                      └── 检测项目目录可访问性
                                      │
                                      ▼
                              (3) 创建数据库备份
                                      │
                                      ▼
                              (4) 逐项目迁移循环
                                      │
                              ┌───────┴───────┐
                              │   开始事务     │
                              │   生成ID       │
                              │   检查冲突     │
                              │   重命名文件夹  │
                              │   验证Git      │
                              │   更新DB       │
                              │   更新project.json│
                              │   提交/回滚事务  │
                              └───────┬───────┘
                                      ▼
                              (5) 输出迁移报告
                                      │
                                      ▼
                              (6) 前端展示结果
```

### 1.2 逐项目迁移子流程

```
  ┌───────────────────────┐
  │ 选择待迁移项目 (id)    │
  └─────────┬─────────────┘
            ▼
  ┌───────────────────────┐
  │ BEGIN TRANSACTION     │  ← 数据库事务开始
  └─────────┬─────────────┘
            ▼
  ┌───────────────────────┐
  │ 生成唯一 project_id   │  ← 最多重试100次
  └─────────┬─────────────┘
            ▼
  ┌───────────────────────┐
  │ 检查目标路径是否存在   │ ──存在──→ 重新生成ID (最多3次)
  └─────────┬─────────────┘          ↓
            │ 不存在                 成功 → 继续
            ▼                       失败 → 跳过此项目
  ┌───────────────────────┐
  │ fs::rename(...)       │  ← 重命名文件夹
  └─────────┬─────────────┘
            ▼
      ┌─────┴─────┐
      │ 成功?      │
      └─────┬─────┘
       成功  │  失败
            ▼            ▼
  ┌──────────────┐  ┌──────────────────────┐
  │ 验证Git仓库    │  │ ROLLBACK            │
  └──────┬───────┘  │ 记录错误日志          │
         ▼          └──────────────────────┘
  ┌──────────────────────┐
  │ UPDATE projects SET  │
  │   project_id = ?,    │
  │   path = ?           │
  │ WHERE id = ?         │
  └──────────┬───────────┘
             ▼
  ┌──────────────────────┐
  │ 更新 project.json    │
  │  (添加 project_id)   │
  └──────────┬───────────┘
             ▼
  ┌──────────────────────┐
  │ COMMIT               │
  └──────────┬───────────┘
             ▼
      迁移成功计数 +1
```

---

## 2. 数据模型设计

### 2.1 迁移结果结构体

```rust
// Rust: project.rs 中新增

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
```

### 2.2 回滚参数结构体

```rust
/// 回滚参数
#[derive(Debug, Deserialize)]
pub struct RollbackParams {
    /// 要回滚的项目 DB ID 列表。为空则回滚全部。
    pub project_ids: Option<Vec<i64>>,
}
```

### 2.3 迁移日志表

```sql
CREATE TABLE IF NOT EXISTS migration_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation TEXT NOT NULL,          -- 'migrate' | 'rollback'
    project_db_id INTEGER,           -- 关联的项目 DB ID
    old_project_id TEXT,             -- 旧 project_id（可能为空）
    new_project_id TEXT,             -- 新 project_id
    old_path TEXT NOT NULL,          -- 旧文件夹路径
    new_path TEXT NOT NULL,          -- 新文件夹路径
    status TEXT NOT NULL DEFAULT 'success',  -- success | failed
    error_message TEXT DEFAULT '',   -- 错误信息
    created_at TEXT NOT NULL         -- 操作时间
);
```

---

## 3. Tauri 命令设计

### 3.1 现有命令增强

```rust
// 增强 migrate_existing_projects 命令签名
#[tauri::command]
pub async fn migrate_existing_projects(
    app_handle: AppHandle,
    dry_run: Option<bool>,         // 新增：预览模式，不做实际变更
) -> Result<MigrateResult, String>
```

### 3.2 新增命令

```rust
// 回滚迁移
#[tauri::command]
pub async fn rollback_migration(
    app_handle: AppHandle,
    params: Option<RollbackParams>,  // None = 全部回滚
) -> Result<MigrateResult, String>

// 检查是否需要迁移（前端轮询用）
#[tauri::command]
pub async fn check_migration_needed(
    app_handle: AppHandle,
) -> Result<i32, String>          // 返回待迁移项目数
```

### 3.3 注册到 lib.rs

```rust
.invoke_handler(tauri::generate_handler![
    // ... 现有命令 ...
    project::migrate_existing_projects,  // 已注册，签名不变
    project::rollback_migration,         // 新增
    project::check_migration_needed,     // 新增
])
```

---

## 4. 前端迁移交互设计

### 4.1 前端接口

```typescript
// src/stores/project.ts 新增

export interface MigrationDetail {
  project_db_id: number;
  old_name: string;
  old_path: string;
  new_path: string;
  project_id: string;
  status: "success" | "skipped" | "failed";
  error?: string;
}

export interface MigrateResult {
  total: number;
  success: number;
  failed: number;
  skipped: number;
  backup_path: string;
  details: MigrationDetail[];
}

// Store 新增方法
async function checkMigrationNeeded(): Promise<number>
async function migrateProjects(dryRun: boolean): Promise<MigrateResult | null>
async function rollbackMigration(): Promise<MigrateResult | null>
```

### 4.2 UI 交互方案

#### 检测阶段（WelcomePage 自动检测）

```
┌─────────────────────────────────────────────────────┐
│  📋 发现 N 个项目需要迁移到新的项目ID系统            │
│  ┌────────────────────────────────────────────────┐  │
│  │ ▸ 《三体》 → 将重命名为 P7K3M9                 │  │
│  │ ▸ 《百年孤独》 → 将重命名为 ABC2X              │  │
│  │ ▸ ...                                        │  │
│  └────────────────────────────────────────────────┘  │
│  [ 预览详情 ]  [ 开始迁移 ]  [ 稍后再说 ]            │
└─────────────────────────────────────────────────────┘
```

#### 迁移执行阶段

```
┌─────────────────────────────────────────────────────┐
│  正在迁移项目... (2/5)                               │
│  ████████████████░░░░░░░░  40%                      │
│                                                      │
│  ✅ 《三体》 → P7K3M9  ✓                              │
│  ✅ 《百年孤独》 → ABC2X  ✓                           │
│  ⏳ 《红楼梦》...                                       │
│  ⬜ 《西游记》                                         │
│  ⬜ 《水浒传》                                         │
│                                                      │
│  [ 取消迁移 ]                                         │
└─────────────────────────────────────────────────────┘
```

#### 迁移完成

```
┌─────────────────────────────────────────────────────┐
│  ✅ 迁移完成！                                       │
│                                                      │
│  • 总计迁移：5 个项目                                 │
│  • 成功：5 | 失败：0 | 跳过：0                        │
│  • 自动备份已创建于：{path}                           │
│                                                      │
│  [ 查看详情 ]  [ 返回首页 ]                           │
└─────────────────────────────────────────────────────┘
```

---

## 5. 回滚架构设计

### 5.1 双重回滚机制

```
┌────────────────────────────────────────────────────────┐
│                  回滚架构                               │
├────────────────────────────────────────────────────────┤
│                                                        │
│  方案一：全量备份回滚（最安全）                           │
│  ─────────────────────────                             │
│  VACUUM INTO 备份 → 关闭应用 → 替换 DB → 手动恢复文件夹名│
│                                                        │
│  方案二：逐项目回滚（仅用于少量项目）                     │
│  ─────────────────────────                             │
│  从 migration_logs 取出 (old_path, new_path)            │
│  → rename 回来 → UPDATE projects SET project_id = NULL  │
│  → 恢复 project.json                                    │
│                                                        │
└────────────────────────────────────────────────────────┘
```

### 5.2 回滚流程

```
  用户触发回滚
      │
      ▼
  检查 migration_logs 表
      │
      ├── 有记录 ─→ 读取 old_path ↔ new_path 映射
      │               │
      │               ▼
      │          BEGIN TRANSACTION
      │               │
      │               ├── fs::rename(new_path, old_path)
      │               ├── UPDATE projects SET project_id = NULL, path = old_path
      │               ├── 恢复 project.json (移除 project_id 字段)
      │               └── COMMIT / ROLLBACK
      │
      └── 无记录 ─→ 告知用户无迁移记录可回滚
                     提示使用全量备份回滚方案
```

### 5.3 备份文件管理

| 备份文件 | 路径 | 说明 |
|---------|------|------|
| 数据库备份 | `{app_data_dir}/backups/inovel_before_migration_YYYYMMDD_HHMMSS.db` | VACUUM INTO 完整快照 |
| 迁移日志 | 数据库 `migration_logs` 表 | 记录每次迁移/回滚操作 |

---

## 6. 幂等性设计

### 6.1 查询过滤条件

```sql
-- 仅选择 `project_id IS NULL OR project_id = ''` 的项目
-- 已迁项目不会被再次包含
SELECT id, name, path FROM projects 
WHERE project_id IS NULL OR project_id = ''
```

### 6.2 重入安全

- 迁移命令可重复执行，已迁移项目自动跳过
- 回滚命令幂等：已回滚的项目不会二次回滚（通过检查 migration_logs 最新状态）
- 迁移中途中断后重执行，已完成项目不会再次处理

---

## 7. 异常处理策略

| 异常类型 | 处理方式 |
|---------|---------|
| `fs::rename` 失败 | 回滚当前事务，记录错误 + 错误原因，继续下一个项目 |
| 数据库写入失败 | 回滚当前事务，所有变更还原，项目保持原状 |
| project.json 写入失败 | 回滚事务（文件夹恢复原名，DB 不变），项目保持原状 |
| Git 验证失败 | 仅记录警告，不阻断迁移（后续可手动修复 Git） |
| ID 生成失败 | 跳过该项目，记录错误 |

---

## 8. 检查点清单

### 迁移前检查点 (Pre-migration)

- [ ] `check_migration_needed` 返回 > 0
- [ ] 数据库 integrity_check 通过
- [ ] 磁盘剩余空间充足
- [ ] 备份创建成功

### 迁移中检查点 (Per-project)

- [ ] 目标路径不存在
- [ ] `fs::rename` 返回 Ok
- [ ] DB UPDATE 影响行数为 1
- [ ] project.json 写入成功
- [ ] 事务已提交

### 迁移后检查点 (Post-migration)

- [ ] `MigrateResult.failed == 0`
- [ ] `get_recent_projects` 返回正确数据
- [ ] 子表数据可通过项目打开
- [ ] 封面路径有效
- [ ] 备份文件存在且可读

---

## 9. 文件变更清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `src-tauri/src/project.rs` | 修改 | 增强 migrate_existing_projects, 新增 rollback_migration, check_migration_needed |
| `src-tauri/src/db.rs` | 修改 | 新增 migration_logs 表 |
| `src-tauri/src/models.rs` | 修改 | 新增 MigrationDetail, MigrateResult 结构体 |
| `src-tauri/src/lib.rs` | 修改 | 注册新命令 |
| `src/stores/project.ts` | 修改 | 新增迁移相关接口和方法 |
| `src/views/WelcomePage.vue` | 修改 | 添加迁移提示横幅和交互 |
