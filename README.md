# inovel — 小说工坊

基于 Tauri 2 + Vue 3 + Rust 的桌面小说创作软件，提供富文本编辑、世界观管理、关系图谱、时间轴、版本快照、数据备份、敏感词检测、灵感看板等功能。

## 技术栈

| 层     | 技术                        |
| ------ | --------------------------- |
| 前端   | Vue 3 + TypeScript + Vite 8 |
| UI 库  | Naive UI + Tailwind CSS 4   |
| 编辑器 | Tiptap 3 (ProseMirror)      |
| 后端   | Rust (Tauri 2)              |
| 数据库 | SQLite (rusqlite)           |
| 版本   | Git (git2)                  |

## 功能特性

### 核心编辑

- **Markdown 富文本编辑器** — Tiptap 实现，支持加粗/斜体/标题/列表/引用/代码块
- **智能符号扩展** — 自动将 `...` 转为 `…`、`--` 转为 `—` 等
- **敏感词高亮** — 自动检测并高亮显示敏感词
- **写作模式** — 普通/打字机/专注三种模式
- **写作目标** — 每日字数目标，进度追踪
- **番茄钟** — 专注计时器，记录专注会话
- **纸张样式** — 横线纸/带边距横线纸/方格纸/点阵纸四种背景样式
- **文本导入** — 导入 .txt 文本文件，支持替换或插入模式

### 内容管理

- **章节管理** — 分卷分章树形结构，拖拽排序，字数统计
- **模板系统** — 内置模板 + 用户自定义模板，支持插入到正文
- **@提及** — 在正文中 `@人物/地点/组织`，悬停预览详情，点击跳转
- **大纲面板** — 快速导航到任意章节

### 世界观构建

- **世界观管理** — 人物/地点/组织 CRUD，自定义字段
- **关系图谱** — 可视化人物关系网络（@vue-flow）
- **故事时间轴** — 按故事时间组织事件，关联章节跳转
- **灵感看板** — 看板式收集灵感碎片，支持多语言国际化

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
│   ├── assets/                   # 静态资源
│   │   └── vue.svg
│   ├── components/               # 组件
│   │   ├── BackupDialog.vue       # 备份对话框
│   │   ├── DeleteConfirmModal.vue # 删除确认弹窗
│   │   ├── ExportDialog.vue       # 导出对话框
│   │   ├── HistoryDialog.vue      # 版本历史对话框
│   │   ├── InspirationBoard.vue   # 灵感看板
│   │   ├── MarkdownEditor.vue     # 核心编辑器
│   │   ├── MentionExtension.ts    # @提及扩展
│   │   ├── MentionNode.vue       # @提及节点渲染
│   │   ├── OutlinePanel.vue       # 大纲面板
│   │   ├── PomodoroTimer.vue     # 番茄钟
│   │   ├── RelationshipGraph.vue  # 关系图谱
│   │   ├── SensitiveHighlightPlugin.ts # 敏感词高亮
│   │   ├── SensitiveWordsManager.vue # 敏感词管理
│   │   ├── ShortcutSettings.vue   # 快捷键设置
│   │   ├── SmartSymbolsExtension.ts # 智能符号扩展
│   │   ├── TemplateSelector.vue   # 模板选择器
│   │   ├── TextImportDialog.vue   # 文本导入对话框
│   │   ├── Timeline.vue           # 故事时间轴
│   │   ├── TreeSidebar.vue        # 章节树侧栏
│   │   └── WorldbuildingPanel.vue # 世界观管理
│   ├── composables/              # 组合式函数
│   │   ├── themeConfig.ts        # 主题配置
│   │   ├── useEditor.ts          # 编辑器核心
│   │   ├── useEditorLayout.ts    # 编辑器布局
│   │   ├── useGlobalShortcuts.ts # 全局快捷键
│   │   ├── useTextBeautify.ts    # 文本美化
│   │   ├── useTextImport.ts      # 文本导入
│   │   ├── useTheme.ts           # 主题管理
│   │   └── useWordCount.ts       # 字数统计
│   ├── router/                   # 路由
│   │   └── index.ts
│   ├── stores/                    # Pinia 状态管理
│   │   ├── editor.ts             # 编辑器状态
│   │   ├── project.ts            # 项目状态
│   │   ├── shortcuts.ts          # 快捷键状态
│   │   ├── template.ts           # 模板状态
│   │   ├── tree.ts               # 目录树状态
│   │   └── worldbuilding.ts      # 世界观状态
│   ├── types/                    # TypeScript 类型
│   │   ├── chapter.ts            # 章节类型
│   │   ├── encryption.ts         # 加密类型
│   │   ├── inspiration.ts        # 灵感类型
│   │   ├── pomodoro.ts           # 番茄钟类型
│   │   └── template.ts           # 模板类型
│   ├── views/                    # 页面
│   │   ├── EditorPage.vue        # 主编辑器页面
│   │   ├── ProjectSettingsPage.vue # 项目设置页面
│   │   ├── ProjectStatsDashboard.vue # 项目统计面板
│   │   ├── SettingsPage.vue      # 全局设置页面
│   │   ├── StatsDashboard.vue    # 全局统计面板
│   │   ├── WelcomePage.vue       # 欢迎页面
│   │   └── WorldbuildingPage.vue # 世界观页面
│   ├── App.vue                   # 根组件
│   ├── main.ts                   # 入口文件
│   ├── style.css                 # 全局样式
│   └── vite-env.d.ts             # Vite 类型声明
├── src-tauri/                    # Rust 后端
│   ├── capabilities/              # Tauri 权限配置
│   │   └── default.json
│   ├── icons/                     # 应用图标
│   ├── resources/                 # 资源文件
│   │   └── names.json            # 名称生成数据
│   ├── src/                       # Rust 源码
│   │   ├── commands/             # Tauri 命令层
│   │   │   ├── backup.rs         # 备份命令
│   │   │   ├── chapter.rs        # 章节 CRUD 命令
│   │   │   ├── encryption.rs     # 加密命令
│   │   │   ├── export.rs         # 导出命令
│   │   │   ├── file.rs           # 文件操作命令
│   │   │   ├── git_snapshot.rs   # Git 版本控制命令
│   │   │   ├── inspiration.rs    # 灵感看板命令
│   │   │   ├── names.rs          # 名称生成器命令
│   │   │   ├── optimization.rs   # 性能优化命令
│   │   │   ├── project.rs        # 项目 CRUD 命令
│   │   │   ├── relationship.rs   # 关系图谱命令
│   │   │   ├── sensitive.rs      # 敏感词检测命令
│   │   │   ├── template.rs       # 模板系统命令
│   │   │   ├── timeline.rs       # 时间轴命令
│   │   │   ├── worldbuilding.rs  # 世界观命令
│   │   │   └── writing.rs        # 写作目标/统计/番茄钟命令
│   │   ├── db/                   # 数据访问层
│   │   │   ├── backups.rs        # 备份数据访问
│   │   │   ├── chapters.rs       # 章节数据访问
│   │   │   ├── common.rs         # 公共数据库工具
│   │   │   ├── init.rs           # 数据库初始化
│   │   │   ├── inspiration.rs    # 灵感数据访问
│   │   │   ├── projects.rs       # 项目数据访问
│   │   │   ├── relationships.rs  # 关系数据访问
│   │   │   ├── sensitive.rs      # 敏感词数据访问
│   │   │   ├── templates.rs      # 模板数据访问
│   │   │   ├── timeline.rs       # 时间轴数据访问
│   │   │   ├── worldbuilding.rs  # 世界观数据访问
│   │   │   └── writing.rs        # 写作统计数据访问
│   │   ├── logging/              # 日志模块
│   │   │   ├── commands.rs       # 日志命令
│   │   │   ├── error_log.rs      # 错误日志
│   │   │   ├── init.rs           # 日志初始化
│   │   │   └── operation.rs      # 操作日志
│   │   ├── optimization/         # 性能优化模块
│   │   │   ├── cache.rs          # 响应缓存
│   │   │   ├── gzip.rs           # Gzip 压缩
│   │   │   ├── merger.rs         # 请求合并
│   │   │   └── pagination.rs     # 分页支持
│   │   ├── services/             # 业务逻辑层
│   │   │   ├── chapter.rs        # 章节业务逻辑
│   │   │   ├── project.rs        # 项目业务逻辑
│   │   │   └── writing.rs        # 写作业务逻辑
│   │   ├── utils/                # 工具函数
│   │   │   ├── time.rs           # 时间处理
│   │   │   └── validation.rs     # 数据验证
│   │   ├── commands.rs           # 命令模块入口
│   │   ├── config.rs             # 应用配置管理
│   │   ├── db.rs                 # 数据库模块入口
│   │   ├── error.rs              # 统一错误类型
│   │   ├── lib.rs                # 入口：模块声明 + 命令注册
│   │   ├── logging.rs            # 日志模块入口
│   │   ├── main.rs               # Tauri 入口
│   │   ├── models.rs             # 数据结构定义
│   │   ├── optimization.rs       # 优化模块入口
│   │   ├── services.rs           # 服务模块入口
│   │   ├── settings.rs           # 设置管理
│   │   ├── state.rs              # 应用状态
│   │   └── utils.rs              # 工具模块入口
│   ├── build.rs                   # 构建配置
│   ├── builtin_templates.json     # 内置模板
│   ├── Cargo.lock                 # Rust 依赖锁定
│   ├── Cargo.toml                 # Rust 依赖配置
│   └── tauri.conf.json           # Tauri 配置
└── package.json
```

## 后端架构

后端采用分层架构设计，模块组织遵循 Rust 最新惯例（不使用 `mod.rs`）：

```
src-tauri/src/
├── commands/      # Tauri 命令层 — 对前端暴露的 API
├── services/      # 业务逻辑层 — 核心业务处理
├── db/            # 数据访问层 — SQLite 操作
├── optimization/  # 性能优化 — Gzip 压缩/缓存/分页/请求合并
├── logging/       # 日志模块 — 错误日志/操作日志
├── utils/         # 工具函数 — 时间处理/数据验证
├── config.rs      # 配置管理（支持热加载）
├── models.rs      # 数据结构定义
├── error.rs       # 统一错误类型
├── settings.rs    # 设置管理
├── state.rs       # 应用状态
└── lib.rs         # 入口：模块声明 + 命令注册
```

## 后端命令一览

所有命令通过 `@tauri-apps/api/core` 的 `invoke()` 调用，约 **70+ 个** Tauri command，覆盖：

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
- 灵感看板获取（支持多语言国际化）

#### 灵感看板国际化设计

灵感看板采用**内部标识符 + 动态翻译**的设计模式：

- **内部标识符（column_key）**：使用稳定的内部键（如 `inspiration`、`dialogue`、`scene`）作为列的唯一标识，不受语言切换影响
- **显示名称（column_name）**：根据当前语言环境动态翻译显示，支持中英文切换
- **数据库迁移**：自动将旧版语言相关的列名映射到新的内部标识符，确保数据兼容性

**默认列标识符与翻译映射：**

| 内部键        | 中文显示 | 英文显示    |
| ------------- | -------- | ----------- |
| `inspiration` | 灵感     | Inspiration |
| `dialogue`    | 对白     | Dialogue    |
| `scene`       | 场景     | Scene       |

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

### 性能优化

- Gzip 压缩响应
- 响应缓存
- 请求合并
- 分页查询

### 文件操作

- 图片保存
- 文件读取

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
