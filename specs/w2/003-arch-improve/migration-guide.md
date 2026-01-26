# 数据库服务架构重构迁移指南

本文档提供详细的分阶段迁移计划，帮助团队安全地从当前架构迁移到新的 trait-based 架构。

## 📋 目录

1. [迁移概览](#迁移概览)
2. [前置准备](#前置准备)
3. [阶段 1：创建新架构](#阶段-1创建新架构非破坏性)
4. [阶段 2：迁移命令处理器](#阶段-2迁移命令处理器)
5. [阶段 3：清理旧代码](#阶段-3清理旧代码)
6. [测试策略](#测试策略)
7. [回滚计划](#回滚计划)
8. [常见问题](#常见问题)

---

## 🎯 迁移概览

### 迁移时间表

| 阶段 | 时间估计 | 风险等级 | 状态 |
|------|---------|---------|------|
| 前置准备 | 0.5 天 | 低 | ⏸️ 待开始 |
| 阶段 1：创建新架构 | 2-3 天 | 中 | ⏸️ 待开始 |
| 阶段 2：迁移命令 | 1-2 天 | 高 | ⏸️ 待开始 |
| 阶段 3：清理 | 0.5 天 | 低 | ⏸️ 待开始 |
| **总计** | **4-6 天** | - | - |

### 迁移原则

- ✅ **渐进式迁移**：每个阶段都可以独立验证
- ✅ **向后兼容**：新架构与旧代码共存
- ✅ **随时可回滚**：每个阶段后都有回滚点
- ✅ **测试驱动**：先写测试，再改代码

---

## 📦 前置准备

### 1. 创建工作分支

```bash
# 从主分支创建特性分支
git checkout main
git pull origin main
git checkout -b feature/db-architecture-refactor

# 确认当前分支
git branch
```

### 2. 备份当前代码

虽然使用 git 可以轻松回滚，但建议创建一个里程碑标签：

```bash
# 创建备份标签
git tag pre-refactor-backup

# 推送标签到远程
git push origin pre-refactor-backup
```

### 3. 环境准备

确保所有依赖已安装：

```bash
# 进入项目目录
cd w2-db-query/src-tauri

# 检查 Rust 版本
rustc --version  # 应该是 1.70+ 或更新

# 确保项目可以编译
cargo build

# 运行现有测试
cargo test
```

### 4. 阅读架构设计文档

确保已经阅读并理解：
- [architecture-design.md](./architecture-design.md) - 完整的架构设计
- [examples/](./examples/) - 代码示例

### 5. 设置测试环境

准备测试数据库：

```bash
# PostgreSQL
export TEST_PG_HOST=localhost
export TEST_PG_PORT=5432
export TEST_PG_NAME=testdb
export TEST_PG_USER=testuser
export TEST_PG_PASS=testpass

# MySQL
export TEST_MYSQL_HOST=localhost
export TEST_MYSQL_PORT=3306
export TEST_MYSQL_NAME=testdb
export TEST_MYSQL_USER=testuser
export TEST_MYSQL_PASS=testpass
```

---

## 🏗️ 阶段 1：创建新架构（非破坏性）

**时间估计**：2-3 天
**风险等级**：中
**目标**：创建新的 trait-based 架构，与现有代码共存

### 1.1 创建目录结构

```bash
cd src-tauri/src/services
mkdir -p database
touch database/{mod.rs,trait.rs,factory.rs,postgres_impl.rs,mysql_impl.rs}
```

### 1.2 定义 Trait（trait.rs）

**文件**：`src/services/database/trait.rs`

**任务**：
1. 从 [examples/trait.rs](./examples/trait.rs) 复制基础代码
2. 根据实际项目调整导入路径
3. 确保 async-trait 依赖在 Cargo.toml 中

**验证**：

```bash
# 编译检查
cargo build --lib

# 运行 trait 单元测试
cargo test --lib services::database::trait
```

**验收标准**：
- ✅ 代码编译通过
- ✅ 所有单元测试通过
- ✅ 没有警告

### 1.3 实现 Factory（factory.rs）

**文件**：`src/services/database/factory.rs`

**任务**：
1. 从 [examples/factory.rs](./examples/factory.rs) 复制代码
2. 调整导入路径（稍后创建的服务）
3. 临时注释掉 PostgresService 和 MySqlService 的导入

**验证**：

```bash
# 编译检查（会报错，因为服务还没实现）
cargo build --lib 2>&1 | head -20
```

**预期错误**：
```
error[E0433]: failed to resolve: use of undeclared type PostgresService
```
这是预期的，下一步会解决。

### 1.4 实现 PostgreSQL 服务（postgres_impl.rs）

**文件**：`src/services/database/postgres_impl.rs`

**任务**：
1. 从现有的 `postgres_service.rs` 迁移逻辑
2. 或者从 [examples/postgres_impl.rs](./examples/postgres_impl.rs) 开始
3. 实现 DatabaseService trait 的所有方法
4. 迁移类型转换逻辑

**关键迁移点**：

| 现有函数 | 新 Trait 方法 | 说明 |
|---------|--------------|------|
| `connect()` | `connect()` | 直接迁移 |
| `test_connection()` | `test_connection()` | 直接迁移 |
| `execute_query()` | `execute_query()` | 返回类型改为 `QueryExecutionResult` |
| `extract_tables()` | `extract_metadata()` | 合并多个提取函数 |
| - | `convert_row_to_json()` | 从命令处理器迁移 |
| - | `get_sql_dialect()` | 新增方言信息 |

**验证**：

```bash
# 编译检查
cargo build --lib

# 运行测试
cargo test --lib services::database::postgres_impl
```

**验收标准**：
- ✅ 代码编译通过
- ✅ 所有测试通过
- ✅ 与现有 PostgreSQL 集成测试通过

### 1.5 实现 MySQL 服务（mysql_impl.rs）

**文件**：`src/services/database/mysql_impl.rs`

**任务**：同 PostgreSQL 服务实现

**验证**：

```bash
# 编译检查
cargo build --lib

# 运行测试
cargo test --lib services::database::mysql_impl
```

**验收标准**：
- ✅ 代码编译通过
- ✅ 所有测试通过
- ✅ 与现有 MySQL 集成测试通过

### 1.6 更新模块导出

**文件**：`src/services/database/mod.rs`

```rust
pub mod factory;
pub mod trait;
pub mod postgres_impl;
pub mod mysql_impl;

pub use factory::*;
pub use trait::*;
```

**文件**：`src/services/mod.rs`

```rust
// 现有导出...
pub mod database;  // 添加这一行

pub use database::*;
```

**验证**：

```bash
# 完整编译
cargo build

# 运行所有测试
cargo test
```

### 1.7 阶段 1 验收

**检查清单**：

- [ ] 所有新文件已创建
- [ ] 所有代码编译通过
- [ ] 所有单元测试通过
- [ ] 所有集成测试通过
- [ ] 没有引入新的警告
- [ ] 代码已提交到 git

**提交代码**：

```bash
git add .
git commit -m "feat: add database service trait and factory (phase 1)

- Define DatabaseService trait
- Implement factory pattern for service creation
- Implement PostgreSQL and MySQL services
- All tests passing
- No breaking changes to existing code"
```

---

## 🔄 阶段 2：迁移命令处理器

**时间估计**：1-2 天
**风险等级**：高
**目标**：更新命令处理器使用新的工厂模式

### 2.1 初始化全局 Factory

**文件**：`src/main.rs`

在 main 函数开始处添加：

```rust
fn main() {
    // 初始化数据库服务工厂
    services::database::init_global_factory();

    tauri::Builder::default()
        // ... 现有代码
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**验证**：

```bash
cargo build
```

### 2.2 更新 database.rs 命令

**文件**：`src/commands/database.rs`

**重构前**：

```rust
pub async fn test_connection(request: TestConnectionRequest) -> Result<bool, String> {
    match request.database_type {
        DatabaseType::PostgreSQL => {
            postgres_service::test_connection(/* ... */).await
        }
        DatabaseType::MySQL => {
            mysql_service::test_connection(/* ... */).await
        }
    }
}
```

**重构后**：

```rust
pub async fn test_connection(request: TestConnectionRequest) -> Result<bool, String> {
    // 使用工厂获取服务
    let factory = services::database::get_global_factory();
    let service = factory
        .get_service(&request.database_type)
        .map_err(|e| e.to_string())?;

    // 使用抽象接口
    service
        .test_connection(
            &request.host,
            request.port,
            &request.database_name,
            &request.user,
            &request.password,
        )
        .await
        .map_err(|e| e.to_string())
}
```

**需要重构的函数**：
- `test_connection()`
- `add_database()`（如果有连接测试）

**验证**：

```bash
# 编译
cargo build

# 手动测试：添加数据库连接
cargo run
```

### 2.3 更新 query.rs 命令

**文件**：`src/commands/query.rs`

**重构前**：

```rust
pub async fn run_sql_query(request: RunQueryRequest) -> Result<QueryResult, String> {
    let result = match connection.database_type {
        DatabaseType::PostgreSQL => {
            let client = postgres_service::connect(/* ... */).await?;
            let (cols, rows, time) = postgres_service::execute_query(&client, &sql).await?;
            convert_postgres_rows(/* ... */)
        }
        DatabaseType::MySQL => {
            let pool = mysql_service::connect(/* ... */).await?;
            let (cols, rows, time) = mysql_service::execute_query(&pool, &sql).await?;
            convert_mysql_rows(/* ... */)
        }
    };
}
```

**重构后**：

```rust
pub async fn run_sql_query(request: RunQueryRequest) -> Result<QueryResult, String> {
    // DDL 检查（保持不变）
    if query_parser::is_ddl_statement(&request.sql).map_err(|e| e.to_string())? {
        return Err("不允许执行 DDL 语句".to_string());
    }

    // 注入 LIMIT（保持不变）
    let parsed_sql = query_parser::inject_limit(&request.sql).map_err(|e| e.to_string())?;

    // 加载连接（保持不变）
    let connections = cache_service::load_connections()
        .map_err(|e| format!("加载连接失败: {}", e))?;

    let connection = connections
        .iter()
        .find(|c| c.id == request.database_id)
        .ok_or_else(|| "数据库连接不存在".to_string())?;

    // === 使用工厂模式 ===
    let factory = services::database::get_global_factory();
    let service = factory
        .get_service(&connection.database_type)
        .map_err(|e| e.to_string())?;

    // 连接
    let db_connection = service
        .connect(
            &connection.host,
            connection.port,
            &connection.database_name,
            &connection.user,
            &connection.password,
        )
        .await
        .map_err(|e| format!("连接失败: {}", e))?;

    // 执行查询
    let exec_result = service
        .execute_query(&db_connection, &parsed_sql, &[])
        .await
        .map_err(|e| e.to_string())?;

    // 转换结果
    let mut result_rows = Vec::new();
    for db_row in &exec_result.rows {
        let row_map = service
            .convert_row_to_json(db_row, &db_row.columns)
            .map_err(|e| e.to_string())?;
        result_rows.push(row_map);
    }

    let columns = exec_result
        .rows
        .first()
        .map(|row| row.columns.clone())
        .unwrap_or_default();

    let result = QueryResult {
        columns,
        rows: result_rows,
        total: result_rows.len(),
        exec_time_ms: exec_result.exec_time_ms,
        sql: parsed_sql,
        truncated: result_rows.len() >= 100,
    };

    // 保存历史（保持不变）
    let _ = cache_service::save_query_history(/* ... */);

    Ok(result)
}
```

**需要删除的辅助函数**：
- `convert_postgres_rows()`
- `convert_mysql_rows()`

**验证**：

```bash
# 编译
cargo build

# 手动测试：执行 SQL 查询
cargo run
```

### 2.4 更新 metadata.rs 命令

**文件**：`src/commands/metadata.rs`

**重构前**：

```rust
pub async fn refresh_metadata(database_id: String) -> Result<DatabaseMetadata, String> {
    match connection.database_type {
        DatabaseType::PostgreSQL => {
            // PostgreSQL 元数据提取
        }
        DatabaseType::MySQL => {
            // MySQL 元数据提取
        }
    }
}
```

**重构后**：

```rust
pub async fn refresh_metadata(database_id: String) -> Result<DatabaseMetadata, String> {
    // 加载连接（保持不变）
    let connections = cache_service::load_connections()
        .map_err(|e| format!("加载连接失败: {}", e))?;

    let connection = connections
        .iter()
        .find(|c| c.id == database_id)
        .ok_or_else(|| "数据库连接不存在".to_string())?;

    // === 使用工厂模式 ===
    let factory = services::database::get_global_factory();
    let service = factory
        .get_service(&connection.database_type)
        .map_err(|e| e.to_string())?;

    // 连接
    let db_connection = service
        .connect(/* ... */)
        .await
        .map_err(|e| format!("连接失败: {}", e))?;

    // 提取元数据
    let metadata = service
        .extract_metadata(&db_connection, &database_id)
        .await
        .map_err(|e| format!("提取元数据失败: {}", e))?;

    // 保存到缓存（保持不变）
    let metadata_json = serde_json::to_string(&metadata)
        .map_err(|e| format!("序列化元数据失败: {}", e))?;

    cache_service::save_metadata(&database_id, &metadata_json)
        .map_err(|e| format!("保存元数据失败: {}", e))?;

    Ok(metadata)
}
```

**验证**：

```bash
# 编译
cargo build

# 手动测试：刷新元数据
cargo run
```

### 2.5 阶段 2 验收

**检查清单**：

- [ ] 所有命令处理器已更新
- [ ] 代码编译通过
- [ ] 所有单元测试通过
- [ ] 所有集成测试通过
- [ ] 手动功能测试通过
- [ ] 代码已提交到 git

**功能测试清单**：

- [ ] 测试 PostgreSQL 连接
- [ ] 测试 MySQL 连接
- [ ] 测试 PostgreSQL 查询执行
- [ ] 测试 MySQL 查询执行
- [ ] 测试 PostgreSQL 元数据提取
- [ ] 测试 MySQL 元数据提取
- [ ] 测试错误处理

**提交代码**：

```bash
git add .
git commit -m "refactor: migrate commands to use factory pattern (phase 2)

- Update database commands to use factory pattern
- Update query commands to use factory pattern
- Update metadata commands to use factory pattern
- Remove database-specific match statements
- Remove duplicate type conversion functions
- All tests passing
- Breaking change: commands now use factory"
```

---

## 🧹 阶段 3：清理旧代码

**时间估计**：0.5 天
**风险等级**：低
**目标**：删除旧的服务文件和未使用的代码

### 3.1 删除旧服务文件

```bash
cd src-tauri/src/services

# 删除旧服务文件
rm postgres_service.rs
rm mysql_service.rs
```

### 3.2 更新模块导出

**文件**：`src/services/mod.rs`

删除旧的导出：

```rust
// 删除这些行
pub mod postgres_service;
pub mod mysql_service;
pub use postgres_service::*;
pub use mysql_service::*;
```

### 3.3 清理依赖（可选）

检查 `Cargo.toml` 是否有未使用的依赖：

```bash
cargo install cargo-udeps
cargo +nightly udeps
```

如果有未使用的依赖，可以删除。

### 3.4 代码审查

检查是否有其他地方引用了旧的服务：

```bash
cd src-tauri

# 搜索旧服务的引用
grep -r "postgres_service" src/
grep -r "mysql_service" src/
```

如果找到引用，更新或删除。

### 3.5 阶段 3 验收

**检查清单**：

- [ ] 旧服务文件已删除
- [ ] 模块导出已更新
- [ ] 没有遗留引用
- [ ] 代码编译通过
- [ ] 所有测试通过
- [ ] 代码已提交到 git

**提交代码**：

```bash
git add .
git commit -m "chore: remove old database service files (phase 3)

- Remove postgres_service.rs
- Remove mysql_service.rs
- Update module exports
- All tests passing"
```

---

## 🧪 测试策略

### 单元测试

每个模块都应该有单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_creation() {
        let factory = DatabaseServiceFactory::new();
        assert!(factory.is_supported(&DatabaseType::PostgreSQL));
    }
}
```

### 集成测试

确保现有的集成测试仍然通过：

```bash
cd src-tauri

# 运行所有测试
cargo test

# 运行集成测试
cargo test --test integration_test
```

### 手动功能测试

创建测试清单：

| 功能 | PostgreSQL | MySQL | 备注 |
|------|-----------|-------|------|
| 连接数据库 | ⬜ | ⬜ |  |
| 测试连接 | ⬜ | ⬜ |  |
| 执行 SELECT | ⬜ | ⬜ |  |
| 执行 JOIN | ⬜ | ⬜ |  |
| 执行聚合 | ⬜ | ⬜ |  |
| 刷新元数据 | ⬜ | ⬜ |  |
| 错误处理 | ⬜ | ⬜ |  |

### 性能测试

确保性能没有回退：

```bash
# 使用发布模式进行性能测试
cargo build --release
cargo test --release -- --nocapture
```

---

## 🔄 回滚计划

### 任何阶段都可以回滚

#### 回滚阶段 3

```bash
# 恢复旧服务文件
git checkout HEAD~1 -- src/services/postgres_service.rs
git checkout HEAD~1 -- src/services/mysql_service.rs
git checkout HEAD~1 -- src/services/mod.rs
```

#### 回滚阶段 2

```bash
# 回滚到阶段 1 的完成状态
git reset --hard <phase-1-commit-hash>
```

#### 回滚阶段 1

```bash
# 回滚到迁移前
git reset --hard pre-refactor-backup
```

#### 完全回滚

```bash
# 删除特性分支
git checkout main
git branch -D feature/db-architecture-refactor

# 或者保留分支但回到初始状态
git checkout pre-refactor-backup
git checkout -b feature/db-architecture-refactor
```

---

## ❓ 常见问题

### Q1: 编译错误 "use of undeclared type DatabaseService"

**原因**：模块导入路径不正确

**解决**：

```rust
// 在文件顶部添加
use crate::services::database::{DatabaseService, *};
```

### Q2: 运行时错误 "Factory not initialized"

**原因**：忘记在 main.rs 中初始化工厂

**解决**：

```rust
// src/main.rs
fn main() {
    services::database::init_global_factory();
    // ...
}
```

### Q3: 如何添加对新数据库的支持？

参考 [architecture-design.md](./architecture-design.md) 中的"添加新数据库指南"部分。

简单来说：
1. 添加枚举变体
2. 创建服务实现
3. 在工厂中注册
4. 完成！

不需要修改任何命令处理器。

### Q4: 旧的和新的可以共存吗？

可以！阶段 1 的目标就是创建非破坏性的新架构。可以同时保留旧服务和新服务。

### Q5: 如何验证迁移没有破坏功能？

1. 运行所有现有测试
2. 进行手动功能测试
3. 对比迁移前后的查询结果
4. 检查性能指标

---

## 📚 相关文档

- [架构设计文档](./architecture-design.md)
- [代码示例](./examples/)
- [验证清单](./validation.md)

---

**文档版本**：1.0.0
**最后更新**：2026-01-26
