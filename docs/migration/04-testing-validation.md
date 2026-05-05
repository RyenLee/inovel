# 测试与数据验证方案

> 项目：inovel v0.1.0 | 生成日期：2026-05-05

---

## 1. 测试环境准备

### 1.1 测试数据生成脚本

创建模拟旧数据的 SQL 和文件脚本，生成包含各种边界情况的测试项目：

```
测试项目清单：
1. 中文书名：《三体》 — 标准场景
2. 英文书名：The Great Gatsby — 纯ASCII
3. 中英混合：三体III·死神永生 — 特殊符号
4. 短书名：A — 单字符
5. 长书名：Lorem ipsum dolor sit amet consectetur adipiscing elit — 长文件名
6. 有 Git 仓库的项目 — 验证 Git 完整性
7. 有封面的项目 — 验证封面路径
8. 有完整子数据的项目（卷+章+角色+写作记录）— 验证完整数据通路
```

### 1.2 测试目录结构

```
tests/
  └── data/
      ├── 三体/                  # 旧格式项目
      │   ├── project.json       # 无 project_id
      │   ├── chapters/
      │   ├── covers/
      │   └── .git/
      ├── The Great Gatsby/
      └── ...
```

---

## 2. 逐项验证矩阵

### 2.1 数据库验证

| # | 验证项 | SQL 检查 | 预期结果 |
|---|--------|---------|---------|
| V1 | 所有项目有 project_id | `SELECT COUNT(*) FROM projects WHERE project_id IS NULL OR project_id = ''` | 0 |
| V2 | project_id 唯一 | `SELECT project_id, COUNT(*) FROM projects GROUP BY project_id HAVING COUNT(*) > 1` | 0 行 |
| V3 | path 已更新为新路径 | `SELECT id, name, path FROM projects` | path 包含 project_id |
| V4 | 子表数据完整 | `SELECT p.id, p.name, COUNT(v.id) FROM projects p LEFT JOIN volumes v ON v.project_id = p.id GROUP BY p.id` | 卷数不变 |
| V5 | 封面路径可访问 | 通过 open_project 获取 cover_path，检查文件存在 | 全部可访问 |

### 2.2 文件系统验证

| # | 验证项 | 检查方式 | 预期结果 |
|---|--------|---------|---------|
| F1 | 文件夹名已变更为 project_id | `ls -d {parent_dir}/*/` | 文件夹名为 PXXXXX 格式 |
| F2 | 旧文件夹不存在 | `ls -d {parent_dir}/{旧书名}/` 2>&1 | 不存在 |
| F3 | Git 仓库可操作 | 每个项目执行 `git log` | 返回历史记录 |
| F4 | 章节文件存在 | `ls {new_path}/chapters/*.md` | 文件完整 |
| F5 | 封面文件存在 | `ls {new_path}/covers/*` | 文件完整 |

### 2.3 JSON 配置验证

| # | 验证项 | 检查方式 | 预期结果 |
|---|--------|---------|---------|
| J1 | project.json 包含 project_id | `jq .project_id {path}/project.json` | 非空字符串 |
| J2 | project_id 格式正确 | 正则 `^P[A-HJ-NP-Z2-9]{5}$` | 全部匹配 |
| J3 | project.json 原始字段保留 | `jq '{name, author, description}'` | 与迁移前一致 |

### 2.4 功能回归验证

| # | 验证项 | 操作 | 预期结果 |
|---|--------|------|---------|
| R1 | 打开项目 | `open_project(id)` | 返回正确项目信息 |
| R2 | 获取最近项目 | `get_recent_projects()` | 列表完整，ID 显示正确 |
| R3 | 更新项目信息 | `update_project(id, ...)` | 更新成功 |
| R4 | 设置封面 | `set_cover(id, path)` | 封面保存到 covers/ 目录 |
| R5 | 新建项目 | `create_project(...)` | 使用新 ID 格式 |
| R6 | 卷/章操作 | CRUD volumes & chapters | 功能正常 |
| R7 | 写作统计 | 获取写作记录 | 关联到正确项目 |

---

## 3. 测试步骤

### 步骤 1：创建测试数据集

```bash
# 创建测试目录
mkdir -p tests/data

# 创建 5 个模拟旧项目
for book in "三体" "The Great Gatsby" "三体III·死神永生" "短篇集A" "LongLongBookName"; do
    mkdir -p "tests/data/$book/chapters"
    mkdir -p "tests/data/$book/covers"
    
    # 创建 project.json（无 project_id）
    cat > "tests/data/$book/project.json" << EOF
{
    "name": "$book",
    "author": "测试作者",
    "description": "测试项目 $book",
    "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
    
    # 创建示例章节
    echo "# 第一章" > "tests/data/$book/chapters/v1_c1.md"
    echo "这是第一章内容" >> "tests/data/$book/chapters/v1_c1.md"
done

# 初始化一个项目的 Git 仓库
cd tests/data/三体 && git init && git add . && git commit -m "初始提交"
cd ../../..
```

### 步骤 2：导入测试数据到数据库

```sql
INSERT INTO projects (name, author, description, path, created_at, last_opened_at)
VALUES 
    ('三体', '刘慈欣', '科幻小说', 'D:/Dev/workspace/2026/05/Rust/inovel/tests/data/三体', '2025-01-01T00:00:00Z', '2025-06-01T00:00:00Z'),
    ('The Great Gatsby', 'Fitzgerald', 'Classic', 'D:/Dev/workspace/2026/05/Rust/inovel/tests/data/The Great Gatsby', '2025-02-01T00:00:00Z', NULL),
    ('三体III·死神永生', '刘慈欣', '续作', 'D:/Dev/workspace/2026/05/Rust/inovel/tests/data/三体III·死神永生', '2025-03-01T00:00:00Z', NULL),
    ('短篇集A', '作者A', '短篇', 'D:/Dev/workspace/2026/05/Rust/inovel/tests/data/短篇集A', '2025-04-01T00:00:00Z', NULL),
    ('LongLongBookName', 'Author', 'Long name test', 'D:/Dev/workspace/2026/05/Rust/inovel/tests/data/LongLongBookName', '2025-05-01T00:00:00Z', NULL);
```

### 步骤 3：执行迁移

```bash
# 1. 启动应用
npm run tauri dev

# 2. 在 WelcomPage 查看迁移提示横幅
# 3. 点击"查看详情"预览
# 4. 点击"开始迁移"执行
```

### 步骤 4：逐项验证

按上述验证矩阵，逐项检查：

```bash
# 验证 V1: 无空 project_id
sqlite3 inovel.db "SELECT COUNT(*) FROM projects WHERE project_id IS NULL OR project_id = ''"

# 验证 V2: project_id 唯一
sqlite3 inovel.db "SELECT project_id, COUNT(*) FROM projects GROUP BY project_id HAVING COUNT(*) > 1"

# 验证 V3: 路径包含新 ID
sqlite3 inovel.db "SELECT id, name, path FROM projects"

# 验证 F1-F5: 文件系统
for dir in tests/data/*/; do
    echo "=== $dir ==="
    ls "$dir"
    echo "---"
done
```

### 步骤 5：回滚测试

```bash
# 1. 在 UI 上点击"回滚"按钮
# 2. 验证文件夹名恢复
# 3. 验证 project_id 已清空
sqlite3 inovel.db "SELECT id, name, project_id, path FROM projects"
```

---

## 4. 验收标准总表

| # | 检查项 | 验收标准 | 状态 |
|---|--------|---------|------|
| TC1 | 迁移成功数 | `MigrateResult.failed == 0` | □ |
| TC2 | project_id 完整性 | 无空值，唯一性约束通过 | □ |
| TC3 | 文件夹重命名 | 文件夹名与 project_id 一致 | □ |
| TC4 | 子表数据关联 | 打开项目后子数据完整 | □ |
| TC5 | 封面路径有效 | 封面图片正常显示 | □ |
| TC6 | Git 可用 | 项目内可查看 git log | □ |
| TC7 | 新项目正常 | 创建新项目使用 ID 格式 | □ |
| TC8 | 备份可恢复 | 备份文件存在且可读 | □ |
| TC9 | 回滚可用 | 回滚后项目恢复旧状态 | □ |
| TC10 | 幂等性 | 重复执行迁移自动跳过 | □ |

---

## 5. 边界情况测试

| 场景 | 测试数据 | 预期行为 |
|------|---------|---------|
| 同名不同路径 | 两个项目的书名相同，但在不同父目录下 | 各自生成不同 ID |
| 空书名 | 书名为空字符串的项目 | 迁移跳过并报错 |
| 路径含特殊字符 | 书名含 `&$@` 字符 | 正确重命名 |
| 超大项目 | 1000+ 章节文件的项目 | 重命名 O(1)，不耗时 |
| 只读文件 | 项目中存在只读文件 | 提示权限不足 |
| 已迁移项目 | 已经拥有 project_id 的项目 | 自动跳过 |
| 无数据项目 | 只有 project.json 的空项目 | 正确迁移 |
| 迁移中断后重入 | 终止迁移进程后再次执行 | 已迁移的跳过，未迁移的继续 |
