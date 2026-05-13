# 灵感看板模块

## 1. 功能概述

灵感看板采用看板式设计，帮助作者收集和管理创作灵感碎片。支持多列布局、拖拽排序、多语言国际化，采用内部标识符 + 动态翻译的设计模式确保数据兼容性。

## 2. 核心功能点

| 功能 | 描述 |
|------|------|
| 创建灵感条目 | 在指定列创建灵感卡片 |
| 更新灵感条目 | 修改灵感内容 |
| 删除灵感条目 | 删除灵感卡片 |
| 排序灵感条目 | 拖拽调整卡片顺序 |
| 获取灵感看板 | 获取完整看板数据（含列和卡片） |
| 获取灵感列表 | 获取所有灵感条目 |

## 3. API 参数说明

### 3.1 create_inspiration_item - 创建灵感条目

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| params | CreateInspirationItemParams | 是 | 创建参数 |

**CreateInspirationItemParams 结构：**

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| project_id | i64 | 是 | 项目 ID |
| column_key | String | 是 | 列标识符 |
| content | String | 是 | 灵感内容 |
| sort_order | i32 | 否 | 排序顺序 |

### 3.2 update_inspiration_item - 更新灵感条目

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| item_id | i64 | 是 | 条目 ID |
| params | UpdateInspirationItemParams | 是 | 更新参数 |

### 3.3 delete_inspiration_item - 删除灵感条目

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| item_id | i64 | 是 | 条目 ID |

### 3.4 reorder_inspiration_items - 重排条目

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| project_id | i64 | 是 | 项目 ID |
| column_key | String | 是 | 列标识符 |
| ordered_ids | Vec<i64> | 是 | 按新顺序排列的 ID 列表 |

### 3.5 get_inspiration_board - 获取看板

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| project_id | i64 | 是 | 项目 ID |

### 3.6 get_inspiration_items - 获取条目列表

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| project_id | i64 | 是 | 项目 ID |

## 4. 数据结构

### 4.1 InspirationItem 灵感条目结构

```json
{
  "id": 1,
  "project_id": 100,
  "column_key": "inspiration",
  "content": "主角失去记忆的设定",
  "sort_order": 0,
  "created_at": "2026-01-01T00:00:00Z",
  "updated_at": "2026-01-01T00:00:00Z"
}
```

### 4.2 InspirationColumn 看板列结构

```json
{
  "column_key": "inspiration",
  "column_name": "灵感",
  "items": [/* InspirationItem[] */]
}
```

### 4.3 InspirationBoard 完整看板

```json
{
  "columns": [
    {
      "column_key": "inspiration",
      "column_name": "灵感",
      "items": []
    },
    {
      "column_key": "dialogue",
      "column_name": "对白",
      "items": []
    },
    {
      "column_key": "scene",
      "column_name": "场景",
      "items": []
    }
  ]
}
```

## 5. 国际化设计

灵感看板采用**内部标识符 + 动态翻译**的设计模式：

| 内部键 | 中文显示 | 英文显示 |
|--------|----------|----------|
| `inspiration` | 灵感 | Inspiration |
| `dialogue` | 对白 | Dialogue |
| `scene` | 场景 | Scene |

## 6. 业务规则

### 6.1 列标识符
- 使用稳定的内部键作为唯一标识
- 不随语言切换改变
- 数据库迁移时自动映射旧数据

### 6.2 排序规则
- 同列内按 `sort_order` 升序排列
- 拖拽操作更新所有受影响条目的排序

## 7. 异常处理

| 错误场景 | 错误信息 | 处理方式 |
|----------|----------|----------|
| 条目不存在 | "灵感条目不存在" | 返回错误 |
| 项目不存在 | "项目不存在" | 返回错误 |
| 列不存在 | "列不存在" | 返回错误 |
| 数据库错误 | "数据库操作失败: {e}" | 返回错误 |

## 8. 相关联功能模块

| 模块 | 关联说明 |
|------|----------|
| [项目管理](project.md) | 灵感看板从属于项目 |
