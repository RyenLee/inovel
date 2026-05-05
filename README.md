# inovel — 交互式小说创作工具

基于 Tauri 2 + Vue 3 + Rust 的桌面小说写作软件，提供富文本编辑、世界观管理、关系图谱、时间轴、版本快照等功能。

## 技术栈

| 层     | 技术                          |
|--------|-------------------------------|
| 前端   | Vue 3 + TypeScript + Vite 6   |
| UI 库  | Naive UI + Tailwind CSS       |
| 编辑器 | Tiptap (ProseMirror)          |
| 后端   | Rust (Tauri 2)                |
| 数据库 | SQLite (rusqlite)             |
| 版本   | Git (git2)                    |

## 功能

- **Markdown 编辑器** — Tiptap 富文本编辑，支持加粗/斜体/标题/列表/引用
- **写作模式** — 普通/打字机/专注三种模式
- **章节管理** — 分卷分章树形结构，拖拽排序
- **世界观管理** — 人物/地点/组织 CRUD，自定义字段
- **关系图谱** — 可视化人物关系网络（@vue-flow）
- **故事时间轴** — 按故事时间组织事件，关联章节跳转
- **@提及** — 在正文中 `@人物/地点/组织`，悬停预览详情，点击跳转
- **版本快照** — 自动 Git 提交，版本历史对话框，版本对比（diff），一键恢复
- **名称生成器** — 随机生成中/英文姓名、地名
- **写作统计** — 每日字数统计、目标进度
- **暗色模式**

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
│   │   ├── MarkdownEditor.vue    # 核心编辑器
│   │   ├── TreeSidebar.vue       # 章节树侧栏
│   │   ├── OutlinePanel.vue      # 大纲面板
│   │   ├── WorldbuildingPanel.vue # 世界观管理
│   │   ├── RelationshipGraph.vue # 关系图谱
│   │   ├── Timeline.vue          # 故事时间轴
│   │   ├── MentionExtension.ts   # @提及扩展
│   │   ├── MentionNode.vue       # @提及节点渲染
│   │   └── HistoryDialog.vue     # 版本历史对话框
│   ├── stores/                   # Pinia 状态管理
│   │   ├── project.ts
│   │   ├── editor.ts
│   │   └── worldbuilding.ts
│   ├── views/
│   │   ├── EditorPage.vue        # 主编辑器页面
│   │   └── HomePage.vue          # 主页
│   └── composables/              # 组合式函数
├── src-tauri/                    # Rust 后端
│   └── src/
│       ├── lib.rs                # 入口：模块声明 + 命令注册
│       ├── main.rs               # Tauri 入口
│       ├── models.rs             # 数据结构（30+ 模型）
│       ├── db.rs                 # 数据库初始化
│       ├── project.rs            # 项目 CRUD
│       ├── chapter.rs            # 卷/章节 CRUD
│       ├── writing.rs            # 写作目标/统计
│       ├── names.rs              # 名称生成器
│       ├── worldbuilding.rs      # 人物/地点/组织
│       ├── relationship.rs       # 关系图谱
│       ├── timeline.rs           # 时间轴事件
│       └── git_snapshot.rs       # Git 版本控制
└── package.json
```

## 后端命令一览

所有命令通过 `@tauri-apps/api/core` 的 `invoke()` 调用，约 **50 个** Tauri command，覆盖：

- 项目：创建/打开/删除/更新
- 章节：增删改查/排序/内容读写
- 世界观：人物/地点/组织的 CRUD
- 关系：创建/更新/删除/列表
- 事件：时间轴事件的 CRUD
- 写作：目标/记录/统计
- 名称：按类型随机生成
- Git：初始化/快照/历史/恢复/差异对比

## 许可证

MIT
