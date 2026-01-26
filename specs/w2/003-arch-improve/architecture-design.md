# 数据库服务架构重构设计

## 📋 目录

1. [问题分析](#问题分析)
2. [设计目标](#设计目标)
3. [核心架构](#核心架构)
4. [实现细节](#实现细节)
5. [迁移路径](#迁移路径)
6. [添加新数据库指南](#添加新数据库指南)

---

## 🔍 问题分析

### 当前架构的问题

**当前实现**（违反开闭原则）：

```rust
// ❌ 每次添加新数据库都需要修改这些地方
pub async fn run_sql_query(request: RunQueryRequest) -> Result<QueryResult, String> {
    match connection.database_type {
        DatabaseType::PostgreSQL => {
            // PostgreSQL 特定代码
        }
        DatabaseType::MySQL => {
            // MySQL 特定代码
        }
        // 添加新数据库？需要修改这里！
    }
}
```

**需要修改的位置统计**：

| 文件 | 修改位置数 | 风险等级 |
|------|----------|---------|
| `models/database.rs` | 1 处（添加枚举） | 低 |
| `services/mod.rs` | 1 处（导出新模块） | 低 |
| `commands/database.rs` | 3 处 match 语句 | 高 |
| `commands/query.rs` | 2 处 match 语句 | 高 |
| `commands/metadata.rs` | 1 处 match 语句 | 高 |
| **总计** | **8 处** | **高风险** |

**违反的原则**：

1. ❌ **开闭原则 (OCP)**：对修改不封闭，添加新数据库需要修改现有代码
2. ❌ **单一职责原则 (SRP)**：命令处理器包含数据库特定逻辑
3. ❌ **依赖倒置原则 (DIP)**：高层模块依赖低层实现，而非抽象
4. ❌ **代码重复**：类型转换逻辑在每个命令中重复

---

## 🎯 设计目标

### 核心目标

**当添加新数据库时，开发者应该只需要：**

1. ✅ 在 `DatabaseType` 枚举中添加一个变体（1 行代码）
2. ✅ 创建一个服务文件实现 `DatabaseService` trait（约 300-500 行）
3. ✅ 在 Factory 中注册服务（3 行代码）
4. ✅ 更新模块导出（1 行代码）

**不应该需要修改：**
- ❌ 命令处理器（`commands/*.rs`）
- ❌ 其他数据库服务
- ❌ 类型转换逻辑
- ❌ 错误处理代码

### SOLID 原则遵循

| 原则 | 实现 |
|------|------|
| **S** - 单一职责 | 每个服务只负责一个数据库类型 |
| **O** - 开闭原则 | 通过 Trait 实现扩展，无需修改现有代码 |
| **L** - 里氏替换 | 任何 DatabaseService 实现都可替换 |
| **I** - 接口隔离 | Trait 方法精简，不强迫实现不需要的方法 |
| **D** - 依赖倒置 | 命令层依赖 DatabaseService 抽象，而非具体实现 |

---

## 🏗️ 核心架构

### 架构图

```
┌─────────────────────────────────────────────────────────────┐
│                      Tauri Commands Layer                    │
│  (commands/database.rs, commands/query.rs, etc.)             │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ depends on
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   DatabaseServiceFactory                     │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  HashMap<DatabaseType, Arc<dyn DatabaseService>>    │    │
│  └─────────────────────────────────────────────────────┘    │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ creates
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                  DatabaseService Trait                       │
│  - connect()                                                │
│  - test_connection()                                        │
│  - execute_query()                                          │
│  - extract_metadata()                                       │
│  - convert_row_to_json()                                    │
│  - get_sql_dialect()                                        │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ implemented by
                         ▼
        ┌────────────────┼────────────────┐
        │                │                │
        ▼                ▼                ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│  PostgreSQL  │ │    MySQL     │ │   SQLite     │
│   Service    │ │   Service    │ │   Service    │
│              │ │              │ │    (Future)   │
└──────────────┘ └──────────────┘ └──────────────┘
```

### 核心组件

#### 1. DatabaseService Trait

```rust
use async_trait::async_trait;

/// 数据库连接抽象
pub enum DbConnection {
    PostgreSQL(tokio_postgres::Client),
    MySQL(mysql_async::Pool),
    // 未来扩展：SQLite(rusqlite::Connection), 等
}

/// 通用行表示
pub struct DbRow {
    pub columns: Vec<String>,
    pub values: Vec<serde_json::Value>,
}

/// 查询执行结果
pub struct QueryExecutionResult {
    pub rows: Vec<DbRow>,
    pub exec_time_ms: u64,
}

/// 查询参数（数据库无关）
#[derive(Debug, Clone)]
pub enum QueryParam {
    Null,
    Bool(bool),
    I32(i32),
    I64(i64),
    F64(f64),
    String(String),
    Bytes(Vec<u8>),
}

/// 核心数据库服务 Trait
#[async_trait]
pub trait DatabaseService: Send + Sync {
    /// 服务名称（用于日志/调试）
    fn service_name(&self) -> &'static str;

    /// 创建数据库连接
    async fn connect(
        &self,
        host: &str,
        port: u16,
        database_name: &str,
        user: &str,
        password: &str,
    ) -> Result<DbConnection, AppError>;

    /// 测试连接
    async fn test_connection(
        &self,
        host: &str,
        port: u16,
        database_name: &str,
        user: &str,
        password: &str,
    ) -> Result<bool, AppError>;

    /// 执行 SQL 查询
    async fn execute_query(
        &self,
        connection: &DbConnection,
        sql: &str,
        params: &[QueryParam],
    ) -> Result<QueryExecutionResult, AppError>;

    /// 提取数据库元数据
    async fn extract_metadata(
        &self,
        connection: &DbConnection,
        connection_id: &str,
    ) -> Result<DatabaseMetadata, AppError>;

    /// 转换行为 JSON
    fn convert_row_to_json(
        &self,
        row: &DbRow,
        columns: &[String],
    ) -> Result<std::collections::HashMap<String, serde_json::Value>, AppError>;

    /// 获取 SQL 方言信息
    fn get_sql_dialect(&self) -> &SqlDialect;
}
```

#### 2. SQL 方言抽象

```rust
/// SQL 方言信息（用于代码生成）
#[derive(Debug, Clone)]
pub struct SqlDialect {
    pub name: &'static str,
    pub string_quote: char,           // 字符串引号：'
    pub identifier_quote: char,        // 标识符引号：" 或 `
    pub supports_limit: bool,
    pub limit_syntax: LimitSyntax,
    pub parameter_syntax: ParameterSyntax,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LimitSyntax {
    Clause,      // LIMIT n (PostgreSQL, MySQL, SQLite)
    Top,         // SELECT TOP n (SQL Server)
    FetchFirst,  // FETCH FIRST n ROWS ONLY (Oracle)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterSyntax {
    DollarNumeric,   // $1, $2 (PostgreSQL)
    QuestionMark,    // ? (MySQL, SQLite)
    Named,           // :name (Oracle, SQLite named)
}
```

#### 3. Factory 模式

```rust
use std::sync::Arc;
use std::collections::HashMap;

/// 数据库服务工厂
pub struct DatabaseServiceFactory {
    services: HashMap<DatabaseType, Arc<dyn DatabaseService>>,
}

impl DatabaseServiceFactory {
    /// 创建新工厂并注册内置服务
    pub fn new() -> Self {
        let mut factory = Self {
            services: HashMap::new(),
        };

        // 注册内置数据库服务
        factory.register_service(
            DatabaseType::PostgreSQL,
            Arc::new(PostgresService::new()),
        );
        factory.register_service(
            DatabaseType::MySQL,
            Arc::new(MySqlService::new()),
        );

        factory
    }

    /// 注册新服务（扩展点）
    pub fn register_service(
        &mut self,
        db_type: DatabaseType,
        service: Arc<dyn DatabaseService>,
    ) {
        self.services.insert(db_type, service);
    }

    /// 获取服务
    pub fn get_service(&self, db_type: &DatabaseType) -> Result<Arc<dyn DatabaseService>, AppError> {
        self.services
            .get(db_type)
            .cloned()
            .ok_or_else(|| AppError::UnsupportedDatabase(format!("{:?}", db_type)))
    }

    /// 获取所有支持的类型
    pub fn supported_types(&self) -> Vec<DatabaseType> {
        self.services.keys().cloned().collect()
    }
}

impl Default for DatabaseServiceFactory {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## 💡 实现细节

### 目录结构

```
src-tauri/src/services/
├── database/
│   ├── mod.rs                 # 模块导出
│   ├── trait.rs               # DatabaseService trait 定义
│   ├── factory.rs             # 工厂模式实现
│   ├── postgres_impl.rs       # PostgreSQL 实现
│   ├── mysql_impl.rs          # MySQL 实现
│   └── sqlite_impl.rs         # SQLite 实现（未来）
├── cache_service.rs           #（保持不变）
├── ai_service.rs              #（保持不变）
├── query_parser.rs            #（保持不变）
└── mod.rs                     # 服务模块导出
```

### PostgreSQL 实现示例

```rust
use super::trait::*;
use async_trait::async_trait;
use tokio_postgres::{Client, NoTls};

pub struct PostgresService;

impl PostgresService {
    pub fn new() -> Self {
        Self
    }

    // 转换 PostgreSQL 行为通用 DbRow
    fn convert_pg_row(pg_row: &tokio_postgres::Row) -> DbRow {
        // 类型转换逻辑（从现有代码迁移）
        // ... 实现略
    }
}

#[async_trait]
impl DatabaseService for PostgresService {
    fn service_name(&self) -> &'static str {
        "PostgreSQL"
    }

    async fn connect(
        &self,
        host: &str,
        port: u16,
        database_name: &str,
        user: &str,
        password: &str,
    ) -> Result<DbConnection, AppError> {
        let connection_string = format!(
            "host={} port={} dbname={} user={} password={}",
            host, port, database_name, user, password
        );

        let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
            .await
            .map_err(|e| AppError::DatabaseConnection(format!("连接失败: {}", e)))?;

        // 启动连接处理器
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("PostgreSQL 连接错误: {}", e);
            }
        });

        Ok(DbConnection::PostgreSQL(client))
    }

    async fn test_connection(
        &self,
        host: &str,
        port: u16,
        database_name: &str,
        user: &str,
        password: &str,
    ) -> Result<bool, AppError> {
        let client = match self.connect(host, port, database_name, user, password).await? {
            DbConnection::PostgreSQL(c) => c,
            _ => return Err(AppError::DatabaseConnection("Invalid connection type".to_string())),
        };

        client
            .query_one("SELECT 1", &[])
            .await
            .map_err(|e| AppError::DatabaseConnection(format!("测试查询失败: {}", e)))?;

        Ok(true)
    }

    async fn execute_query(
        &self,
        connection: &DbConnection,
        sql: &str,
        _params: &[QueryParam],
    ) -> Result<QueryExecutionResult, AppError> {
        let client = match connection {
            DbConnection::PostgreSQL(c) => c,
            _ => return Err(AppError::DatabaseConnection("Invalid connection type".to_string())),
        };

        let start_time = std::time::Instant::now();

        let rows = client
            .query(sql, &[])
            .await
            .map_err(|e| AppError::QueryExecution(format!("查询执行失败: {}", e)))?;

        let exec_time_ms = start_time.elapsed().as_millis() as u64;

        let db_rows: Vec<DbRow> = rows.iter().map(Self::convert_pg_row).collect();

        Ok(QueryExecutionResult {
            rows: db_rows,
            exec_time_ms,
        })
    }

    async fn extract_metadata(
        &self,
        connection: &DbConnection,
        connection_id: &str,
    ) -> Result<DatabaseMetadata, AppError> {
        let client = match connection {
            DbConnection::PostgreSQL(c) => c,
            _ => return Err(AppError::DatabaseConnection("Invalid connection type".to_string())),
        };

        // 复用现有的元数据提取逻辑
        // ... 实现略
    }

    fn convert_row_to_json(
        &self,
        row: &DbRow,
        columns: &[String],
    ) -> Result<std::collections::HashMap<String, serde_json::Value>, AppError> {
        let mut map = std::collections::HashMap::new();
        for (i, col_name) in columns.iter().enumerate() {
            if let Some(value) = row.values.get(i) {
                map.insert(col_name.clone(), value.clone());
            }
        }
        Ok(map)
    }

    fn get_sql_dialect(&self) -> &SqlDialect {
        &POSTGRES_DIALECT
    }
}

const POSTGRES_DIALECT: SqlDialect = SqlDialect {
    name: "PostgreSQL",
    string_quote: '\'',
    identifier_quote: '"',
    supports_limit: true,
    limit_syntax: LimitSyntax::Clause,
    parameter_syntax: ParameterSyntax::DollarNumeric,
};
```

### MySQL 实现示例

```rust
use super::trait::*;
use async_trait::async_trait;
use mysql_async::{Pool, Row as MySqlRow};

pub struct MySqlService;

impl MySqlService {
    pub fn new() -> Self {
        Self
    }

    fn convert_mysql_row(mysql_row: &MySqlRow) -> DbRow {
        // 类型转换逻辑（从现有代码迁移）
        // ... 实现略
    }
}

#[async_trait]
impl DatabaseService for MySqlService {
    fn service_name(&self) -> &'static str {
        "MySQL"
    }

    async fn connect(
        &self,
        host: &str,
        port: u16,
        database_name: &str,
        user: &str,
        password: &str,
    ) -> Result<DbConnection, AppError> {
        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            user, password, host, port, database_name
        );

        let opts = mysql_async::Opts::from_url(&url)
            .map_err(|e| AppError::DatabaseConnection(format!("无效的连接URL: {}", e)))?;

        let pool = mysql_async::Pool::new(opts);

        // 测试连接
        let conn = pool
            .get_conn()
            .await
            .map_err(|e| AppError::DatabaseConnection(format!("连接失败: {}", e)))?;

        let _ = conn.disconnect().await;

        Ok(DbConnection::MySQL(pool))
    }

    async fn test_connection(
        &self,
        host: &str,
        port: u16,
        database_name: &str,
        user: &str,
        password: &str,
    ) -> Result<bool, AppError> {
        let pool = match self.connect(host, port, database_name, user, password).await? {
            DbConnection::MySQL(p) => p,
            _ => return Err(AppError::DatabaseConnection("Invalid connection type".to_string())),
        };

        let mut conn = pool
            .get_conn()
            .await
            .map_err(|e| AppError::DatabaseConnection(format!("获取连接失败: {}", e)))?;

        let _result: Option<u32> = conn
            .query_first("SELECT 1")
            .await
            .map_err(|e| AppError::DatabaseConnection(format!("测试查询失败: {}", e)))?;

        let _ = conn.disconnect().await;
        Ok(true)
    }

    async fn execute_query(
        &self,
        connection: &DbConnection,
        sql: &str,
        _params: &[QueryParam],
    ) -> Result<QueryExecutionResult, AppError> {
        let pool = match connection {
            DbConnection::MySQL(p) => p,
            _ => return Err(AppError::DatabaseConnection("Invalid connection type".to_string())),
        };

        let start_time = std::time::Instant::now();

        let mut conn = pool
            .get_conn()
            .await
            .map_err(|e| AppError::QueryExecution(format!("获取连接失败: {}", e)))?;

        let rows: Vec<MySqlRow> = conn
            .query(sql)
            .await
            .map_err(|e| AppError::QueryExecution(format!("查询执行失败: {}", e)))?;

        let exec_time_ms = start_time.elapsed().as_millis() as u64;

        let db_rows = rows.iter().map(Self::convert_mysql_row).collect();

        Ok(QueryExecutionResult {
            rows: db_rows,
            exec_time_ms,
        })
    }

    async fn extract_metadata(
        &self,
        connection: &DbConnection,
        connection_id: &str,
    ) -> Result<DatabaseMetadata, AppError> {
        let pool = match connection {
            DbConnection::MySQL(p) => p,
            _ => return Err(AppError::DatabaseConnection("Invalid connection type".to_string())),
        };

        // 复用现有的元数据提取逻辑
        // ... 实现略
    }

    fn convert_row_to_json(
        &self,
        row: &DbRow,
        columns: &[String],
    ) -> Result<std::collections::HashMap<String, serde_json::Value>, AppError> {
        let mut map = std::collections::HashMap::new();
        for (i, col_name) in columns.iter().enumerate() {
            if let Some(value) = row.values.get(i) {
                map.insert(col_name.clone(), value.clone());
            }
        }
        Ok(map)
    }

    fn get_sql_dialect(&self) -> &SqlDialect {
        &MYSQL_DIALECT
    }
}

const MYSQL_DIALECT: SqlDialect = SqlDialect {
    name: "MySQL",
    string_quote: '\'',
    identifier_quote: '`',
    supports_limit: true,
    limit_syntax: LimitSyntax::Clause,
    parameter_syntax: ParameterSyntax::QuestionMark,
};
```

### 重构后的命令处理器

**之前**（违反开闭原则）：

```rust
// ❌ 添加新数据库需要修改这里
#[tauri::command]
pub async fn run_sql_query(request: RunQueryRequest) -> Result<QueryResult, String> {
    // ... 前置检查略

    let result = match connection.database_type {
        DatabaseType::PostgreSQL => {
            let client = postgres_service::connect(/* ... */).await?;
            let (cols, rows, time) = postgres_service::execute_query(&client, &sql).await?;
            // 转换 PostgreSQL 行为 JSON
            convert_postgres_rows(/* ... */)
        }
        DatabaseType::MySQL => {
            let pool = mysql_service::connect(/* ... */).await?;
            let (cols, rows, time) = mysql_service::execute_query(&pool, &sql).await?;
            // 转换 MySQL 行为 JSON
            convert_mysql_rows(/* ... */)
        }
        // 添加新数据库？需要修改这里！
    };

    Ok(result)
}
```

**之后**（符合开闭原则）：

```rust
// ✅ 添加新数据库不需要修改这里！
#[tauri::command]
pub async fn run_sql_query(request: RunQueryRequest) -> Result<QueryResult, String> {
    // ... 前置检查略

    // === 零修改区域开始 ===
    // 通过工厂获取服务 - 无数据库特定代码！
    let factory = get_global_factory();
    let service = factory
        .get_service(&connection.database_type)
        .map_err(|e| e.to_string())?;

    // 使用抽象接口连接
    let db_connection = service
        .connect(/* ... */)
        .await
        .map_err(|e| format!("连接失败: {}", e))?;

    // 使用抽象接口执行查询
    let exec_result = service
        .execute_query(&db_connection, &parsed_sql, &[])
        .await
        .map_err(|e| e.to_string())?;

    // 使用通用转换
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
    // === 零修改区域结束 ===

    Ok(result)
}
```

---

## 🚀 迁移路径

### 阶段 1：创建新架构（非破坏性）

**时间估计**：2-3 天

**任务**：

1. **创建数据库服务模块**
   ```bash
   mkdir -p src-tauri/src/services/database
   touch src-tauri/src/services/database/{mod.rs,trait.rs,factory.rs}
   ```

2. **定义核心 Trait**（trait.rs）
3. **实现 Factory**（factory.rs）
4. **迁移 PostgreSQL 服务**（postgres_impl.rs）
5. **迁移 MySQL 服务**（mysql_impl.rs）

**测试**：
- 编译通过
- 现有测试通过

### 阶段 2：迁移命令处理器

**时间估计**：1-2 天

**任务**：

1. **初始化全局 Factory**（在 `main.rs` 中）
2. **更新 `commands/database.rs`** 使用工厂模式
3. **更新 `commands/query.rs`** 使用工厂模式
4. **更新 `commands/metadata.rs`** 使用工厂模式

**测试**：
- 所有集成测试通过
- 手动测试所有功能

### 阶段 3：清理

**时间估计**：0.5 天

**任务**：

1. 删除旧服务文件：
   ```bash
   rm src-tauri/src/services/postgres_service.rs
   rm src-tauri/src/services/mysql_service.rs
   ```

2. 更新 `services/mod.rs` 导出

3. 最终测试

### 回滚计划

如果迁移失败，可以：
1. 恢复 `commands/*.rs` 到旧版本
2. 删除 `services/database/` 目录
3. 恢复 `services/postgres_service.rs` 和 `mysql_service.rs`

---

## 📚 添加新数据库指南

### 示例：添加 SQLite 支持

**只需 4 个步骤，零修改现有代码！**

#### 步骤 1：添加枚举变体

**文件**：`src/models/database.rs`

```rust
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    SQLite,  // ✅ 添加这一行
}
```

#### 步骤 2：创建服务文件

**文件**：`src/services/database/sqlite_impl.rs`

```rust
use super::trait::*;
use async_trait::async_trait;
use rusqlite::Connection as SqliteConnection;

pub struct SqliteService;

impl SqliteService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl DatabaseService for SqliteService {
    fn service_name(&self) -> &'static str {
        "SQLite"
    }

    async fn connect(
        &self,
        _host: &str,
        _port: u16,
        database_name: &str,  // SQLite 使用文件路径
        _user: &str,
        _password: &str,
    ) -> Result<DbConnection, AppError> {
        let conn = SqliteConnection::open(database_name)
            .map_err(|e| AppError::DatabaseConnection(format!("SQLite 连接失败: {}", e)))?;

        Ok(DbConnection::SQLite(conn))
    }

    async fn test_connection(
        &self,
        host: &str,
        port: u16,
        database_name: &str,
        user: &str,
        password: &str,
    ) -> Result<bool, AppError> {
        self.connect(host, port, database_name, user, password).await?;
        Ok(true)
    }

    async fn execute_query(
        &self,
        connection: &DbConnection,
        sql: &str,
        _params: &[QueryParam],
    ) -> Result<QueryExecutionResult, AppError> {
        let conn = match connection {
            DbConnection::SQLite(c) => c,
            _ => return Err(AppError::DatabaseConnection("Invalid connection type".to_string())),
        };

        let start_time = std::time::Instant::now();

        // 执行查询并转换...
        // 实现略

        Ok(QueryExecutionResult {
            rows: vec![],
            exec_time_ms: start_time.elapsed().as_millis() as u64,
        })
    }

    async fn extract_metadata(
        &self,
        connection: &DbConnection,
        connection_id: &str,
    ) -> Result<DatabaseMetadata, AppError> {
        // 从 sqlite_master 提取元数据
        // 实现略
    }

    fn convert_row_to_json(
        &self,
        row: &DbRow,
        columns: &[String],
    ) -> Result<std::collections::HashMap<String, serde_json::Value>, AppError> {
        let mut map = std::collections::HashMap::new();
        for (i, col_name) in columns.iter().enumerate() {
            if let Some(value) = row.values.get(i) {
                map.insert(col_name.clone(), value.clone());
            }
        }
        Ok(map)
    }

    fn get_sql_dialect(&self) -> &SqlDialect {
        &SQLITE_DIALECT
    }
}

const SQLITE_DIALECT: SqlDialect = SqlDialect {
    name: "SQLite",
    string_quote: '\'',
    identifier_quote: '"',
    supports_limit: true,
    limit_syntax: LimitSyntax::Clause,
    parameter_syntax: ParameterSyntax::QuestionMark,
};
```

#### 步骤 3：注册到 Factory

**文件**：`src/services/database/factory.rs`

```rust
impl DatabaseServiceFactory {
    pub fn new() -> Self {
        let mut factory = Self {
            services: HashMap::new(),
        };

        factory.register_service(
            DatabaseType::PostgreSQL,
            Arc::new(PostgresService::new()),
        );
        factory.register_service(
            DatabaseType::MySQL,
            Arc::new(MySqlService::new()),
        );
        factory.register_service(  // ✅ 添加这些行
            DatabaseType::SQLite,
            Arc::new(SqliteService::new()),
        );

        factory
    }
}
```

#### 步骤 4：更新模块导出

**文件**：`src/services/database/mod.rs`

```rust
pub mod factory;
pub mod trait;
pub mod postgres_impl;
pub mod mysql_impl;
pub mod sqlite_impl;  // ✅ 添加这一行

pub use factory::*;
pub use trait::*;
```

#### 步骤 5：更新 DbConnection 枚举

**文件**：`src/services/database/trait.rs`

```rust
pub enum DbConnection {
    PostgreSQL(tokio_postgres::Client),
    MySQL(mysql_async::Pool),
    SQLite(rusqlite::Connection),  // ✅ 添加这一行
}
```

#### 步骤 6：添加依赖

**文件**：`Cargo.toml`

```toml
[dependencies]
# ... 现有依赖
rusqlite = "0.38"  # ✅ 添加这一行（如果还没有）
```

**完成！** 🎉

不需要修改：
- ❌ `commands/query.rs`
- ❌ `commands/metadata.rs`
- ❌ `commands/database.rs`
- ❌ `postgres_impl.rs`
- ❌ `mysql_impl.rs`

---

## 🎁 附加功能

### 连接池管理器

```rust
/// 连接池管理器 - 抽象池生命周期
pub struct ConnectionPoolManager {
    pools: std::collections::HashMap<String, DbConnection>,
}

impl ConnectionPoolManager {
    pub fn new() -> Self {
        Self {
            pools: std::collections::HashMap::new(),
        }
    }

    pub async fn get_or_create_connection(
        &mut self,
        connection_id: &str,
        service: &Arc<dyn DatabaseService>,
        host: &str,
        port: u16,
        database_name: &str,
        user: &str,
        password: &str,
    ) -> Result<&DbConnection, AppError> {
        if !self.pools.contains_key(connection_id) {
            let conn = service
                .connect(host, port, database_name, user, password)
                .await?;
            self.pools.insert(connection_id.to_string(), conn);
        }

        Ok(self.pools.get(connection_id).unwrap())
    }

    pub fn remove_connection(&mut self, connection_id: &str) {
        self.pools.remove(connection_id);
    }

    pub fn cleanup(&mut self) {
        self.pools.clear();
    }
}
```

### 错误处理增强

**文件**：`src/utils/error.rs`

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库连接错误: {0}")]
    DatabaseConnection(String),

    #[error("查询执行失败: {0}")]
    QueryExecution(String),

    #[error("不支持的数据库类型: {0}")]
    UnsupportedDatabase(String),

    #[error("元数据提取失败: {0}")]
    MetadataExtraction(String),

    #[error("类型转换失败: {0}")]
    TypeConversion(String),

    #[error("缓存操作失败: {0}")]
    CacheError(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

---

## ✅ 验证清单

### 实现前

- [ ] 阅读完整的架构设计文档
- [ ] 理解 SOLID 原则
- [ ] 熟悉 Rust trait 和 async trait
- [ ] 备份现有代码（git commit 或分支）

### 实现中

- [ ] 创建 `services/database/` 目录结构
- [ ] 定义 `DatabaseService` trait
- [ ] 实现 `DatabaseServiceFactory`
- [ ] 迁移 PostgreSQL 服务
- [ ] 迁移 MySQL 服务
- [ ] 更新所有命令处理器
- [ ] 更新错误处理

### 实现后

- [ ] 所有现有测试通过
- [ ] 手动测试所有数据库连接
- [ ] 手动测试所有查询类型
- [ ] 手动测试元数据提取
- [ ] 性能测试（确保无回退）
- [ ] 代码审查
- [ ] 更新文档

---

## 📊 架构对比

### 修改点对比

| 操作 | 当前架构 | 新架构 |
|------|---------|--------|
| 添加新数据库 | 修改 8 处 | 修改 4 处，零破坏性修改 |
| 修改查询逻辑 | 修改所有命令 | 修改一个 trait 方法 |
| 添加新方法 | 修改所有服务 | 修改 trait + 所有实现 |
| 测试新数据库 | 需要修改现有测试 | 独立测试 |

### 代码量对比

| 组件 | 当前 | 新架构 | 变化 |
|------|------|--------|------|
| Trait 定义 | 0 | ~150 行 | +150 |
| Factory | 0 | ~100 行 | +100 |
| PostgreSQL 实现 | ~400 行 | ~450 行 | +50 |
| MySQL 实现 | ~350 行 | ~400 行 | +50 |
| 命令处理器 | ~800 行 | ~400 行 | -400 |
| **总计** | **~1550 行** | **~1400 行** | **-150 行 (-10%)** |

### 维护成本对比

| 场景 | 当前架构 | 新架构 |
|------|---------|--------|
| 添加新数据库 | 修改 8 个文件，约 50 行代码 | 创建 1 个文件，约 300 行代码 |
| 修复 PostgreSQL Bug | 可能影响其他数据库 | 只修改 PostgreSQL 实现 |
| 更新错误处理 | 修改所有命令 | 修改 trait 和错误类型 |
| 添加日志 | 修改所有服务 | 修改 trait 或添加装饰器 |

---

## 🎓 设计原则总结

### SOLID 原则映射

| 原则 | 当前架构问题 | 新架构解决方案 |
|------|-------------|---------------|
| **S** - 单一职责 | 命令处理器包含数据库逻辑 | 每个服务只处理一个数据库 |
| **O** - 开闭原则 | 添加数据库需修改现有代码 | 通过 Trait 扩展，无需修改 |
| **L** - 里氏替换 | 无抽象，无法替换 | 所有服务可互相替换 |
| **I** - 接口隔离 | 无接口定义 | 精简的 Trait 接口 |
| **D** - 依赖倒置 | 依赖具体实现 | 依赖 DatabaseService 抽象 |

### 设计模式使用

| 模式 | 用途 | 位置 |
|------|------|------|
| **Factory** | 创建数据库服务 | `DatabaseServiceFactory` |
| **Strategy** | 数据库特定行为 | `DatabaseService` 实现 |
| **Adapter** | 统一不同数据库接口 | `DbConnection` 转换 |
| **Template Method** | 通用查询流程 | `execute_query` 骨架 |

---

## 📖 参考资料

### Rust 相关

- [The Rust Book - Trait System](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [Async Trait in Rust](https://docs.rs/async-trait/latest/async_trait/)
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)

### SOLID 原则

- [SOLID Principles Wikipedia](https://en.wikipedia.org/wiki/SOLID)
- [SOLID in Rust](https://blog.logrocket.com/solid-principles-rust/)

### 数据库抽象

- [Database Abstraction Layer Best Practices](https://www.craftinglabs.com/blog/database-abstraction-layers/)
- [Repository Pattern in Rust](https://kamu.dev/repository-pattern-in-rust/)

---

## 🤝 贡献指南

如果您想改进此架构设计：

1. Fork 项目仓库
2. 创建特性分支：`git checkout -b feature/db-architecture`
3. 提交更改：`git commit -m 'Improve database architecture'`
4. 推送到分支：`git push origin feature/db-architecture`
5. 创建 Pull Request

---

## 📝 变更日志

| 版本 | 日期 | 更改 |
|------|------|------|
| 1.0.0 | 2026-01-26 | 初始架构设计文档 |

---

**文档版本**：1.0.0
**最后更新**：2026-01-26
**作者**：Claude (rust-system-architect agent)
**审核状态**：待审核
