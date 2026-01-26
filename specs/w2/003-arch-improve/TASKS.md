# 数据库服务架构重构任务清单

本文档包含数据库服务架构重构的所有任务，按照三个阶段组织。

## 📊 任务概览

| 阶段 | 任务数 | 预计时间 | 状态 |
|------|-------|---------|------|
| 阶段 1：创建新架构 | 6 | 2-3 天 | ⏸️ 待开始 |
| 阶段 2：迁移命令 | 5 | 1-2 天 | ⏸️ 待开始 |
| 阶段 3：清理代码 | 3 | 0.5 天 | ⏸️ 待开始 |
| **总计** | **14** | **4-6 天** | - |

---

## 🏗️ 阶段 1：创建新架构（非破坏性）

### ✅ 任务 1：创建服务模块目录结构

**状态**：⏸️ 待开始

**文件**：
- `src-tauri/src/services/database/mod.rs`
- `src-tauri/src/services/database/trait.rs`
- `src-tauri/src/services/database/factory.rs`
- `src-tauri/src/services/database/postgres_impl.rs`
- `src-tauri/src/services/database/mysql_impl.rs`

**命令**：
```bash
cd src-tauri/src/services
mkdir -p database
touch database/{mod.rs,trait.rs,factory.rs,postgres_impl.rs,mysql_impl.rs}
```

**验收标准**：
- [ ] 所有文件已创建
- [ ] 目录结构正确

---

### ✅ 任务 2：定义 DatabaseService trait

**状态**：⏸️ 待开始

**文件**：`src-tauri/src/services/database/trait.rs`

**参考**：`spec/w2/003-arch-improve/examples/trait.rs`

**内容**：
- 定义 `DbConnection` 枚举
- 定义 `DbRow` 结构体
- 定义 `QueryExecutionResult` 结构体
- 定义 `QueryParam` 枚举
- 定义 `SqlDialect` 结构体
- 定义 `LimitSyntax` 和 `ParameterSyntax` 枚举
- 定义 `DatabaseService` trait（所有方法）
- 添加辅助函数 `convert_row_to_json_default()`
- 添加单元测试

**验收标准**：
- [ ] 代码编译通过
- [ ] 所有单元测试通过
- [ ] 没有编译警告

**测试命令**：
```bash
cargo build --lib
cargo test --lib services::database::trait
```

---

### ✅ 任务 3：实现 DatabaseServiceFactory

**状态**：⏸️ 待开始

**文件**：`src-tauri/src/services/database/factory.rs`

**参考**：`spec/w2/003-arch-improve/examples/factory.rs`

**内容**：
- 定义 `DatabaseServiceFactory` 结构体
- 实现 `new()` 方法（注册 PostgreSQL 和 MySQL）
- 实现 `register_service()` 方法
- 实现 `get_service()` 方法
- 实现 `supported_types()` 方法
- 实现 `is_supported()` 方法
- 定义全局工厂实例
- 实现 `init_global_factory()` 函数
- 实现 `get_global_factory()` 函数
- 添加单元测试

**验收标准**：
- [ ] 代码编译通过（可能有服务未实现的错误，这是预期的）
- [ ] 工厂模式逻辑正确
- [ ] 单元测试通过

**测试命令**：
```bash
cargo build --lib
cargo test --lib services::database::factory
```

---

### ✅ 任务 4：实现 PostgreSQL 服务

**状态**：⏸️ 待开始

**文件**：`src-tauri/src/services/database/postgres_impl.rs`

**参考**：`spec/w2/003-arch-improve/examples/postgres_impl.rs`

**内容**：
- 定义 `PostgresService` 结构体
- 实现 `DatabaseService` trait：
  - `service_name()`
  - `connect()`
  - `test_connection()`
  - `execute_query()`
  - `extract_metadata()`
  - `convert_row_to_json()`
  - `get_sql_dialect()`
- 实现 `convert_pg_row_to_db_row()` 类型转换
- 实现 `extract_tables()` 和 `extract_views()`
- 定义 `POSTGRES_DIALECT` 常量
- 迁移现有 `postgres_service.rs` 的逻辑
- 添加单元测试

**类型转换规则**：
- UUID → String
- int4/int8 → Number
- float4/float8 → Number
- bool → Bool
- timestamp/timestamptz → String (ISO 8601)
- date → String (YYYY-MM-DD)
- json/jsonb → JSON 对象

**验收标准**：
- [ ] 代码编译通过
- [ ] 所有单元测试通过
- [ ] PostgreSQL 集成测试通过

**测试命令**：
```bash
cargo build --lib
cargo test --lib services::database::postgres_impl
cargo test --test integration_test
```

---

### ✅ 任务 5：实现 MySQL 服务

**状态**：⏸️ 待开始

**文件**：`src-tauri/src/services/database/mysql_impl.rs`

**参考**：`spec/w2/003-arch-improve/examples/mysql_impl.rs`

**内容**：
- 定义 `MySqlService` 结构体
- 实现 `DatabaseService` trait（所有方法）
- 实现 `convert_mysql_row_to_db_row()` 类型转换
- 实现 `extract_tables()` 和 `extract_views()`
- 定义 `MYSQL_DIALECT` 常量
- 迁移现有 `mysql_service.rs` 的逻辑
- 添加单元测试

**类型转换规则**：
- MYSQL_TYPE_TINY (bool) → Bool
- Value::Int → Number
- Value::UInt → Number (或 String 如果太大)
- Value::Float/Double → Number
- Value::Bytes → String (UTF-8)
- Value::NULL → Null

**验收标准**：
- [ ] 代码编译通过
- [ ] 所有单元测试通过
- [ ] MySQL 集成测试通过

**测试命令**：
```bash
cargo build --lib
cargo test --lib services::database::mysql_impl
cargo test --test integration_test
```

---

### ✅ 任务 6：更新模块导出和验证

**状态**：⏸️ 待开始

**文件**：
- `src-tauri/src/services/database/mod.rs`
- `src-tauri/src/services/mod.rs`

**内容**：

**database/mod.rs**:
```rust
pub mod factory;
pub mod trait;
pub mod postgres_impl;
pub mod mysql_impl;

pub use factory::*;
pub use trait::*;
```

**services/mod.rs**:
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

# 运行 clippy
cargo clippy -- -D warnings

# 检查格式
cargo fmt -- --check
```

**验收标准**：
- [ ] 所有代码编译通过
- [ ] 所有测试通过
- [ ] 没有 clippy 警告
- [ ] 代码格式正确
- [ ] 创建 git commit

**提交命令**：
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

### ✅ 任务 7：在 main.rs 中初始化全局 Factory

**状态**：⏸️ 待开始

**文件**：`src-tauri/src/main.rs`

**内容**：
在 `main()` 函数开始处添加：
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

**验收标准**：
- [ ] 代码编译通过
- [ ] 应用可以正常启动

---

### ✅ 任务 8：重构 database.rs 命令

**状态**：⏸️ 待开始

**文件**：`src-tauri/src/commands/database.rs`

**需要重构的函数**：
- `test_connection()`
- `add_database()`（如果有连接测试）

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

**验收标准**：
- [ ] 代码编译通过
- [ ] 手动测试：PostgreSQL 连接成功
- [ ] 手动测试：MySQL 连接成功
- [ ] 错误处理正确

---

### ✅ 任务 9：重构 query.rs 命令

**状态**：⏸️ 待开始

**文件**：`src-tauri/src/commands/query.rs`

**需要重构的函数**：
- `run_sql_query()`

**需要删除的函数**：
- `convert_postgres_rows()`
- `convert_mysql_rows()`

**重构要点**：
1. 使用 `factory.get_service()` 获取服务
2. 使用 `service.connect()` 连接
3. 使用 `service.execute_query()` 执行
4. 使用 `service.convert_row_to_json()` 转换结果
5. 删除数据库特定的 match 语句

**验收标准**：
- [ ] 代码编译通过
- [ ] 手动测试各种查询场景：
  - [ ] 简单 SELECT
  - [ ] JOIN 查询
  - [ ] 聚合查询
  - [ ] WHERE 条件
  - [ ] ORDER BY
  - [ ] LIMIT
  - [ ] NULL 值处理
  - [ ] 类型转换
  - [ ] 错误查询
- [ ] PostgreSQL 和 MySQL 都测试通过

---

### ✅ 任务 10：重构 metadata.rs 命令

**状态**：⏸️ 待开始

**文件**：`src-tauri/src/commands/metadata.rs`

**需要重构的函数**：
- `refresh_metadata()`
- `get_database_metadata()`（如果有逻辑需要更新）

**重构要点**：
1. 使用 `factory.get_service()` 获取服务
2. 使用 `service.connect()` 连接
3. 使用 `service.extract_metadata()` 提取元数据
4. 删除数据库特定的 match 语句
5. 保持缓存逻辑不变

**验收标准**：
- [ ] 代码编译通过
- [ ] 手动测试：PostgreSQL 元数据刷新成功
- [ ] 手动测试：MySQL 元数据刷新成功
- [ ] 表信息正确
- [ ] 列信息正确
- [ ] 视图信息正确

---

### ✅ 任务 11：验证并提交代码

**状态**：⏸️ 待开始

**完整验证流程**：

```bash
# 1. 编译检查
cargo build

# 2. 运行所有单元测试
cargo test

# 3. 运行所有集成测试
cargo test --test '*'

# 4. 代码质量检查
cargo clippy -- -D warnings

# 5. 格式检查
cargo fmt -- --check

# 6. 构建发布版本
cargo build --release
```

**功能测试清单**：
- [ ] PostgreSQL 连接测试
- [ ] MySQL 连接测试
- [ ] PostgreSQL 查询执行（各种类型）
- [ ] MySQL 查询执行（各种类型）
- [ ] PostgreSQL 元数据提取
- [ ] MySQL 元数据提取
- [ ] 错误处理验证
- [ ] 性能测试（无回退）

**验收标准**：
- [ ] 所有测试通过
- [ ] 没有编译警告
- [ ] 代码质量合格
- [ ] 功能测试通过
- [ ] 创建 git commit

**提交命令**：
```bash
git add .
git commit -m "refactor: migrate commands to use factory pattern (phase 2)

- Update database commands to use factory pattern
- Update query commands to use factory pattern
- Update metadata commands to use factory pattern
- Remove database-specific match statements
- Remove duplicate type conversion functions
- All tests passing"
```

---

## 🧹 阶段 3：清理旧代码

### ✅ 任务 12：删除旧服务文件

**状态**：⏸️ 待开始

**需要删除的文件**：
- `src-tauri/src/services/postgres_service.rs`
- `src-tauri/src/services/mysql_service.rs`

**命令**：
```bash
cd src-tauri/src/services
rm postgres_service.rs mysql_service.rs
```

**需要更新的文件**：
- `src-tauri/src/services/mod.rs`

**删除的导出**：
```rust
// 删除这些行
pub mod postgres_service;
pub mod mysql_service;
pub use postgres_service::*;
pub use mysql_service::*;
```

**验收标准**：
- [ ] 旧文件已删除
- [ ] 模块导出已更新
- [ ] 项目可以编译

**验证命令**：
```bash
# 检查文件已删除
ls -la postgres_service.rs mysql_service.rs  # 应该失败

# 编译检查
cargo build
```

---

### ✅ 任务 13：清理遗留引用和依赖

**状态**：⏸️ 待开始

**检查遗留引用**：
```bash
# 搜索旧服务引用
grep -r "postgres_service" src/
grep -r "mysql_service" src/
```

**如果找到引用**：
- 更新引用使用新的工厂模式
- 或删除不需要的引用

**可选：清理未使用的依赖**：
```bash
# 安装 udeps
cargo install cargo-udeps

# 检查未使用的依赖
cargo +nightly udeps

# 如果发现未使用的依赖，从 Cargo.toml 删除
```

**验收标准**：
- [ ] 没有遗留的旧服务引用
- [ ] 项目编译通过
- [ ] 未使用的依赖已清理（可选）

---

### ✅ 任务 14：最终验证和提交

**状态**：⏸️ 待开始

**完整重新编译**：
```bash
# 清理构建缓存
cargo clean

# 完整重新编译
cargo build
```

**完整测试套件**：
```bash
# 所有单元测试
cargo test

# 所有集成测试
cargo test --test '*'

# 代码质量
cargo clippy -- -D warnings

# 格式检查
cargo fmt -- --check
```

**最终验收清单**：

**代码质量**：
- [ ] 代码符合 Rust 命名规范
- [ ] 所有公共 API 有文档注释
- [ ] 错误处理完整
- [ ] 没有 TODO 或 FIXME 留下

**功能测试**：
- [ ] PostgreSQL 连接
- [ ] MySQL 连接
- [ ] PostgreSQL 查询（各种类型）
- [ ] MySQL 查询（各种类型）
- [ ] PostgreSQL 元数据
- [ ] MySQL 元数据
- [ ] 错误处理

**性能测试**：
- [ ] 连接建立时间 < 1 秒
- [ ] 简单查询 < 100ms
- [ ] 复杂查询 < 1 秒
- [ ] 元数据提取 < 5 秒
- [ ] 内存使用正常

**SOLID 原则验证**：
- [ ] 单一职责：每个服务只负责一个数据库
- [ ] 开闭原则：可以添加新数据库而无需修改现有代码
- [ ] 里氏替换：所有服务可以互换使用
- [ ] 接口隔离：Trait 方法精简
- [ ] 依赖倒置：命令层依赖抽象

**扩展性验证**：
- [ ] 添加新数据库只需要 4 个步骤
- [ ] 不需要修改命令处理器
- [ ] 不需要修改其他数据库服务

**Git 提交**：
```bash
git add .
git commit -m "chore: remove old database service files (phase 3)

- Remove postgres_service.rs
- Remove mysql_service.rs
- Update module exports
- Clean up legacy references
- All tests passing
- Ready to merge"

# 可选：打标签
git tag v2.0.0-arch-refactor
git push origin v2.0.0-arch-refactor
```

**准备合并**：
```bash
# 切换到主分支
git checkout main
git pull origin main

# 合并特性分支
git merge feature/db-architecture-refactor

# 推送
git push origin main
```

**验收标准**：
- [ ] 所有验证通过
- [ ] 代码已提交
- [ ] 合并到主分支
- [ ] 更新版本号
- [ ] 创建变更日志

---

## 📊 进度追踪

### 阶段完成度

| 阶段 | 完成任务 | 总任务 | 进度 |
|------|---------|--------|------|
| 阶段 1：创建新架构 | 0/6 | 6 | 0% |
| 阶段 2：迁移命令 | 0/5 | 5 | 0% |
| 阶段 3：清理代码 | 0/3 | 3 | 0% |
| **总计** | **0/14** | **14** | **0%** |

### 预计时间表

| 日期 | 任务 | 预计时间 | 状态 |
|------|------|---------|------|
| Day 1-2 | 任务 1-3：目录结构、Trait、Factory | 1-2 天 | ⏸️ |
| Day 2-3 | 任务 4-5：PostgreSQL 和 MySQL 服务 | 1-2 天 | ⏸️ |
| Day 3 | 任务 6：模块导出和验证 | 0.5 天 | ⏸️ |
| Day 4 | 任务 7-8：初始化和 database.rs | 0.5-1 天 | ⏸️ |
| Day 4-5 | 任务 9-10：query.rs 和 metadata.rs | 0.5-1 天 | ⏸️ |
| Day 5 | 任务 11：验证阶段 2 | 0.5 天 | ⏸️ |
| Day 6 | 任务 12-14：清理和最终验证 | 0.5-1 天 | ⏸️ |

---

## 🚀 快速开始

### 第一步：准备环境

```bash
# 创建特性分支
git checkout -b feature/db-architecture-refactor

# 创建备份标签
git tag pre-refactor-backup

# 确保项目可以编译
cd w2-db-query/src-tauri
cargo build
cargo test
```

### 第二步：开始任务 1

```bash
cd src-tauri/src/services
mkdir -p database
touch database/{mod.rs,trait.rs,factory.rs,postgres_impl.rs,mysql_impl.rs}
```

### 第三步：按照任务清单依次完成

参考本文档中的详细说明，按照任务 1 → 任务 2 → ... → 任务 14 的顺序完成。

### 第四步：每个阶段后验证

- 阶段 1 完成后：运行所有测试，创建 commit
- 阶段 2 完成后：完整功能测试，创建 commit
- 阶段 3 完成后：最终验收，合并到主分支

---

## 📚 相关文档

- [README.md](./README.md) - 规范概述
- [architecture-design.md](./architecture-design.md) - 详细架构设计
- [migration-guide.md](./migration-guide.md) - 迁移指南
- [validation.md](./validation.md) - 验证清单
- [examples/](./examples/) - 代码示例

---

## 🆘 遇到问题？

### 常见问题

查看 [migration-guide.md](./migration-guide.md#常见问题) 部分。

### 回滚

如果任何阶段出现问题：

```bash
# 回滚到上一个 commit
git reset --hard HEAD~1

# 或回滚到备份点
git reset --hard pre-refactor-backup
```

---

**文档版本**：1.0.0
**创建日期**：2026-01-26
**状态**：准备开始
