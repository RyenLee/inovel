# inovel 功能模块文档

本文档详细描述 inovel 小说工坊的各个功能模块的技术规格和使用说明。

## 文档结构

### 核心功能模块

| 模块                                   | 文档               | 描述                               |
| -------------------------------------- | ------------------ | ---------------------------------- |
| [项目管理](modules/project.md)         | project.md         | 项目的创建、打开、删除、迁移等     |
| [章节管理](modules/chapter.md)         | chapter.md         | 卷/章节的 CRUD、排序、内容管理     |
| [世界观构建](modules/worldbuilding.md) | worldbuilding.md   | 人物、地点、组织、关系图谱、时间轴 |
| [写作工具](modules/writing.md)         | writing.md         | 写作目标、字数统计、番茄钟、敏感词 |
| [模板系统](modules/template.md)        | template.md        | 内置模板、用户模板管理             |
| [灵感看板](modules/inspiration.md)     | inspiration.md     | 灵感条目管理、看板视图             |
| [导出功能](modules/export.md)          | export.md          | TXT/Markdown/EPUB/HTML 导出        |
| [版本控制](modules/version-control.md) | version-control.md | Git 快照、版本历史、差异对比       |
| [备份功能](modules/backup.md)          | backup.md          | 手动/增量备份、恢复                |
| [配置管理](modules/config.md)          | config.md          | 应用配置、导入/导出、回滚          |
| [加密功能](modules/encryption.md)      | encryption.md      | 项目加密、解密、密码验证           |
| [名称生成](modules/names.md)           | names.md           | 中英文姓名、地名生成               |

## 技术架构

### 前端技术栈

- Vue 3 + TypeScript + Vite
- Naive UI + Tailwind CSS 4
- Tiptap 3 (ProseMirror)
- Pinia 状态管理

### 后端技术栈

- Rust (Tauri 2)
- SQLite (rusqlite)
- Git (git2)
- AES-256 加密

### 前后端通信

- 所有命令通过 `@tauri-apps/api/core` 的 `invoke()` 调用
- 参数统一使用下划线命名（snake_case）
- 返回值遵循统一的错误处理规范

## 文档编写规范

每个功能模块文档包含以下章节：

1. **功能概述** - 功能简介和使用场景
2. **核心功能点** - 主要功能列表
3. **详细操作流程** - 用户操作步骤
4. **输入输出参数说明** - API 参数详解
5. **业务规则** - 业务逻辑约束
6. **异常处理机制** - 错误处理方式
7. **相关联功能模块** - 模块依赖关系
8. **界面交互说明** - UI/UX 说明（如适用）

## 版本信息

- 文档版本：1.0.0
- 更新日期：2026-05-13
- 软件版本：inovel v1.1.1
