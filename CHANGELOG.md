# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.3] - 2026-05-13

### Fixed
- 修复增量备份重复文件名错误
- 修复备份目录被包含在备份中的问题
- 修复 git diff 导致的文件重复问题

### Changed
- 更新 GitHub Actions 工作流配置
- 升级 Ubuntu runner 版本到 24.04
- 添加 Node.js 24 支持

### Dependencies
- 更新 Tauri 依赖版本
- 更新前端依赖版本

## [1.1.2] - 2026-05-12

### Changed
- 版本号更新

## [1.1.1] - 2026-05-11

### Added
- 新增枚举字典支持
- 优化编辑器功能
- 添加统一配置管理系统
- 添加国际化支持 (i18n)
- 支持中文简体、繁体和英文

### Fixed
- 修复配置系统路径管理问题

### Changed
- 重构配置系统为基于安装目录的简单路径管理
- 更新 README 文档

## [1.1.0] - 2026-05-10

### Added
- 正式版本发布
- 完整的小说创作功能
- 章节管理系统
- 世界观构建功能
- 时间线功能
- 灵感看板
- 模板系统
- 备份功能
- 导出功能
- 敏感词管理
- 任务清单
- 番茄钟

### Changed
- 统一使用 CSS 变量实现主题切换
- 优化应用性能与资源管理
- 添加性能优化模块
- 重构代码结构

### Fixed
- 修复构建报错
- 修复编辑器样式问题

## [1.0.0] - 2026-05-01

### Added
- 初始版本发布
- 基础编辑器功能
- HTML 到纯文本和 Markdown 转换功能
- 项目管理功能

### Changed
- 更新项目版本至 1.0.0
- 优化发布流程

---

[Unreleased]: https://github.com/RyenLee/inovel/compare/v1.1.3...HEAD
[1.1.3]: https://github.com/RyenLee/inovel/compare/v1.1.2...v1.1.3
[1.1.2]: https://github.com/RyenLee/inovel/compare/v1.1.1...v1.1.2
[1.1.1]: https://github.com/RyenLee/inovel/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/RyenLee/inovel/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/RyenLee/inovel/releases/tag/v1.0.0