# Instructions

## constitution

针对 `./w2-db-query` 项目的规范：

- Rust 使用官方标准规范：The Standard Style
- 前端使用 TypeScript
- 前端后端都要有严格的类型标注。
- 所有后端生成的 JSON 数据，使用 camelCase 格式。
- 不需要 authentication，任何用户都可以使用。
- 文档输出语言以中文为主，英文为辅。

## 基本思路

这是一个数据库查询的桌面应用程序。用户可以添加 db url，系统会自动连接数据库后会获取数据库的 metadata，然后将数据库中的 table, view 等信息展示出来。然后用户可以自己输入 sql 查询，也可以通过自然语言生成 sql 查询。

### 要点

- 用户配置数据库连接等信息和数据库的 metadata 都需要存储到 sqlite 数据库中。可以根据 postgresql 的规范来查询系统中的表和视图的信息，然后用 LLM 来将这些信息转换成 json 格式，然后存储到 sqlite 数据库中。这些信息以后可以复用。
- 当用户使用 LLM 来生成查询的时候，我们可以将系统中的表和视图信息作为 context 传递给 LLM，然后 LLM 会根据这些信息生成 sql 查询。
  - 如果查询不包含 limit 子句，则默认添加 limit 100 子句。
- sql 查询结果应当是 key-value 格式，key 是 column 的名称，value 是 column 的值。UI 页面通过表格的形式将其展示出来。

### 技术栈

后端：Rust / Tauri / tokio-postgres / sqlparser-rs / openai sdk
前端：React / refine 5 / tailwind css / ant design 来实现。sql editor 使用 monaco editor 来实现。

### 配置

OpenAI API Key: 从环境变量 OPENAI_API_KEY 中获取。
db metadata：存储在 sqlite 数据库中，放在 `./db_query.db` 中。

### 后端 API

提供的 Tauri API 接口定义，大致有这些：

```rust
// Tauri 后端 Rust API 示例：定义 tauri::command 函数，供前端调用

/**
    获取所有已连接的数据库
 */
#[tauri::command]
async fn list_databases() -> Result<Vec<DatabaseInfo>, String> {
    todo!()
}

/**
    添加新的数据库连接配置，并立即尝试连接数据库
*/
#[tauri::command]
async fn add_database(config: DatabaseInfo) -> Result<(), String> {
    todo!()
}

/**
    获取某个数据库的 metadata
*/
#[tauri::command]
async fn get_database_metadata(database_id: String) -> Result<DatabaseMetadata, String> {
    todo!()
}

/**
    使用 sql 查询某个数据库，返回查询结果
*/
#[tauri::command]
async fn run_sql_query(database_id: String, sql: String) -> Result<QueryResult, String> {
    todo!()
}

/**
    使用自然语言查询某个数据库，返回查询结果
*/
#[tauri::command]
async fn run_sql_query_with_nl(database_id: String, prompt: String) -> Result<QueryResult, String> {
    todo!()
}
```

**类型定义（供参考，实际项目需详细定义）**:

```rust
struct DatabaseInfo {
    id: String,
    name: String,
    host: String,
    port: u16,
    database_name: String,
    user: String,
    password: String,
}

struct DatabaseMetadata {
    tables: Vec<TableInfo>,
    views: Vec<ViewInfo>,
    // ...
}

struct QueryResult {
    columns: Vec<String>,
    rows: Vec<std::collections::HashMap<String, serde_json::Value>>,
    total: usize,
    exec_time_ms: u64,
    sql: String,
}

```
