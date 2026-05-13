# 世界观构建模块

## 1. 功能概述

世界观构建模块为小说创作提供完整的世界观设定管理功能，包括人物（Character）、地点（Location）、组织（Organization）的 CRUD 操作，以及关系图谱（Relationship）和故事时间轴（Timeline）的可视化呈现。通过 @提及 功能，可以在正文中快速引用和跳转世界观元素。

## 2. 核心功能点

### 2.1 人物管理

| 功能 | 描述 |
|------|------|
| 创建人物 | 创建新的角色记录 |
| 更新人物 | 更新角色的基本信息和自定义字段 |
| 删除人物 | 删除角色记录 |
| 列出人物 | 获取项目的所有角色列表 |

### 2.2 地点管理

| 功能 | 描述 |
|------|------|
| 创建地点 | 创建新的地点记录 |
| 更新地点 | 更新地点的基本信息和自定义字段 |
| 删除地点 | 删除地点记录 |
| 列出地点 | 获取项目的所有地点列表 |

### 2.3 组织管理

| 功能 | 描述 |
|------|------|
| 创建组织 | 创建新的组织记录 |
| 更新组织 | 更新组织的基本信息和自定义字段 |
| 删除组织 | 删除组织记录 |
| 列出组织 | 获取项目的所有组织列表 |

### 2.4 关系图谱

| 功能 | 描述 |
|------|------|
| 创建关系 | 创建两人物/组织之间的关系 |
| 更新关系 | 更新关系的类型和描述 |
| 删除关系 | 删除关系记录 |
| 获取关系列表 | 获取项目的所有关系 |

### 2.5 时间轴

| 功能 | 描述 |
|------|------|
| 创建事件 | 在时间轴上创建事件 |
| 更新事件 | 更新事件的时间和描述 |
| 删除事件 | 删除时间轴事件 |
| 列出事件 | 获取时间轴上的所有事件 |

## 3. API 参数说明

### 3.1 人物管理

#### create_character - 创建人物

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| params | CreateCharacterParams | 是 | 创建参数 |

**CreateCharacterParams 结构：**

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| project_id | i64 | 是 | 项目 ID |
| name | String | 是 | 人物名称 |
| description | String | 否 | 人物描述 |
| custom_fields | HashMap<String, String> | 否 | 自定义字段 |

#### list_characters - 列出人物

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| project_id | i64 | 是 | 项目 ID |

#### update_character - 更新人物

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| character_id | i64 | 是 | 人物 ID |
| params | UpdateCharacterParams | 是 | 更新参数 |

#### delete_character - 删除人物

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| character_id | i64 | 是 | 人物 ID |

### 3.2 地点管理

#### create_location - 创建地点

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| params | CreateLocationParams | 是 | 创建参数 |

#### update_location - 更新地点

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| location_id | i64 | 是 | 地点 ID |
| params | UpdateLocationParams | 是 | 更新参数 |

#### delete_location - 删除地点

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| location_id | i64 | 是 | 地点 ID |

#### list_locations - 列出地点

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| project_id | i64 | 是 | 项目 ID |

### 3.3 组织管理

#### create_organization - 创建组织

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| params | CreateOrganizationParams | 是 | 创建参数 |

#### update_organization - 更新组织

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| organization_id | i64 | 是 | 组织 ID |
| params | UpdateOrganizationParams | 是 | 更新参数 |

#### delete_organization - 删除组织

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| organization_id | i64 | 是 | 组织 ID |

#### list_organizations - 列出组织

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| project_id | i64 | 是 | 项目 ID |

### 3.4 关系图谱

#### create_relationship - 创建关系

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| params | CreateRelationshipParams | 是 | 创建参数 |

**CreateRelationshipParams 结构：**

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| project_id | i64 | 是 | 项目 ID |
| source_id | i64 | 是 | 源实体 ID（人物或组织） |
| source_type | String | 是 | 源实体类型（"character" 或 "organization"） |
| target_id | i64 | 是 | 目标实体 ID |
| target_type | String | 是 | 目标实体类型 |
| relationship_type | String | 是 | 关系类型 |
| description | String | 否 | 关系描述 |

#### get_relationships - 获取关系列表

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| project_id | i64 | 是 | 项目 ID |

#### update_relationship - 更新关系

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| relationship_id | i64 | 是 | 关系 ID |
| params | UpdateRelationshipParams | 是 | 更新参数 |

#### delete_relationship - 删除关系

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| relationship_id | i64 | 是 | 关系 ID |

### 3.5 时间轴

#### create_event - 创建事件

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| params | CreateEventParams | 是 | 创建参数 |

**CreateEventParams 结构：**

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| project_id | i64 | 是 | 项目 ID |
| title | String | 是 | 事件标题 |
| description | String | 否 | 事件描述 |
| story_time | String | 是 | 故事时间（如 "第1年"、"第三章"） |
| sort_order | i32 | 否 | 排序顺序 |

#### list_events - 列出事件

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| project_id | i64 | 是 | 项目 ID |

#### update_event - 更新事件

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| event_id | i64 | 是 | 事件 ID |
| params | UpdateEventParams | 是 | 更新参数 |

#### delete_event - 删除事件

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| event_id | i64 | 是 | 事件 ID |

## 4. 数据结构

### 4.1 Character 人物结构

```json
{
  "id": 1,
  "project_id": 100,
  "name": "张三",
  "description": "主角，一位年轻的剑客",
  "custom_fields": {
    "年龄": "25",
    "武器": "长剑"
  },
  "created_at": "2026-01-01T00:00:00Z",
  "updated_at": "2026-01-01T00:00:00Z"
}
```

### 4.2 Location 地点结构

```json
{
  "id": 1,
  "project_id": 100,
  "name": "青云山",
  "description": "修仙门派的所在地",
  "custom_fields": {},
  "created_at": "2026-01-01T00:00:00Z",
  "updated_at": "2026-01-01T00:00:00Z"
}
```

### 4.3 Organization 组织结构

```json
{
  "id": 1,
  "project_id": 100,
  "name": "青云派",
  "description": "正道修仙门派",
  "custom_fields": {},
  "created_at": "2026-01-01T00:00:00Z",
  "updated_at": "2026-01-01T00:00:00Z"
}
```

### 4.4 Relationship 关系结构

```json
{
  "id": 1,
  "project_id": 100,
  "source_id": 1,
  "source_type": "character",
  "target_id": 2,
  "target_type": "character",
  "relationship_type": "师徒",
  "description": "张三是李四的师父",
  "created_at": "2026-01-01T00:00:00Z",
  "updated_at": "2026-01-01T00:00:00Z"
}
```

### 4.5 Event 时间轴事件结构

```json
{
  "id": 1,
  "project_id": 100,
  "title": "入门大典",
  "description": "张三正式拜入青云派",
  "story_time": "第1年春天",
  "sort_order": 0,
  "created_at": "2026-01-01T00:00:00Z",
  "updated_at": "2026-01-01T00:00:00Z"
}
```

## 5. 业务规则

### 5.1 @提及规则
- 在正文中输入 `@人物名` 自动触发人物提及
- 在正文中输入 `@地点名` 自动触发地点提及
- 在正文中输入 `@组织名` 自动触发组织提及
- 悬停显示详情弹窗，点击跳转到世界观详情页

### 5.2 关系类型
- 支持自定义关系类型
- 关系可以是单向或双向
- 关系可连接人物与人物、人与组织、组织与组织

### 5.3 时间排序
- 事件按 `story_time` 字段排序
- `sort_order` 用于同时间的二次排序

### 5.4 删除规则
- 删除人物时，同时删除其所有关系
- 删除地点/组织时不断言删除关系（由前端处理）

## 6. 异常处理

| 错误场景 | 错误信息 | 处理方式 |
|----------|----------|----------|
| 实体不存在 | "{type} 不存在" | 返回错误 |
| 项目不存在 | "项目不存在" | 返回错误 |
| 数据库错误 | "数据库操作失败: {e}" | 返回错误 |
| 参数无效 | "无效的参数: {detail}" | 返回错误 |

## 7. 相关联功能模块

| 模块 | 关联说明 |
|------|----------|
| [项目管理](project.md) | 世界观元素从属于项目 |
| [章节管理](chapter.md) | 可在章节中 @提及 世界观元素 |
| [编辑器](../components/markdown-editor.md) | 支持 @提及 扩展 |

## 8. 界面交互说明

### 8.1 世界观页面
- 左侧分类导航（人物/地点/组织）
- 中间列表展示
- 右侧详情编辑面板
- 支持自定义字段的添加和编辑

### 8.2 关系图谱
- 使用 @vue-flow 库实现可视化
- 支持拖拽布局
- 节点显示实体名称
- 边显示关系类型

### 8.3 时间轴
- 垂直时间轴布局
- 显示事件标题和故事时间
- 支持编辑和删除操作
- 可关联到具体章节
