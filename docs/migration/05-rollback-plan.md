# 回滚方案操作手册

> 项目：inovel v0.1.0 | 生成日期：2026-05-05

---

## 概述

本手册提供两种回滚方案，确保在任何意外情况下都能安全恢复数据。

| 方案 | 恢复粒度 | 操作复杂度 | 是否需要代码支持 |
|------|---------|-----------|----------------|
| **方案一：全量备份回滚** | 整体回滚 | 低 | 否（手动操作） |
| **方案二：逐项目自动化回滚** | 单个/批量 | 中 | 是（rollback_migration 命令） |

---

## 方案一：全量备份回滚（最安全）

在迁移脚本执行前会自动创建数据库完整快照。此方案通过替换数据库文件来实现"一键还原"。

### 适用场景
- 迁移后出现严重问题（如大量数据丢失、应用无法启动）
- 需要完全恢复到迁移前的状态
- 不需要保留迁移后的任何变更

### 操作步骤

#### 步骤 1：定位备份文件

迁移成功后，`MigrateResult.backup_path` 会返回备份文件路径。

默认备份路径为：
```
{app_data_dir}/backups/inovel_before_migration_{YYYYMMDD_HHMMSS}.db
```

Windows 典型路径：
```
C:\Users\{用户名}\AppData\Roaming\inovel\backups\inovel_before_migration_20260505_093000.db
```

如果找不到备份文件，检查以下位置：
1. `{app_data_dir}/backups/` 目录
2. Tauri 日志文件中的备份路径记录

#### 步骤 2：关闭应用

确保 inovel 应用完全退出（检查任务管理器）。

#### 步骤 3：确认备份完整性

```bash
# 检查备份文件存在
ls -la {backup_path}

# 检查备份数据库完整性
sqlite3 {backup_path} "PRAGMA integrity_check;"
# 预期输出：ok

# 查看备份中的数据
sqlite3 {backup_path} "SELECT id, name, project_id, path FROM projects;"
# 预期：project_id 全部为 NULL
```

#### 步骤 4：替换数据库文件

```bash
# 1. 备份当前数据库（以防万一）
copy "{app_data_dir}\inovel.db" "{app_data_dir}\inovel_after_migration_backup.db"

# 2. 用备份文件替换当前数据库
copy "{backup_path}" "{app_data_dir}\inovel.db"
```

#### 步骤 5：恢复文件夹名

由于全量备份只恢复了数据库，磁盘上的文件夹名仍为新 ID。需要手动将文件夹名恢复为旧书名。

```bash
# 从备份文件中获取旧路径信息
sqlite3 {backup_path} "
    SELECT 'rename ' || path || ' to ' || 
    CASE 
        WHEN path LIKE '%\\三体%' THEN REPLACE(path, 'P7K3M9', '三体')
        -- 根据实际情况替换
    END
    FROM projects;
"
```

或者使用应用内的回滚命令（如果数据库替换后应用仍能启动）：

在 WelcomePage 点击「回滚」按钮，使用逐项目回滚功能。

#### 步骤 6：验证恢复结果

```bash
# 验证项目列表
sqlite3 "{app_data_dir}\inovel.db" "SELECT id, name, path FROM projects;"

# 启动应用验证
npm run tauri dev
```

---

## 方案二：逐项目自动化回滚（推荐）

通过 Tauri 命令 `rollback_migration` 实现，无需手动替换文件。

### 适用场景
- 只有少数项目出现问题
- 需要选择性回滚
- 希望保留其他已迁移项目的状态

### 操作方式一：UI 操作（推荐）

1. 在 WelcomePage 首页的迁移提示横幅中，点击 **「回滚」** 按钮
2. 在弹出的确认对话框中点击 **「确认回滚」**
3. 等待回滚完成
4. 验证项目列表已恢复

### 操作方式二：编程调用

```typescript
// 回滚所有项目
const result = await invoke("rollback_migration", { params: null });
console.log(`回滚完成: ${result.success} 成功, ${result.failed} 失败`);

// 回滚指定项目（通过 DB ID）
const result = await invoke("rollback_migration", {
    params: { project_ids: [1, 2, 3] }
});
```

### 操作方式三：后端 CLI 式调用（调试用）

```bash
# 通过 Tauri 调试窗口执行
curl -X POST http://localhost:1420/rollback -H "Content-Type: application/json"
```

### 回滚行为说明

| 操作 | 说明 |
|------|------|
| 回滚全部 | 遍历所有 `migration_logs` 中 `operation='migrate' AND status='success'` 且未被回滚的记录 |
| 回滚指定 | 通过 `project_ids` 参数指定要回滚的项目 DB ID |
| 幂等性 | 已回滚的项目不会二次回滚（通过检查 migration_logs 中的 rollback 记录） |
| 事务性 | 每个项目的回滚为独立事务 |
| 可中断 | 回滚过程中断后可以安全重入 |

---

## 回滚前检查清单

在执行回滚前，请确认以下所有检查点：

- [ ] **确认问题范围**：确定是全量回滚还是部分回滚
- [ ] **确认备份存在**：备份文件路径已知且文件可读
- [ ] **确认备份完整性**：`PRAGMA integrity_check` 返回 `ok`
- [x] **关闭应用**：确保 inovel 已完全关闭
- [ ] **备份当前状态**：方案一第 4 步已执行当前状态备份
- [ ] **通知用户**：如有多用户/生产环境，告知维护窗口

---

## 回滚后验证清单

回滚完成后，执行以下验证：

- [ ] **项目列表恢复**：`get_recent_projects` 返回迁移前的项目列表
- [ ] **文件夹名恢复**：文件夹名从 `PXXXXX` 恢复为旧书名
- [ ] **子表数据正常**：打开项目后，卷/章/角色/地点等数据完整
- [ ] **新项目不受影响**：迁移后创建的新项目（已有 project_id）是否受影响（应不受影响）
- [ ] **应用日志**：检查 Tauri 日志无异常错误
- [ ] **备份删除**：确认回滚后的临时备份是否清理（或有明确标记）

---

## 回滚失败处理

如果在回滚过程中遇到以下问题：

### 问题 1：文件无法重命名

**现象**：`fs::rename` 返回 Permission denied 错误。

**原因**：项目文件被其他进程占用（如编辑器、杀毒软件、文件资源管理器）。

**解决方案**：
1. 关闭所有编辑器窗口
2. 关闭文件资源管理器中打开的目录
3. 重启电脑后重试
4. 使用方案一（全量备份替换）

### 问题 2：数据库写入失败

**现象**：ROLLBACK 后 DB 状态不一致。

**原因**：SQLite 文件锁定或磁盘空间不足。

**解决方案**：
1. 检查磁盘剩余空间
2. 使用方案一替换数据库文件
3. 手动执行 `VACUUM` 修复

### 问题 3：migration_logs 丢失

**现象**：`rollback_migration` 返回 0 条待回滚记录。

**原因**：数据库被重建或 migration_logs 表被清空。

**解决方案**：
1. 仅能使用方案一（全量备份恢复）
2. 如果备份也不存在，需手动通过旧项目名匹配恢复

---

## 紧急联系人

若回滚过程中遇到无法自行解决的问题：

| 角色 | 联系方式 | 说明 |
|------|---------|------|
| 项目维护者 | 见项目 README | 代码级别的问题 |
| 数据恢复 | 全量备份优先 | 数据恢复的第一优先级 |

---

## 附录：手动恢复脚本（备份和 migration_logs 均丢失的极端情况）

如果数据库备份和 migration_logs 同时丢失，可尝试以下手动恢复步骤：

```bash
# 1. 停止应用
# 2. 从磁盘文件系统中查找旧项目
# 3. 手动将文件夹名从 ID 改回书名
# 4. 手动修改数据库：
sqlite3 inovel.db "
    -- 清空不正确的 project_id
    UPDATE projects SET project_id = NULL;
    
    -- 手动修正路径（用实际旧书名替换）
    UPDATE projects SET path = REPLACE(path, 'PXXXXX', '旧书名') 
    WHERE path LIKE '%PXXXXX%';
"
```
