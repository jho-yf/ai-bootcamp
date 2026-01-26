# 数据库服务架构重构验证清单

本文档提供完整的验证清单，确保架构重构的正确性和完整性。

## 📋 验证清单概览

- [前置准备](#前置准备)
- [阶段 1 验证](#阶段-1-验证创建新架构)
- [阶段 2 验证](#阶段-2-验证迁移命令处理器)
- [阶段 3 验证](#阶段-3-验证清理旧代码)
- [最终验收](#最终验收)

---

## 🔍 前置准备

### 环境检查

- [ ] Rust 版本 >= 1.70
  ```bash
  rustc --version
  ```

- [ ] 项目可以编译
  ```bash
  cd w2-db-query/src-tauri
  cargo build
  ```

- [ ] 所有现有测试通过
  ```bash
  cargo test
  ```

### 依赖检查

- [ ] `async-trait` 在 Cargo.toml 中
  ```toml
  [dependencies]
  async-trait = "0.1"
  ```

- [ ] 所有数据库驱动已安装
  - [ ] `tokio-postgres`
  - [ ] `mysql_async`
  - [ ] `rusqlite` (用于缓存)

### 代码备份

- [ ] 创建了特性分支
  ```bash
  git checkout -b feature/db-architecture-refactor
  ```

- [ ] 创建了备份标签
  ```bash
  git tag pre-refactor-backup
  ```

- [ ] 阅读了架构设计文档
- [ ] 理解了 SOLID 原则
- [ ] 熟悉了 trait 和 async trait

### 测试环境

- [ ] PostgreSQL 测试数据库可用
  ```bash
  psql -h localhost -U testuser -d testdb -c "SELECT 1;"
  ```

- [ ] MySQL 测试数据库可用
  ```bash
  mysql -h localhost -u testuser -p testdb -e "SELECT 1;"
  ```

- [ ] 环境变量已设置
  ```bash
  export TEST_PG_HOST=localhost
  export TEST_PG_PORT=5432
  # ... 其他变量
  ```

---

## ✅ 阶段 1 验证（创建新架构）

### 文件结构

- [ ] 创建了 `src/services/database/` 目录
- [ ] 创建了以下文件：
  - [ ] `trait.rs` - Trait 定义
  - [ ] `factory.rs` - 工厂实现
  - [ ] `postgres_impl.rs` - PostgreSQL 服务
  - [ ] `mysql_impl.rs` - MySQL 服务
  - [ ] `mod.rs` - 模块导出

### Trait 定义验证

**文件**：`src/services/database/trait.rs`

- [ ] 定义了 `DbConnection` 枚举
- [ ] 定义了 `DbRow` 结构体
- [ ] 定义了 `QueryExecutionResult` 结构体
- [ ] 定义了 `QueryParam` 枚举
- [ ] 定义了 `SqlDialect` 结构体
- [ ] 定义了 `DatabaseService` trait

**编译检查**：

```bash
# 应该编译通过
cargo build --lib

# 运行单元测试
cargo test --lib services::database::trait
```

- [ ] 编译通过，无错误
- [ ] 编译通过，无警告
- [ ] 所有单元测试通过

### Factory 实现验证

**文件**：`src/services/database/factory.rs`

- [ ] 定义了 `DatabaseServiceFactory` 结构体
- [ ] 实现了 `new()` 方法
- [ ] 实现了 `register_service()` 方法
- [ ] 实现了 `get_service()` 方法
- [ ] 实现了 `supported_types()` 方法
- [ ] 定义了全局工厂实例
- [ ] 实现了 `init_global_factory()` 函数
- [ ] 实现了 `get_global_factory()` 函数

**编译检查**：

```bash
# 编译（可能会有错误，因为服务还没实现）
cargo build --lib 2>&1 | grep -A5 "error\|warning"
```

- [ ] 预期的错误是因为服务还没实现
- [ ] 没有其他意外错误

### PostgreSQL 服务验证

**文件**：`src/services/database/postgres_impl.rs`

- [ ] 定义了 `PostgresService` 结构体
- [ ] 实现了 `DatabaseService` trait
- [ ] 实现了 `service_name()` 方法
- [ ] 实现了 `connect()` 方法
- [ ] 实现了 `test_connection()` 方法
- [ ] 实现了 `execute_query()` 方法
- [ ] 实现了 `extract_metadata()` 方法
- [ ] 实现了 `convert_row_to_json()` 方法
- [ ] 实现了 `get_sql_dialect()` 方法
- [ ] 定义了 `POSTGRES_DIALECT` 常量

**编译检查**：

```bash
cargo build --lib
```

- [ ] 编译通过，无错误
- [ ] 编译通过，无警告

**单元测试**：

```bash
cargo test --lib services::database::postgres_impl
```

- [ ] `test_service_name` 通过
- [ ] `test_sql_dialect` 通过
- [ ] 所有其他测试通过

**集成测试**：

```bash
# 设置环境变量
export TEST_PG_HOST=localhost
export TEST_PG_PORT=5432
export TEST_PG_NAME=testdb
export TEST_PG_USER=testuser
export TEST_PG_PASS=testpass

# 运行集成测试
cargo test --test integration_test
```

- [ ] PostgreSQL 连接测试通过
- [ ] PostgreSQL 查询测试通过
- [ ] PostgreSQL 元数据提取测试通过

### MySQL 服务验证

**文件**：`src/services/database/mysql_impl.rs`

- [ ] 定义了 `MySqlService` 结构体
- [ ] 实现了 `DatabaseService` trait
- [ ] 实现了所有必需方法
- [ ] 定义了 `MYSQL_DIALECT` 常量

**编译检查**：

```bash
cargo build --lib
```

- [ ] 编译通过，无错误
- [ ] 编译通过，无警告

**单元测试**：

```bash
cargo test --lib services::database::mysql_impl
```

- [ ] 所有测试通过

**集成测试**：

```bash
# 设置环境变量
export TEST_MYSQL_HOST=localhost
export TEST_MYSQL_PORT=3306
export TEST_MYSQL_NAME=testdb
export TEST_MYSQL_USER=testuser
export TEST_MYSQL_PASS=testpass

# 运行集成测试
cargo test --test integration_test
```

- [ ] MySQL 连接测试通过
- [ ] MySQL 查询测试通过
- [ ] MySQL 元数据提取测试通过

### 模块导出验证

**文件**：`src/services/database/mod.rs`

```rust
pub mod factory;
pub mod trait;
pub mod postgres_impl;
pub mod mysql_impl;

pub use factory::*;
pub use trait::*;
```

- [ ] 所有模块正确导出

**文件**：`src/services/mod.rs`

```rust
pub mod database;
pub use database::*;
```

- [ ] database 模块已导出

**编译检查**：

```bash
cargo build
```

- [ ] 完整项目编译通过
- [ ] 没有编译错误
- [ ] 没有编译警告

### 阶段 1 功能测试

- [ ] 可以创建工厂实例
- [ ] 可以获取 PostgreSQL 服务
- [ ] 可以获取 MySQL 服务
- [ ] 尝试获取不存在的服务返回错误
- [ ] 所有现有测试仍然通过

### 代码质量

- [ ] 代码符合 Rust 命名规范
- [ ] 没有使用 `unsafe`（除了全局工厂）
- [ ] 没有使用 `unwrap()` 或 `expect()`（除非在测试中）
- [ ] 所有公共函数有文档注释
- [ ] 错误处理完整

### Git 提交

```bash
git status
git add .
git commit -m "feat: add database service trait and factory (phase 1)

- Define DatabaseService trait
- Implement factory pattern for service creation
- Implement PostgreSQL and MySQL services
- All tests passing
- No breaking changes to existing code"
```

- [ ] 代码已提交
- [ ] 提交信息清晰
- [ ] 没有遗留的未提交文件

---

## ✅ 阶段 2 验证（迁移命令处理器）

### main.rs 初始化

**文件**：`src/main.rs`

- [ ] 在 `main()` 函数开始处调用了 `init_global_factory()`
- [ ] 调用位置在其他初始化之前

**编译检查**：

```bash
cargo build
```

- [ ] 编译通过

### database.rs 命令验证

**文件**：`src/commands/database.rs`

- [ ] `test_connection()` 使用工厂模式
- [ ] 删除了数据库特定的 match 语句
- [ ] 错误处理正确

**编译检查**：

```bash
cargo build
```

- [ ] 编译通过

**手动测试**：

```bash
cargo run
```

- [ ] 测试 PostgreSQL 连接成功
- [ ] 测试 MySQL 连接成功
- [ ] 错误处理正确显示

### query.rs 命令验证

**文件**：`src/commands/query.rs`

- [ ] `run_sql_query()` 使用工厂模式
- [ ] 删除了数据库特定的 match 语句
- [ ] 删除了 `convert_postgres_rows()` 函数
- [ ] 删除了 `convert_mysql_rows()` 函数
- [ ] 使用 `service.convert_row_to_json()` 统一转换
- [ ] DDL 检查仍然工作
- [ ] LIMIT 注入仍然工作
- [ ] 查询历史保存仍然工作

**编译检查**：

```bash
cargo build
```

- [ ] 编译通过

**手动测试**：

| 测试场景 | PostgreSQL | MySQL | 通过 |
|---------|-----------|-------|------|
| 简单 SELECT | ⬜ | ⬜ |  |
| JOIN 查询 | ⬜ | ⬜ |  |
| 聚合查询 | ⬜ | ⬜ |  |
| WHERE 条件 | ⬜ | ⬜ |  |
| ORDER BY | ⬜ | ⬜ |  |
| LIMIT | ⬜ | ⬜ |  |
| NULL 值处理 | ⬜ | ⬜ |  |
| 类型转换 | ⬜ | ⬜ |  |
| 错误查询 | ⬜ | ⬜ |  |

### metadata.rs 命令验证

**文件**：`src/commands/metadata.rs`

- [ ] `refresh_metadata()` 使用工厂模式
- [ ] `get_database_metadata()` 工作正常
- [ ] 删除了数据库特定的 match 语句
- [ ] 缓存逻辑仍然工作

**编译检查**：

```bash
cargo build
```

- [ ] 编译通过

**手动测试**：

```bash
cargo run
```

- [ ] PostgreSQL 元数据刷新成功
- [ ] MySQL 元数据刷新成功
- [ ] 表信息正确
- [ ] 列信息正确
- [ ] 视图信息正确

### ai.rs 命令验证

**文件**：`src/commands/ai.rs`（如果有 AI 功能）

- [ ] AI 生成 SQL 使用正确的方言
- [ ] NL 查询功能正常

### 阶段 2 完整测试

**所有单元测试**：

```bash
cargo test
```

- [ ] 所有单元测试通过

**所有集成测试**：

```bash
cargo test --test '*'
```

- [ ] 所有集成测试通过

**性能测试**：

```bash
# 构建发布版本
cargo build --release

# 测试查询性能（应该与之前相当或更好）
./target/release/app-name
```

- [ ] 查询性能没有回退
- [ ] 连接建立时间正常
- [ ] 内存使用正常

### 代码质量检查

```bash
# 运行 clippy
cargo clippy -- -D warnings
```

- [ ] 没有 clippy 警告

```bash
# 运行 fmt 检查
cargo fmt -- --check
```

- [ ] 代码格式正确

### Git 提交

```bash
git status
git add .
git commit -m "refactor: migrate commands to use factory pattern (phase 2)

- Update database commands to use factory pattern
- Update query commands to use factory pattern
- Update metadata commands to use factory pattern
- Remove database-specific match statements
- Remove duplicate type conversion functions
- All tests passing"
```

- [ ] 代码已提交
- [ ] 提交信息清晰

---

## ✅ 阶段 3 验证（清理旧代码）

### 删除旧文件

```bash
cd src-tauri/src/services
ls -la postgres_service.rs mysql_service.rs
```

- [ ] 确认旧文件存在
- [ ] 删除 `postgres_service.rs`
- [ ] 删除 `mysql_service.rs`

**验证删除**：

```bash
ls -la postgres_service.rs mysql_service.rs
# 应该显示 "No such file or directory"
```

### 更新模块导出

**文件**：`src/services/mod.rs`

- [ ] 删除了 `pub mod postgres_service;`
- [ ] 删除了 `pub mod mysql_service;`
- [ ] 删除了 `pub use postgres_service::*;`
- [ ] 删除了 `pub use mysql_service::*;`

**编译检查**：

```bash
cargo build
```

- [ ] 编译通过
- [ ] 没有遗留引用

### 检查遗留引用

```bash
grep -r "postgres_service" src/ || echo "No references found"
grep -r "mysql_service" src/ || echo "No references found"
```

- [ ] 没有遗留引用

### 依赖清理（可选）

```bash
cargo install cargo-udeps
cargo +nightly udeps
```

- [ ] 检查未使用的依赖
- [ ] 删除未使用的依赖（如果有）

### 最终编译和测试

```bash
# 清理构建
cargo clean

# 完整重新编译
cargo build

# 运行所有测试
cargo test

# 运行 clippy
cargo clippy -- -D warnings

# 检查格式
cargo fmt -- --check
```

- [ ] 编译通过
- [ ] 所有测试通过
- [ ] 没有 clippy 警告
- [ ] 代码格式正确

### Git 提交

```bash
git status
git add .
git commit -m "chore: remove old database service files (phase 3)

- Remove postgres_service.rs
- Remove mysql_service.rs
- Update module exports
- All tests passing"
```

- [ ] 代码已提交

---

## 🎯 最终验收

### 代码审查

- [ ] 所有代码符合项目规范
- [ ] 所有公共 API 有文档注释
- [ ] 错误处理完整
- [ ] 没有 `TODO` 或 `FIXME`（或者有明确的跟踪）
- [ ] 没有调试代码（`println!`, `dbg!` 等）

### 完整功能测试

| 功能 | 测试场景 | 状态 |
|------|---------|------|
| **连接管理** |  |  |
| | 创建 PostgreSQL 连接 | ⬜ |
| | 创建 MySQL 连接 | ⬜ |
| | 测试连接 | ⬜ |
| | 删除连接 | ⬜ |
| **查询执行** |  |  |
| | 简单 SELECT | ⬜ |
| | JOIN 查询 | ⬜ |
| | 聚合函数 | ⬜ |
| | 子查询 | ⬜ |
| | NULL 处理 | ⬜ |
| | 类型转换 | ⬜ |
| | 错误处理 | ⬜ |
| **元数据** |  |  |
| | 刷新元数据 | ⬜ |
| | 表信息 | ⬜ |
| | 列信息 | ⬜ |
| | 视图信息 | ⬜ |
| | 键信息 | ⬜ |
| **AI 功能** |  |  |
| | NL 转换 SQL | ⬜ |
| | 执行 NL 查询 | ⬜ |

### 性能基准

- [ ] 连接建立时间 < 1 秒
- [ ] 简单查询 < 100ms
- [ ] 复杂查询 < 1 秒
- [ ] 元数据提取 < 5 秒
- [ ] 内存使用正常

### SOLID 原则验证

- [ ] **单一职责**：每个服务只负责一个数据库
- [ ] **开闭原则**：可以添加新数据库而无需修改现有代码
- [ ] **里氏替换**：所有服务可以互换使用
- [ ] **接口隔离**：Trait 方法精简，没有不必要的方法
- [ ] **依赖倒置**：命令层依赖抽象，不依赖具体实现

### 扩展性验证

**验证添加新数据库的容易程度**：

- [ ] 添加新数据库只需要 4 个步骤
- [ ] 不需要修改命令处理器
- [ ] 不需要修改其他数据库服务
- [ ] Trait 定义清晰，易于实现

### 文档完整性

- [ ] README.md 已更新（如果有）
- [ ] API 文档完整
- [ ] 迁移指南清晰
- [ ] 代码注释充分

### 最终检查清单

- [ ] 所有测试通过
- [ ] 代码审查完成
- [ ] 功能测试完成
- [ ] 性能测试完成
- [ ] 文档更新完成
- [ ] 没有 TODO 留下（或有明确的跟踪）
- [ ] Git 历史清晰
- [ ] 准备好合并到主分支

### 合并准备

```bash
# 确保主分支最新
git checkout main
git pull origin main

# 合并特性分支
git merge feature/db-architecture-refactor

# 如果有冲突，解决后：
git add .
git commit -m "Merge feature/db-architecture-refactor into main"

# 推送到远程
git push origin main

# 可选：创建发布标签
git tag v2.0.0-arch-refactor
git push origin v2.0.0-arch-refactor
```

- [ ] 合并成功
- [ ] 推送到远程
- [ ] 更新版本号
- [ ] 创建变更日志

---

## 📊 验证总结

| 阶段 | 测试项 | 通过 | 备注 |
|------|-------|------|------|
| 前置准备 | 15 | ⬜ |  |
| 阶段 1 | 20 | ⬜ |  |
| 阶段 2 | 25 | ⬜ |  |
| 阶段 3 | 10 | ⬜ |  |
| 最终验收 | 30 | ⬜ |  |
| **总计** | **100** | ⬜ |  |

### 验收标准

- [ ] 至少 95% 的检查项通过
- [ ] 所有关键功能（连接、查询、元数据）正常工作
- [ ] 没有严重的性能回退
- [ ] 代码质量符合标准
- [ ] 文档完整

### 签字确认

- [ ] 开发者确认：________________  日期：______
- [ ] 代码审查确认：________________  日期：______
- [ ] QA 测试确认：__________________  日期：______

---

## 🐛 问题跟踪

记录验证过程中发现的问题：

| ID | 问题描述 | 严重性 | 状态 | 负责人 |
|----|---------|--------|------|--------|
| 1 |  |  |  |  |
| 2 |  |  |  |  |

---

**文档版本**：1.0.0
**最后更新**：2026-01-26
