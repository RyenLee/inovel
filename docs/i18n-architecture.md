# iNovel 多语言架构设计文档 — 任务清单功能实现

| 属性 | 值 |
|------|------|
| 文档版本 | 1.0 |
| 创建日期 | 2026-05-09 |
| 适用项目 | iNovel（Vue 3 + TypeScript + Tauri） |
| 已实现语言 | 中文（zh-CN，默认）、英语（en-US） |

---

## 1. 架构总览

### 1.1 设计原则

- **文本与逻辑分离**：所有用户可见文本存储在独立 JSON 语言资源文件中，组件代码仅通过 `t('key')` 引用
- **开闭原则**：新增语言只需添加语言资源文件和目录入口，无需修改任何组件代码
- **数据与语言解耦**：任务数据（TaskItem）与界面语言完全独立，语言切换不丢失任何数据
- **渐进式回退**：缺失翻译自动降级到 zh-CN 默认语言

### 1.2 技术栈

| 组件 | 版本 | 用途 |
|------|------|------|
| vue-i18n | @next (v11+) | Vue 3 国际化核心库 |
| Naive UI locale | 2.44+ | 组件库内置国际化支持 |
| TypeScript | 6.0+ | 类型安全的翻译键 |
| Vitest | 4.1+ | 兼容性测试框架 |

### 1.3 文件结构

```
src/
├── i18n/
│   ├── index.ts                    # i18n 实例配置、setLocale/getLocale、缺失键追踪
│   └── composables/
│       └── useLocale.ts            # 组件级 composable（t, locale, switchLocale）
├── locales/
│   ├── zh-CN/
│   │   ├── index.ts                # 聚合导出
│   │   ├── common.json             # 通用文本（操作、状态、时间、对话框）
│   │   └── task.json               # 任务清单专用文本
│   └── en-US/
│       ├── index.ts                # 聚合导出
│       ├── common.json             # 通用文本（英文）
│       └── task.json               # 任务清单专用文本（英文）
├── components/
│   └── TaskChecklist.vue           # 任务清单组件（完全 i18n 化）
├── views/
│   ├── TaskChecklistPage.vue       # 任务清单页面
│   └── SettingsPage.vue            # 设置页（含语言切换）
└── tests/
    └── i18n.spec.ts                # 兼容性测试（13 项）
```

---

## 2. 语言资源管理机制

### 2.1 资源文件格式

每个语言包按功能域拆分为独立 JSON 文件，使用点分隔层级命名法：

```json
{
  "namespace": {
    "element": {
      "qualifier": "翻译文本 {placeholder}"
    }
  }
}
```

**命名规则**：
- 第一级：功能域（common、task、editor、sidebar 等）
- 第二级：元素类型（action、status、dialog 等）
- 第三级：具体标识（save、cancel、confirmDelete 等）

### 2.2 新增语言包流程

1. 创建 `src/locales/{locale}/` 目录
2. 从 `zh-CN/` 复制所有 JSON 文件
3. 翻译 JSON 中的值（保持键结构完全一致）
4. 创建 `src/locales/{locale}/index.ts` 聚合导出
5. 在 `src/i18n/index.ts` 中导入新语言包并添加到 `messages`
6. 在 `useLocale.ts` 的 `availableLocales` 中添加新选项
7. 运行 `vitest run src/tests/i18n.spec.ts` 验证键完整性

### 2.3 当前语言包统计

| 语言包 | 文件数 | 翻译键数 | 占位符数 |
|--------|--------|---------|---------|
| zh-CN  | 2      | 57      | 3       |
| en-US  | 2      | 57      | 3       |

---

## 3. 核心实现

### 3.1 i18n 实例配置

```typescript
// src/i18n/index.ts
const i18n = createI18n({
  legacy: false,                    // 使用 Composition API 模式
  locale: getStoredLocale(),        // localStorage > 系统语言 > 默认值
  fallbackLocale: 'zh-CN',          // 缺失翻译时回退到中文
  messages: { 'zh-CN': zhCN, 'en-US': enUS },
  missing: handleMissing,           // 开发环境缺失键警告
  missingWarn: true,                // 控制台缺失警告
  fallbackWarn: true,               // 控制台回退警告
})
```

### 3.2 语言检测优先级

```
localStorage('inovel_locale') → navigator.language → 'zh-CN'(默认)
```

### 3.3 useLocale Composable

```typescript
const { t, locale, isZhCN, isEnUS, availableLocales, switchLocale, currentLocale } = useLocale()
```

| 属性/方法 | 类型 | 说明 |
|-----------|------|------|
| `t` | Function | 翻译函数，`t('task.title')` → "任务清单" / "Task Checklist" |
| `locale` | Ref\<string\> | 当前语言代码 |
| `isZhCN` | ComputedRef\<boolean\> | 是否为中文 |
| `isEnUS` | ComputedRef\<boolean\> | 是否为英文 |
| `availableLocales` | Array | 可用语言列表 |
| `switchLocale` | Function | 切换语言（持久化 + 更新 DOM lang 属性） |
| `currentLocale` | ComputedRef | 当前语言代码 |

### 3.4 Naive UI 国际化集成

```vue
<!-- App.vue -->
<n-config-provider
  :locale="isZhCN ? zhCN : enUS"
  :date-locale="isZhCN ? dateZhCN : dateEnUS"
>
```

Naive UI 的日期选择器、分页、对话框等内置组件文本随语言自动切换。

---

## 4. 回退机制

### 4.1 三级回退链

```
当前语言翻译 → zh-CN 翻译 → 键名本身（如 "task.title"）
```

1. **第一级**：查找当前语言（如 en-US）中对应键的翻译
2. **第二级**：当前语言缺失时，自动回退到 `fallbackLocale: 'zh-CN'`
3. **第三级**：zh-CN 也缺失时，显示键名字符串（开发环境同时输出控制台警告）

### 4.2 缺失键追踪

```typescript
const missingKeys = new Set<string>()

function handleMissing(locale: string, key: string): string {
  const cacheKey = `${locale}::${key}`
  if (!missingKeys.has(cacheKey)) {
    missingKeys.add(cacheKey)
    console.warn(`[i18n] Missing translation: locale="${locale}", key="${key}"`)
  }
  return key  // 返回键名作为最终回退
}

export function getMissingKeys(): string[] {
  return Array.from(missingKeys)
}
```

- 仅在开发环境启用（`import.meta.env.DEV`）
- 去重输出，避免控制台刷屏
- 可通过 `getMissingKeys()` 获取所有缺失键列表

---

## 5. 语言切换数据完整性

### 5.1 切换流程

```
用户选择新语言
  → switchLocale(newLocale)
    → i18n.global.locale.value = newLocale    // 更新响应式 locale
    → localStorage.setItem('inovel_locale')    // 持久化
    → document.documentElement.lang = newLocale // 更新 DOM
  → Vue 响应式系统自动重新渲染所有 t() 调用
  → Naive UI locale 切换（通过 computed 属性）
```

### 5.2 数据不丢失保证

- 任务数据（`TaskItem[]`）存储在组件 `ref` 中，与语言无关
- 语言切换仅影响 `t()` 返回的文本，不触发任何数据变更
- JSON 导入/导出使用语言无关的数据结构
- 组件状态（筛选、排序、模态框）在切换后保持不变

---

## 6. UI 自适应设计

### 6.1 文本扩展率分析

| 中文原文 | 英文翻译 | 扩展率 | 适配策略 |
|---------|---------|--------|---------|
| 新建任务 | New Task | 1.5x | 按钮弹性宽度 |
| 清除已完成 | Clear Completed | 2.0x | 按钮弹性宽度 |
| 无截止日期 | No due date | 2.3x | 文本自适应 |
| 按优先级排序 | Sort by Priority | 2.3x | 下拉选择器弹性宽度 |
| 成功导入 {count} 项任务 | Successfully imported {count} task(s) | 2.0x | 消息提示自适应 |

### 6.2 自适应技术方案

| 技术手段 | 应用位置 | 说明 |
|---------|---------|------|
| `flex-wrap: wrap` | 工具栏、元数据区域 | 长文本自动换行 |
| `min-width` + 弹性宽度 | 下拉选择器 | 保证最小宽度，允许扩展 |
| `word-break: break-word` | 任务名称 | 超长名称自动断行 |
| `max-width: 90vw` | 模态框 | 防止溢出视口 |
| `whitespace-nowrap` | 页面标题 | 标题不换行 |
| CSS `overflow` | 进度条区域 | 防止文本溢出 |

---

## 7. 兼容性测试

### 7.1 测试覆盖

| 测试类别 | 测试项数 | 状态 |
|---------|---------|------|
| 键完整性 | 4 | ✅ 通过 |
| 占位符一致性 | 2 | ✅ 通过 |
| 回退机制 | 2 | ✅ 通过 |
| 语言切换数据完整性 | 2 | ✅ 通过 |
| 文本扩展兼容性 | 1 | ✅ 通过 |
| 新语言包导入兼容性 | 2 | ✅ 通过 |
| **合计** | **13** | **✅ 全部通过** |

### 7.2 测试详情

**键完整性测试**：
- zh-CN 和 en-US 的 common.json 键集合完全一致
- zh-CN 和 en-US 的 task.json 键集合完全一致
- en-US 中无空翻译值

**占位符一致性测试**：
- 所有 `{placeholder}` 在两种语言中名称和数量一致
- 防止运行时插值错误

**回退机制测试**：
- fallbackLocale 配置为 zh-CN
- zh-CN 所有键值均为有效非空字符串

**数据完整性测试**：
- 任务数据序列化/反序列化后字段完整
- 语言偏好持久化模式验证

**文本扩展测试**：
- 英文文本扩展率不超过 5 倍（超出需 UI 适配）

**新语言包兼容性测试**：
- 验证新语言包必须遵循与 zh-CN 相同的键结构
- 检测多余键（extra keys）

### 7.3 构建验证

| 检查项 | 结果 |
|--------|------|
| vue-tsc --noEmit | ✅ 0 错误 |
| vitest run | ✅ 30/30 通过 |
| vite build | ✅ 构建成功 |

---

## 8. TaskChecklist 功能说明

### 8.1 功能列表

| 功能 | 说明 | i18n 支持 |
|------|------|----------|
| 新建任务 | 名称、优先级、截止日期、负责人、备注 | ✅ |
| 编辑任务 | 修改所有字段 | ✅ |
| 删除任务 | 带确认对话框 | ✅ |
| 完成/取消完成 | 复选框切换 | ✅ |
| 筛选 | 全部/待完成/已完成 | ✅ |
| 排序 | 按名称/日期/优先级 | ✅ |
| 进度条 | 完成百分比 | ✅ |
| 逾期标记 | 红色左边框 + 错误色文本 | ✅ |
| 优先级标签 | 高/中/低 色彩编码 | ✅ |
| 导出 JSON | 下载任务数据文件 | ✅ |
| 导入 JSON | 从文件加载任务 | ✅ |
| 清除已完成 | 批量删除已完成任务 | ✅ |

### 8.2 数据结构

```typescript
interface TaskItem {
  id: string           // 唯一标识
  name: string         // 任务名称
  completed: boolean   // 完成状态
  priority: TaskPriority  // 'high' | 'medium' | 'low'
  dueDate: number | null  // 截止日期时间戳
  assignee: string     // 负责人
  notes: string        // 备注
  tags: string[]       // 标签
  createdAt: number    // 创建时间
  updatedAt: number    // 更新时间
}
```

---

## 9. 访问路径

- 任务清单页面：`/#/tasks`
- 语言切换：设置页（`/#/settings`）→ 语言 / Language 卡片
