# inovel — 小说工坊

基于 Tauri 2 + Vue 3 + Rust 的桌面小说创作软件，提供富文本编辑、世界观管理、关系图谱、时间轴、版本快照、数据备份、敏感词检测、灵感看板等功能。

## 技术栈

| 层     | 技术                          |
|--------|-------------------------------|
| 前端   | Vue 3 + TypeScript + Vite 8   |
| UI 库  | Naive UI + Tailwind CSS 4     |
| 编辑器 | Tiptap 3 (ProseMirror)        |
| 后端   | Rust (Tauri 2)                |
| 数据库 | SQLite (rusqlite)             |
| 版本   | Git (git2)                    |

## 功能特性

### 核心编辑
- **Markdown 富文本编辑器** — Tiptap 实现，支持加粗/斜体/标题/列表/引用/代码块
- **智能符号扩展** — 自动将 `...` 转为 `…`、`--` 转为 `—` 等
- **敏感词高亮** — 自动检测并高亮显示敏感词
- **写作模式** — 普通/打字机/专注三种模式
- **写作目标** — 每日字数目标，进度追踪
- **番茄钟** — 专注计时器，记录专注会话

### 内容管理
- **章节管理** — 分卷分章树形结构，拖拽排序，字数统计
- **模板系统** — 内置模板 + 用户自定义模板，支持插入到正文
- **@提及** — 在正文中 `@人物/地点/组织`，悬停预览详情，点击跳转
- **大纲面板** — 快速导航到任意章节

### 世界观构建
- **世界观管理** — 人物/地点/组织 CRUD，自定义字段
- **关系图谱** — 可视化人物关系网络（@vue-flow）
- **故事时间轴** — 按故事时间组织事件，关联章节跳转
- **灵感看板** — 看板式收集灵感碎片

### 项目工具
- **项目加密** — AES-256 加密，保护项目内容
- **版本快照** — 自动 Git 提交，版本历史对话框，版本对比（diff），一键恢复
- **数据备份** — 手动/增量备份，恢复历史版本
- **敏感词管理** — 批量导入/导出敏感词库，全文扫描
- **导出功能** — 支持导出为 TXT/Markdown/EPUB/HTML 格式

### 辅助功能
- **名称生成器** — 随机生成中/英文姓名、地名
- **写作统计** — 每日/每周/每月字数统计，柱状图展示
- **快捷键设置** — 自定义全局快捷键
- **暗色模式**
- **项目迁移** — 自动检测并迁移旧格式项目

## 快速开始

```bash
# 安装依赖
npm install

# 启动开发服务器（Web 端调试）
npm run dev

# 启动 Tauri 桌面应用
npm run tauri dev

# 构建发布版本
npm run tauri build
```

## 项目结构

```
inovel/
├── src/                          # 前端源码
│   ├── components/
│   │   ├── MarkdownEditor.vue     # 核心编辑器
│   │   ├── TreeSidebar.vue        # 章节树侧栏
│   │   ├── OutlinePanel.vue        # 大纲面板
│   │   ├── WorldbuildingPanel.vue  # 世界观管理
│   │   ├── RelationshipGraph.vue  # 关系图谱
│   │   ├── Timeline.vue            # 故事时间轴
│   │   ├── HistoryDialog.vue       # 版本历史对话框
│   │   ├── BackupDialog.vue        # 备份对话框
│   │   ├── ExportDialog.vue        # 导出对话框
│   │   ├── InspirationBoard.vue    # 灵感看板
│   │   ├── PomodoroTimer.vue       # 番茄钟
│   │   ├── SensitiveWordsManager.vue # 敏感词管理
│   │   ├── ShortcutSettings.vue    # 快捷键设置
│   │   ├── TemplateSelector.vue     # 模板选择器
│   │   ├── MentionExtension.ts     # @提及扩展
│   │   ├── MentionNode.vue         # @提及节点渲染
│   │   ├── SensitiveHighlightPlugin.ts # 敏感词高亮
│   │   └── SmartSymbolsExtension.ts # 智能符号扩展
│   ├── views/
│   │   ├── EditorPage.vue          # 主编辑器页面
│   │   ├── WorldbuildingPage.vue   # 世界观页面
│   │   ├── ProjectSettingsPage.vue  # 项目设置页面
│   │   ├── ProjectStatsDashboard.vue # 项目统计面板
│   │   ├── StatsDashboard.vue       # 全局统计面板
│   │   ├── SettingsPage.vue         # 全局设置页面
│   │   └── WelcomePage.vue          # 欢迎页面
│   ├── stores/                     # Pinia 状态管理
│   │   ├── project.ts              # 项目状态
│   │   ├── editor.ts               # 编辑器状态
│   │   ├── worldbuilding.ts        # 世界观状态
│   │   ├── shortcuts.ts             # 快捷键状态
│   │   ├── template.ts             # 模板状态
│   │   └── tree.ts                 # 目录树状态
│   ├── composables/                # 组合式函数
│   │   ├── useGlobalShortcuts.ts   # 全局快捷键
│   │   ├── useTheme.ts             # 主题管理
│   │   └── themeConfig.ts          # 主题配置
│   └── types/                      # TypeScript 类型
├── src-tauri/                      # Rust 后端
│   └── src/
│       ├── lib.rs                  # 入口：模块声明 + 命令注册
│       ├── main.rs                 # Tauri 入口
│       ├── models.rs               # 数据结构定义
│       ├── db.rs                   # 数据库初始化
│       ├── config.rs                # 应用配置管理
│       ├── project.rs              # 项目 CRUD
│       ├── chapter.rs              # 卷/章节 CRUD
│       ├── writing.rs              # 写作目标/统计/番茄钟
│       ├── names.rs                # 名称生成器
│       ├── worldbuilding.rs        # 人物/地点/组织
│       ├── relationship.rs          # 关系图谱
│       ├── timeline.rs              # 时间轴事件
│       ├── git_snapshot.rs         # Git 版本控制
│       ├── backup.rs               # 数据备份
│       ├── export.rs               # 导出功能
│       ├── encryption.rs           # 项目加密
│       ├── sensitive.rs            # 敏感词检测
│       ├── template.rs             # 模板系统
│       └── inspiration.rs          # 灵感看板
└── package.json
```

## 后端命令一览

所有命令通过 `@tauri-apps/api/core` 的 `invoke()` 调用，约 **60+ 个** Tauri command，覆盖：

### 项目管理
- 创建/打开/删除/更新项目
- 项目加密/解密/验证密码/修改密码
- 项目迁移（自动检测旧格式）
- 项目封面设置

### 章节管理
- 卷/章节 CRUD
- 内容读写
- 排序（拖拽重排）
- 状态管理（草稿/已完成等）
- 字数统计

### 世界观构建
- 人物/地点/组织 CRUD
- 关系创建/更新/删除/列表
- 时间轴事件 CRUD

### 写作工具
- 写作目标设置/统计
- 每日字数记录
- 番茄钟会话记录/统计
- 敏感词导入/导出/扫描

### 模板系统
- 内置模板列表
- 用户模板 CRUD
- 模板插入到正文

### 灵感管理
- 灵感条目 CRUD
- 看板重排
- 灵感看板获取

### 导出功能
- 导出为 TXT/Markdown/EPUB/HTML
- 获取导出内容
- 打开导出目录

### 版本控制
- Git 仓库初始化
- 创建快照（自动 Git 提交）
- 获取快照历史
- 恢复指定版本
- 版本差异对比

### 备份功能
- 手动/增量备份
- 备份列表查看
- 备份恢复
- 备份日志/统计

### 配置管理
- 获取/更新/重置应用配置
- 配置文件路径
- 路径验证

## 数据存储

- **应用配置**: `{APP_DATA}/config.json`
- **数据库**: `{APP_DATA}/inovel.db`
- **项目数据**: `{APP_DATA}/projects/`
- **备份文件**: `{APP_DATA}/backups/`
- **导出文件**: `{APP_DATA}/exports/`
- **日志文件**: `{APP_DATA}/logs/`

> 支持环境变量: `${APP_DATA}`、`${APP_INSTALL_DIR}`、`${HOME}` 等

## 许可证

MIT
