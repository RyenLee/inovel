# iNovel 国际化（i18n）设计文档 — 英语语言支持

| 属性     | 值                                       |
| -------- | ---------------------------------------- |
| 文档版本 | 2.0                                      |
| 创建日期 | 2026-05                                  |
| 适用项目 | iNovel（Vue 3 + TypeScript + Tauri）     |
| 目标语言 | 中文（zh-CN，默认）→ 英语（en-US，新增） |

---

## 目录

1. [现状分析](#1-现状分析)
2. [语言架构设计](#2-语言架构设计)
3. [翻译管理工作流程](#3-翻译管理工作流程)
4. [技术实现方案](#4-技术实现方案)
5. [UI 适配要求](#5-ui-适配要求)
6. [测试策略](#6-测试策略)
7. [性能考量](#7-性能考量)
8. [维护程序](#8-维护程序)
9. [术语表与翻译对照表](#9-术语表与翻译对照表)
10. [实施路线图](#10-实施路线图)

---

## 1. 现状分析

### 1.1 硬编码中文统计

通过对项目源码的全面扫描，当前硬编码中文字符串分布如下：

| 层级                | 文件数 | 中文出现次数 | 说明                         |
| ------------------- | ------ | ------------ | ---------------------------- |
| Vue 组件（`.vue`）  | 24     | 6,370        | 模板文本、属性绑定、消息提示 |
| TypeScript（`.ts`） | 23     | 3,030        | Store、Composable、类型定义  |
| Rust 后端（`.rs`）  | 37     | 20,066       | 错误消息、日志、枚举字典     |
| **合计**            | **84** | **29,466**   | —                            |

### 1.2 前端字符串类型分类

| 字符串类型          | 典型示例                         | 出现位置            | 优先级 |
| ------------------- | -------------------------------- | ------------------- | ------ |
| **UI 标签**         | `label="书名"`、`title="快捷键"` | 模板属性            | P0     |
| **按钮文本**        | `新建项目`、`保存设置`、`取消`   | `<n-button>` 内容   | P0     |
| **消息提示**        | `message.success("保存成功")`    | Script 逻辑         | P0     |
| **对话框标题/内容** | `title="确认删除项目"`           | `<n-modal>` 属性    | P0     |
| **占位文本**        | `placeholder="请输入书名"`       | `<n-input>` 属性    | P0     |
| **表单验证**        | `message: '请输入书名'`          | `:rule` 属性        | P1     |
| **空状态描述**      | `description="暂无项目"`         | `<n-empty>` 属性    | P1     |
| **枚举字典**        | `{ name: '男', code: 'male' }`   | `enumDictionary.ts` | P1     |
| **内置模板**        | `builtin_templates.json`         | Tauri 资源          | P2     |
| **错误页面**        | `加载失败`、`重新加载`           | `main.ts`           | P2     |
| **注释**            | `// 禁用浏览器前进/后退功能`     | 各文件              | 不翻译 |

### 1.3 后端字符串类型分类

| 字符串类型           | 典型示例                               | 处理策略              |
| -------------------- | -------------------------------------- | --------------------- |
| **AppError 枚举**    | `"数据库错误: {0}"`、`"验证错误: {0}"` | 使用错误码 + 前端映射 |
| **format! 错误消息** | `format!("打开数据库失败: {}", e)`     | 改用错误码            |
| **枚举字典数据**     | `enum_dict.rs` 中的中文枚举名          | 数据库存储多语言字段  |
| **日志消息**         | `info!("项目创建成功")`                | 保持中文（仅开发用）  |
| **内置资源**         | `builtin_templates.json`、`names.json` | 提供多语言版本        |

### 1.4 涉及的关键文件清单

**Views（6 个）：**

- `WelcomePage.vue` — 欢迎页、项目管理、迁移
- `EditorPage.vue` — 编辑器主页面
- `SettingsPage.vue` — 应用设置
- `ProjectSettingsPage.vue` — 项目设置
- `StatsDashboard.vue` — 全局统计
- `ProjectStatsDashboard.vue` — 项目统计

**Components（17 个）：**

- `TreeSidebar.vue`、`MarkdownEditor.vue`、`WorldbuildingPanel.vue`
- `Timeline.vue`、`RelationshipGraph.vue`、`InspirationBoard.vue`
- `BackupDialog.vue`、`ExportDialog.vue`、`HistoryDialog.vue`
- `PomodoroTimer.vue`、`OutlinePanel.vue`、`TemplateSelector.vue`
- `SensitiveWordsManager.vue`、`ShortcutSettings.vue`
- `TextImportDialog.vue`、`DeleteConfirmModal.vue`、`MentionNode.vue`

**Composables（8 个）：**

- `useEditor.ts`、`useEditorLayout.ts`、`useTextBeautify.ts`
- `useTextImport.ts`、`useFolderDialog.ts`、`useWordCount.ts`
- `useGlobalShortcuts.ts`、`useTheme.ts`

**Stores（7 个）：**

- `project.ts`、`editor.ts`、`tree.ts`、`template.ts`
- `worldbuilding.ts`、`shortcuts.ts`、`enumDictionary.ts`

**入口/配置：**

- `main.ts`、`App.vue`、`router/index.ts`

---

## 2. 语言架构设计

### 2.1 资源目录结构

```
src/
├── locales/                         # 语言资源根目录
│   ├── zh-CN/                       # 简体中文（默认语言）
│   │   ├── common.json              # 通用：按钮、标签、状态、时间
│   │   ├── project.json             # 项目：创建、编辑、删除、迁移
│   │   ├── editor.json              # 编辑器：工具栏、快捷键、模式
│   │   ├── sidebar.json             # 侧边栏：章节树、卷管理
│   │   ├── worldbuilding.json       # 世界观：人物、地点、组织
│   │   ├── timeline.json            # 时间线：事件管理
│   │   ├── relationship.json        # 关系图：人物关系
│   │   ├── settings.json            # 设置：应用设置、项目设置
│   │   ├── stats.json               # 统计：写作数据、热力图
│   │   ├── backup.json              # 备份：创建、恢复、管理
│   │   ├── export.json              # 导出：格式选择、进度
│   │   ├── inspiration.json         # 灵感：看板、卡片
│   │   ├── pomodoro.json            # 番茄钟：计时、设置
│   │   ├── sensitive.json           # 敏感词：管理、检测
│   │   ├── template.json            # 模板：选择、管理
│   │   ├── import.json              # 导入：文本导入
│   │   └── error.json               # 错误消息：后端错误码映射
│   └── en-US/                       # 美式英语
│       ├── common.json
│       ├── project.json
│       ├── editor.json
│       ├── sidebar.json
│       ├── worldbuilding.json
│       ├── timeline.json
│       ├── relationship.json
│       ├── settings.json
│       ├── stats.json
│       ├── backup.json
│       ├── export.json
│       ├── inspiration.json
│       ├── pomodoro.json
│       ├── sensitive.json
│       ├── template.json
│       ├── import.json
│       └── error.json
└── i18n/
    ├── index.ts                     # i18n 实例创建与配置
    ├── types.ts                     # 类型定义（Schema 类型）
    └── composables/
        └── useLocale.ts             # 语言切换 composable
```

### 2.2 资源文件职责划分

| 文件                 | 职责                                        | 对应组件                              | 预估键数 |
| -------------------- | ------------------------------------------- | ------------------------------------- | -------- |
| `common.json`        | 通用按钮、标签、状态词、时间格式、确认/取消 | 所有组件                              | ~80      |
| `project.json`       | 项目 CRUD、迁移、加密/解密                  | WelcomePage, ProjectSettingsPage      | ~60      |
| `editor.json`        | 编辑器模式、保存、快照、名称生成            | EditorPage, MarkdownEditor            | ~70      |
| `sidebar.json`       | 章节树、卷管理、状态切换                    | TreeSidebar, OutlinePanel             | ~50      |
| `worldbuilding.json` | 人物/地点/组织属性、表单                    | WorldbuildingPanel                    | ~60      |
| `timeline.json`      | 时间线事件 CRUD                             | Timeline                              | ~30      |
| `relationship.json`  | 关系图节点/边操作                           | RelationshipGraph                     | ~30      |
| `settings.json`      | 应用/项目设置表单                           | SettingsPage, ProjectSettingsPage     | ~50      |
| `stats.json`         | 统计面板、热力图、趋势                      | StatsDashboard, ProjectStatsDashboard | ~40      |
| `backup.json`        | 备份创建/恢复/删除                          | BackupDialog                          | ~25      |
| `export.json`        | 导出格式、进度                              | ExportDialog                          | ~20      |
| `inspiration.json`   | 灵感看板、卡片管理                          | InspirationBoard                      | ~25      |
| `pomodoro.json`      | 番茄钟计时、设置                            | PomodoroTimer                         | ~30      |
| `sensitive.json`     | 敏感词管理                                  | SensitiveWordsManager                 | ~20      |
| `template.json`      | 模板选择器                                  | TemplateSelector                      | ~20      |
| `import.json`        | 文本导入                                    | TextImportDialog                      | ~15      |
| `error.json`         | 后端错误码 → 前端友好消息                   | 全局                                  | ~60      |
| **合计**             |                                             |                                       | **~685** |

### 2.3 键名命名规范

采用 **点分隔层级命名法**，格式为 `namespace.element.qualifier`：

```
{文件域}.{UI元素}.{语义限定词}
```

**规则：**

| 规则          | 说明                   | 正确示例                       | 错误示例                                  |
| ------------- | ---------------------- | ------------------------------ | ----------------------------------------- |
| 使用英文键名  | 键名不使用中文         | `project.create.title`         | `project.新建.标题`                       |
| 语义化命名    | 键名表达含义而非位置   | `editor.save.success`          | `editor.button1.text`                     |
| 层级不超过4层 | 避免过深嵌套           | `sidebar.chapter.status.draft` | `app.editor.sidebar.chapter.status.draft` |
| 复用通用键    | 通用操作使用 common 域 | `common.action.save`           | 每个域单独定义"保存"                      |

**示例：**

```jsonc
// common.json
{
  "action": {
    "save": "保存",
    "cancel": "取消",
    "confirm": "确认",
    "delete": "删除",
    "edit": "编辑",
    "create": "新建",
    "close": "关闭",
    "refresh": "刷新",
    "back": "返回"
  },
  "status": {
    "loading": "加载中...",
    "success": "操作成功",
    "error": "操作失败",
    "empty": "暂无数据"
  },
  "time": {
    "day": "天",
    "minute": "分钟",
    "second": "秒"
  }
}

// project.json
{
  "header": {
    "title": "小说工坊"
  },
  "create": {
    "button": "新建项目",
    "title": "新建项目",
    "success": "项目创建成功！",
    "error": "创建项目失败"
  },
  "form": {
    "name": "书名",
    "namePlaceholder": "请输入书名（必填）",
    "nameRequired": "请输入书名",
    "author": "笔名",
    "authorPlaceholder": "请输入作者笔名",
    "description": "简介",
    "descriptionPlaceholder": "请输入小说简介",
    "path": "存储路径",
    "pathPlaceholder": "请选择项目存储路径（必填）",
    "pathRequired": "请选择存储路径",
    "pathSelect": "选择"
  }
}
```

### 2.4 语言资源访问方式

| 访问场景          | 方式                        | 示例                                           |
| ----------------- | --------------------------- | ---------------------------------------------- |
| 组件模板中        | `{{ t('key') }}`            | `{{ t('project.header.title') }}`              |
| 组件属性绑定      | `:title="t('key')"`         | `:title="t('common.action.save')"`             |
| Script 逻辑中     | `const { t } = useI18n()`   | `message.success(t('project.create.success'))` |
| Naive UI 组件属性 | `:label="t('key')"`         | `:label="t('project.form.name')"`              |
| 带插值            | `t('key', { count: 5 })`    | `t('stats.wordsCount', { count: 1234 })`       |
| 带复数            | `t('key', { count: n }, n)` | `t('project.items', { count: 3 }, 3)`          |

### 2.5 语言偏好持久化

| 存储位置                         | 用途             | 读取时机             |
| -------------------------------- | ---------------- | -------------------- |
| `localStorage` (`inovel_locale`) | 前端语言偏好     | 应用启动时           |
| Tauri `settings.rs`              | 后端感知当前语言 | 后端返回本地化消息时 |
| 系统语言检测                     | 首次启动默认值   | 无存储偏好时         |

**语言检测优先级：**

```
localStorage 存储值 > 系统语言检测 > 默认值(zh-CN)
```

---

## 3. 翻译管理工作流程

### 3.1 整体流程

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  字符串提取   │───▶│  翻译创建     │───▶│  审查校验     │───▶│  集成发布     │
│  Extract     │    │  Translate   │    │  Review      │    │  Integrate   │
└──────────────┘    └──────────────┘    └──────────────┘    └──────────────┘
       │                   │                   │                   │
   开发者识别          翻译人员填写         审查人员核对         开发者合并
   硬编码字符串       en-US JSON 文件      准确性/一致性        代码+测试
```

### 3.2 各阶段详细说明

#### 阶段一：字符串提取

| 步骤 | 操作                         | 负责人 | 产出物               |
| ---- | ---------------------------- | ------ | -------------------- |
| 1.1  | 识别组件中所有硬编码中文     | 开发者 | 标记清单             |
| 1.2  | 将中文替换为 `t('key')` 调用 | 开发者 | 代码变更             |
| 1.3  | 在 `zh-CN/` 下创建对应键值   | 开发者 | zh-CN JSON           |
| 1.4  | 在 `en-US/` 下创建占位键     | 开发者 | en-US JSON（待翻译） |

**提取规则：**

- 必须提取：所有用户可见文本（按钮、标签、提示、错误消息）
- 不提取：代码注释、console.log 内容、开发者调试信息
- 特殊处理：含变量的字符串使用插值语法 `t('key', { var })`

#### 阶段二：翻译创建

| 步骤 | 操作                       | 负责人   | 产出物      |
| ---- | -------------------------- | -------- | ----------- |
| 2.1  | 翻译 en-US JSON 中的占位键 | 翻译人员 | 翻译后 JSON |
| 2.2  | 标注不确定/需上下文的条目  | 翻译人员 | 疑问清单    |
| 2.3  | 补充上下文截图或说明       | 开发者   | 上下文文档  |

**翻译规范：**

- 保持占位符格式：`{name}` → `{name}`，不翻译占位符本身
- 保持标点习惯：英文使用英文标点，句末加句号
- 尊重英文大小写规范：按钮文本首字母大写（Title Case），描述文本句首大写
- 术语一致性：参照本文档第 9 节术语对照表

#### 阶段三：审查校验

| 审查维度   | 检查内容                           | 通过标准            |
| ---------- | ---------------------------------- | ------------------- |
| **准确性** | 翻译是否准确传达原文含义           | 无误译、漏译        |
| **一致性** | 同一术语在不同位置翻译是否一致     | 与术语表 100% 一致  |
| **完整性** | en-US JSON 键是否与 zh-CN 完全对应 | 键集合完全一致      |
| **格式**   | 占位符、换行、特殊字符是否保留     | 格式无损坏          |
| **长度**   | 英文文本是否导致 UI 溢出           | 无溢出（见第 5 节） |

#### 阶段四：集成发布

| 步骤 | 操作                   | 负责人 |
| ---- | ---------------------- | ------ |
| 4.1  | 合并翻译文件到代码仓库 | 开发者 |
| 4.2  | 运行自动化测试         | CI     |
| 4.3  | 手动验证语言切换       | QA     |
| 4.4  | 发布版本               | DevOps |

### 3.3 翻译质量保证机制

```
┌─────────────────────────────────────────────────┐
│              翻译质量保证流水线                    │
├─────────────────────────────────────────────────┤
│                                                 │
│  ① JSON 语法校验                                │
│     └─ 确保所有翻译文件为合法 JSON               │
│                                                 │
│  ② 键完整性校验                                 │
│     └─ zh-CN 与 en-US 键集合完全匹配            │
│                                                 │
│  ③ 占位符一致性校验                             │
│     └─ {var} 在两种语言中数量和名称一致          │
│                                                 │
│  ④ 术语一致性校验                               │
│     └─ 同一中文词在不同位置对应同一英文翻译       │
│                                                 │
│  ⑤ 长度预警                                     │
│     └─ 英文文本超过中文 150% 时标记警告          │
│                                                 │
└─────────────────────────────────────────────────┘
```

---

## 4. 技术实现方案

### 4.1 i18n 库选型

**选定：`vue-i18n@next`（v10.x）**

| 评估维度              | vue-i18n            | i18next     | lingui      |
| --------------------- | ------------------- | ----------- | ----------- |
| Vue 3 Composition API | ✅ 原生支持         | ⚠️ 需适配层 | ⚠️ 需适配层 |
| TypeScript 类型安全   | ✅ 完整类型         | ✅ 完整类型 | ✅ 完整类型 |
| SFC 集成              | ✅ `{{ t() }}` 原生 | ⚠️ 需配置   | ⚠️ 需配置   |
| Naive UI 兼容         | ✅ 已验证           | ✅ 已验证   | ⚠️ 未验证   |
| 按需加载              | ✅ 支持             | ✅ 支持     | ✅ 支持     |
| 社区活跃度            | ⭐⭐⭐⭐⭐          | ⭐⭐⭐⭐⭐  | ⭐⭐⭐      |
| 学习曲线              | 低                  | 中          | 中          |
| 包体积                | ~12KB gzip          | ~15KB gzip  | ~10KB gzip  |

**选择理由：** vue-i18n 是 Vue 官方推荐的 i18n 方案，与 Vue 3 Composition API 深度集成，TypeScript 支持完善，Naive UI 社区广泛使用，学习成本最低。

### 4.2 依赖安装

```bash
npm install vue-i18n@next
```

### 4.3 核心配置

**`src/i18n/index.ts`：**

```typescript
import { createI18n } from "vue-i18n";
import zhCN from "../locales/zh-CN";
import enUS from "../locales/en-US";

export type MessageSchema = typeof zhCN;

const i18n = createI18n<[MessageSchema], "zh-CN" | "en-US">({
  legacy: false,
  locale: localStorage.getItem("inovel_locale") || detectSystemLocale(),
  fallbackLocale: "zh-CN",
  messages: {
    "zh-CN": zhCN,
    "en-US": enUS,
  },
  missingWarn: import.meta.env.DEV,
  fallbackWarn: import.meta.env.DEV,
});

function detectSystemLocale(): "zh-CN" | "en-US" {
  const lang = navigator.language;
  if (lang.startsWith("zh")) return "zh-CN";
  return "en-US";
}

export default i18n;

export async function setLocale(locale: "zh-CN" | "en-US") {
  i18n.global.locale.value = locale;
  localStorage.setItem("inovel_locale", locale);
  document.documentElement.setAttribute("lang", locale);
}
```

**`src/i18n/composables/useLocale.ts`：**

```typescript
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { setLocale } from "../index";
import type { MessageSchema } from "../index";

export function useLocale() {
  const { t, locale } = useI18n<[MessageSchema], "zh-CN" | "en-US">();

  const isZhCN = computed(() => locale.value === "zh-CN");
  const isEnUS = computed(() => locale.value === "en-US");

  const availableLocales = [
    { label: "简体中文", value: "zh-CN" as const },
    { label: "English", value: "en-US" as const },
  ];

  const switchLocale = async (newLocale: "zh-CN" | "en-US") => {
    await setLocale(newLocale);
  };

  return {
    t,
    locale,
    isZhCN,
    isEnUS,
    availableLocales,
    switchLocale,
  };
}
```

**`src/main.ts` 集成点：**

```typescript
import { createApp } from "vue";
import { createPinia } from "pinia";
import i18n from "./i18n";
import "./style.css";
import App from "./App.vue";
import router from "./router";

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);
app.use(i18n);

app.mount("#app");
```

### 4.4 Naive UI 国际化集成

Naive UI 组件库自带国际化支持，需要通过 `n-config-provider` 的 `locale` 属性传递：

**`src/App.vue` 修改：**

```vue
<script setup lang="ts">
import { computed } from "vue";
import { zhCN, dateZhCN, enUS, dateEnUS } from "naive-ui";
import { useLocale } from "./i18n/composables/useLocale";

const { locale } = useLocale();

const naiveUILocale = computed(() => (locale.value === "en-US" ? enUS : zhCN));
const naiveUIDateLocale = computed(() =>
  locale.value === "en-US" ? dateEnUS : dateZhCN,
);
</script>

<template>
  <n-config-provider
    :theme="theme"
    :theme-overrides="themeOverrides"
    :locale="naiveUILocale"
    :date-locale="naiveUIDateLocale"
  >
    <!-- ... -->
  </n-config-provider>
</template>
```

### 4.5 字符串提取方法

#### 方法一：渐进式手动提取（推荐）

按组件逐个提取，确保每个组件提取完整：

**提取前（WelcomePage.vue）：**

```vue
<n-button type="primary" @click="showModal = true">
  新建项目
</n-button>
```

```typescript
message.success("项目创建成功！");
```

**提取后：**

```vue
<n-button type="primary" @click="showModal = true">
  {{ t('project.create.button') }}
</n-button>
```

```typescript
const { t } = useLocale();
message.success(t("project.create.success"));
```

#### 方法二：自动化提取工具

```bash
# 安装提取工具
npm install @intlify/cli --save-dev

# 提取所有源文件中的 t() 调用键
npx intlify extract --src ./src --output ./src/locales/extracted.json
```

### 4.6 回退机制

采用三级回退策略：

```
用户选择的语言 (en-US)
       │
       │ 键不存在
       ▼
回退语言 (zh-CN)
       │
       │ 键仍不存在
       ▼
键名本身作为显示文本
```

**回退配置：**

```typescript
const i18n = createI18n({
  legacy: false,
  locale: "en-US",
  fallbackLocale: "zh-CN", // 第一级回退
  missing: (locale, key) => {
    // 第二级回退 + 开发警告
    if (import.meta.env.DEV) {
      console.warn(`[i18n] Missing key: "${key}" in locale: "${locale}"`);
    }
    return key;
  },
});
```

**回退场景示例：**

| 场景       | en-US 值           | zh-CN 值     | 最终显示                |
| ---------- | ------------------ | ------------ | ----------------------- |
| 正常       | `"Create Project"` | `"新建项目"` | `Create Project`        |
| en-US 缺键 | _(缺失)_           | `"新建项目"` | `新建项目`              |
| 双语缺键   | _(缺失)_           | _(缺失)_     | `project.create.button` |

### 4.7 后端错误消息国际化策略

后端 Rust 代码中的中文错误消息不应直接翻译，而是采用 **错误码 + 前端映射** 策略：

**后端改造（Rust）：**

```rust
// 改造前
#[error("数据库错误: {0}")]
Database(#[from] rusqlite::Error),

// 改造后
#[error("DB_ERROR:{0}")]
Database(#[from] rusqlite::Error),
```

**前端错误码映射（`error.json`）：**

```json
{
  "DB_ERROR": {
    "zh-CN": "数据库错误",
    "en-US": "Database Error"
  },
  "VALIDATION_ERROR": {
    "zh-CN": "验证错误",
    "en-US": "Validation Error"
  },
  "NOT_FOUND": {
    "zh-CN": "未找到",
    "en-US": "Not Found"
  },
  "ENCRYPTION_ERROR": {
    "zh-CN": "加密错误",
    "en-US": "Encryption Error"
  }
}
```

**前端错误处理工具：**

```typescript
// src/i18n/errorMapper.ts
export function mapBackendError(
  error: string,
  t: (key: string) => string,
): string {
  const codeMatch = error.match(/^([A-Z_]+):/);
  if (codeMatch) {
    const code = codeMatch[1];
    const detail = error.substring(code.length + 1);
    const localizedPrefix = t(`error.${code}`);
    return detail ? `${localizedPrefix}: ${detail}` : localizedPrefix;
  }
  return error;
}
```

### 4.8 枚举字典国际化

当前 `enumDictionary.ts` 中的硬编码中文枚举需要支持多语言：

**方案：后端枚举字典增加多语言字段**

```typescript
// 改造后的枚举定义
export interface EnumDefinition {
  category: string;
  code: string;
  name_zh: string; // 中文名称
  name_en: string; // 英文名称
  description_zh: string | null;
  description_en: string | null;
  sort_order: number;
}

// 前端根据当前语言返回对应名称
export function getLocalizedName(
  enumDef: EnumDefinition,
  locale: string,
): string {
  return locale === "en-US" ? enumDef.name_en : enumDef.name_zh;
}
```

### 4.9 内置模板国际化

`builtin_templates.json` 中的模板内容需要提供英文版本：

**方案：增加 `locale` 字段，按语言加载**

```jsonc
// builtin_templates_zh.json
[
  {
    "id": "builtin_blank",
    "name": "空白页",
    "description": "从空白页面开始写作",
    "category": "章节",
    "content": "# 新章节\n\n"
  }
]

// builtin_templates_en.json
[
  {
    "id": "builtin_blank",
    "name": "Blank Page",
    "description": "Start writing from a blank page",
    "category": "Chapter",
    "content": "# New Chapter\n\n"
  }
]
```

---

## 5. UI 适配要求

### 5.1 文本长度变化分析

英文文本相比中文通常**长 20%~40%**，部分场景差异更大：

| 中文原文                   | 英文翻译                          | 扩展率 | 风险等级 |
| -------------------------- | --------------------------------- | ------ | -------- |
| 新建项目                   | Create Project                    | +100%  | 🟡 中    |
| 保存                       | Save                              | +100%  | 🟢 低    |
| 取消                       | Cancel                            | +100%  | 🟢 低    |
| 确认删除项目               | Confirm Delete Project            | +120%  | 🟡 中    |
| 本月字数                   | Words This Month                  | +150%  | 🔴 高    |
| 日均字数                   | Daily Average                     | +100%  | 🟡 中    |
| 写作天数                   | Writing Days                      | +100%  | 🟡 中    |
| 打字机模式已开启           | Typewriter Mode Enabled           | +140%  | 🟡 中    |
| 项目路径已失效             | Project Path Invalid              | +80%   | 🟢 低    |
| 发现 3 个项目需要迁移      | 3 projects need migration         | +50%   | 🟢 低    |
| 编辑器每隔设定时间自动保存 | Auto-save at configured intervals | +60%   | 🟢 低    |

### 5.2 各 UI 元素适配策略

#### 按钮

| 当前实现                        | 问题             | 适配方案                            |
| ------------------------------- | ---------------- | ----------------------------------- |
| `<n-button>新建项目</n-button>` | 英文更长可能溢出 | 使用 `whitespace-nowrap` + 弹性宽度 |
| 按钮组内多个按钮                | 空间不足时挤压   | 允许换行或使用图标替代文字          |

**适配前：**

```vue
<n-button type="primary">新建项目</n-button>
```

**适配后：**

```vue
<n-button type="primary" class="whitespace-nowrap">
  {{ t('project.create.button') }}
</n-button>
```

#### 卡片标题与统计数字

| 当前实现             | 问题                         | 适配方案                          |
| -------------------- | ---------------------------- | --------------------------------- |
| `<p>本月字数</p>`    | 英文 "Words This Month" 过长 | 缩短为 "Monthly Words" 或换行显示 |
| `<p>日均字数</p>`    | 英文过长                     | 缩短为 "Daily Avg." 或两行显示    |
| `{{ totalDays }} 天` | "天" 需翻译                  | `t('common.time.day')`            |

**适配后：**

```vue
<p class="text-xs text-gray-500">{{ t('stats.monthlyWords') }}</p>
<p class="text-xl font-bold">{{ totalWordsThisMonth.toLocaleString() }}</p>
```

#### 表单标签

| 当前实现                   | 问题                        | 适配方案                                           |
| -------------------------- | --------------------------- | -------------------------------------------------- |
| `label="每日字数目标"`     | 英文 "Daily Word Goal" 较长 | Naive UI Form 自动换行                             |
| `placeholder="请输入书名"` | 需翻译                      | `:placeholder="t('project.form.namePlaceholder')"` |

#### 模态框

| 当前实现               | 问题             | 适配方案                             |
| ---------------------- | ---------------- | ------------------------------------ |
| `style="width: 520px"` | 固定宽度可能不够 | 使用 `max-width` + 百分比宽度        |
| `title="确认删除项目"` | 需翻译           | `:title="t('project.delete.title')"` |

**适配后：**

```vue
<n-modal
  v-model:show="showModal"
  preset="card"
  :title="isEditing ? t('project.edit.title') : t('project.create.title')"
  style="max-width: 520px; width: 90vw"
  :mask-closable="false"
>
```

#### 日期/时间格式

| 当前实现                      | 适配方案                          |
| ----------------------------- | --------------------------------- |
| `toLocaleDateString("zh-CN")` | 根据语言动态切换 locale           |
| `"30 秒"`、`"1 分钟"`         | 使用 `t('common.time.second')` 等 |

**适配后：**

```typescript
const dateLocale = computed(() =>
  locale.value === "en-US" ? "en-US" : "zh-CN",
);
new Date(project.created_at).toLocaleDateString(dateLocale.value);
```

### 5.3 需要重点关注的组件

| 组件                     | 风险点                     | 适配优先级 |
| ------------------------ | -------------------------- | ---------- |
| `WelcomePage.vue`        | 统计卡片文字、项目卡片信息 | P0         |
| `SettingsPage.vue`       | 表单标签、下拉选项         | P0         |
| `TreeSidebar.vue`        | 章节状态标签、右键菜单     | P0         |
| `WorldbuildingPanel.vue` | 大量表单标签               | P1         |
| `StatsDashboard.vue`     | 统计图表标题               | P1         |
| `BackupDialog.vue`       | 表格列标题                 | P1         |
| `PomodoroTimer.vue`      | 计时器显示文本             | P2         |

### 5.4 CSS 适配规范

```css
/* 禁止固定宽度截断文本 */
.i18n-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 允许文本换行的容器 */
.i18n-wrap {
  overflow-wrap: break-word;
  word-break: break-word;
}

/* 弹性按钮组 */
.i18n-button-group {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
```

---

## 6. 测试策略

### 6.1 测试层级

```
┌─────────────────────────────────────────────────────────┐
│                      E2E 测试                           │
│          语言切换全流程、跨页面一致性                      │
├─────────────────────────────────────────────────────────┤
│                    集成测试                              │
│       组件渲染验证、Naive UI locale 集成                  │
├─────────────────────────────────────────────────────────┤
│                    单元测试                              │
│    翻译键完整性、插值正确性、回退机制、错误码映射           │
├─────────────────────────────────────────────────────────┤
│                    静态检查                              │
│       JSON 语法、键完整性、占位符一致性                    │
└─────────────────────────────────────────────────────────┘
```

### 6.2 静态检查（CI 阶段）

**翻译文件完整性校验脚本：**

```typescript
// scripts/check-i18n.ts
import zhCN from "../src/locales/zh-CN";
import enUS from "../src/locales/en-US";

function getAllKeys(obj: Record<string, unknown>, prefix = ""): string[] {
  const keys: string[] = [];
  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;
    if (typeof value === "object" && value !== null) {
      keys.push(...getAllKeys(value as Record<string, unknown>, fullKey));
    } else {
      keys.push(fullKey);
    }
  }
  return keys;
}

function checkCompleteness() {
  const zhKeys = new Set(getAllKeys(zhCN));
  const enKeys = new Set(getAllKeys(enUS));

  const missingInEn = [...zhKeys].filter((k) => !enKeys.has(k));
  const missingInZh = [...enKeys].filter((k) => !zhKeys.has(k));

  if (missingInEn.length > 0) {
    console.error("❌ Missing in en-US:", missingInEn);
  }
  if (missingInZh.length > 0) {
    console.error("❌ Missing in zh-CN:", missingInZh);
  }
  if (missingInEn.length === 0 && missingInZh.length === 0) {
    console.log("✅ All locale keys are in sync");
  }

  process.exit(missingInEn.length + missingInZh.length > 0 ? 1 : 0);
}

checkCompleteness();
```

**占位符一致性校验：**

```typescript
// scripts/check-placeholders.ts
function checkPlaceholders(
  zhObj: Record<string, unknown>,
  enObj: Record<string, unknown>,
) {
  const placeholderRegex = /\{(\w+)\}/g;
  const issues: string[] = [];

  function traverse(zh: unknown, en: unknown, path: string) {
    if (typeof zh === "string" && typeof en === "string") {
      const zhPlaceholders = [...zh.matchAll(placeholderRegex)].map(
        (m) => m[1],
      );
      const enPlaceholders = [...en.matchAll(placeholderRegex)].map(
        (m) => m[1],
      );

      const missing = zhPlaceholders.filter((p) => !enPlaceholders.includes(p));
      const extra = enPlaceholders.filter((p) => !zhPlaceholders.includes(p));

      if (missing.length > 0 || extra.length > 0) {
        issues.push(`Key "${path}": missing=${missing}, extra=${extra}`);
      }
    }
  }

  // ... traverse logic

  return issues;
}
```

### 6.3 单元测试

```typescript
// src/tests/i18n.spec.ts
import { describe, it, expect, beforeEach } from "vitest";
import { createI18n } from "vue-i18n";
import zhCN from "../locales/zh-CN";
import enUS from "../locales/en-US";

describe("i18n 国际化", () => {
  let i18n: ReturnType<typeof createI18n>;

  beforeEach(() => {
    i18n = createI18n({
      legacy: false,
      locale: "zh-CN",
      fallbackLocale: "zh-CN",
      messages: { "zh-CN": zhCN, "en-US": enUS },
    });
  });

  describe("中文语言包", () => {
    it("应正确加载中文通用键", () => {
      expect(i18n.global.t("common.action.save")).toBe("保存");
      expect(i18n.global.t("common.action.cancel")).toBe("取消");
    });

    it("应正确加载项目相关键", () => {
      expect(i18n.global.t("project.header.title")).toBe("小说工坊");
    });
  });

  describe("英文语言包", () => {
    beforeEach(() => {
      i18n.global.locale.value = "en-US";
    });

    it("应正确加载英文通用键", () => {
      expect(i18n.global.t("common.action.save")).toBe("Save");
      expect(i18n.global.t("common.action.cancel")).toBe("Cancel");
    });

    it("应正确加载项目相关键", () => {
      expect(i18n.global.t("project.header.title")).toBe("iNovel");
    });
  });

  describe("回退机制", () => {
    it("英文缺键时应回退到中文", () => {
      i18n.global.locale.value = "en-US";
      i18n.global.setLocaleMessage("en-US", {});
      expect(i18n.global.t("common.action.save")).toBe("保存");
    });
  });

  describe("插值", () => {
    it("应正确替换占位符", () => {
      i18n.global.locale.value = "en-US";
      expect(i18n.global.t("stats.wordsCount", { count: 1234 })).toBe(
        "1,234 words",
      );
    });
  });
});
```

### 6.4 集成测试

```typescript
// src/tests/i18n-component.spec.ts
import { mount } from "@vue/test-utils";
import { describe, it, expect } from "vitest";
import { createI18n } from "vue-i18n";
import DeleteConfirmModal from "../components/DeleteConfirmModal.vue";

describe("DeleteConfirmModal 国际化", () => {
  function createWrapper(locale: string) {
    const i18n = createI18n({
      legacy: false,
      locale,
      messages: { "zh-CN": zhCN, "en-US": enUS },
    });

    return mount(DeleteConfirmModal, {
      global: { plugins: [i18n] },
      props: { show: true, title: "Test" },
    });
  }

  it("中文环境下应显示中文按钮", () => {
    const wrapper = createWrapper("zh-CN");
    expect(wrapper.text()).toContain("取消");
    expect(wrapper.text()).toContain("删除");
  });

  it("英文环境下应显示英文按钮", () => {
    const wrapper = createWrapper("en-US");
    expect(wrapper.text()).toContain("Cancel");
    expect(wrapper.text()).toContain("Delete");
  });
});
```

### 6.5 手动测试清单

| 测试项     | 验证内容                   | 通过标准             |
| ---------- | -------------------------- | -------------------- |
| 语言切换   | 设置页面切换语言           | 所有页面文本即时更新 |
| 持久化     | 重启应用后语言保持         | 与上次选择一致       |
| 欢迎页     | 项目列表、统计卡片         | 无文本溢出、布局正常 |
| 编辑器     | 工具栏、侧边栏、快捷键提示 | 所有文本已翻译       |
| 世界观面板 | 人物/地点/组织表单         | 标签和占位符已翻译   |
| 统计面板   | 图表标题、热力图提示       | 日期格式正确         |
| 设置页面   | 表单标签、下拉选项         | 选项文本已翻译       |
| 备份/导出  | 对话框标题和按钮           | 所有文本已翻译       |
| 错误消息   | 触发各类错误               | 显示友好英文消息     |
| Naive UI   | 日期选择器、分页等         | 组件内置文本已翻译   |
| 空状态     | 无项目、无数据             | 空状态提示已翻译     |
| 对话框     | 确认/取消按钮              | 按钮文本已翻译       |

---

## 7. 性能考量

### 7.1 加载策略

| 策略         | 说明                          | 适用场景            |
| ------------ | ----------------------------- | ------------------- |
| **同步加载** | 将当前语言包直接打包进 bundle | 默认语言（zh-CN）   |
| **按需加载** | 切换语言时动态 import         | 非默认语言（en-US） |
| **预加载**   | 空闲时预加载备选语言          | 用户可能切换语言    |

### 7.2 具体实现

**默认语言同步加载 + 非默认语言异步加载：**

```typescript
// src/i18n/index.ts
import { createI18n } from "vue-i18n";
import zhCN from "../locales/zh-CN";

const i18n = createI18n({
  legacy: false,
  locale: localStorage.getItem("inovel_locale") || "zh-CN",
  fallbackLocale: "zh-CN",
  messages: {
    "zh-CN": zhCN,
  },
});

const loadedLocales = new Set(["zh-CN"]);

export async function loadLocaleAsync(locale: string): Promise<void> {
  if (loadedLocales.has(locale)) return;

  const messages = await import(`../locales/${locale}/index.ts`);
  i18n.global.setLocaleMessage(locale, messages.default);
  loadedLocales.add(locale);
}

export async function setLocale(locale: "zh-CN" | "en-US") {
  await loadLocaleAsync(locale);
  i18n.global.locale.value = locale;
  localStorage.setItem("inovel_locale", locale);
  document.documentElement.setAttribute("lang", locale);
}

export default i18n;
```

**空闲时预加载：**

```typescript
// 在 App.vue 的 onMounted 中
onMounted(() => {
  if ("requestIdleCallback" in window) {
    requestIdleCallback(() => {
      const altLocale = locale.value === "zh-CN" ? "en-US" : "zh-CN";
      loadLocaleAsync(altLocale);
    });
  }
});
```

### 7.3 资源体积估算

| 语言包 | 预估键数 | 预估 JSON 体积 | Gzip 后 |
| ------ | -------- | -------------- | ------- |
| zh-CN  | ~685     | ~15 KB         | ~4 KB   |
| en-US  | ~685     | ~18 KB         | ~5 KB   |

**总体影响：**

- vue-i18n 库：~12 KB gzip
- 默认语言包（zh-CN）：~4 KB gzip
- 非默认语言包按需加载：~5 KB gzip
- **首屏增加：~16 KB gzip**（库 + 默认语言包）

### 7.4 缓存策略

| 缓存层       | 机制                | 说明                           |
| ------------ | ------------------- | ------------------------------ |
| Vite 构建    | 模块哈希            | 语言文件变更时仅更新对应 chunk |
| HTTP 缓存    | Cache-Control       | 静态资源长期缓存               |
| 内存缓存     | `loadedLocales` Set | 避免重复加载已加载的语言       |
| localStorage | `inovel_locale`     | 持久化用户语言偏好             |

### 7.5 运行时性能

| 关注点         | 影响                      | 优化措施                            |
| -------------- | ------------------------- | ----------------------------------- |
| `t()` 调用频率 | 每次渲染都执行            | 键查找为 O(1) 对象属性访问，可忽略  |
| 响应式更新     | 语言切换时所有 `t()` 重算 | vue-i18n 内部已优化，仅更新依赖组件 |
| 大型 JSON 解析 | 首次加载时解析            | JSON 解析速度极快（<1ms for 20KB）  |

---

## 8. 维护程序

### 8.1 新增字符串流程

```
开发者编写代码
      │
      ▼
在组件中使用 t('new.feature.label')
      │
      ▼
在 zh-CN 对应文件中添加键值
      │
      ▼
在 en-US 对应文件中添加键值（可先留空）
      │
      ▼
提交 PR，CI 自动检查键完整性
      │
      ▼
翻译人员补充英文翻译
      │
      ▼
审查人员校验
      │
      ▼
合并发布
```

### 8.2 更新已有翻译

| 变更类型     | 流程                                | 影响范围  |
| ------------ | ----------------------------------- | --------- |
| 修改中文原文 | 同步修改 zh-CN → 通知翻译更新 en-US | 对应键    |
| 修改英文翻译 | 直接修改 en-US                      | 对应键    |
| 删除字符串   | 同时从 zh-CN 和 en-US 删除          | 对应键    |
| 移动字符串   | 同时移动两个语言包中的键            | 旧键+新键 |

### 8.3 版本控制规范

- 语言文件与代码在同一仓库，同一 PR 管理
- 新增功能 PR 必须包含对应的语言文件变更
- CI 流水线包含键完整性检查步骤
- 发布时在 CHANGELOG 中记录翻译变更

### 8.4 翻译覆盖率监控

**CI 检查脚本（集成到 `release.yml`）：**

```yaml
# .github/workflows/release.yml 新增步骤
- name: Check i18n completeness
  run: npx tsx scripts/check-i18n.ts
```

**覆盖率报告：**

```typescript
// scripts/i18n-coverage.ts
function calculateCoverage(zhKeys: Set<string>, enKeys: Set<string>) {
  const total = zhKeys.size;
  const translated = [...zhKeys].filter((k) => enKeys.has(k)).length;
  const coverage = ((translated / total) * 100).toFixed(1);

  console.log(`翻译覆盖率: ${coverage}% (${translated}/${total})`);

  if (parseFloat(coverage) < 100) {
    const missing = [...zhKeys].filter((k) => !enKeys.has(k));
    console.warn("未翻译的键:", missing.slice(0, 20));
  }
}
```

### 8.5 废弃字符串清理

定期（每个大版本）执行清理：

1. 扫描代码中所有 `t('key')` 调用，提取实际使用的键集合
2. 对比语言文件中的键集合
3. 删除不再使用的键
4. 提交清理 PR

---

## 9. 术语表与翻译对照表

### 9.1 核心术语对照

| 中文       | English            | 上下文      | 备注               |
| ---------- | ------------------ | ----------- | ------------------ |
| 小说工坊   | iNovel             | 应用名称    | 保持品牌名         |
| 项目       | Project            | 项目管理    | 不用 "Item"        |
| 书名       | Book Title         | 项目表单    | 不用 "Name"        |
| 笔名       | Pen Name           | 作者信息    | 不用 "Author"      |
| 简介       | Synopsis           | 项目描述    | 不用 "Description" |
| 章节       | Chapter            | 章节管理    | —                  |
| 卷         | Volume             | 卷管理      | —                  |
| 草稿       | Draft              | 章节状态    | —                  |
| 进行中     | In Progress        | 章节状态    | —                  |
| 已完成     | Completed          | 章节状态    | —                  |
| 已修订     | Revised            | 章节状态    | —                  |
| 世界观     | Worldbuilding      | 世界观面板  | 不用 "Worldview"   |
| 人物       | Character          | 世界观-人物 | 不用 "Person"      |
| 地点       | Location           | 世界观-地点 | 不用 "Place"       |
| 组织       | Organization       | 世界观-组织 | —                  |
| 时间线     | Timeline           | 时间线面板  | —                  |
| 关系图     | Relationship Graph | 关系图面板  | —                  |
| 灵感看板   | Inspiration Board  | 灵感面板    | —                  |
| 番茄钟     | Pomodoro Timer     | 番茄钟      | —                  |
| 敏感词     | Sensitive Words    | 敏感词管理  | —                  |
| 模板       | Template           | 模板选择    | —                  |
| 快照       | Snapshot           | Git 快照    | —                  |
| 备份       | Backup             | 备份管理    | —                  |
| 导出       | Export             | 导出功能    | —                  |
| 导入       | Import             | 文本导入    | —                  |
| 打字机模式 | Typewriter Mode    | 编辑器模式  | —                  |
| 聚焦模式   | Focus Mode         | 编辑器模式  | —                  |
| 禅模式     | Zen Mode           | 编辑器模式  | —                  |
| 写作统计   | Writing Stats      | 统计面板    | —                  |
| 热力图     | Heatmap            | 统计面板    | —                  |
| 字数       | Word Count         | 统计        | —                  |
| 加密       | Encryption         | 项目加密    | —                  |
| 解密       | Decryption         | 项目解密    | —                  |
| 迁移       | Migration          | 数据迁移    | —                  |
| 回滚       | Rollback           | 迁移回滚    | —                  |
| 存储       | Storage            | 存储路径    | —                  |
| 自动保存   | Auto Save          | 编辑器设置  | —                  |

### 9.2 通用操作术语

| 中文 | English  | 使用场景   |
| ---- | -------- | ---------- |
| 保存 | Save     | 保存按钮   |
| 取消 | Cancel   | 取消按钮   |
| 确认 | Confirm  | 确认按钮   |
| 删除 | Delete   | 删除操作   |
| 编辑 | Edit     | 编辑操作   |
| 新建 | Create   | 新建操作   |
| 关闭 | Close    | 关闭对话框 |
| 刷新 | Refresh  | 刷新数据   |
| 返回 | Back     | 导航返回   |
| 选择 | Select   | 文件选择   |
| 搜索 | Search   | 搜索框     |
| 重置 | Reset    | 重置设置   |
| 应用 | Apply    | 应用设置   |
| 下载 | Download | 下载操作   |
| 上传 | Upload   | 上传操作   |
| 复制 | Copy     | 复制操作   |
| 粘贴 | Paste    | 粘贴操作   |

### 9.3 消息提示术语

| 中文           | English                      | 类型    |
| -------------- | ---------------------------- | ------- |
| 保存成功       | Saved successfully           | success |
| 保存失败       | Save failed                  | error   |
| 操作成功       | Operation successful         | success |
| 操作失败       | Operation failed             | error   |
| 加载中...      | Loading...                   | info    |
| 加载失败       | Failed to load               | error   |
| 请输入{field}  | Please enter {field}         | warning |
| 请选择{field}  | Please select {field}        | warning |
| 确认删除？     | Confirm deletion?            | warning |
| 此操作不可撤销 | This action cannot be undone | warning |

---

## 10. 实施路线图

### 10.1 分阶段实施计划

| 阶段               | 内容                                                              | 涉及文件数 | 预估工作量 |
| ------------------ | ----------------------------------------------------------------- | ---------- | ---------- |
| **P0：基础设施**   | 安装 vue-i18n、创建目录结构、配置 i18n 实例、集成 Naive UI locale | ~5         | 基础搭建   |
| **P1：核心页面**   | WelcomePage、SettingsPage、ProjectSettingsPage                    | ~3         | 高优先级   |
| **P2：编辑器**     | EditorPage、TreeSidebar、MarkdownEditor                           | ~3         | 高优先级   |
| **P3：世界观**     | WorldbuildingPanel、Timeline、RelationshipGraph                   | ~3         | 中优先级   |
| **P4：辅助功能**   | BackupDialog、ExportDialog、PomodoroTimer 等                      | ~8         | 中优先级   |
| **P5：后端适配**   | 错误码映射、枚举字典多语言、模板多语言                            | ~10        | 低优先级   |
| **P6：测试与优化** | 完善测试、性能优化、清理废弃字符串                                | 全部       | 收尾       |

### 10.2 每阶段交付物

| 阶段 | 交付物                      | 验收标准                    |
| ---- | --------------------------- | --------------------------- |
| P0   | i18n 基础设施 + 语言切换 UI | 可在设置页切换语言          |
| P1   | 核心页面中英文切换          | 欢迎页/设置页所有文本可切换 |
| P2   | 编辑器中英文切换            | 编辑器所有文本可切换        |
| P3   | 世界观功能中英文切换        | 世界观面板所有文本可切换    |
| P4   | 全部功能中英文切换          | 所有组件文本可切换          |
| P5   | 后端错误消息本地化          | 错误提示显示对应语言        |
| P6   | 完整测试 + 文档             | CI 通过、覆盖率 100%        |

---

**文档版本：** 2.0  
**创建日期：** 2026-05  
**最后更新：** 2026-05  
**维护者：** 开发团队
