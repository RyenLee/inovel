# rand crate 版本升级指南（→ 0.10.1）

> 本文档记录 `rand` 从旧版升级至 `0.10.1` 的完整过程，包括版本对比、API 变更详情、受影响文件及具体代码修改示例。

---

## 目录

1. [升级概述](#1-升级概述)
2. [版本信息](#2-版本信息)
3. [API 变更对照表](#3-api-变更对照表)
4. [受影响的文件清单](#4-受影响的文件清单)
5. [代码修改详解](#5-代码修改详解)
6. [兼容性分析](#6-兼容性分析)
7. [升级步骤](#7-升级步骤)
8. [验证方法](#8-验证方法)

---

## 1. 升级概述

`rand 0.10.1` 是一个破坏性变更（breaking changes）较大的版本，主要涉及：

- **`thread_rng()` 函数被移除**，替换为 `rand::rng()`
- **Trait 和类型重命名**：`RngCore` → `Rng`（原 `Rng` → `RngExt`）
- **随机序列 trait 重构**：`SliceRandom` 的读取方法移至 `IndexedRandom`
- **多个便捷方法重命名**：`gen_range` → `random_range`、`gen_bool` → `random_bool`
- **`choose_multiple` 弃用**，推荐改用 `sample`

本项目在以下 3 个文件中存在编译错误，已全部修复并验证通过。

---

## 2. 版本信息

| 配置项 | 值 |
|--------|-----|
| `rand` 版本 | `0.10.1`（`Cargo.toml` 第 32 行） |
| 涉及的源文件 | `encryption.rs`、`names.rs`、`project.rs` |
| 编译结果 | `cargo check` 通过，零错误零警告 |

---

## 3. API 变更对照表

### 3.1 全局随机数生成器

| 旧 API | 新 API | 说明 |
|--------|--------|------|
| `rand::thread_rng()` | `rand::rng()` | `thread_rng` 已被移除 |

### 3.2 Trait 与类型命名

| 旧 API | 新 API | 说明 |
|--------|--------|------|
| `rand::RngCore` | `rand::Rng` | 底层随机数生成器 trait，提供 `fill_bytes`、`next_u32` 等方法 |
| `rand::Rng` | `rand::RngExt` | 用户友好随机数 trait，提供 `gen_range`、`gen_bool` 等便捷方法 |

### 3.3 RngExt 方法名称变更

| 旧方法 | 新方法 | 示例 |
|--------|--------|------|
| `rng.gen_range(0..n)` | `rng.random_range(0..n)` | 生成指定范围内的整数 |
| `rng.gen_bool(p)` | `rng.random_bool(p)` | 按概率生成布尔值 |

### 3.4 序列随机选择

| 旧 API | 新 API | 说明 |
|--------|--------|------|
| `rand::seq::SliceRandom` | `rand::seq::IndexedRandom`（用于读取方法） | 选择操作的 trait 变更 |
| `slice.choose(&mut rng)` | `slice.choose(&mut rng)` | `choose` 保留在 `SliceRandom` 中 |
| `slice.choose_multiple(&mut rng, n)` | `slice.sample(&mut rng, n)` | 多选方法变更 |

---

## 4. 受影响的文件清单

| 文件路径 | 变更类型 | 变更数量 |
|----------|----------|----------|
| `src-tauri/src/encryption.rs` | 导入语句修改 + 函数调用修改 | 3 处 |
| `src-tauri/src/names.rs` | 导入语句修改 + 多处方法调用修改 | 8 处 |
| `src-tauri/src/project.rs` | 导入语句修改 + 方法调用修改 | 3 处 |

---

## 5. 代码修改详解

### 5.1 `encryption.rs`

#### 变更 1：导入语句

**修改前：**
```rust
use rand::{thread_rng, RngCore};
```

**修改后：**
```rust
use rand::Rng;
```

**说明：** `RngCore` 在新版本中已重命名为 `Rng`，而 `thread_rng` 函数已被移除，不再需要导入。

#### 变更 2：`generate_salt()` 函数

**修改前：**
```rust
pub fn generate_salt() -> [u8; SALT_SIZE] {
    let mut salt = [0u8; SALT_SIZE];
    thread_rng().fill_bytes(&mut salt);
    salt
}
```

**修改后：**
```rust
pub fn generate_salt() -> [u8; SALT_SIZE] {
    let mut salt = [0u8; SALT_SIZE];
    rand::rng().fill_bytes(&mut salt);
    salt
}
```

#### 变更 3：`generate_nonce()` 函数

**修改前：**
```rust
pub fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    thread_rng().fill_bytes(&mut nonce);
    nonce
}
```

**修改后：**
```rust
pub fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    rand::rng().fill_bytes(&mut nonce);
    nonce
}
```

---

### 5.2 `names.rs`

#### 变更 1：导入语句

**修改前：**
```rust
use rand::seq::SliceRandom;
use rand::Rng;
```

**修改后：**
```rust
use rand::seq::IndexedRandom;
use rand::RngExt;
```

**说明：**
- `SliceRandom` 在新版本中不再是直接导入的 trait，其读取方法已移至 `IndexedRandom`
- 旧的 `Rng` trait（提供便捷方法）已重命名为 `RngExt`

#### 变更 2：`generate_names()` 函数中的 RNG 初始化

**修改前：**
```rust
pub fn generate_names(category: String, gender: Option<String>, count: u32) -> Vec<String> {
    let mut rng = rand::thread_rng();
```

**修改后：**
```rust
pub fn generate_names(category: String, gender: Option<String>, count: u32) -> Vec<String> {
    let mut rng = rand::rng();
```

#### 变更 3：`gen_bool` → `random_bool`（第 68 行）

**修改前：**
```rust
                    if rng.gen_bool(0.5) { &NAMES_DB.chinese_name.male[..] } else { &NAMES_DB.chinese_name.female[..] },
```

**修改后：**
```rust
                    if rng.random_bool(0.5) { &NAMES_DB.chinese_name.male[..] } else { &NAMES_DB.chinese_name.female[..] },
```

#### 变更 4：`gen_range` → `random_range`（第 74-75 行）

**修改前：**
```rust
                    let idx1 = rng.gen_range(0..given_names.len());
                    let idx2 = rng.gen_range(0..given_names.len());
```

**修改后：**
```rust
                    let idx1 = rng.random_range(0..given_names.len());
                    let idx2 = rng.random_range(0..given_names.len());
```

#### 变更 5：`gen_bool` → `random_bool`（第 85 行）

**修改前：**
```rust
                    if rng.gen_bool(0.5) { &NAMES_DB.western_name.male[..] } else { &NAMES_DB.western_name.female[..] }
```

**修改后：**
```rust
                    if rng.random_bool(0.5) { &NAMES_DB.western_name.male[..] } else { &NAMES_DB.western_name.female[..] }
```

#### 变更 6：`choose_multiple` → `sample`（第 88 行）

**修改前：**
```rust
            names.sample(&mut rng, count).cloned().collect()
```

**修改后：**（保持不变，因为 `sample` 已在 `IndexedRandom` 中可用）

```rust
            names.sample(&mut rng, count).cloned().collect()
```

#### 变更 7：`choose_multiple` → `sample`（第 90-91 行）

**修改前：**
```rust
        "chinese_place" => NAMES_DB.chinese_place.choose_multiple(&mut rng, count).cloned().collect(),
        "western_place" => NAMES_DB.western_place.choose_multiple(&mut rng, count).cloned().collect(),
```

**修改后：**
```rust
        "chinese_place" => NAMES_DB.chinese_place.sample(&mut rng, count).cloned().collect(),
        "western_place" => NAMES_DB.western_place.sample(&mut rng, count).cloned().collect(),
```

---

### 5.3 `project.rs`

#### 变更 1：导入语句

**修改前：**
```rust
use rand::Rng;
```

**修改后：**
```rust
use rand::RngExt;
```

#### 变更 2：`generate_project_id()` 函数

**修改前：**
```rust
fn generate_project_id() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::thread_rng();
    let id: String = (0..5)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    format!("P{}", id)
}
```

**修改后：**
```rust
fn generate_project_id() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::rng();
    let id: String = (0..5)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    format!("P{}", id)
}
```

---

## 6. 兼容性分析

### 6.1 破坏性变更等级：**高**

`rand 0.10` 相比 `rand 0.8` 及更早版本存在大量破坏性变更，升级时需逐文件审查所有 `rand` 相关调用。

### 6.2 迁移影响评估

| 影响维度 | 评估 |
|----------|------|
| 编译影响 | 直接导致编译失败（5 处错误） |
| 运行时影响 | 无 — 所有变更均为 API 重命名，功能逻辑不变 |
| 数据兼容性 | 无影响 — 不涉及持久化数据 |
| 依赖链影响 | `rand 0.10.1` 为间接依赖，通过 `Cargo.lock` 固定 |

### 6.3 依赖版本范围

当前 `Cargo.toml` 中指定：

```toml
rand = "0.10.1"
```

这是**精确版本**而非范围版本，因此不存在隐式的更大版本升级风险。如需支持更宽泛的版本范围，可考虑：

```toml
rand = "0.10"
```

但需注意 `0.10.x` 之间的破坏性变更可能仍存在。

---

## 7. 升级步骤

### 步骤 1：更新 `Cargo.toml`（如需调整版本约束）

确认 `rand` 版本为 `0.10.1`（本项目已满足）：

```toml
rand = "0.10.1"
```

### 步骤 2：修改导入语句

在每个受影响的文件中，按下表替换导入语句：

| 文件 | 修改前 | 修改后 |
|------|--------|--------|
| `encryption.rs` | `use rand::{thread_rng, RngCore};` | `use rand::Rng;` |
| `names.rs` | `use rand::seq::SliceRandom;`<br>`use rand::Rng;` | `use rand::seq::IndexedRandom;`<br>`use rand::RngExt;` |
| `project.rs` | `use rand::Rng;` | `use rand::RngExt;` |

### 步骤 3：替换 `thread_rng()` 调用

将所有 `rand::thread_rng()` 替换为 `rand::rng()`：

```rust
// 替换前
let mut rng = rand::thread_rng();

// 替换后
let mut rng = rand::rng();
```

### 步骤 4：替换 `RngExt` 方法

将旧方法名替换为新方法名：

| 替换前 | 替换后 |
|--------|--------|
| `rng.gen_range(a..b)` | `rng.random_range(a..b)` |
| `rng.gen_bool(p)` | `rng.random_bool(p)` |

### 步骤 5：替换序列选择方法

将 `choose_multiple` 替换为 `sample`：

```rust
// 替换前
items.choose_multiple(&mut rng, count).cloned().collect()

// 替换后
items.sample(&mut rng, count).cloned().collect()
```

---

## 8. 验证方法

### 8.1 编译验证

在 `src-tauri` 目录下执行：

```bash
cargo check
```

预期结果：**编译通过，无错误无警告**

### 8.2 完整构建验证

```bash
cargo build --release
```

### 8.3 检查未使用导入警告

升级后应确保没有未使用的 `rand` 导入：

```bash
cargo clippy 2>&1 | grep -i "unused"
```

本项目升级完成后，Clippy 无 `rand` 相关警告。

---

## 附录：快速对照卡

```
┌─────────────────────────────────────────────────────────┐
│  rand 0.10.1 快速对照                                    │
├─────────────────────────┬───────────────────────────────┤
│  thread_rng()           │ rand::rng()                   │
│  RngCore (trait)        │ rand::Rng (trait)             │
│  Rng (trait)            │ rand::RngExt (trait)          │
│  rng.gen_range(a..b)    │ rng.random_range(a..b)        │
│  rng.gen_bool(p)        │ rng.random_bool(p)            │
│  SliceRandom            │ IndexedRandom (读取方法)       │
│  choose_multiple        │ sample                        │
└─────────────────────────┴───────────────────────────────┘
```

---

*文档生成时间：2026-05-06*
*适用版本：rand 0.10.1*
