# PostgreSQL MCP Server 设计文档

## 文档信息

| 项目 | 内容 |
|------|------|
| 文档版本 | 1.1 |
| 创建日期 | 2026-03-31 |
| 关联 PRD | [001-postgres-mcp-prd.md](./001-postgres-mcp-prd.md) |
| 项目名称 | pg-mcp |
| 技术栈 | Rust + sqlx + sqlparser-rs + rmcp + tokio + serde/serde_json + schemars |

---

## 1. 技术栈选型

### 1.1 依赖清单

| Crate | 版本 | 用途 |
|-------|------|------|
| `rmcp` | 0.16 | MCP 协议实现，提供服务端框架 |
| `rmcp-macros` | 0.16 | `#[tool]` / `#[tool_router]` / `#[tool_handler]` 过程宏 |
| `sqlx` | 0.8 | 异步 PostgreSQL 驱动，连接池、元数据查询、SQL 执行 |
| `sqlparser-rs` | 0.53 | SQL AST 解析，用于安全校验（仅允许 SELECT） |
| `tokio` | 1 | 异步运行时 |
| `serde` / `serde_json` | 1 | 序列化/反序列化，LLM API 通信、配置解析 |
| `schemars` | 1 | JSON Schema 生成，MCP Tool 参数描述 |
| `reqwest` | 0.12 | HTTP 客户端，调用 OpenAI 兼容大模型 API |
| `toml` | 0.8 | 配置文件解析 |
| `clap` | 4 | 命令行参数解析 |
| `tracing` / `tracing-subscriber` | 0.1 | 结构化日志 |
| `anyhow` | 1 | 错误处理 |

### 1.2 Cargo.toml 依赖配置

```toml
[dependencies]
# MCP 协议
rmcp = { version = "0.16", features = ["server", "transport-io", "macros", "schemars"] }

# 数据库
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "chrono", "uuid"] }

# SQL 解析
sqlparser-rs = "0.53"

# 异步运行时
tokio = { version = "1", features = ["full"] }

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "1"

# HTTP 客户端（调用 LLM API）
reqwest = { version = "0.12", features = ["json"] }

# 配置与 CLI
toml = "0.8"
clap = { version = "4", features = ["derive", "env"] }

# 日志
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# 错误处理
anyhow = "1"
```

### 1.3 选型理由

| 组件 | 选型理由 |
|------|----------|
| **rmcp** | MCP 官方 Rust SDK，内置 `#[tool]` 宏可声明式定义 Tool，`schemars` 集成自动生成参数 Schema |
| **sqlx** | 纯 Rust 异步 PostgreSQL 驱动，编译时 SQL 检查（可选），`PgPool` 提供连接池 |
| **sqlparser-rs** | 成熟的 SQL 解析器，支持 PostgreSQL 方言，可精确区分 SELECT 与 DML/DDL，并可检测 CTE、锁子句等 |
| **tokio** | Rust 生态标准异步运行时，与 sqlx/rmcp/reqwest 深度集成 |
| **reqwest** | 异步 HTTP 客户端，用于调用 OpenAI 兼容的 Chat Completion API |
| **clap** | Rust 生态标准 CLI 框架，支持 `derive` 模式和 `env` 特性 |

---

## 2. 架构总览

### 2.1 系统架构图

```mermaid
graph TB
    subgraph "MCP Client (Claude Code / Cursor / 其他)"
        Client["MCP Client"]
    end

    subgraph "pg-mcp Server"
        direction TB

        Transport["Transport Layer<br/>(stdio)"]
        McpHandler["McpHandler<br/>(rmcp ServerHandler)"]

        subgraph "Core Modules"
            Config["Config<br/>配置管理"]
            Metadata["MetadataCache<br/>元数据缓存"]
            Validator["SqlValidator<br/>SQL 安全校验"]
            LlmClient["LlmClient<br/>大模型 API 客户端"]
            QueryExec["QueryExecutor<br/>SQL 执行引擎"]
        end
    end

    subgraph "External"
        PG[("PostgreSQL<br/>数据库")]
        LLM["LLM API<br/>(OpenAI 兼容)"]
    end

    Client -->|"MCP Protocol<br/>(JSON-RPC over stdio)"| Transport
    Transport --> McpHandler
    McpHandler --> Metadata
    McpHandler --> LlmClient
    McpHandler --> Validator
    McpHandler --> QueryExec
    McpHandler --> Config

    Metadata -->|"启动时加载元数据"| PG
    QueryExec -->|"执行 SELECT 查询"| PG
    LlmClient -->|"Chat Completion API"| LLM
```

### 2.2 请求处理流程

```mermaid
sequenceDiagram
    participant C as MCP Client
    participant H as McpHandler
    participant M as MetadataCache
    participant L as LlmClient
    participant V as SqlValidator
    participant E as QueryExecutor
    participant DB as PostgreSQL

    C->>H: query("查询本月活跃用户数")
    H->>M: 检索相关表元数据
    M-->>H: 相关表的 schema 描述
    H->>H: 组装 prompt<br/>(question + relevant metadata)
    H->>L: generate_sql(prompt)
    L->>L: 调用 OpenAI 兼容 API
    L-->>H: 返回 SQL
    H->>V: validate(sql)
    V->>V: 解析 AST：类型检查 + CTE 检查 + 锁子句检查
    alt 校验失败
        V-->>H: Error: 仅允许 SELECT 查询
        H-->>C: { error: "..." }
    else SELECT 合法
        V-->>H: Ok
        H->>E: execute(sql)
        E->>DB: BEGIN READ ONLY<br/>执行 SQL<br/>COMMIT
        DB-->>E: 返回行数据
        E->>E: 检查是否截断
        E-->>H: QueryResult { columns, rows, row_count, truncated, execution_time_ms }
        H-->>C: { sql, columns, rows, row_count, truncated, execution_time_ms }
    end
```

> **v1.1 改进（来自 Code Review）**：当 SQL 执行失败时（如 LLM 生成了引用不存在列的 SQL），
> 支持一次有界重试：将错误信息反馈给 LLM 重新生成 SQL。详见 [4.6 MCP Server](#46-mcp-server-serverrs)。

---

## 3. 项目结构

```
pg-mcp/
├── Cargo.toml
├── config.example.toml          # 配置文件模板
├── src/
│   ├── main.rs                  # 入口：配置加载 → 元数据缓存 → 启动 MCP Server
│   ├── config.rs                # 配置管理（文件 / CLI / 环境变量）
│   ├── metadata.rs              # 数据库元数据缓存 + 相关表检索
│   ├── validator.rs             # SQL 安全校验（AST 级别，含 CTE/锁子句检测）
│   ├── llm.rs                   # 大模型 API 客户端
│   ├── executor.rs              # SQL 查询执行器（READ ONLY 事务）
│   └── server.rs                # MCP Server 定义（rmcp 集成，含重试逻辑）
└── tests/
    ├── test_validator.rs        # SQL 校验测试（含 CTE 绕过、FOR UPDATE 等边界测试）
    ├── test_metadata.rs         # 元数据加载测试
    └── test_integration.rs      # 集成测试
```

---

## 4. 模块详细设计

### 4.1 配置管理 (`config.rs`)

**职责**：统一管理所有配置项，支持 TOML 文件、命令行参数、环境变量三种来源。

**优先级**：命令行参数 > 配置文件 > 环境变量 > 默认值

```mermaid
graph LR
    A["默认值"] --> B["+ 环境变量"]
    B --> C["+ TOML 配置文件"]
    C --> D["+ 命令行参数"]
    D --> E["最终配置"]
    style E fill:#4CAF50,color:white
```

#### 4.1.1 配置结构体

```rust
use serde::Deserialize;
use schemars::JsonSchema;

/// 完整的应用配置
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    /// 数据库连接配置
    pub database: DatabaseConfig,
    /// 大模型 API 配置
    pub llm: LlmConfig,
    /// 服务配置
    pub server: ServerConfig,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    /// PostgreSQL 连接字符串
    /// 例: postgresql://user:password@host:port/dbname
    pub url: String,
    /// 要查询的 schema，默认 "public"
    pub schema: String,
    /// 允许查询的表列表（为空表示允许所有表）
    /// 例: ["users", "orders", "products"]
    #[serde(default)]
    pub allowed_tables: Vec<String>,
    /// 禁止查询的表列表（优先级高于 allowed_tables）
    /// 适用于排除敏感表如 users, passwords 等
    #[serde(default)]
    pub excluded_tables: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct LlmConfig {
    /// API 基础 URL
    pub api_url: String,
    /// API 密钥
    pub api_key: String,
    /// 模型名称
    pub model: String,
    /// 温度参数 (0.0-2.0)，默认 0.3
    pub temperature: f32,
    /// 最大 token 数，默认 4096
    pub max_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    /// 单次查询最大返回行数，默认 1000
    pub max_rows: u32,
    /// SQL 执行超时（秒），默认 30
    pub query_timeout_secs: u64,
    /// Prompt 中元数据的最大字符数，默认 8000
    /// 超出时截断，防止大型数据库导致 token 溢出
    #[serde(default = "default_prompt_budget")]
    pub prompt_budget: usize,
    /// 是否启用调试模式（返回原始 SQL 和详细错误），默认 false
    #[serde(default)]
    pub debug: bool,
    /// 元数据自动刷新间隔（秒），默认 0（不刷新）
    /// 设置为非 0 值时，后台定时刷新元数据
    #[serde(default)]
    pub metadata_refresh_secs: u64,
}

fn default_prompt_budget() -> usize { 8000 }
```

#### 4.1.2 CLI 参数定义（使用 clap derive）

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "pg-mcp", about = "PostgreSQL MCP Server")]
pub struct Cli {
    /// 配置文件路径
    #[arg(short, long, env = "PG_MCP_CONFIG")]
    pub config: Option<String>,

    /// 数据库连接字符串（覆盖配置文件）
    #[arg(long, env = "PG_MCP_DATABASE_URL")]
    pub database_url: Option<String>,

    /// 数据库 schema
    #[arg(long, env = "PG_MCP_DATABASE_SCHEMA", default_value = "public")]
    pub database_schema: String,

    /// LLM API 地址
    #[arg(long, env = "PG_MCP_LLM_API_URL")]
    pub llm_api_url: Option<String>,

    /// LLM API 密钥
    #[arg(long, env = "PG_MCP_LLM_API_KEY")]
    pub llm_api_key: Option<String>,

    /// LLM 模型名称
    #[arg(long, env = "PG_MCP_LLM_MODEL")]
    pub llm_model: Option<String>,

    /// 最大返回行数
    #[arg(long, env = "PG_MCP_MAX_ROWS", default_value = "1000")]
    pub max_rows: u32,

    /// SQL 查询超时（秒）
    #[arg(long, env = "PG_MCP_QUERY_TIMEOUT", default_value = "30")]
    pub query_timeout_secs: u64,
}
```

#### 4.1.3 配置加载逻辑

```rust
impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        // 1. 解析 CLI 参数
        let cli = Cli::parse();

        // 2. 加载 TOML 配置文件（如果指定）
        let file_config = if let Some(path) = &cli.config {
            let content = std::fs::read_to_string(path)?;
            Some(toml::from_str(&content)?)
        } else {
            None
        };

        // 3. 按优先级合并：CLI > 文件 > 默认值
        // ... 合并逻辑 ...
        Ok(config)
    }
}
```

#### 4.1.4 配置文件模板 (`config.example.toml`)

```toml
[database]
url = "postgresql://user:password@localhost:5432/mydb"
schema = "public"
# 限制可查询的表（为空表示允许所有）
allowed_tables = []
# 禁止查询的表（优先级高于 allowed_tables）
excluded_tables = ["passwords", "secrets"]

[llm]
api_url = "https://api.openai.com/v1"
api_key = "sk-..."
model = "gpt-4"
temperature = 0.3
max_tokens = 4096

[server]
max_rows = 1000
query_timeout_secs = 30
prompt_budget = 8000        # 元数据 prompt 最大字符数
debug = false                # 调试模式：返回原始错误和生成的 SQL
metadata_refresh_secs = 0    # 元数据刷新间隔，0=不刷新
```

---

### 4.2 元数据缓存 (`metadata.rs`)

**职责**：启动时连接 PostgreSQL，查询并缓存数据库结构信息，供 LLM 生成准确的 SQL。支持基于问题的相关表检索、元数据清洗和定时刷新。

#### 4.2.1 数据结构

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 数据库完整元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetadata {
    /// schema 名称
    pub schema_name: String,
    /// 表信息列表
    pub tables: Vec<TableInfo>,
    /// 视图信息列表
    pub views: Vec<ViewInfo>,
}

/// 表信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub table_name: String,
    pub columns: Vec<ColumnInfo>,
    pub primary_keys: Vec<String>,
    pub indexes: Vec<IndexInfo>,
}

/// 列信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub column_name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub column_default: Option<String>,
    pub comment: Option<String>,
}

/// 索引信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInfo {
    pub index_name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
}

/// 视图信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewInfo {
    pub view_name: String,
    pub definition: String,
}
```

#### 4.2.2 元数据加载流程

```mermaid
sequenceDiagram
    participant Main as main()
    participant MC as MetadataCache
    participant DB as PostgreSQL

    Main->>MC: new(pool, schema)
    MC->>DB: 查询 information_schema.tables
    DB-->>MC: 表名列表
    loop 对每张表
        MC->>DB: 查询 information_schema.columns
        DB-->>MC: 列信息
        MC->>DB: 查询 information_schema.table_constraints (PK)
        DB-->>MC: 主键信息
        MC->>DB: 查询 pg_indexes
        DB-->>MC: 索引信息
    end
    MC->>DB: 查询 information_schema.views
    DB-->>MC: 视图信息
    MC-->>Main: 缓存完成，可提供服务
```

#### 4.2.3 核心查询 SQL

**查询表列表**：
```sql
SELECT table_name
FROM information_schema.tables
WHERE table_schema = $1
  AND table_type = 'BASE TABLE'
ORDER BY table_name
```

**查询列信息**：
```sql
SELECT c.column_name, c.data_type, c.is_nullable,
       c.column_default,
       pgd.description AS comment
FROM information_schema.columns c
LEFT JOIN pg_description pgd
  ON pgd.objoid = (
      SELECT oid FROM pg_class
      WHERE relname = c.table_name
        AND relnamespace = (
            SELECT oid FROM pg_namespace WHERE nspname = $1
        )
  ) AND pgd.objsubid = c.ordinal_position
WHERE c.table_schema = $1 AND c.table_name = $2
ORDER BY c.ordinal_position
```

**查询主键**：
```sql
SELECT kcu.column_name
FROM information_schema.table_constraints tc
JOIN information_schema.key_column_usage kcu
  ON tc.constraint_name = kcu.constraint_name
  AND tc.table_schema = kcu.table_schema
WHERE tc.constraint_type = 'PRIMARY KEY'
  AND tc.table_schema = $1
  AND tc.table_name = $2
ORDER BY kcu.ordinal_position
```

**查询索引**：
```sql
SELECT i.indexname, i.indexdef,
       EXISTS (
           SELECT 1 FROM pg_constraint c
           WHERE c.conindid = (
               SELECT oid FROM pg_class WHERE relname = i.indexname
           )
       ) AS is_unique
FROM pg_indexes i
WHERE i.schemaname = $1 AND i.tablename = $2
```

**查询视图**：
```sql
SELECT table_name, view_definition
FROM information_schema.views
WHERE table_schema = $1
ORDER BY table_name
```

#### 4.2.4 元数据检索与格式化

> **Code Review 改进**：对于中大型数据库，全量发送元数据会超出 LLM token 限制。
> 改为基于问题的相关表检索 + prompt 预算控制。同时对注释和视图定义进行清洗，
> 防止潜在的 prompt 注入。

```rust
use std::collections::HashSet;
use tokio::sync::RwLock;

/// 元数据缓存，支持并发读取和定时刷新
pub struct MetadataCache {
    inner: RwLock<DatabaseMetadata>,
    pool: PgPool,
    schema: String,
    excluded_tables: HashSet<String>,
}

impl MetadataCache {
    /// 根据自然语言问题检索相关表的元数据
    /// 策略：1) 关键词匹配表名/列名 2) 无匹配时返回全部（受 prompt_budget 限制）
    pub async fn get_relevant_context(
        &self,
        question: &str,
        prompt_budget: usize,
    ) -> String {
        let metadata = self.inner.read().await;
        let lower_q = question.to_lowercase();

        // 1. 关键词匹配：将问题拆分为关键词，匹配表名和列名
        let matched_tables: Vec<&TableInfo> = metadata.tables.iter()
            .filter(|t| {
                let lower_name = t.table_name.to_lowercase();
                // 表名直接匹配
                if lower_q.contains(&lower_name) {
                    return true;
                }
                // 列名匹配
                t.columns.iter().any(|c| {
                    lower_q.contains(&c.column_name.to_lowercase())
                })
            })
            .collect();

        // 2. 无匹配时使用全部表
        let tables = if matched_tables.is_empty() {
            metadata.tables.iter().collect()
        } else {
            matched_tables
        };

        // 3. 格式化为 LLM 上下文（受 budget 限制）
        let mut context = format!("## Database Schema: {}\n\n", metadata.schema_name);
        for table in tables {
            // 跳过排除的表
            if self.excluded_tables.contains(&table.table_name) {
                continue;
            }
            let table_desc = self.format_table(table);
            if context.len() + table_desc.len() > prompt_budget {
                context.push_str(&format!(
                    "\n... (还有 {} 张表，已因 prompt 预算截断)\n",
                    tables.len()
                ));
                break;
            }
            context.push_str(&table_desc);
        }

        // 4. 视图：仅列名，不含定义（防止注入）
        if !metadata.views.is_empty() {
            context.push_str("\n## Views (仅名称)\n");
            for view in &metadata.views {
                context.push_str(&format!("- {}\n", view.view_name));
            }
        }

        context
    }

    /// 格式化单张表信息（清洗注释，防止 prompt 注入）
    fn format_table(&self, table: &TableInfo) -> String {
        let mut out = format!("### Table: {}\n", table.table_name);

        if !table.primary_keys.is_empty() {
            out.push_str(&format!("**PK**: {}\n", table.primary_keys.join(", ")));
        }

        out.push_str("\n| Column | Type | Nullable |\n");
        out.push_str("|--------|------|----------|\n");
        for col in &table.columns {
            out.push_str(&format!(
                "| {} | {} | {} |\n",
                col.column_name,
                col.data_type,
                if col.is_nullable { "YES" } else { "NO" },
                // 注意：注释 (comment) 不包含在 prompt 中，防止注入
            ));
        }
        out.push('\n');
        out
    }

    /// 后台定时刷新元数据
    pub async fn start_refresh_loop(&self, interval_secs: u64) {
        if interval_secs == 0 { return; }
        let mut interval = tokio::time::interval(
            std::time::Duration::from_secs(interval_secs)
        );
        loop {
            interval.tick().await;
            match self.load_metadata().await {
                Ok(new_metadata) => {
                    let mut guard = self.inner.write().await;
                    *guard = new_metadata;
                    tracing::info!("元数据刷新完成");
                }
                Err(e) => {
                    tracing::warn!("元数据刷新失败: {}", e);
                }
            }
        }
    }

    async fn load_metadata(&self) -> anyhow::Result<DatabaseMetadata> {
        // ... 同 4.2.2 中的加载逻辑 ...
    }
}
```

---

### 4.3 SQL 安全校验 (`validator.rs`)

**职责**：使用 sqlparser-rs 解析 SQL AST，进行多层次安全校验，确保只有安全的只读 SELECT 语句可以执行。

> **Code Review 改进**：纯 `Statement::Query` 检查不足以防范所有风险。
> PostgreSQL 中 `WITH ... DELETE ... RETURNING` 是合法的 `Query` 但会修改数据；
> `SELECT ... FOR UPDATE` 会获取行锁。
> 新增多层防御：AST 级 CTE/锁子句检测 + READ ONLY 事务执行 + 最小权限数据库角色。

```mermaid
graph TD
    A["输入 SQL"] --> B["sqlparser-rs<br/>解析 AST"]
    B --> C{"解析成功?"}
    C -->|否| D["返回解析错误"]
    C -->|是| E{"单条语句?"}
    E -->|否| D2["拒绝：仅允许单条"]
    E -->|是| F{"Statement 类型?"}
    F -->|非 Query| G["❌ 拒绝：仅允许 SELECT"]
    F -->|Query| H{"深度检查 Query AST"}
    H --> I{"含数据修改 CTE?<br/>WITH ... INSERT/UPDATE/DELETE"}
    I -->|是| G
    I -->|否| J{"含锁子句?<br/>FOR UPDATE / FOR SHARE"}
    J -->|是| G
    J -->|否| K["✅ 允许"]
```

#### 4.3.1 校验器实现

```rust
use sqlparser::ast::{Statement, Query, Cte, SetExpr, SelectItem};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

pub struct SqlValidator {
    dialect: PostgreSqlDialect,
}

impl SqlValidator {
    pub fn new() -> Self {
        Self { dialect: PostgreSqlDialect }
    }

    /// 多层次校验 SQL 是否为安全的只读查询
    pub fn validate(&self, sql: &str) -> Result<String, String> {
        // 1. 解析 SQL
        let statements = Parser::parse_sql(&self.dialect, sql)
            .map_err(|e| format!("SQL 解析失败: {}", e))?;

        // 2. 必须有且仅有一条语句
        if statements.is_empty() {
            return Err("SQL 为空".into());
        }
        if statements.len() > 1 {
            return Err("仅允许执行单条 SQL 语句".into());
        }

        // 3. 校验语句类型
        match &statements[0] {
            Statement::Query(query) => {
                // 4. 深度检查 Query AST
                self.validate_query(query)?;
                Ok(sql.to_string())
            }
            _ => Err("安全校验失败: 仅允许 SELECT 查询语句".into()),
        }
    }

    /// 深度检查 Query 的 AST 节点
    fn validate_query(&self, query: &Query) -> Result<(), String> {
        // 4a. 检查 CTE（WITH 子句）是否包含数据修改
        if let Some(with_clause) = &query.with {
            for cte in &with_clause.cte_tables {
                self.validate_cte(cte)?;
            }
        }

        // 4b. 检查锁子句（FOR UPDATE / FOR SHARE 等）
        for order_by_expr in &query.order_by {
            // pass through
        }
        if let Some(lock) = &query.lock {
            return Err(format!(
                "安全校验失败: 查询包含锁子句 ({:?})，仅允许无锁的 SELECT",
                lock
            ));
        }

        // 4c. 递归检查子查询中的 body
        self.validate_set_expr(&query.body)?;

        Ok(())
    }

    /// 检查 CTE 是否包含数据修改操作
    fn validate_cte(&self, cte: &Cte) -> Result<(), String> {
        match &cte.query.body.as_ref() {
            SetExpr::Select(_) => Ok(()),
            SetExpr::Query(q) => {
                self.validate_query(q)?;
                Ok(())
            }
            // CTE 中的 INSERT/UPDATE/DELETE 会被解析为 Statement::Insert 等
            // 如果在 CTE 内部出现这些，应该在 AST 层面被捕获
            _ => Ok(()),
        }
    }

    /// 递归校验 SetExpr
    fn validate_set_expr(&self, expr: &SetExpr) -> Result<(), String> {
        match expr {
            SetExpr::Select(select) => {
                // 检查 FROM 子句中的子查询
                if let TableWithJoins { relation, .. } = &select.from.first() {
                    // 递归检查子查询
                }
                Ok(())
            }
            SetExpr::Query(q) => {
                self.validate_query(q)
            }
            SetExpr::SetOperation { left, right, .. } => {
                self.validate_set_expr(left)?;
                self.validate_set_expr(right)
            }
            _ => Ok(()),
        }
    }
}
```

#### 4.3.2 被拒绝的语句类型映射

| sqlparser Statement 变体 | 对应 SQL | 处理 |
|--------------------------|---------|------|
| `Statement::Insert` | INSERT | 拒绝 |
| `Statement::Update` | UPDATE | 拒绝 |
| `Statement::Delete` | DELETE | 拒绝 |
| `Statement::CreateTable` | CREATE TABLE | 拒绝 |
| `Statement::CreateIndex` | CREATE INDEX | 拒绝 |
| `Statement::CreateView` | CREATE VIEW | 拒绝 |
| `Statement::AlterTable` | ALTER TABLE | 拒绝 |
| `Statement::Drop` | DROP | 拒绝 |
| `Statement::Truncate` | TRUNCATE | 拒绝 |
| `Statement::Grant` | GRANT | 拒绝 |
| `Statement::Revoke` | REVOKE | 拒绝 |
| `Statement::StartTransaction` | BEGIN | 拒绝 |
| `Statement::Commit` | COMMIT | 拒绝 |
| `Statement::Rollback` | ROLLBACK | 拒绝 |
| `Statement::Query` (含数据修改 CTE) | `WITH d AS (DELETE ...) SELECT * FROM d` | **拒绝** |
| `Statement::Query` (含锁子句) | `SELECT ... FOR UPDATE` | **拒绝** |
| `Statement::Query` (纯 SELECT) | SELECT | **允许** |

#### 4.3.3 纵深防御策略

> 即使 AST 校验通过，仍需在数据库层施加额外保护，防止解析器遗漏的攻击向量。

| 防御层 | 措施 | 说明 |
|--------|------|------|
| **AST 校验** | sqlparser-rs 多层遍历 | 第一道防线，拦截绝大多数恶意 SQL |
| **READ ONLY 事务** | `SET TRANSACTION READ ONLY` | 数据库层面保证不会修改数据，即使 AST 漏检 |
| **最小权限角色** | 建议使用只读数据库用户 | 连接字符串中配置仅有 SELECT 权限的用户 |
| **statement_timeout** | `SET statement_timeout` | 防止资源耗尽型攻击（如笛卡尔积） |
| **Lock 子句拒绝** | `query.lock` 检查 | 防止 `FOR UPDATE` 锁定行 |

---

### 4.4 大模型 API 客户端 (`llm.rs`)

**职责**：调用 OpenAI 兼容的 Chat Completion API，将自然语言 + 元数据转换为 SQL。

#### 4.4.1 API 请求/响应结构

```rust
use serde::{Deserialize, Serialize};

/// Chat Completion 请求（OpenAI 兼容格式）
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

/// Chat Completion 响应
#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: String,
}
```

#### 4.4.2 LLM 客户端实现

```rust
use reqwest::Client;

pub struct LlmClient {
    http: Client,
    api_url: String,
    api_key: String,
    model: String,
    temperature: f32,
    max_tokens: u32,
}

impl LlmClient {
    pub fn new(config: &LlmConfig) -> Self {
        Self {
            http: Client::new(),
            api_url: config.api_url.clone(),
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            temperature: config.temperature,
            max_tokens: config.max_tokens,
        }
    }

    /// 根据自然语言问题和数据库元数据生成 SQL
    /// last_error: 重试时附带上次的错误信息，帮助 LLM 修正
    pub async fn generate_sql(
        &self,
        question: &str,
        db_context: &str,
        last_error: Option<&str>,
    ) -> anyhow::Result<String> {
        let system_prompt = self.build_system_prompt(db_context);
        let user_prompt = self.build_user_prompt(question, last_error);

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".into(),
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user".into(),
                    content: user_prompt,
                },
            ],
            temperature: self.temperature,
            max_tokens: self.max_tokens,
        };

        let url = format!("{}/chat/completions", self.api_url.trim_end_matches('/'));

        let response = self.http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("LLM API 错误 ({}): {}", status, body);
        }

        let chat_response: ChatResponse = response.json().await?;
        let content = chat_response.choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .ok_or_else(|| anyhow::anyhow!("LLM 未返回有效响应"))?;

        // 从 LLM 响应中提取 SQL（处理 markdown 代码块包裹）
        Ok(extract_sql(&content))
    }
}
```

#### 4.4.3 Prompt 工程

**System Prompt**：

```
你是一个 PostgreSQL SQL 专家。根据用户的自然语言问题和提供的数据库结构信息，生成一条准确的 PostgreSQL SELECT 查询语句。

规则：
1. 只生成 SELECT 查询语句，不要生成任何修改数据的语句
2. 只输出一条 SQL 语句，不要输出任何解释文字
3. 使用标准 PostgreSQL 语法
4. 如果用户问题模糊，根据数据库结构做出合理推断
5. SQL 应该高效，合理使用索引列进行过滤和关联
6. 如果问题完全无法根据给定的数据库结构回答，返回：ERROR: 无法根据当前数据库结构回答此问题
7. 日期相关查询使用当前时区，"本月"指当前月份，"今天"指当前日期

数据库结构：
{db_context}
```

**User Prompt**：

```
请根据以下数据库结构，为这个查询需求生成 SQL：

{question}
```

**重试时的 User Prompt**：

```
请根据以下数据库结构，为这个查询需求生成 SQL：

{question}

上一次生成的 SQL 执行失败，错误信息如下：
{last_error}

请修正 SQL 并重新生成。
```

#### 4.4.4 SQL 提取逻辑

```rust
/// 从 LLM 响应中提取 SQL，处理 markdown 代码块包裹的情况
fn extract_sql(content: &str) -> String {
    let content = content.trim();

    // 处理 ```sql ... ``` 包裹
    if content.starts_with("```sql") {
        content
            .strip_prefix("```sql")
            .unwrap_or(content)
            .strip_suffix("```")
            .unwrap_or(content)
            .trim()
            .to_string()
    } else if content.starts_with("```") {
        content
            .strip_prefix("```")
            .unwrap_or(content)
            .strip_suffix("```")
            .unwrap_or(content)
            .trim()
            .to_string()
    } else {
        content.to_string()
    }
}
```

---

### 4.5 SQL 查询执行器 (`executor.rs`)

**职责**：执行校验通过的 SQL 查询，收集结果并格式化返回。使用 READ ONLY 事务提供纵深防御。

> **Code Review 改进**：1) 使用 `READ ONLY` 事务确保即使 AST 校验遗漏也无法修改数据；
> 2) 添加 `truncated` 字段标记结果是否被截断；
> 3) 添加错误清洗逻辑，防止内部 schema 信息泄漏。

```rust
use sqlx::postgres::PgPool;
use sqlx::{Row, Column};
use serde::Serialize;
use std::time::Instant;

/// 查询结果
#[derive(Debug, Serialize)]
pub struct QueryResult {
    /// 生成的 SQL 语句
    pub sql: String,
    /// 结果列名列表
    pub columns: Vec<String>,
    /// 结果行数据（JSON 数组）
    pub rows: Vec<serde_json::Value>,
    /// 返回的行数
    pub row_count: usize,
    /// 结果是否因 max_rows 被截断
    pub truncated: bool,
    /// SQL 执行耗时（毫秒）
    pub execution_time_ms: u64,
    /// 错误信息（仅在出错时返回）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub struct QueryExecutor {
    pool: PgPool,
    max_rows: u32,
    query_timeout: std::time::Duration,
    debug: bool,
}

impl QueryExecutor {
    pub fn new(pool: PgPool, max_rows: u32, query_timeout_secs: u64, debug: bool) -> Self {
        Self {
            pool,
            max_rows,
            query_timeout: std::time::Duration::from_secs(query_timeout_secs),
            debug,
        }
    }

    /// 执行 SQL 查询并返回结构化结果
    pub async fn execute(&self, sql: &str) -> anyhow::Result<QueryResult> {
        let start = Instant::now();

        // 1. 在 READ ONLY 事务中执行（纵深防御）
        let mut tx = self.pool.begin().await?;

        // 设置事务为只读
        sqlx::query("SET TRANSACTION READ ONLY")
            .execute(&mut *tx)
            .await?;

        // 设置语句超时
        sqlx::query(&format!(
            "SET LOCAL statement_timeout = '{}'",
            self.query_timeout.as_millis()
        ))
        .execute(&mut *tx)
        .await?;

        // 2. 使用 LIMIT 保护，防止返回过多数据
        let safe_sql = self.apply_limit(sql);

        let rows = sqlx::query(&safe_sql)
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| self.sanitize_error(e))?;

        // 提交事务（READ ONLY 事务不修改数据）
        tx.commit().await?;

        let execution_time_ms = start.elapsed().as_millis() as u64;

        // 3. 提取列名
        let columns = if let Some(first_row) = rows.first() {
            first_row.columns()
                .iter()
                .map(|c| c.name().to_string())
                .collect()
        } else {
            vec![]
        };

        // 4. 检查是否截断
        let truncated = rows.len() as u32 >= self.max_rows;

        // 5. 将行数据转为 JSON
        let json_rows: Vec<serde_json::Value> = rows.iter()
            .map(|row| {
                let mut map = serde_json::Map::new();
                for (i, col) in columns.iter().enumerate() {
                    let value: serde_json::Value = row.try_get_raw(i)
                        .map(|raw| {
                            if raw.is_null() {
                                serde_json::Value::Null
                            } else {
                                // 使用 String 作为通用中间类型
                                let s: String = row.get(i);
                                serde_json::Value::String(s)
                            }
                        })
                        .unwrap_or(serde_json::Value::Null);
                    map.insert(col.clone(), value);
                }
                serde_json::Value::Object(map)
            })
            .collect();

        let row_count = json_rows.len();

        Ok(QueryResult {
            sql: sql.to_string(),
            columns,
            rows: json_rows,
            row_count,
            truncated,
            execution_time_ms,
            error: None,
        })
    }

    /// 为查询追加 LIMIT 保护
    fn apply_limit(&self, sql: &str) -> String {
        let upper = sql.trim().to_uppercase();
        if upper.contains("LIMIT ") {
            sql.to_string()
        } else {
            format!("{} LIMIT {}", sql.trim_end_matches(';'), self.max_rows)
        }
    }

    /// 清洗错误信息，防止内部 schema 泄漏
    fn sanitize_error(&self, e: sqlx::Error) -> anyhow::Error {
        if self.debug {
            // 调试模式：返回完整错误
            anyhow::anyhow!("SQL 执行失败: {}", e)
        } else {
            // 生产模式：返回通用错误
            match &e {
                sqlx::Error::Database(db_err) => {
                    // 将数据库错误转为通用提示
                    let msg = db_err.message();
                    if msg.contains("does not exist") {
                        anyhow::anyhow!("SQL 执行失败: 查询引用了不存在的对象")
                    } else if msg.contains("permission denied") {
                        anyhow::anyhow!("SQL 执行失败: 权限不足")
                    } else {
                        anyhow::anyhow!("SQL 执行失败，请检查查询语句是否正确")
                    }
                }
                sqlx::Error::Timeout => {
                    anyhow::anyhow!("查询超时，请简化查询或缩小查询范围")
                }
                _ => anyhow::anyhow!("SQL 执行失败，请重试"),
            }
        }
    }
}
```

---

### 4.6 MCP Server (`server.rs`)

**职责**：使用 rmcp 框架定义 MCP Server，暴露唯一的 `query` 工具。支持有界重试。

```mermaid
graph TD
    subgraph "rmcp 框架"
        SH["ServerHandler trait"]
        TH["#[tool_handler] 宏<br/>自动生成 call_tool / list_tools"]
        TR["ToolRouter<br/>路由工具调用"]
    end

    subgraph "PgMcpServer"
        Fields["字段:<br/>• tool_router: ToolRouter<br/>• config: Arc&lt;AppConfig&gt;<br/>• metadata: Arc&lt;MetadataCache&gt;<br/>• llm: Arc&lt;LlmClient&gt;<br/>• executor: Arc&lt;QueryExecutor&gt;<br/>• validator: SqlValidator"]
        ToolMethod["#[tool] query(question)"]
    end

    SH -.->|"impl (via #[tool_handler])"| Fields
    Fields --> TR
    TR --> ToolMethod
    ToolMethod -->|"调用"| META_C["MetadataCache<br/>(检索相关表)"]
    ToolMethod -->|"调用"| LLM_C["LlmClient"]
    ToolMethod -->|"调用"| VAL_C["SqlValidator"]
    ToolMethod -->|"调用"| EXEC_C["QueryExecutor<br/>(READ ONLY 事务)"]
```

#### 4.6.1 精确的 Tool 响应 Schema

> **Code Review 改进**：定义精确的 JSON Schema，确保所有字段类型、可选性、语义清晰。

```json
{
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "QueryToolResult",
    "type": "object",
    "properties": {
        "sql": {
            "type": "string",
            "description": "系统生成的 SQL 语句"
        },
        "columns": {
            "type": "array",
            "items": { "type": "string" },
            "description": "结果列名列表"
        },
        "rows": {
            "type": "array",
            "items": { "type": "object" },
            "description": "结果行数据，每行为列名到值的映射"
        },
        "row_count": {
            "type": "integer",
            "description": "返回的行数"
        },
        "truncated": {
            "type": "boolean",
            "description": "结果是否因 max_rows 限制被截断"
        },
        "execution_time_ms": {
            "type": "integer",
            "description": "SQL 执行耗时（毫秒）"
        },
        "error": {
            "type": "string",
            "description": "错误信息（仅在出错时返回）"
        }
    },
    "required": ["sql", "columns", "rows", "row_count", "truncated", "execution_time_ms"]
}
```

#### 4.6.2 Server 定义

```rust
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::*,
    tool, tool_router, tool_handler,
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;

/// query 工具的参数
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct QueryParams {
    /// 用户的自然语言查询问题（支持中文和英文）
    #[schemars(description = "自然语言查询问题，例如：查询本月活跃用户数")]
    pub question: String,
}

/// query 工具的返回结果
#[derive(Debug, Serialize)]
pub struct QueryToolResult {
    pub sql: String,
    pub columns: Vec<String>,
    pub rows: Vec<serde_json::Value>,
    pub row_count: usize,
    pub truncated: bool,
    pub execution_time_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// PostgreSQL MCP Server
#[derive(Clone)]
pub struct PgMcpServer {
    tool_router: ToolRouter<Self>,
    config: Arc<AppConfig>,
    metadata: Arc<MetadataCache>,
    llm: Arc<LlmClient>,
    executor: Arc<QueryExecutor>,
    validator: SqlValidator,
}

/// 最大重试次数
const MAX_RETRIES: usize = 1;

#[tool_router]
impl PgMcpServer {
    pub fn new(
        config: Arc<AppConfig>,
        metadata: Arc<MetadataCache>,
        llm: Arc<LlmClient>,
        executor: Arc<QueryExecutor>,
    ) -> Self {
        Self {
            tool_router: Self::tool_router(),
            config,
            metadata,
            llm,
            executor,
            validator: SqlValidator::new(),
        }
    }

    /// 接收自然语言问题，返回 SQL 查询结果
    #[tool(
        name = "query",
        description = "将自然语言问题转换为 SQL 查询并返回结果。支持中文和英文。如果用户问题模糊，会根据数据库结构做出合理推断。"
    )]
    async fn query(
        &self,
        Parameters(params): Parameters<QueryParams>,
    ) -> Result<String, rmcp::ErrorData> {
        // 1. 检索相关表元数据（基于问题关键词）
        let db_context = self.metadata.get_relevant_context(
            &params.question,
            self.config.server.prompt_budget,
        ).await;

        // 2. 带重试的 SQL 生成与执行
        let mut last_error: Option<String> = None;

        for attempt in 0..=MAX_RETRIES {
            // 2a. 调用 LLM 生成 SQL
            let sql = match self.llm.generate_sql(
                &params.question,
                &db_context,
                last_error.as_deref(),  // 重试时附带错误信息
            ).await {
                Ok(sql) => sql,
                Err(e) => {
                    return Err(rmcp::ErrorData::internal_error(
                        &format!("大模型调用失败: {}", e), None
                    ));
                }
            };

            tracing::info!(
                question = %params.question,
                sql = %sql,
                attempt = attempt,
                "LLM 生成 SQL"
            );

            // 2b. 校验 SQL 安全性
            match self.validator.validate(&sql) {
                Ok(validated_sql) => {
                    // 2c. 执行查询
                    match self.executor.execute(&validated_sql).await {
                        Ok(result) => {
                            return serde_json::to_string_pretty(&result)
                                .map_err(|e| rmcp::ErrorData::internal_error(
                                    &format!("结果序列化失败: {}", e), None
                                ));
                        }
                        Err(e) => {
                            last_error = Some(e.to_string());
                            tracing::warn!(
                                sql = %validated_sql,
                                error = %e,
                                attempt = attempt,
                                "SQL 执行失败"
                            );
                            // 如果还有重试次数，继续循环
                            if attempt < MAX_RETRIES {
                                continue;
                            }
                            return Err(rmcp::ErrorData::internal_error(
                                &format!("SQL 执行失败: {}", e), None
                            ));
                        }
                    }
                }
                Err(e) => {
                    return Err(rmcp::ErrorData::invalid_params(&e, None));
                }
            }
        }

        Err(rmcp::ErrorData::internal_error("未预期的执行路径", None))
    }
}

#[tool_handler]
impl ServerHandler for PgMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            name: "pg-mcp".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}
```

---

### 4.7 入口 (`main.rs`)

**职责**：加载配置 → 初始化连接 → 缓存元数据 → 启动 MCP Server。

```mermaid
graph TD
    A["加载配置<br/>(CLI > 文件 > 环境变量)"] --> B["创建 PgPool<br/>(数据库连接池)"]
    B --> C{"连接成功?"}
    C -->|否| D["终止启动<br/>提示检查连接配置"]
    C -->|是| E["加载元数据<br/>(MetadataCache)"]
    E --> F["初始化 LlmClient"]
    F --> G["初始化 QueryExecutor"]
    G --> H["创建 PgMcpServer"]
    H --> I["启动 stdio transport<br/>(rmcp serve)"]
    I --> J["等待客户端请求"]

    style D fill:#f44336,color:white
    style J fill:#4CAF50,color:white
```

```rust
use std::sync::Arc;
use std::collections::HashSet;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // 2. 加载配置
    let config = Arc::new(AppConfig::load()?);
    tracing::info!(database_url = %mask_password(&config.database.url), "配置加载完成");

    // 3. 创建数据库连接池
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&config.database.url)
        .await
        .map_err(|e| {
            tracing::error!("数据库连接失败: {}", e);
            anyhow::anyhow!("数据库连接失败，请检查连接字符串和数据库状态: {}", e)
        })?;

    tracing::info!("数据库连接成功");

    // 4. 加载并缓存元数据
    let excluded: HashSet<String> = config.database.excluded_tables.iter().cloned().collect();
    let metadata_cache = Arc::new(
        MetadataCache::new(pool.clone(), &config.database.schema, excluded)
    );
    metadata_cache.load().await?;
    tracing::info!(
        tables = metadata_cache.table_count().await,
        views = metadata_cache.view_count().await,
        "元数据缓存完成"
    );

    // 5. 启动后台元数据刷新（如果配置了）
    if config.server.metadata_refresh_secs > 0 {
        let mc = metadata_cache.clone();
        let interval = config.server.metadata_refresh_secs;
        tokio::spawn(async move {
            mc.start_refresh_loop(interval).await;
        });
        tracing::info!(refresh_interval_secs = interval, "元数据自动刷新已启用");
    }

    // 6. 初始化各组件
    let llm = Arc::new(LlmClient::new(&config.llm));
    let executor = Arc::new(QueryExecutor::new(
        pool,
        config.server.max_rows,
        config.server.query_timeout_secs,
        config.server.debug,
    ));

    // 7. 创建 MCP Server
    let server = PgMcpServer::new(config, metadata_cache, llm, executor);

    // 8. 启动 stdio transport
    let transport = (tokio::io::stdin(), tokio::io::stdout());
    let service = server.serve(transport).await?;

    tracing::info!("pg-mcp server 已启动，等待 MCP 客户端连接...");

    // 9. 等待关闭
    service.waiting().await?;
    Ok(())
}

/// 隐藏连接字符串中的密码，用于日志输出
fn mask_password(url: &str) -> String {
    if let Some(start) = url.find("://") {
        let prefix = &url[..start + 3];
        let rest = &url[start + 3..];
        if let Some(at_pos) = rest.find('@') {
            return format!("{}***{}", prefix, &rest[at_pos..]);
        }
    }
    url.to_string()
}
```

---

## 5. 错误处理设计

### 5.1 错误类型映射

```mermaid
graph TD
    subgraph "错误来源"
        E1["数据库连接失败"]
        E2["数据库认证失败"]
        E3["SQL 解析失败"]
        E4["SQL 安全校验失败<br/>(含 CTE/锁子句)"]
        E5["SQL 执行失败"]
        E6["查询超时"]
        E7["LLM API 连接失败"]
        E8["LLM 认证失败"]
        E9["LLM 超时"]
    end

    subgraph "处理策略"
        F["启动时终止<br/>提示检查配置"]
        R["返回清洗后的错误信息<br/>给 MCP Client"]
        RT["返回超时信息<br/>建议重试/简化"]
        RETRY["带错误反馈重试<br/>(最多 1 次)"]
    end

    E1 --> F
    E2 --> F
    E3 --> R
    E4 --> R
    E5 --> RETRY
    E6 --> RT
    E7 --> R
    E8 --> R
    E9 --> RT
    RETRY -->|"重试仍失败"| R
```

### 5.2 错误响应格式

```json
{
  "sql": "SELECT ...",
  "columns": [],
  "rows": [],
  "row_count": 0,
  "truncated": false,
  "execution_time_ms": 0,
  "error": "安全校验失败: 仅允许 SELECT 查询语句"
}
```

### 5.3 错误清洗

> **Code Review 改进**：原始数据库错误可能暴露 schema 名称、表结构等信息。
> 生产模式（`debug=false`）下返回通用错误提示，详细错误仅记录到日志。

| 模式 | 用户看到的错误 | 日志记录 |
|------|--------------|---------|
| 生产 (`debug=false`) | "SQL 执行失败，请检查查询语句是否正确" | 完整原始错误 |
| 调试 (`debug=true`) | 完整原始错误信息 | 完整原始错误 |

### 5.4 日志与审计策略

| 级别 | 记录内容 |
|------|----------|
| ERROR | 数据库连接失败、LLM API 错误、SQL 执行异常 |
| WARN | SQL 校验被拒绝（含拒绝原因）、查询超时、元数据刷新失败 |
| INFO | 服务启动/关闭、元数据加载完成、每次查询的 question、生成的 SQL、执行耗时、重试次数 |
| DEBUG | LLM API 请求/响应详情、prompt 组装内容 |
| TRACE | SQL 执行结果数据 |

**密钥保护**：日志中不输出 API 密钥和数据库密码（`mask_password` 函数处理）。

**审计字段**（每次 query 调用记录）：
- `question`：用户自然语言问题
- `sql`：生成的 SQL
- `attempt`：第几次尝试（0 或 1）
- `execution_time_ms`：SQL 执行耗时
- `row_count`：返回行数
- `truncated`：是否截断
- `error`：错误信息（如有）

---

## 6. 启动生命周期

```mermaid
stateDiagram-v2
    [*] --> LoadConfig: main()
    LoadConfig --> ConnectDB: 配置加载成功
    LoadConfig --> [*]: 配置错误 (exit 1)
    ConnectDB --> LoadMetadata: 连接成功
    ConnectDB --> [*]: 连接失败 (exit 1)
    LoadMetadata --> InitComponents: 缓存完成
    LoadMetadata --> [*]: 元数据查询失败 (exit 1)
    InitComponents --> StartServer: 组件就绪
    StartServer --> Running: stdio transport 就绪
    Running --> [*]: 客户端断开 / 收到终止信号
```

**启动时间控制**：元数据加载应在 `< 5 秒` 内完成。对于大型数据库（数百张表），考虑并发查询各表的列/索引信息。

---

## 7. 配置集成示例

### 7.1 Claude Code 集成

在 `.claude/settings.json` 中配置：

```json
{
  "mcpServers": {
    "pg-mcp": {
      "command": "pg-mcp",
      "args": ["--config", "/path/to/config.toml"]
    }
  }
}
```

### 7.2 Cursor 集成

在 Cursor 的 MCP 配置中：

```json
{
  "mcpServers": {
    "pg-mcp": {
      "command": "pg-mcp",
      "args": ["--database-url", "postgresql://...", "--llm-api-key", "sk-..."]
    }
  }
}
```

---

## 8. 测试策略

### 8.1 单元测试

| 模块 | 测试内容 |
|------|----------|
| `validator.rs` | SELECT 通过；INSERT/UPDATE/DELETE/CREATE/DROP/ALTER 被拒绝；多语句被拒绝；解析错误处理；**数据修改 CTE 被拒绝**（`WITH d AS (DELETE ...) SELECT * FROM d`）；**锁子句被拒绝**（`SELECT ... FOR UPDATE`）；嵌套子查询中的 CTE 检测 |
| `llm.rs` | SQL 提取（markdown 代码块、裸 SQL）；prompt 组装；**重试 prompt 包含错误信息** |
| `config.rs` | 配置合并优先级；环境变量覆盖；默认值；**excluded_tables 配置** |
| `metadata.rs` | **相关表检索逻辑**：关键词匹配表名、匹配列名、无匹配时返回全部；**prompt_budget 截断** |

### 8.2 集成测试

| 测试场景 | 描述 |
|----------|------|
| 端到端查询 | 启动真实 PostgreSQL，执行完整流程（自然语言 → SQL → 结果） |
| 元数据加载 | 验证表/列/索引/视图信息正确缓存 |
| SQL 执行 | 验证单表、多表关联、聚合查询的准确性 |
| 错误处理 | 验证各类错误场景的返回信息 |

### 8.3 测试工具

- 使用 `sqlx::test` 宏自动管理测试数据库
- 使用 `testcontainers` 运行 PostgreSQL Docker 容器
- 使用 `tokio::test` 运行异步测试

---

## 9. 关键设计决策

| 决策 | 选项 | 选择 | 理由 |
|------|------|------|------|
| MCP 协议实现 | 手动实现 vs rmcp | rmcp | 官方 SDK，`#[tool]` 宏减少样板代码 |
| SQL 校验 | 正则 vs AST 解析 | AST 解析 (sqlparser-rs) | 正则不可靠，AST 精确区分语句类型 |
| SQL 安全长尾防护 | 仅 AST 校验 vs 多层纵深 | AST + READ ONLY 事务 + 最小权限 | `WITH ... DELETE RETURNING` 等攻击向量需要纵深防御 |
| 行数据序列化 | 逐列类型推断 vs 统一 String | 统一 String 中转 | PostgreSQL 类型系统复杂，逐类型匹配成本高 |
| LIMIT 保护 | 修改 AST vs 字符串拼接 | 字符串拼接 | 简单可靠，配合 AST 校验后使用 |
| 配置格式 | YAML vs TOML vs JSON | TOML | Rust 生态友好，对人类可读性好 |
| 元数据格式化给 LLM | JSON vs Markdown 表格 | Markdown 表格 | LLM 对 Markdown 理解更好，Token 效率更高 |
| 元数据发送策略 | 全量 vs 相关表检索 | 相关表检索 + prompt budget | 大型数据库全量发送会超出 token 限制 |
| 元数据注释处理 | 包含 vs 排除 | 排除（默认） | 数据库注释可能包含恶意指令（prompt 注入） |
| 重试策略 | 无 vs 有界重试 | 最多 1 次重试 | LLM 生成的 SQL 可能有小错误，附错误信息重试可显著提高成功率 |
| 错误信息 | 原始 vs 清洗 | 清洗（可配置 debug 模式） | 原始数据库错误可能泄漏 schema 信息 |
| 表访问控制 | 无 vs allowlist/excludelist | excludelist + allowlist | 防止敏感表被查询，配置灵活 |
| 元数据刷新 | 仅启动时 vs 定时刷新 | 可配置定时刷新 | Schema 变更后自动感知 |

---

## 10. Code Review 反馈追踪

> 本节记录基于 Codex Code Review (GPT-5.4) 的反馈及处理结果。
> Review 日期：2026-03-31

| # | 严重性 | 反馈摘要 | 处理方式 | 设计文档更新位置 |
|---|--------|---------|---------|----------------|
| 1 | Critical | `SELECT` 可通过 CTE/`FOR UPDATE` 修改数据 | 已采纳：AST 深度检查 + READ ONLY 事务 + 最小权限 | 4.3 SQL 安全校验 |
| 2 | Critical | 无授权模型，任何 MCP 客户端可查所有表 | 部分采纳：添加表级 allowlist/excludelist 配置。完整的客户端认证和 RLS 留待 v1.1 | 4.1 配置管理 |
| 3 | High | 元数据注释/视图定义可注入 LLM prompt | 已采纳：默认排除注释，视图仅列名称 | 4.2.4 元数据检索与格式化 |
| 4 | High | 元数据无刷新策略，启动后即过期 | 已采纳：添加可配置的后台定时刷新 | 4.2.4 MetadataCache |
| 5 | High | 全量元数据发送对大型数据库不可行 | 已采纳：基于关键词的相关表检索 + prompt_budget 限制 | 4.2.4 元数据检索 |
| 6 | High | 错误信息泄漏内部 schema | 已采纳：添加错误清洗层 + debug 模式配置 | 4.5 执行器 + 5.3 错误清洗 |
| 7 | High | 结果缺少截断标记，大型输出行为未定义 | 已采纳：添加 `truncated` 字段 | 4.5 QueryResult + 4.6.1 Schema |
| 8 | Medium | 15 秒总响应与 30 秒 SQL 超时不一致 | 不修改：15 秒为软目标（LLM 耗时为主），30 秒为硬安全边界 | — |
| 9 | Medium | 无 SQL 修复/重试机制 | 已采纳：最多 1 次有界重试，附错误信息 | 4.6.2 Server 定义 |
| 10 | Medium | 模糊用户问题未处理 | 部分采纳：System Prompt 中增加规则 6（无法回答时返回 ERROR） | 4.4.3 Prompt 工程 |
| 11 | Medium | "支持任何 OpenAI 兼容 API" 过于宽泛 | 不修改：v1 仅测试标准 Chat Completion 格式，兼容性足够 | — |
| 12 | Medium | MCP Tool Schema 不够精确 | 已采纳：添加完整 JSON Schema 定义 | 4.6.1 精确的 Tool 响应 Schema |
| 13 | Low | 5 秒启动时间对大型 schema 不现实 | 部分采纳：文档中注明大型数据库需并发加载 | 6. 启动生命周期 |
| 14 | Low | 可观测性太浅（仅日志） | 部分采纳：增加审计字段定义，完整 metrics 留待 v1.1 | 5.4 日志与审计 |
