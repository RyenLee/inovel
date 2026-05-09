# 统一配置项目信息功能设计文档

## 1. 功能概述

本功能为 iNovel 应用提供一套独立的统一配置管理系统，支持系统版本从 1.0.0 升级至 1.1.0。配置系统独立于现有前后端项目代码，提供集中管理、动态更新、版本控制和安全加密等核心能力。

## 2. 架构设计

### 2.1 模块架构

```
┌─────────────────────────────────────────────────────────────┐
│                    配置管理系统                              │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │   前端模块    │    │   后端模块    │    │   存储模块    │  │
│  │  Config UI   │    │ Config API   │    │ Config Store │  │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘  │
│         │                   │                   │           │
│         │  Tauri IPC        │  文件操作          │           │
│         └───────────────────┴───────────────────┘           │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 目录结构

```
src-tauri/src/
├── config_manager/           # 配置管理核心模块
│   ├── mod.rs               # 模块导出
│   ├── model.rs             # 数据模型定义
│   ├── api.rs               # 配置管理API
│   ├── encryption.rs        # 加密模块
│   ├── loader.rs            # 配置加载/保存
│   └── history.rs           # 历史记录管理
└── commands/
    └── config.rs            # Tauri命令注册

src/
├── config/                  # 前端配置模块
│   ├── types.ts             # TypeScript类型定义
│   ├── api.ts               # 前端API封装
│   └── store.ts             # 状态管理
├── components/
│   └── ConfigManager.vue    # 配置管理界面
└── views/
    └── ConfigManagerPage.vue # 配置管理页面
```

## 3. 配置项定义

### 3.1 配置分类

| 分类 | 名称 | 说明 |
|------|------|------|
| app | 应用配置 | 应用名称、版本号、运行环境等 |
| api | API配置 | API地址、超时时间等 |
| security | 安全配置 | API密钥、安全令牌等（加密存储） |
| feature | 功能开关 | 自动保存、云同步等功能开关 |

### 3.2 配置项列表

| 配置键 | 分类 | 默认值 | 是否加密 | 说明 |
|--------|------|--------|----------|------|
| app_name | app | iNovel | 否 | 应用名称 |
| version | app | 1.1.0 | 否 | 应用版本号 |
| environment | app | development | 否 | 运行环境 |
| api_base_url | api | http://localhost:8080 | 否 | API基础地址 |
| api_timeout | api | 30000 | 否 | API超时时间(毫秒) |
| api_key | security | (空) | 是 | API密钥 |
| secret_token | security | (空) | 是 | 安全令牌 |
| auto_save_enabled | feature | true | 否 | 自动保存开关 |
| sync_enabled | feature | false | 否 | 云同步开关 |

## 4. 后端API接口

### 4.1 获取配置

**接口**: `get_config`

**参数**: 无

**返回**: `AppConfig` 对象

### 4.2 获取配置项

**接口**: `get_config_value`

**参数**: `key: string`

**返回**: `ConfigValue | undefined`

### 4.3 设置配置项

**接口**: `set_config_value`

**参数**: 
- `key: string` - 配置键
- `value: string` - 配置值
- `encrypted: boolean` - 是否加密

**返回**: `boolean` - 是否成功

### 4.4 批量设置配置

**接口**: `set_config_values`

**参数**: `values: Record<string, string>`

**返回**: `ConfigUpdateResult`

### 4.5 更新版本号

**接口**: `update_app_version`

**参数**: `new_version: string`

**返回**: `boolean` - 是否成功

### 4.6 重新加载配置

**接口**: `reload_config`

**参数**: 无

**返回**: `boolean` - 是否成功

### 4.7 导出配置

**接口**: `export_config`

**参数**: `path: string`

**返回**: `ExportResult`

### 4.8 导入配置

**接口**: `import_config`

**参数**: `path: string`

**返回**: `ConfigQueryResult`

### 4.9 重置配置

**接口**: `reset_config`

**参数**: 无

**返回**: `ConfigQueryResult`

### 4.10 获取历史记录

**接口**: `get_config_history`

**参数**: 
- `page: number` - 页码
- `page_size: number` - 每页数量

**返回**: `HistoryQueryResult`

### 4.11 回滚配置

**接口**: `rollback_config`

**参数**: `history_id: string`

**返回**: `ConfigQueryResult`

## 5. 前端API封装

### 5.1 导入方式

```typescript
import * as configApi from '@/config/api'
import { useConfigStore } from '@/config/store'
```

### 5.2 使用示例

```typescript
// 获取配置存储
const configStore = useConfigStore()

// 加载配置
await configStore.loadConfig()

// 获取配置项
const version = configStore.appVersion.value

// 设置配置项
await configStore.setValue('api_base_url', 'https://api.example.com')

// 更新版本
await configStore.updateVersion('1.1.0')

// 导出配置
await configStore.exportTo('/path/to/config.json')
```

## 6. 加密机制

### 6.1 加密算法

采用 AES-256-GCM 对称加密算法：
- 密钥长度：256位
- 随机初始化向量：12字节
- 加密模式：GCM (Galois/Counter Mode)

### 6.2 密钥管理

- 密钥来源：环境变量 `CONFIG_ENCRYPTION_KEY`
- 默认密钥：`iNovelConfigEncryptionKey256BitLength!!`
- 生产环境：建议通过环境变量设置安全密钥

### 6.3 加密流程

```
明文 → AES-256-GCM加密 → Base64编码 → 存储
读取 → Base64解码 → AES-256-GCM解密 → 明文
```

## 7. 历史记录与版本控制

### 7.1 历史记录结构

| 字段 | 类型 | 说明 |
|------|------|------|
| id | string | 记录唯一标识(UUID) |
| snapshot | ConfigSnapshot | 配置快照 |
| action | HistoryAction | 操作类型 |
| operator | string | 操作人(可选) |

### 7.2 操作类型

| 类型 | 说明 |
|------|------|
| Created | 配置创建 |
| Updated | 配置更新 |
| RolledBack | 配置回滚 |
| Exported | 配置导出 |

### 7.3 历史记录限制

- 最大历史记录数：50条
- 超出限制时自动删除最旧记录

## 8. 集成方案

### 8.1 前后端集成

1. **后端集成**: 在 `lib.rs` 中注册配置管理命令
2. **前端集成**: 通过 Vue Router 添加配置管理页面路由
3. **状态同步**: 使用 Vue 响应式系统实现配置实时更新

### 8.2 升级兼容性

- 支持从 1.0.0 版本无缝升级至 1.1.0
- 自动检测并迁移旧版配置格式
- 保留原有配置项，新增配置项使用默认值

### 8.3 回滚机制

1. **配置回滚**: 通过历史记录回滚到任意版本
2. **文件备份**: 每次配置更新前自动备份
3. **异常恢复**: 配置文件损坏时自动使用默认配置

## 9. 安全考虑

### 9.1 敏感配置加密

- 安全分类下的配置项自动加密存储
- 传输过程中保持加密状态
- 前端展示敏感配置时显示掩码

### 9.2 访问控制

- 配置管理界面需要管理员权限
- 关键操作需要二次确认
- 操作日志记录所有配置变更

### 9.3 数据完整性

- 配置文件使用 JSON 格式存储
- 支持配置校验和验证
- 异常配置自动降级到默认值

## 10. 使用指南

### 10.1 访问配置管理

1. 登录应用后，点击侧边栏"配置管理"菜单
2. 或直接访问 `/config` 路由

### 10.2 编辑配置

1. 选择配置分类标签
2. 在输入框中修改配置值
3. 点击"保存"按钮确认修改

### 10.3 导出配置

1. 点击"导出配置"按钮
2. 选择保存路径和文件名
3. 配置文件将以 JSON 格式导出（敏感配置已加密）

### 10.4 导入配置

1. 点击"导入配置"按钮
2. 选择要导入的配置文件
3. 确认覆盖现有配置

### 10.5 重置配置

1. 点击"重置配置"按钮
2. 确认操作（此操作不可撤销）
3. 系统恢复到默认配置

### 10.6 回滚配置

1. 在历史记录列表中找到目标版本
2. 点击"回滚"按钮
3. 确认回滚操作

## 11. 注意事项

1. **敏感配置**: 安全分类下的配置项会自动加密，导出后仍保持加密状态
2. **配置路径**: 配置文件存储在应用数据目录，不建议手动修改
3. **版本更新**: 更新版本号时请遵循语义化版本规范
4. **数据备份**: 修改重要配置前建议先导出备份
5. **权限管理**: 配置管理功能应限制给管理员用户

## 12. 故障排除

### 12.1 配置加载失败

**原因**: 配置文件损坏或格式错误

**解决方案**:
- 检查配置文件是否存在
- 使用"重置配置"恢复默认值
- 从备份文件导入配置

### 12.2 敏感配置无法解密

**原因**: 加密密钥不一致

**解决方案**:
- 确认环境变量 `CONFIG_ENCRYPTION_KEY` 正确设置
- 使用相同密钥重新加密配置

### 12.3 配置更新不生效

**原因**: 缓存未刷新或配置未保存

**解决方案**:
- 点击"重新加载"刷新配置
- 检查网络连接和权限设置

## 13. 版本历史

| 版本 | 日期 | 更新内容 |
|------|------|----------|
| 1.0.0 | 2024-01-01 | 初始版本 |
| 1.1.0 | 2026-05-09 | 新增统一配置管理功能 |

## 14. 相关资源

- [Tauri API 文档](https://tauri.app/docs/)
- [Vue Router 文档](https://router.vuejs.org/)
- [AES-GCM 加密标准](https://en.wikipedia.org/wiki/Galois/Counter_Mode)
- [语义化版本规范](https://semver.org/)