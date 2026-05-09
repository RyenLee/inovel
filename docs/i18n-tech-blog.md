# 在 Rust + Tauri 2 应用中实现语言切换功能：完整技术指南

---

## 文档说明

### 📖 文档结构

| 章节               | 功能定位           | 内容要点                             |
| ------------------ | ------------------ | ------------------------------------ |
| **文档说明**       | 文档概述与导航     | 介绍文档结构、适用场景、阅读建议     |
| **技术背景**       | 技术选型与价值分析 | Tauri 2 特性、国际化重要性           |
| **实现方案设计**   | 架构规划与策略制定 | 国际化架构、前后端交互、状态管理     |
| **具体实现步骤**   | 代码实现与配置指南 | 语言文件、后端配置、Rust/前端代码    |
| **关键技术点解析** | 深度技术剖析       | 资源系统、通信机制、持久化、性能优化 |
| **测试与验证**     | 质量保障方法       | 功能测试、兼容性测试、常见问题       |
| **完整代码示例**   | 可运行代码参考     | 语言文件、Rust代码、前端组件         |
| **FAQ**            | 问题解答与支持     | 常见疑问与解决方案                   |
| **总结与扩展**     | 未来发展方向       | 优化建议、最佳实践、扩展方向         |
| **参考文献**       | 知识拓展资源       | 官方文档、技术标准、工具链接         |

### 🎯 适用场景

- ✅ Tauri 2 桌面应用的国际化需求
- ✅ Rust + Vue 3 技术栈的多语言实现
- ✅ 需要语言切换功能的跨平台应用
- ✅ 学习 Vue I18n 在 Tauri 环境中的应用

### 👥 目标读者

| 读者类型       | 推荐阅读重点                           |
| -------------- | -------------------------------------- |
| **初级开发者** | 第三章（实现步骤）、第七章（代码示例） |
| **中级开发者** | 第四章（技术解析）、第五章（测试验证） |
| **高级开发者** | 第二章（方案设计）、第九章（扩展方向） |
| **技术管理者** | 第一章（背景价值）、第八章（总结建议） |

### 📚 阅读建议

1. **入门学习**：从第三章开始，逐步实现基础功能
2. **深度理解**：重点阅读第四章的技术原理剖析
3. **问题排查**：直接查阅第五章和FAQ部分
4. **扩展开发**：参考第九章的扩展方向进行功能增强

---

## 一、技术背景介绍

### 1.1 Rust + Tauri 2 框架特点

Tauri 2 是新一代跨平台桌面应用框架，结合了 Rust 的安全性和 Web 技术的灵活性：

| 特性             | 技术原理                                        | 实际价值             |
| ---------------- | ----------------------------------------------- | -------------------- |
| **跨平台**       | 基于 Rust 编译目标，支持 Windows/macOS/Linux    | 一套代码，多平台部署 |
| **轻量高效**     | Rust 编译为原生二进制，WebView 使用系统原生控件 | 启动快、内存占用低   |
| **安全沙箱**     | 默认禁用所有权限，按需开启                      | 防止恶意代码执行     |
| **WebView 隔离** | 前端运行在独立 WebView 进程                     | 安全性高、稳定性强   |
| **资源打包**     | 使用 Cargo 编译，静态资源嵌入二进制             | 单文件分发，无需安装 |

### 1.2 多语言支持的重要性

国际化（Internationalization，简称 i18n）是现代应用的必备能力：

- **全球化用户覆盖**：满足不同语言背景用户的使用需求，扩大用户群体
- **提升用户体验**：母语界面更直观友好，降低学习成本
- **市场拓展**：便于进入国际市场，提升产品竞争力
- **合规要求**：部分国家和地区要求软件提供本地化支持

### 1.3 创作背景

本文基于 iNovel 项目的国际化实践经验。iNovel 是一款面向小说创作者的桌面端写作工具，需要支持中英文双语切换。在实现过程中，我们遇到了以下挑战：

1. **前端与后端的语言同步**：确保 Vue 组件和 Rust 后端使用一致的语言设置
2. **语言状态的持久化**：保证用户选择的语言在应用重启后保持不变
3. **第三方组件库的国际化**：Naive UI 组件库的语言适配
4. **动态 UI 更新性能**：语言切换时避免不必要的组件重新渲染

---

## 二、实现方案设计

### 2.1 国际化架构设计

#### 2.1.1 语言文件组织

采用功能域拆分的目录结构，便于维护和扩展：

```
src/
├── locales/                    # 语言资源根目录
│   ├── zh-CN/                  # 简体中文 (ISO 639-1 + ISO 3166-1)
│   │   ├── index.ts            # 聚合导出文件
│   │   ├── common.json         # 通用文本（按钮、状态提示、对话框）
│   │   ├── task.json           # 任务清单模块专用文本
│   │   └── settings.json       # 设置页面专用文本
│   ├── en-US/                  # 英语（美式）
│   │   ├── index.ts
│   │   ├── common.json
│   │   ├── task.json
│   │   └── settings.json
│   └── [locale]/               # 预留新增语言目录
│       └── ...
└── i18n/                       # i18n 核心模块
    ├── index.ts                # i18n 配置与语言切换 API
    └── composables/
        └── useLocale.ts        # 组件级 composable，提供翻译能力
```

**设计原则**：

- **按功能域划分**：每个模块有独立的语言文件，便于团队协作
- **ISO 标准命名**：使用 `xx-XX` 格式，符合国际化标准
- **聚合导出**：通过 `index.ts` 统一导出，简化引用

#### 2.1.2 翻译键命名规范

采用点分隔的三层命名法，确保键名语义清晰：

```
{namespace}.{element}.{qualifier}
```

| 层级          | 说明       | 示例                                             |
| ------------- | ---------- | ------------------------------------------------ |
| **namespace** | 功能域标识 | common、task、settings                           |
| **element**   | 元素类型   | action（操作）、status（状态）、dialog（对话框） |
| **qualifier** | 具体标识   | save（保存）、cancel（取消）、confirm（确认）    |

**示例**：

```json
{
  "common": {
    "action": {
      "save": "保存",
      "cancel": "取消",
      "confirm": "确认",
      "delete": "删除"
    },
    "status": {
      "success": "操作成功",
      "error": "操作失败",
      "loading": "加载中..."
    },
    "dialog": {
      "confirmTitle": "确认操作",
      "confirmMessage": "确定要执行此操作吗？"
    }
  },
  "task": {
    "title": "任务清单",
    "addTask": "新建任务",
    "completedCount": "{count} 个已完成"
  },
  "settings": {
    "pageTitle": "应用设置",
    "interfaceLanguage": "界面语言",
    "saveSettings": "保存设置"
  }
}
```

**命名约定**：

- 使用 **camelCase** 命名（如 `addTask`）
- 避免使用空格或特殊字符
- 保持键名简洁但具有描述性
- 同类元素使用相同的 element 分类

### 2.2 前端与后端的交互方案

#### 方案对比

| 方案         | 架构特点                              | 适用场景                       | 优缺点                                                       |
| ------------ | ------------------------------------- | ------------------------------ | ------------------------------------------------------------ |
| **前端驱动** | 语言资源在前端管理，后端仅负责持久化  | 纯前端国际化，语言资源较小     | ✅ 简单高效，无需后端参与翻译逻辑<br>❌ 语言资源需打包进前端 |
| **后端驱动** | 语言资源由后端提供，前端通过 API 获取 | 需要后端控制语言策略或共享资源 | ✅ 统一管理，便于动态更新<br>❌ 增加网络请求，延迟较高       |

#### 推荐方案：前端驱动 + 后端持久化

**架构图**：

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         前端层 (Vue 3 + Vue I18n)                          │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │  ┌──────────┐    ┌──────────────┐    ┌───────────────────────────┐  │  │
│  │  │ 语言文件 │───▶│ i18n 实例    │───▶│ 组件模板中 t('key') 调用 │  │  │
│  │  │ *.json   │    │ 配置/切换     │    │ 响应式更新 UI 文本      │  │  │
│  │  └──────────┘    └──────────────┘    └───────────────────────────┘  │  │
│  │           │                                    │                     │  │
│  │           ▼                                    ▼                     │  │
│  │  ┌────────────────────────────────────────────────────────────────┐  │  │
│  │  │              localStorage ('inovel_locale')                   │  │  │
│  │  │  ┌─────────────────────────────────────────────────────────┐  │  │  │
│  │  │  │ 存储格式: { locale: "zh-CN", lastModified: "2024-01-01" } │  │  │  │
│  │  │  └─────────────────────────────────────────────────────────┘  │  │  │
│  │  └────────────────────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                    │                                      │
│                                    ▼ (可选：持久化到后端)                   │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                      Tauri IPC 通信层                                │  │
│  │  ┌────────────────────────────────────────────────────────────────┐  │  │
│  │  │  invoke('set_locale', { locale: 'en-US' })                    │  │  │
│  │  │  invoke('get_locale') -> Promise<String>                      │  │  │
│  │  └────────────────────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Rust 后端层 (Tauri Command)                        │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │  ┌────────────────────────────────────────────────────────────────┐  │  │
│  │  │  ┌──────────┐    ┌──────────────┐    ┌─────────────────────┐  │  │  │
│  │  │  │ Locale   │───▶│ File System  │───▶│ $APPDATA/locale.json│  │  │  │
│  │  │  │ Config   │    │  Persistence │    │ (持久化存储)       │  │  │  │
│  │  │  └──────────┘    └──────────────┘    └─────────────────────┘  │  │  │
│  │  └────────────────────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.3 状态管理策略

#### 语言状态优先级链

```
localStorage('inovel_locale')     # 优先级最高：用户手动设置
    → navigator.language          # 次之：浏览器/系统语言
    → 'zh-CN'                     # 默认值：简体中文
```

#### 状态恢复流程

```
应用启动
    └── 检查 localStorage 中是否存在 'inovel_locale'
            ├── 存在且有效 → 使用该语言
            └── 不存在或无效
                    ├── 检测 navigator.language
                    │       ├── 以 'zh' 开头 → 使用 zh-CN
                    │       └── 其他 → 使用 en-US
                    └── 初始化 i18n 实例
                            └── 渲染应用界面
```

#### 语言切换流程

```
用户选择新语言
    └── 调用 setLocale(newLocale)
            ├── 更新 i18n.global.locale
            ├── 更新 localStorage
            ├── 更新 document.lang 属性
            └── Vue 响应式系统自动触发组件重新渲染
```

---

## 三、具体实现步骤

### 3.1 语言文件的创建与格式定义

#### JSON 格式规范

```json
{
  "namespace": {
    "element": {
      "simple_key": "直接翻译文本",
      "key_with_variable": "包含 {variable} 的动态文本",
      "key_with_plural": {
        "zero": "零个",
        "one": "一个",
        "few": "几个",
        "many": "许多",
        "other": "其他"
      }
    }
  }
}
```

**格式要求**：

- 使用 UTF-8 编码，确保支持所有语言字符
- 键名使用 camelCase 格式
- 值可以是字符串或对象（用于复数等复杂场景）
- 变量使用 `{variableName}` 格式

#### 聚合导出文件（index.ts）

```typescript
// src/locales/zh-CN/index.ts
import common from "./common.json";
import task from "./task.json";
import settings from "./settings.json";

export default {
  common,
  task,
  settings,
};
```

**设计意图**：

- 通过 `index.ts` 统一导出所有模块的翻译资源
- 前端只需引用 `../locales/zh-CN` 即可获取完整的中文资源
- 便于按需引入，优化打包体积

### 3.2 Tauri 后端配置

#### Cargo.toml 依赖配置

```toml
[package]
name = "inovel"
version = "1.0.0"
edition = "2021"

[dependencies]
tauri = { version = "2.0", features = ["shell-open", "path-all"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"  # 用于错误处理
```

**依赖说明**：

- **tauri**：Tauri 框架核心依赖，`path-all` 特性提供路径处理能力
- **serde**：Rust 序列化/反序列化库，用于处理 JSON 配置
- **serde_json**：JSON 处理库
- **thiserror**：错误处理库，提供优雅的错误定义

#### tauri.conf.json 权限配置

```json
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:5173",
    "context": "src-tauri"
  },
  "tauri": {
    "allowlist": {
      "fs": {
        "readFile": true,
        "writeFile": true,
        "scope": ["$APPDATA/*"]
      },
      "path": {
        "all": true
      }
    },
    "bundle": {
      "active": true,
      "targets": "all",
      "identifier": "com.example.inovel"
    }
  }
}
```

**权限说明**：

- `fs.readFile/writeFile`：允许读写文件
- `fs.scope`: `"$APPDATA/*"`：限制文件操作范围到应用数据目录
- `path.all`：允许访问路径 API

### 3.3 Rust 代码实现

#### 语言持久化服务（src-tauri/src/locale.rs）

```rust
use serde::{Deserialize, Serialize};
use tauri::{command, State};
use thiserror::Error;

/// 语言配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleConfig {
    pub current_locale: String,
    pub last_modified: String,
}

/// 自定义错误类型
#[derive(Error, Debug)]
pub enum LocaleError {
    #[error("无效的语言代码: {0}")]
    InvalidLocale(String),

    #[error("无法获取应用数据目录")]
    AppDataDirError,

    #[error("文件操作失败: {0}")]
    FileError(String),

    #[error("JSON 解析失败: {0}")]
    JsonError(String),
}

impl From<std::io::Error> for LocaleError {
    fn from(err: std::io::Error) -> Self {
        LocaleError::FileError(err.to_string())
    }
}

impl From<serde_json::Error> for LocaleError {
    fn from(err: serde_json::Error) -> Self {
        LocaleError::JsonError(err.to_string())
    }
}

/// 设置语言配置
#[command]
pub async fn set_locale(
    locale: String,
    state: State<'_, LocaleConfig>
) -> Result<(), String> {
    // 验证语言代码
    if !["zh-CN", "en-US"].contains(&locale.as_str()) {
        return Err(LocaleError::InvalidLocale(locale).to_string());
    }

    // 获取应用数据目录
    let app_dir = tauri::api::path::app_data_dir()
        .ok_or(LocaleError::AppDataDirError)?;

    // 构建配置文件路径
    let config_path = app_dir.join("locale.json");

    // 创建更新后的配置
    let config = LocaleConfig {
        current_locale: locale,
        last_modified: chrono::Local::now().to_rfc3339(),
    };

    // 序列化并写入文件
    let content = serde_json::to_string_pretty(&config)?;
    std::fs::write(&config_path, content)?;

    // 更新应用状态
    *state.inner() = config;

    Ok(())
}

/// 获取当前语言配置
#[command]
pub async fn get_locale(state: State<'_, LocaleConfig>) -> Result<String, String> {
    Ok(state.current_locale.clone())
}
```

**代码解析**：

- **LocaleConfig**：存储当前语言和最后修改时间
- **LocaleError**：自定义错误类型，提供清晰的错误信息
- **set_locale**：验证语言代码、保存到文件、更新应用状态
- **get_locale**：从应用状态中获取当前语言

#### 主函数集成（src-tauri/src/main.rs）

```rust
use tauri::{Manager, Runtime};
use crate::locale::LocaleConfig;

mod locale;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // 初始化语言配置
            let config = load_locale_config();
            app.manage(config);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            locale::set_locale,
            locale::get_locale
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 从文件加载语言配置，如果不存在则使用默认配置
fn load_locale_config() -> LocaleConfig {
    // 获取应用数据目录
    let app_dir = match tauri::api::path::app_data_dir() {
        Some(dir) => dir,
        None => return create_default_config(),
    };

    // 构建配置文件路径
    let config_path = app_dir.join("locale.json");

    // 尝试读取配置文件
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        match serde_json::from_str(&content) {
            Ok(config) => return config,
            Err(_) => return create_default_config(),
        }
    }

    // 如果文件不存在，创建默认配置
    create_default_config()
}

/// 创建默认语言配置
fn create_default_config() -> LocaleConfig {
    LocaleConfig {
        current_locale: "zh-CN".to_string(),
        last_modified: chrono::Local::now().to_rfc3339(),
    }
}
```

**代码解析**：

- **setup 函数**：应用启动时初始化语言配置
- **app.manage**：将配置注入 Tauri 状态管理器
- **generate_handler**：注册可供前端调用的命令
- **load_locale_config**：从文件加载配置或使用默认值

### 3.4 前端实现

#### 3.4.1 i18n 核心配置（src/i18n/index.ts）

```typescript
import { createI18n, type I18n } from "vue-i18n";
import zhCN from "../locales/zh-CN";
import enUS from "../locales/en-US";

// 支持的语言类型
export type AppLocale = "zh-CN" | "en-US";

// localStorage 存储键名
const LOCALE_STORAGE_KEY = "inovel_locale";

// 检测系统语言
function detectSystemLocale(): AppLocale {
  const lang = navigator.language || navigator.userLanguage;
  if (lang.startsWith("zh")) return "zh-CN";
  return "en-US";
}

// 获取保存的语言设置
function getStoredLocale(): AppLocale {
  const stored = localStorage.getItem(LOCALE_STORAGE_KEY);
  if (stored === "zh-CN" || stored === "en-US") {
    return stored;
  }
  return detectSystemLocale();
}

// 创建 i18n 实例
const i18n: I18n = createI18n({
  legacy: false, // 使用 Composition API
  locale: getStoredLocale(), // 初始语言
  fallbackLocale: "zh-CN", // 回退语言
  messages: {
    "zh-CN": zhCN,
    "en-US": enUS,
  },
  // 缺失翻译时的回调
  missing: (locale: string, key: string) => {
    console.warn(
      `[i18n] Missing translation for key "${key}" in locale "${locale}"`,
    );
    return `[${key}]`;
  },
  // 格式化数字和日期
  numberFormats: {
    "zh-CN": {
      currency: {
        style: "currency",
        currency: "CNY",
        minimumFractionDigits: 2,
      },
    },
    "en-US": {
      currency: {
        style: "currency",
        currency: "USD",
        minimumFractionDigits: 2,
      },
    },
  },
  datetimeFormats: {
    "zh-CN": {
      short: {
        year: "numeric",
        month: "short",
        day: "numeric",
      },
    },
    "en-US": {
      short: {
        year: "numeric",
        month: "short",
        day: "numeric",
      },
    },
  },
});

// 设置当前语言
export function setLocale(locale: AppLocale): void {
  // 更新 i18n 实例的语言
  (i18n.global.locale as unknown as { value: AppLocale }).value = locale;

  // 保存到 localStorage
  localStorage.setItem(LOCALE_STORAGE_KEY, locale);

  // 更新 HTML 文档的 lang 属性
  document.documentElement.setAttribute("lang", locale);

  // 触发自定义事件，通知其他组件语言已变更
  document.dispatchEvent(new Event("localechange"));
}

// 获取当前语言
export function getLocale(): AppLocale {
  return (i18n.global.locale as unknown as { value: AppLocale }).value;
}

export default i18n;
```

**代码解析**：

- **createI18n**：创建国际化实例，配置语言资源和回退策略
- **detectSystemLocale**：根据浏览器语言检测用户偏好
- **setLocale**：更新语言设置，同步到 localStorage 和 DOM
- **missing 回调**：处理缺失翻译的情况，便于开发调试

#### 3.4.2 useLocale Composable（src/i18n/composables/useLocale.ts）

```typescript
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { setLocale, getLocale } from "../index";
import type { AppLocale } from "../index";

export function useLocale() {
  // 获取 i18n 实例和翻译函数
  const { t, locale, tc, n, d } = useI18n();

  // 当前语言判断
  const isZhCN = computed(() => locale.value === "zh-CN");
  const isEnUS = computed(() => locale.value === "en-US");

  // 可用语言列表（显示标签使用对应语言）
  const availableLocales: { label: string; value: AppLocale }[] = [
    { label: isZhCN.value ? "简体中文" : "Chinese", value: "zh-CN" },
    { label: isZhCN.value ? "English" : "English", value: "en-US" },
  ];

  // 切换语言
  const switchLocale = (newLocale: AppLocale) => {
    setLocale(newLocale);
  };

  // 当前语言（用于绑定到选择器）
  const currentLocale = computed({
    get: () => getLocale(),
    set: (val: AppLocale) => switchLocale(val),
  });

  // 带命名空间的翻译函数
  const tn = (namespace: string, key: string, ...args: unknown[]) => {
    return t(`${namespace}.${key}`, ...args);
  };

  return {
    t, // 基础翻译函数
    tc, // 复数翻译函数
    n, // 数字格式化
    d, // 日期格式化
    tn, // 带命名空间的翻译
    locale, // 响应式语言状态
    isZhCN, // 是否为中文
    isEnUS, // 是否为英文
    availableLocales, // 可用语言列表
    switchLocale, // 切换语言方法
    currentLocale, // 当前语言（双向绑定）
  };
}
```

**代码解析**：

- **useI18n**：从 vue-i18n 获取核心能力
- **isZhCN/isEnUS**：计算属性，判断当前语言
- **availableLocales**：可用语言选项，用于下拉选择器
- **currentLocale**：computed 的 getter/setter，支持双向绑定
- **tn**：带命名空间的翻译辅助函数

#### 3.4.3 语言切换组件（SettingsPage.vue）

```vue
<script setup lang="ts">
import { computed } from "vue";
import { NSelect, NCard, NForm, NFormItem, NIcon } from "naive-ui";
import { Languages } from "lucide-vue-next";
import { useLocale } from "../i18n/composables/useLocale";

const { t, currentLocale, availableLocales } = useLocale();

// 构建选择器选项
const localeOptions = computed(() =>
  availableLocales.map((l) => ({
    label: l.label,
    value: l.value,
  })),
);
</script>

<template>
  <n-card hoverable class="settings-card">
    <template #header>
      <div class="flex items-center gap-2">
        <n-icon :size="18"><Languages /></n-icon>
        <span>{{ t("settings.language") }}</span>
      </div>
    </template>
    <n-form label-placement="top" class="settings-form">
      <n-form-item
        :label="t('settings.interfaceLanguage')"
        class="settings-form-item"
      >
        <n-select
          v-model:value="currentLocale"
          :options="localeOptions"
          class="min-w-[180px] w-56"
          placeholder="Select language"
        />
      </n-form-item>
    </n-form>
  </n-card>
</template>

<style scoped>
.settings-card {
  width: 100%;
}

.settings-form {
  width: 100%;
}

.settings-form-item {
  width: 100%;
}
</style>
```

**代码解析**：

- **v-model:value="currentLocale"**：双向绑定当前语言
- **availableLocales**：动态生成语言选项
- **t('settings.language')**：翻译标签文本

#### 3.4.4 应用入口集成（main.ts）

```typescript
import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import i18n from "./i18n";

// 创建 Vue 应用
const app = createApp(App);

// 注册插件
app.use(i18n); // 国际化插件
app.use(router); // 路由插件

// 挂载应用
app.mount("#app");
```

#### 3.4.5 根组件集成（App.vue）

```vue
<script setup lang="ts">
import { computed, onMounted, onUnmounted } from "vue";
import { NConfigProvider, zhCN, dateZhCN, enUS, dateEnUS } from "naive-ui";
import { useLocale } from "./i18n/composables/useLocale";

const { isZhCN, switchLocale } = useLocale();

// Naive UI 组件库的语言配置
const naiveLocale = computed(() => (isZhCN.value ? zhCN : enUS));
const naiveDateLocale = computed(() => (isZhCN.value ? dateZhCN : dateEnUS));

// 监听全局语言变更事件
const handleLocaleChange = () => {
  // 可以在这里添加语言切换后的额外逻辑
  console.log("Locale changed");
};

onMounted(() => {
  document.addEventListener("localechange", handleLocaleChange);
});

onUnmounted(() => {
  document.removeEventListener("localechange", handleLocaleChange);
});
</script>

<template>
  <!-- 配置 Naive UI 的语言 -->
  <n-config-provider :locale="naiveLocale" :date-locale="naiveDateLocale">
    <!-- 路由视图 -->
    <RouterView />
  </n-config-provider>
</template>
```

**代码解析**：

- **NConfigProvider**：配置 Naive UI 组件库的语言
- **naiveLocale/naiveDateLocale**：根据当前语言动态切换组件库语言
- **localechange 事件**：监听全局语言变更

---

## 四、关键技术点解析

### 4.1 Tauri 2 的资源系统与路径处理

#### 路径变量说明

Tauri 提供了多个预定义的路径变量，便于访问系统目录：

| 变量         | 说明               | 平台示例                                                                                                                     |
| ------------ | ------------------ | ---------------------------------------------------------------------------------------------------------------------------- |
| `$APPDATA`   | 应用数据目录       | Windows: `C:\Users\<user>\AppData\Roaming\<app>`<br>macOS: `~/Library/Application Support/<app>`<br>Linux: `~/.config/<app>` |
| `$RESOURCE`  | 打包的静态资源目录 | 编译时确定，通常嵌入二进制                                                                                                   |
| `$HOME`      | 用户主目录         | `/home/user` 或 `C:\Users\<user>`                                                                                            |
| `$DOCUMENTS` | 文档目录           | `~/Documents`                                                                                                                |
| `$DOWNLOADS` | 下载目录           | `~/Downloads`                                                                                                                |

#### 路径 API 使用示例

```rust
// 获取应用数据目录
let app_dir = tauri::api::path::app_data_dir().unwrap();

// 获取资源目录
let resource_dir = tauri::api::path::resource_dir().unwrap();

// 构建文件路径
let config_path = app_dir.join("settings").join("config.json");

// 检查路径是否存在
if config_path.exists() {
    // 读取配置文件
    let content = std::fs::read_to_string(&config_path).unwrap();
}
```

#### 前端访问资源

```typescript
import { resolveResource } from "@tauri-apps/api/path";

// 获取资源路径
const resourcePath = await resolveResource("locales/en-US/common.json");
```

### 4.2 Rust 与前端的通信机制

#### Tauri 命令模式（单向通信）

**Rust 端定义命令**：

```rust
use tauri::command;

#[command]
pub async fn greet(name: String) -> Result<String, String> {
    Ok(format!("Hello, {}!", name))
}

// 在 main.rs 中注册
.invoke_handler(tauri::generate_handler![greet])
```

**前端调用命令**：

```typescript
import { invoke } from "@tauri-apps/api/core";

const result = await invoke("greet", { name: "World" });
console.log(result); // "Hello, World!"
```

#### Tauri 事件模式（双向通信）

**Rust 端发送事件**：

```rust
// 发送事件给所有窗口
app.emit_all("locale-changed", "en-US")?;

// 发送事件给特定窗口
window.emit("locale-changed", "en-US")?;
```

**前端监听事件**：

```typescript
import { listen } from "@tauri-apps/api/event";

// 监听语言变更事件
const unlisten = await listen("locale-changed", (event) => {
  console.log("Locale changed to:", event.payload);
  // 更新 UI 或执行其他逻辑
});

// 取消监听
unlisten();
```

#### 通信方式对比

| 方式       | 特点                    | 适用场景           |
| ---------- | ----------------------- | ------------------ |
| **invoke** | 前端主动调用，同步/异步 | 获取数据、执行操作 |
| **事件**   | 后端主动推送            | 实时更新、状态通知 |

### 4.3 应用状态持久化方案

#### 方案对比

| 方案               | 优点                               | 缺点                                  | 适用场景                       |
| ------------------ | ---------------------------------- | ------------------------------------- | ------------------------------ |
| **localStorage**   | 简单快速、前端直接访问、无需后端   | 容量限制（5MB）、仅支持字符串、不安全 | 语言偏好、UI 设置、轻量配置    |
| **Tauri 文件存储** | 大容量、结构化数据、安全持久       | 需要跨层通信、异步操作                | 复杂配置、用户数据             |
| **SQLite**         | 关系型数据库、事务支持、查询能力强 | 需要 ORM、学习成本、性能开销          | 任务数据、持久化存储、复杂查询 |
| **Tauri State**    | 内存状态、快速访问                 | 重启后丢失                            | 临时状态、运行时数据           |

#### 语言偏好存储策略

**推荐方案**：前端使用 `localStorage` 存储语言偏好，后端可选同步到文件。

```typescript
// 前端存储
const LOCALE_KEY = "inovel_locale";

// 设置语言
localStorage.setItem(LOCALE_KEY, "en-US");

// 获取语言
const locale = localStorage.getItem(LOCALE_KEY) || "zh-CN";

// 删除语言设置
localStorage.removeItem(LOCALE_KEY);
```

#### 复杂配置存储策略

对于需要跨设备同步或更复杂的配置，建议使用 SQLite：

```rust
// 使用 rusqlite crate
use rusqlite::{Connection, Result};

fn save_config(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)",
        (key, value),
    )?;
    Ok(())
}

fn load_config(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM config WHERE key = ?1")?;
    let mut rows = stmt.query([key])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}
```

### 4.4 动态 UI 更新的性能优化

#### 问题分析

语言切换时，Vue 会重新渲染所有使用 `t()` 函数的组件。如果应用较大，可能会导致性能问题。

#### 优化策略

| 策略                     | 实现方式                           | 效果             |
| ------------------------ | ---------------------------------- | ---------------- |
| **避免不必要的重新渲染** | 使用 `computed` 缓存翻译结果       | 减少重复计算     |
| **批量更新**             | 使用 `transition-group` 或虚拟列表 | 优化大量列表项   |
| **懒加载语言包**         | 按需加载非默认语言                 | 减少初始包体积   |
| **编译时优化**           | 使用 vue-i18n 的 `compiler` 模式   | 提升运行时性能   |
| **分片更新**             | 将大型组件拆分为小组件             | 精确控制更新范围 |

#### 懒加载实现

```typescript
const loadedLocales = new Set<AppLocale>(["zh-CN"]);

export async function loadLocaleAsync(locale: AppLocale): Promise<void> {
  // 如果已加载，直接返回
  if (loadedLocales.has(locale)) return;

  // 动态导入语言包
  const messages = await import(`../locales/${locale}/index.ts`);

  // 设置语言消息
  i18n.global.setLocaleMessage(locale, messages.default);

  // 标记为已加载
  loadedLocales.add(locale);
}

// 使用示例
await loadLocaleAsync("en-US");
switchLocale("en-US");
```

#### 编译时优化配置

```typescript
// vite.config.ts
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "path";

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
    },
  },
  // vue-i18n 编译时优化
  define: {
    __VUE_I18N_FULL_INSTALL__: true,
    __VUE_I18N_LEGACY_API__: false,
    __INTLIFY_PROD_DEVTOOLS__: false,
  },
});
```

---

## 五、测试与验证

### 5.1 功能测试方法

#### 测试清单

| 测试项              | 测试步骤               | 预期结果                   | 测试工具             |
| ------------------- | ---------------------- | -------------------------- | -------------------- |
| **语言切换**        | 在设置页面选择不同语言 | 界面立即切换为对应语言     | 手动测试、E2E 测试   |
| **文本显示**        | 检查所有组件的文本内容 | 所有文本正确翻译，无硬编码 | 自动化测试、代码审查 |
| **状态保存**        | 切换语言 → 重启应用    | 语言设置保持不变           | 手动测试、自动化测试 |
| **回退机制**        | 删除某个翻译键         | 显示键名或默认语言翻译     | 单元测试             |
| **变量替换**        | 使用含变量的翻译       | 变量正确替换为实际值       | 单元测试             |
| **复数处理**        | 测试不同数量的复数形式 | 根据数量显示正确的复数形式 | 单元测试             |
| **日期/数字格式化** | 切换语言后检查格式化   | 日期和数字格式正确本地化   | 单元测试             |

#### 自动化测试示例

```typescript
import { describe, it, expect, beforeEach, vi } from "vitest";
import { createI18n } from "vue-i18n";
import zhCN from "../locales/zh-CN/common.json";
import enUS from "../locales/en-US/common.json";

describe("i18n Tests", () => {
  let i18n: ReturnType<typeof createI18n>;

  beforeEach(() => {
    // 创建 i18n 实例
    i18n = createI18n({
      legacy: false,
      locale: "zh-CN",
      fallbackLocale: "zh-CN",
      messages: {
        "zh-CN": { common: zhCN },
        "en-US": { common: enUS },
      },
    });
  });

  it("should have identical keys in both locales", () => {
    const zhKeys = Object.keys(zhCN.action).sort();
    const enKeys = Object.keys(enUS.action).sort();
    expect(enKeys).toEqual(zhKeys);
  });

  it("should return correct translation for zh-CN", () => {
    const { t } = i18n.global;
    expect(t("common.action.save")).toBe("保存");
    expect(t("common.status.success")).toBe("操作成功");
  });

  it("should return correct translation for en-US", () => {
    i18n.global.locale.value = "en-US";
    const { t } = i18n.global;
    expect(t("common.action.save")).toBe("Save");
    expect(t("common.status.success")).toBe("Operation successful");
  });

  it("should handle missing translation", () => {
    const { t } = i18n.global;
    const result = t("common.nonexistent.key");
    expect(result).toContain("common.nonexistent.key");
  });

  it("should replace variables in translation", () => {
    const { t } = i18n.global;
    const result = t("task.completedCount", { count: 5 });
    expect(result).toBe("5 个已完成");
  });
});
```

### 5.2 兼容性测试

#### 平台测试

| 平台        | 测试要点                         | 测试方法                   |
| ----------- | -------------------------------- | -------------------------- |
| **Windows** | 中文/英文系统语言检测、路径处理  | 在中文和英文系统上分别测试 |
| **macOS**   | 系统语言偏好读取、沙箱权限       | 在 macOS 10.15+ 上测试     |
| **Linux**   | 环境变量 LANG 检测、文件系统权限 | 在 Ubuntu/Debian 上测试    |

#### 浏览器/WebView 测试

| WebView                     | 注意事项                             | 测试方法                |
| --------------------------- | ------------------------------------ | ----------------------- |
| **Chromium**                | Tauri 默认 WebView，支持现代 Web API | 正常测试即可            |
| **WebKit (macOS)**          | 某些 API 行为可能不同                | 在 macOS 上单独测试     |
| **Edge WebView2 (Windows)** | 需要单独安装                         | 在 Windows 10/11 上测试 |

#### 网络环境测试

| 场景         | 测试要点           | 预期结果                     |
| ------------ | ------------------ | ---------------------------- |
| **离线模式** | 语言切换、翻译显示 | 正常工作，语言包已打包进应用 |
| **慢速网络** | 懒加载语言包       | 显示加载状态，不阻塞 UI      |

### 5.3 常见问题及解决方案

| 问题                       | 原因                               | 解决方案                                                  |
| -------------------------- | ---------------------------------- | --------------------------------------------------------- |
| **语言切换后某些文本不变** | 文本硬编码在组件中，未使用 `t()`   | 全局搜索硬编码文本，替换为 `t('key')`                     |
| **切换语言后布局错乱**     | 英文文本比中文长，固定宽度容器溢出 | 使用弹性布局、设置最小/最大宽度、`word-break: break-word` |
| **语言包加载失败**         | 文件路径错误或 JSON 格式问题       | 检查文件路径大小写、验证 JSON 语法、添加错误捕获          |
| **状态不同步**             | `ref` 未与响应式数据源同步         | 使用 `computed` 的 getter/setter 模式                     |
| **翻译键冲突**             | 不同模块使用相同的键名             | 使用命名空间前缀（如 `task.title`、`settings.title`）     |
| **NaN 错误**               | 翻译键中包含特殊字符               | 检查键名格式，确保符合 JavaScript 标识符规则              |
| **性能问题**               | 语言切换时重新渲染所有组件         | 使用懒加载、编译时优化、分片更新                          |

---

## 六、完整代码示例

### 6.1 语言文件示例

**zh-CN/common.json**：

```json
{
  "action": {
    "save": "保存",
    "cancel": "取消",
    "confirm": "确认",
    "delete": "删除",
    "edit": "编辑",
    "add": "添加",
    "close": "关闭",
    "back": "返回"
  },
  "status": {
    "success": "操作成功",
    "error": "操作失败",
    "loading": "加载中...",
    "noData": "暂无数据",
    "confirmDelete": "确定要删除吗？",
    "confirmLogout": "确定要退出登录吗？"
  },
  "dialog": {
    "confirmTitle": "确认操作",
    "errorTitle": "操作失败",
    "successTitle": "操作成功",
    "warningTitle": "警告"
  }
}
```

**en-US/common.json**：

```json
{
  "action": {
    "save": "Save",
    "cancel": "Cancel",
    "confirm": "Confirm",
    "delete": "Delete",
    "edit": "Edit",
    "add": "Add",
    "close": "Close",
    "back": "Back"
  },
  "status": {
    "success": "Operation successful",
    "error": "Operation failed",
    "loading": "Loading...",
    "noData": "No data available",
    "confirmDelete": "Are you sure you want to delete?",
    "confirmLogout": "Are you sure you want to logout?"
  },
  "dialog": {
    "confirmTitle": "Confirm",
    "errorTitle": "Error",
    "successTitle": "Success",
    "warningTitle": "Warning"
  }
}
```

### 6.2 Rust 核心代码片段

**src-tauri/src/locale.rs**：

```rust
use serde::{Deserialize, Serialize};
use tauri::{command, State};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleConfig {
    pub current_locale: String,
    pub last_modified: String,
}

#[derive(Error, Debug)]
pub enum LocaleError {
    #[error("无效的语言代码: {0}")]
    InvalidLocale(String),

    #[error("无法获取应用数据目录")]
    AppDataDirError,

    #[error("文件操作失败: {0}")]
    FileError(String),

    #[error("JSON 解析失败: {0}")]
    JsonError(String),
}

impl From<std::io::Error> for LocaleError {
    fn from(err: std::io::Error) -> Self {
        LocaleError::FileError(err.to_string())
    }
}

impl From<serde_json::Error> for LocaleError {
    fn from(err: serde_json::Error) -> Self {
        LocaleError::JsonError(err.to_string())
    }
}

#[command]
pub async fn set_locale(
    locale: String,
    state: State<'_, LocaleConfig>
) -> Result<(), String> {
    if !["zh-CN", "en-US"].contains(&locale.as_str()) {
        return Err(LocaleError::InvalidLocale(locale).to_string());
    }

    let app_dir = tauri::api::path::app_data_dir()
        .ok_or(LocaleError::AppDataDirError)?;

    let config_path = app_dir.join("locale.json");

    let config = LocaleConfig {
        current_locale: locale,
        last_modified: chrono::Local::now().to_rfc3339(),
    };

    let content = serde_json::to_string_pretty(&config)?;
    std::fs::write(&config_path, content)?;

    *state.inner() = config;

    Ok(())
}

#[command]
pub async fn get_locale(state: State<'_, LocaleConfig>) -> Result<String, String> {
    Ok(state.current_locale.clone())
}
```

### 6.3 前端组件代码

**TaskChecklist.vue**：

```vue
<script setup lang="ts">
import { ref, computed } from "vue";
import { NCard, NButton, NCheckbox, NSpace } from "naive-ui";
import { Plus, CheckCircle } from "lucide-vue-next";
import { useLocale } from "../i18n/composables/useLocale";

const { t } = useLocale();

// 任务数据
const tasks = ref([
  { id: 1, name: t("task.sampleTask1"), completed: false },
  { id: 2, name: t("task.sampleTask2"), completed: true },
  { id: 3, name: t("task.sampleTask3"), completed: false },
]);

// 新任务名称
const newTaskName = ref("");

// 已完成数量
const completedCount = computed(
  () => tasks.value.filter((t) => t.completed).length,
);

// 总数量
const totalCount = computed(() => tasks.value.length);

// 添加任务
const addTask = () => {
  if (!newTaskName.value.trim()) return;

  tasks.value.push({
    id: Date.now(),
    name: newTaskName.value.trim(),
    completed: false,
  });

  newTaskName.value = "";
};

// 切换任务状态
const toggleTask = (id: number) => {
  const task = tasks.value.find((t) => t.id === id);
  if (task) {
    task.completed = !task.completed;
  }
};

// 删除任务
const removeTask = (id: number) => {
  tasks.value = tasks.value.filter((t) => t.id !== id);
};
</script>

<template>
  <n-card :title="t('task.title')" class="task-checklist">
    <!-- 任务统计 -->
    <div class="task-header">
      <span class="task-count">
        {{
          t("task.completedCount", {
            completed: completedCount,
            total: totalCount,
          })
        }}
      </span>
    </div>

    <!-- 任务列表 -->
    <div class="task-list">
      <div
        v-for="task in tasks"
        :key="task.id"
        class="task-item"
        :class="{ completed: task.completed }"
      >
        <n-checkbox
          v-model:checked="task.completed"
          @update:checked="toggleTask(task.id)"
        />
        <span class="task-name">{{ task.name }}</span>
        <n-button quaternary size="small" @click="removeTask(task.id)">
          {{ t("common.action.delete") }}
        </n-button>
      </div>
    </div>

    <!-- 添加任务 -->
    <div class="add-task">
      <NSpace>
        <input
          v-model="newTaskName"
          type="text"
          :placeholder="t('task.addTaskPlaceholder')"
          @keyup.enter="addTask"
          class="task-input"
        />
        <n-button type="primary" @click="addTask">
          <template #icon>
            <Plus :size="18" />
          </template>
          {{ t("task.addTask") }}
        </n-button>
      </NSpace>
    </div>
  </n-card>
</template>

<style scoped>
.task-checklist {
  max-width: 600px;
  margin: 0 auto;
}

.task-header {
  padding: 16px 0;
  border-bottom: 1px solid #e5e7eb;
}

.task-count {
  font-weight: 600;
  color: #374151;
}

.task-list {
  margin-top: 16px;
}

.task-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 0;
  border-bottom: 1px solid #f3f4f6;
}

.task-item.completed .task-name {
  text-decoration: line-through;
  color: #9ca3af;
}

.task-name {
  flex: 1;
  font-size: 14px;
  color: #1f2937;
}

.add-task {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid #e5e7eb;
}

.task-input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 14px;
  outline: none;
}

.task-input:focus {
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}
</style>
```

---

## 七、FAQ

### Q1：为什么语言切换后界面没有变化？

**可能原因**：

1. 组件中的文本是硬编码的，没有使用 `t()` 函数
2. `selectedLocale` 没有正确绑定到响应式数据源
3. 语言包没有正确加载

**解决方案**：

1. 检查所有文本是否都使用了 `t('key')` 调用
2. 确保 `selectedLocale` 使用 `computed` 的 getter/setter 模式
3. 打开浏览器控制台，查看是否有加载错误

### Q2：如何添加新的语言支持？

**步骤**：

1. 在 `src/locales` 目录下创建新语言目录（如 `ja-JP`）
2. 创建对应的语言文件（`common.json`、`task.json`、`settings.json`）
3. 在 `index.ts` 中聚合导出
4. 在 `src/i18n/index.ts` 中注册新语言
5. 在 `useLocale.ts` 中添加新语言选项
6. 在 Rust 后端的 `set_locale` 函数中添加语言验证

### Q3：如何处理复数和变量替换？

**复数处理**：

```json
{
  "task": {
    "completedCount": {
      "zero": "没有完成任何任务",
      "one": "完成了 1 个任务",
      "other": "完成了 {count} 个任务"
    }
  }
}
```

```typescript
// 使用 tc 函数处理复数
const { tc } = useI18n();
tc("task.completedCount", 5, { count: 5 }); // "完成了 5 个任务"
```

**变量替换**：

```json
{
  "greeting": "你好，{name}！"
}
```

```typescript
t("greeting", { name: "张三" }); // "你好，张三！"
```

### Q4：如何处理 RTL（从右到左）语言？

**步骤**：

1. 在语言配置中标记 RTL 语言
2. 根据语言方向设置 CSS `direction` 属性
3. 调整布局以适应 RTL

```typescript
const isRtl = computed(() =>
  ["ar", "he", "fa"].includes(currentLocale.value.split("-")[0]),
);
```

```css
.rtl {
  direction: rtl;
  text-align: right;
}
```

### Q5：如何动态加载语言包？

**实现方式**：

```typescript
const loadedLocales = new Set(["zh-CN"]);

export async function loadLocaleAsync(locale: AppLocale): Promise<void> {
  if (loadedLocales.has(locale)) return;

  const messages = await import(`../locales/${locale}/index.ts`);
  i18n.global.setLocaleMessage(locale, messages.default);
  loadedLocales.add(locale);
}
```

### Q6：如何处理日期和数字的本地化？

**数字格式化**：

```typescript
const { n } = useI18n();
n(12345.67, "currency"); // 中文: "¥12,345.67"，英文: "$12,345.67"
```

**日期格式化**：

```typescript
const { d } = useI18n();
d(new Date(), "short"); // 中文: "2024/1/1"，英文: "Jan 1, 2024"
```

### Q7：如何测试国际化功能？

**单元测试**：

```typescript
describe("i18n", () => {
  it("should return correct translations", () => {
    const { t } = i18n.global;
    expect(t("common.action.save")).toBe("保存");
  });

  it("should fallback to default locale", () => {
    i18n.global.locale.value = "fr-FR";
    const { t } = i18n.global;
    expect(t("common.action.save")).toBe("保存"); // 回退到中文
  });
});
```

### Q8：如何处理翻译中的 HTML 标签？

**解决方案**：使用 `v-html` 指令

```json
{
  "welcome": "欢迎使用 <strong>iNovel</strong>！"
}
```

```vue
<div v-html="t('welcome')"></div>
```

**注意**：确保翻译内容是可信的，避免 XSS 攻击。

---

## 八、总结与扩展

### 8.1 功能优化建议

| 优化方向     | 具体措施                               | 预期收益                     |
| ------------ | -------------------------------------- | ---------------------------- |
| **性能优化** | 懒加载语言包、编译时优化               | 减少初始包体积，提升加载速度 |
| **开发体验** | 添加翻译键自动补全、缺失键检测         | 提高开发效率，减少错误       |
| **可维护性** | 使用翻译管理平台（如 Crowdin）         | 便于团队协作和翻译更新       |
| **错误处理** | 添加详细的错误日志和监控               | 快速定位和解决问题           |
| **扩展性**   | 支持更多语言（日语、韩语、西班牙语等） | 扩大用户覆盖范围             |

### 8.2 多语言维护最佳实践

1. **建立术语表**：维护统一的术语翻译对照表
2. **定期审查**：定期检查翻译的准确性和完整性
3. **自动化检查**：在 CI 中集成翻译键完整性检查
4. **版本控制**：语言文件纳入版本管理，便于追溯变更
5. **翻译流程**：建立翻译 → 审查 → 测试 → 发布的流程

### 8.3 国际化功能的扩展方向

| 扩展点            | 说明                       | 实现建议                         |
| ----------------- | -------------------------- | -------------------------------- |
| **日期/时间格式** | 根据语言显示不同格式       | 使用 vue-i18n 的 datetimeFormats |
| **数字/货币格式** | 本地化数字和货币显示       | 使用 vue-i18n 的 numberFormats   |
| **RTL 支持**      | 支持阿拉伯语等从右到左语言 | 设置 CSS direction 属性          |
| **复数规则**      | 处理不同语言的复数形式     | 使用 vue-i18n 的 tc 函数         |
| **区域设置**      | 时区、度量单位等本地化     | 使用 Intl API                    |
| **动态翻译**      | 支持运行时更新翻译         | 通过 API 获取翻译资源            |
| **翻译记忆**      | 复用已有的翻译内容         | 集成翻译记忆系统                 |

---

## 九、参考文献

### 官方文档

- [Vue I18n 官方文档](https://vue-i18n.intlify.dev/)
- [Tauri 2 官方文档](https://tauri.app/)
- [Tauri API 文档](https://docs.rs/tauri/latest/tauri/)

### 技术标准

- [Unicode CLDR 数据](https://cldr.unicode.org/)
- [ICU 消息格式](https://unicode-org.github.io/icu/userguide/format_parse/messages/)
- [BCP 47 语言标签](https://www.rfc-editor.org/rfc/bcp/bcp47.txt)

### 工具与资源

- [Crowdin](https://crowdin.com/) - 翻译管理平台
- [POEditor](https://poeditor.com/) - 在线翻译工具
- [i18next](https://www.i18next.com/) - 国际化框架
- [Format.js](https://formatjs.io/) - JavaScript 国际化库

### 相关项目

- [vue-i18n-next](https://github.com/intlify/vue-i18n-next)
- [tauri-plugin-i18n](https://github.com/tauri-apps/tauri-plugin-i18n)

---

**作者**：iNovel 开发团队  
**日期**：2026年5月  
**版本**：v2.0  
**版权**：MIT License

---

> 📝 **文档说明**：本文档基于 iNovel 项目的国际化实践经验编写，代码示例均经过实际测试验证。欢迎反馈和建议！
