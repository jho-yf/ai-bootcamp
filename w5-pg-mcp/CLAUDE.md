# Instructions

## Project Overview

PostgreSQL MCP Server (`pg-mcp`) — a Rust MCP (Model Context Protocol) server that exposes a single `query` tool. It accepts natural language questions, converts them to SQL via an OpenAI-compatible LLM, validates the SQL is read-only (SELECT only), executes it against PostgreSQL, and returns structured results.

**Specs**: `specs/w5/001-postgres-mcp-prd.md` (PRD), `specs/w5/002-postgres-mcp-design.md` (design)

## Rust

- Use **Rust 2024 edition**. Check crate docs for the latest API before use.
- Prefer `mpsc channel` over shared mutable state; for rarely-changing data (e.g. config), prefer `ArcSwap` over `Arc<Mutex<_>>`.
- If a concurrent map is needed, prefer `DashMap` over `Mutex/RwLock<HashMap>`.
- No `unsafe` code. Use `dotenvy` for env vars in tests.
- Never use `unwrap()` or `expect()` in production code — propagate errors with `?` or handle them explicitly. `unwrap()`/`expect()` are acceptable in test-only helper functions.
- Use native `async trait` support (Rust 2024), not the `async_trait` crate.
- Use `thiserror` for library-level error types (validator, executor, metadata, llm modules). Use `anyhow` only in `main.rs` and integration tests as a top-level error catch-all.
- Prefer `&str` parameters over `String` for function inputs. Return `String` only when ownership is needed.
- Use `#[derive(Clone)]` sparingly — only for types that genuinely need it (e.g., `PgMcpServer` because rmcp requires it).
- Use `impl Trait` in return position only when the type is complex and unnameable; otherwise, be explicit.

## Architecture (SOLID / DRY)

### Single Responsibility

Each module owns exactly one domain:

| Module | Responsibility |
|--------|---------------|
| `config.rs` | Configuration loading and merging (CLI > TOML > env > defaults) |
| `metadata.rs` | Database metadata querying and caching |
| `validator.rs` | SQL AST parsing and SELECT-only enforcement |
| `llm.rs` | OpenAI-compatible Chat Completion API client |
| `executor.rs` | SQL execution and result formatting |
| `server.rs` | MCP Server definition (rmcp integration, tool routing) |
| `main.rs` | Application bootstrap and wiring |

Do not cross boundaries: `validator` must not know about `executor`, `llm` must not know about `validator`, etc. `server.rs` is the orchestrator that composes them.

### Dependency Inversion

- Core modules (`validator`, `executor`, `metadata`, `llm`) must not depend on `server.rs` or `main.rs`.
- Share state via `Arc<T>` passed at construction time — no global statics, no `lazy_static!` / `std::sync::OnceLock` for mutable state.
- Keep `PgMcpServer` thin: it delegates to `LlmClient`, `SqlValidator`, `QueryExecutor`. No business logic in the server handler beyond orchestration and error mapping.

### Open/Closed

- Each module exposes a public struct with a clear constructor (`new(...)`) and a small, focused public API (1–3 methods).
- Internal helpers are `pub(crate)` or private. Never expose internal implementation details.

### DRY

- Extract repeated SQL query patterns in `metadata.rs` into helper functions.
- Extract the JSON row-serialization logic in `executor.rs` into a dedicated function or small module to keep `execute()` readable.
- Share error message formatting patterns — don't scatter `format!("...")` strings across modules.

## Code Quality

### Error Handling

- Define a typed error enum per domain module using `thiserror`:

```rust
// In validator.rs
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("SQL parse error: {0}")]
    ParseError(String),
    #[error("Security check failed: {0}")]
    NotSelect(String),
    #[error("Empty SQL")]
    Empty,
    #[error("Multiple statements not allowed")]
    MultipleStatements,
}
```

- Map domain errors to `rmcp::ErrorData` in `server.rs` only — the mapping layer lives in one place.
- Never discard an error with `let _ = ...` unless the operation is explicitly fire-and-forget with a comment explaining why.

### Logging

- Use `tracing` macros (`tracing::info!`, `tracing::warn!`, `tracing::error!`, `tracing::debug!`) with structured fields.
- **Never log API keys or database passwords.** Use `mask_password()` for connection strings in log output.
- Log every query: `tracing::info!(question = %q, sql = %sql, elapsed_ms = %ms, "query executed")`.
- Use `tracing::debug!` for LLM request/response bodies. Use `tracing::trace!` for raw row data.

### Input Validation

- `config.rs`: validate `temperature` is in `[0.0, 2.0]`, `max_tokens > 0`, `url` is non-empty, `schema` is non-empty. Return clear error messages on invalid config.
- `validator.rs`: reject empty SQL, reject multi-statement SQL, reject anything that is not `Statement::Query(_)` via `sqlparser-rs` AST analysis.
- `executor.rs`: always append `LIMIT` if the SQL does not already contain one, to prevent unbounded result sets.

### Security

- SQL validation via AST parsing (not regex) — regex is unreliable for distinguishing statement types.
- The `LIMIT` protection is applied after AST validation; string-based approach is acceptable here because the SQL has already been verified as a single SELECT statement.
- The database connection should use the minimum-privilege principle — recommend read-only DB users.

## Testing

### Test Categories

1. **Unit tests** (`#[cfg(test)] mod tests` within each module file):
   - `validator.rs`: SELECT passes; INSERT/UPDATE/DELETE/CREATE/DROP/ALTER/TRUNCATE/GRANT/COMMIT are rejected; multi-statement rejected; empty input rejected; parse errors handled; SQL with subqueries containing non-SELECT (e.g., `INSERT ... SELECT`) are rejected.
   - `llm.rs`: `extract_sql()` handles ` ```sql...``` `, ` ```...``` `, and bare SQL; prompt assembly correctness.
   - `config.rs`: merge priority (CLI > file > env > default); missing required fields produce clear errors; type validation.
   - `executor.rs`: `apply_limit()` adds LIMIT when missing; preserves existing LIMIT; handles trailing semicolons.

2. **Integration tests** (`tests/` directory):
   - `test_metadata.rs`: Use `testcontainers` (PostgreSQL) to verify metadata loading — tables, columns, PKs, indexes, views are all correctly cached.
   - `test_integration.rs`: End-to-end flow with a real PostgreSQL — seed test data, call the query pipeline, verify SQL correctness and result structure.

3. **Property-based testing** (optional but encouraged):
   - For `validator.rs`, consider `proptest` to fuzz arbitrary SQL strings and confirm no panics and correct allow/deny behavior.

### Test Quality Rules

- Every `pub` function must have at least one test.
- Test names must describe the scenario: `test_reject_insert_statement`, `test_extract_sql_from_markdown_code_block`, `test_config_cli_overrides_file`.
- Use `tokio::test` for async tests.
- Use `testcontainers` for PostgreSQL-dependent tests — never require a manually running database.
- Tests must be deterministic: no `sleep()`, no reliance on wall-clock ordering.
- For error paths, assert both the error kind/type and the error message content (or at least a substring).
- Use `#[serial_test::serial]` only when truly necessary (e.g., env var mutation); prefer isolated test fixtures.

## Performance

- **Connection pooling**: Use `sqlx::postgres::PgPoolOptions` with `max_connections(10)` and `acquire_timeout(5s)`. The pool is shared via `Arc` — do not clone the pool, clone the `Arc`.
- **Metadata loading**: Query tables, then fan out column/PK/index queries concurrently using `tokio::join!` or `futures::join_all` per table. Target < 5s total startup for databases with up to 100 tables.
- **LLM context**: Pre-format `DatabaseMetadata::to_llm_context()` once at startup and cache the resulting `String` in an `Arc<String>`. Do not re-format on every query.
- **Query execution**: Set `statement_timeout` via `SET LOCAL statement_timeout` in each query transaction, or append a timeout using `tokio::time::timeout` wrapping the query.
- **Row serialization**: Pre-allocate `Vec::with_capacity(row_count)` when the row count is known. Use `serde_json::to_string` once — avoid repeated serialization.
- **Avoid unnecessary clones**: Pass references (`&str`, `&DatabaseMetadata`) into functions. Clone only at ownership boundaries (e.g., `rmcp` requires `Clone` on the server).
- **No blocking in async context**: Never call `std::thread::sleep` or blocking I/O inside async functions. Use `tokio::fs`, `tokio::time::sleep`, etc.

## Module API Contracts

```
config.rs
  pub struct AppConfig { pub database: DatabaseConfig, pub llm: LlmConfig, pub server: ServerConfig }
  pub struct DatabaseConfig { pub url: String, pub schema: String }
  pub struct LlmConfig { pub api_url, api_key, model: String, pub temperature: f32, pub max_tokens: u32 }
  pub struct ServerConfig { pub max_rows: u32, pub query_timeout_secs: u64 }
  impl AppConfig { pub fn load() -> Result<Self> }

metadata.rs
  pub struct DatabaseMetadata { pub schema_name: String, pub tables: Vec<TableInfo>, pub views: Vec<ViewInfo> }
  impl DatabaseMetadata { pub fn to_llm_context(&self) -> String }
  pub struct MetadataLoader;
  impl MetadataLoader { pub async fn load(pool: &PgPool, schema: &str) -> Result<DatabaseMetadata> }

validator.rs
  pub enum ValidationError { ParseError(String), NotSelect(String), Empty, MultipleStatements }
  pub struct SqlValidator;
  impl SqlValidator { pub fn validate(&self, sql: &str) -> Result<ValidatedSql, ValidationError> }

llm.rs
  pub struct LlmClient { /* private fields */ }
  impl LlmClient { pub fn new(config: &LlmConfig) -> Self; pub async fn generate_sql(&self, question: &str, db_context: &str) -> Result<String> }

executor.rs
  pub struct QueryResult { pub sql: String, pub columns: Vec<String>, pub rows: Vec<serde_json::Value>, pub row_count: usize, pub execution_time_ms: u64 }
  pub struct QueryExecutor { /* private fields */ }
  impl QueryExecutor { pub fn new(pool: PgPool, max_rows: u32, query_timeout_secs: u64) -> Self; pub async fn execute(&self, sql: &str) -> Result<QueryResult> }

server.rs
  pub struct PgMcpServer { /* private fields */ }
  impl PgMcpServer { pub fn new(metadata, llm, executor) -> Self }
  // #[tool] query(question) — auto-routed via rmcp macros
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `rmcp` | 0.16 | MCP protocol (server, transport-io, macros, schemars features) |
| `sqlx` | 0.8 | Async PostgreSQL driver (runtime-tokio, postgres, chrono, uuid features) |
| `sqlparser-rs` | 0.53 | SQL AST parsing for security validation |
| `tokio` | 1 | Async runtime (full feature) |
| `serde` / `serde_json` | 1 | Serialization |
| `schemars` | 1 | JSON Schema generation for MCP tool params |
| `reqwest` | 0.12 | HTTP client for LLM API (json feature) |
| `toml` | 0.8 | Config file parsing |
| `clap` | 4 | CLI args (derive, env features) |
| `tracing` / `tracing-subscriber` | 0.1 | Structured logging (env-filter feature) |
| `thiserror` | 2 | Typed error enums |
| `anyhow` | 1 | Top-level error handling (main.rs / tests only) |

Dev dependencies: `testcontainers`, `testcontainers-modules` (postgres), `tokio-test`, `proptest` (optional)
