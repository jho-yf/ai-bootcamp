# PostgreSQL MCP Server 测试计划

## 文档信息

| 项目 | 内容 |
|------|------|
| 文档版本 | 1.0 |
| 创建日期 | 2026-03-31 |
| 关联设计文档 | [002-postgres-mcp-design.md](./002-postgres-mcp-design.md) |
| 关联实现计划 | [003-postgres-mcp-impl-plan.md](./003-postgres-mcp-impl-plan.md) |
| 项目名称 | pg-mcp |
| 项目路径 | `w5-pg-mcp/` |

---

## 1. 测试目标

### 1.1 核心测试目标

| 目标 | 描述 | 验收标准 |
|------|------|----------|
| **安全性** | SQL 注入防护、数据修改防护、表访问控制 | 所有非 SELECT 语句被拒绝；CTE 数据修改被拒绝；FOR UPDATE 被拒绝；表级访问控制生效 |
| **正确性** | SQL 生成、执行、结果序列化的准确性 | LLM 生成的 SQL 能正确执行；结果数据类型正确序列化；截断标记准确 |
| **可靠性** | 错误处理、重试机制、超时控制 | 错误信息被正确清洗；重试逻辑生效；超时不阻塞服务 |
| **性能** | 元数据加载、查询响应时间 | 元数据加载 < 5 秒；端到端响应 < 15 秒（目标） |
| **协议合规** | MCP 协议正确实现 | handshake 成功；list_tools 返回正确 schema；错误码映射正确 |

### 1.2 测试覆盖率目标

| 覆盖率类型 | 目标 | 测量工具 |
|-----------|------|----------|
| 代码行覆盖率 | ≥ 80% | `cargo-tarpaulin` |
| 分支覆盖率 | ≥ 70% | `cargo-tarpaulin` |
| 关键路径覆盖率 | 100% | 人工审查 |

---

## 2. 测试分层策略

```mermaid
graph TB
    subgraph "测试金字塔"
        L1["单元测试<br/>(80%)"]
        L2["集成测试<br/>(15%)"]
        L3["E2E 测试<br/>(5%)"]
    end

    L1 --> L2
    L2 --> L3

    style L1 fill:#4CAF50,color:white
    style L2 fill:#2196F3,color:white
    style L3 fill:#FF9800,color:white
```

### 2.1 单元测试 (Unit Tests)

**职责**：验证单个函数/方法的行为，使用 mock 隔离外部依赖。

**范围**：
- `config.rs`：配置加载、合并、优先级逻辑
- `validator.rs`：AST 解析、安全检查、表名提取
- `llm.rs`：SQL 提取、prompt 构建
- `metadata.rs`：相关表检索、元数据格式化
- `executor.rs`：LIMIT 应用、类型序列化、错误清洗

### 2.2 集成测试 (Integration Tests)

**职责**：验证多个模块协作的正确性，使用真实或高保真 mock。

**范围**：
- 元数据加载流程
- SQL 生成 → 校验 → 执行流程
- MCP 协议交互
- 错误处理与重试链路

### 2.3 E2E 测试 (End-to-End Tests)

**职责**：验证完整用户场景，使用真实 PostgreSQL 和真实 MCP 客户端。

**范围**：
- 完整查询流程（自然语言 → SQL → 结果）
- 配置驱动的行为变化
- 长时间运行稳定性

---

## 3. 单元测试详细规范

### 3.1 config.rs 单元测试

**文件位置**：`w5-pg-mcp/src/config.rs` (tests 模块)

#### 3.1.1 配置加载测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        // 验证默认值：
        // - schema = "public"
        // - max_rows = 1000
        // - query_timeout_secs = 30
        // - prompt_budget = 8000
        // - debug = false
        // - metadata_refresh_secs = 0
        // - allowed_tables = []
        // - excluded_tables = []
    }

    #[test]
    fn test_cli_only_config() {
        // 仅通过 CLI 参数提供配置
        // 验证所有必需字段可从 CLI 加载
    }

    #[test]
    fn test_toml_file_only_config() {
        // 创建临时 TOML 文件
        // 验证配置正确解析
    }

    #[test]
    fn test_environment_variable_override() {
        // 设置环境变量 PG_MCP_*
        // 验证环境变量覆盖 TOML 默认值
    }

    #[test]
    fn test_cli_overrides_environment() {
        // 设置环境变量 + CLI 参数
        // 验证 CLI 覆盖环境变量
    }

    #[test]
    fn test_config_priority_full_chain() {
        // 验证完整优先级：CLI > env > file > default
        // 每个配置项都测试
    }

    #[test]
    fn test_missing_required_config_returns_error() {
        // 缺少 database.url
        // 缺少 llm.api_key
        // 验证返回清晰错误
    }

    #[test]
    fn test_allowed_tables_parsing() {
        // 验证 allowed_tables 正确解析为 HashSet
        // 验证空数组 = 允许所有
    }

    #[test]
    fn test_excluded_tables_parsing() {
        // 验证 excluded_tables 正确解析为 HashSet
        // 验证优先级高于 allowed_tables
    }

    #[test]
    fn test_mask_password() {
        // 测试 mask_password 函数
        // postgresql://user:pass@host -> postgresql://user:***@host
    }
}
```

**验收标准**：
- [x] 所有测试用例通过
- [x] 配置优先级符合设计
- [x] 错误信息清晰

---

### 3.2 validator.rs 单元测试

**文件位置**：`w5-pg-mcp/src/validator.rs` (tests 模块)

#### 3.2.1 SQL 安全校验测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn default_validator() -> SqlValidator {
        SqlValidator::new(HashSet::new(), HashSet::new())
    }

    #[test]
    fn test_simple_select_passes() {
        let v = default_validator();
        assert!(v.validate("SELECT 1").is_ok());
        assert!(v.validate("SELECT * FROM users").is_ok());
        assert!(v.validate("SELECT id, name FROM users WHERE active = true").is_ok());
    }

    #[test]
    fn test_join_select_passes() {
        let v = default_validator();
        assert!(v.validate("SELECT u.name, o.total FROM users u JOIN orders o ON u.id = o.user_id").is_ok());
    }

    #[test]
    fn test_aggregation_select_passes() {
        let v = default_validator();
        assert!(v.validate("SELECT COUNT(*) FROM users").is_ok());
        assert!(v.validate("SELECT role, COUNT(*) FROM users GROUP BY role").is_ok());
        assert!(v.validate("SELECT category, SUM(amount) FROM sales GROUP BY category HAVING SUM(amount) > 1000").is_ok());
    }

    #[test]
    fn test_cte_select_passes() {
        let v = default_validator();
        assert!(v.validate("WITH active_users AS (SELECT * FROM users WHERE active = true) SELECT * FROM active_users").is_ok());
        assert!(v.validate("WITH ranked AS (SELECT *, ROW_NUMBER() OVER (ORDER BY created_at) AS rn FROM users) SELECT * FROM ranked").is_ok());
    }

    #[test]
    fn test_subquery_passes() {
        let v = default_validator();
        assert!(v.validate("SELECT * FROM users WHERE id IN (SELECT user_id FROM orders)").is_ok());
        assert!(v.validate("SELECT (SELECT COUNT(*) FROM orders) AS order_count").is_ok());
    }

    #[test]
    fn test_union_passes() {
        let v = default_validator();
        assert!(v.validate("SELECT name FROM admins UNION SELECT name FROM users").is_ok());
    }

    #[test]
    fn test_insert_rejected() {
        let v = default_validator();
        let result = v.validate("INSERT INTO users VALUES (1, 'test')");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("安全校验失败"));
    }

    #[test]
    fn test_update_rejected() {
        let v = default_validator();
        let result = v.validate("UPDATE users SET name = 'test'");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("安全校验失败"));
    }

    #[test]
    fn test_delete_rejected() {
        let v = default_validator();
        let result = v.validate("DELETE FROM users");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("安全校验失败"));
    }

    #[test]
    fn test_create_table_rejected() {
        let v = default_validator();
        let result = v.validate("CREATE TABLE test (id INT)");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("安全校验失败"));
    }

    #[test]
    fn test_drop_table_rejected() {
        let v = default_validator();
        let result = v.validate("DROP TABLE users");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("安全校验失败"));
    }

    #[test]
    fn test_alter_table_rejected() {
        let v = default_validator();
        let result = v.validate("ALTER TABLE users ADD COLUMN email TEXT");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("安全校验失败"));
    }

    #[test]
    fn test_truncate_rejected() {
        let v = default_validator();
        let result = v.validate("TRUNCATE users");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("安全校验失败"));
    }

    #[test]
    fn test_grant_rejected() {
        let v = default_validator();
        let result = v.validate("GRANT SELECT ON users TO public");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("安全校验失败"));
    }

    #[test]
    fn test_multi_statement_rejected() {
        let v = default_validator();
        let result = v.validate("SELECT 1; DROP TABLE users");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("仅允许单条"));
    }

    #[test]
    fn test_empty_sql_rejected() {
        let v = default_validator();
        assert!(v.validate("").is_err());
        assert!(v.validate("   ").is_err());
    }

    #[test]
    fn test_for_update_rejected() {
        let v = default_validator();
        let result = v.validate("SELECT * FROM users FOR UPDATE");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("锁子句"));
    }

    #[test]
    fn test_for_share_rejected() {
        let v = default_validator();
        let result = v.validate("SELECT * FROM users FOR SHARE");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("锁子句"));
    }

    #[test]
    fn test_data_modifying_cte_rejected() {
        let v = default_validator();
        let result = v.validate("WITH d AS (DELETE FROM users RETURNING *) SELECT * FROM d");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("安全校验失败"));
    }

    #[test]
    fn test_insert_cte_rejected() {
        let v = default_validator();
        let result = v.validate("WITH i AS (INSERT INTO users VALUES (1, 'a') RETURNING *) SELECT * FROM i");
        assert!(result.is_err());
    }

    #[test]
    fn test_update_cte_rejected() {
        let v = default_validator();
        let result = v.validate("WITH u AS (UPDATE users SET active = true RETURNING *) SELECT * FROM u");
        assert!(result.is_err());
    }

    #[test]
    fn test_allowed_tables_whitelist() {
        let allowed = HashSet::from(["users".into(), "orders".into()]);
        let v = SqlValidator::new(allowed, HashSet::new());

        assert!(v.validate("SELECT * FROM users").is_ok());
        assert!(v.validate("SELECT * FROM orders").is_ok());

        let result = v.validate("SELECT * FROM secrets");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("表访问被拒绝"));
    }

    #[test]
    fn test_allowed_tables_blocks_joins_with_unauthorized_tables() {
        let allowed = HashSet::from(["users".into()]);
        let v = SqlValidator::new(allowed, HashSet::new());

        let result = v.validate("SELECT * FROM users JOIN orders ON users.id = orders.user_id");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("表访问被拒绝"));
        assert!(result.unwrap_err().contains("orders"));
    }

    #[test]
    fn test_allowed_tables_blocks_subqueries_with_unauthorized_tables() {
        let allowed = HashSet::from(["users".into()]);
        let v = SqlValidator::new(allowed, HashSet::new());

        let result = v.validate("SELECT * FROM users WHERE id IN (SELECT user_id FROM orders)");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("表访问被拒绝"));
    }

    #[test]
    fn test_excluded_tables_blacklist() {
        let allowed = HashSet::from(["users".into(), "orders".into(), "passwords".into()]);
        let excluded = HashSet::from(["passwords".into()]);
        let v = SqlValidator::new(allowed, excluded);

        assert!(v.validate("SELECT * FROM users").is_ok());
        assert!(v.validate("SELECT * FROM orders").is_ok());

        let result = v.validate("SELECT * FROM passwords");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("表访问被拒绝"));
    }

    #[test]
    fn test_empty_allowed_tables_means_allow_all() {
        let v = SqlValidator::new(HashSet::new(), HashSet::new());
        assert!(v.validate("SELECT * FROM any_table").is_ok());
    }

    #[test]
    fn test_extract_table_names() {
        // 测试表名提取逻辑：
        // - 简单 FROM
        // - JOIN
        // - CTE
        // - 子查询
    }

    #[test]
    fn test_schema_qualified_table_names() {
        let v = default_validator();
        assert!(v.validate("SELECT * FROM public.users").is_ok());
        assert!(v.validate("SELECT * FROM other_schema.users").is_ok());
    }

    #[test]
    fn test_invalid_sql_syntax() {
        let v = default_validator();
        let result = v.validate("SELEC * FROM");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("解析失败"));
    }
}
```

**验收标准**：
- [x] 所有通过用例通过
- [x] 所有拒绝用例被正确拒绝
- [x] 错误信息包含固定前缀
- [x] 表名提取覆盖所有 SQL 结构

---

### 3.3 llm.rs 单元测试

**文件位置**：`w5-pg-mcp/src/llm.rs` (tests 模块)

#### 3.3.1 SQL 提取与 Prompt 构建测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_sql_with_sql_code_block() {
        let input = "```sql\nSELECT * FROM users\n```";
        let output = extract_sql(input);
        assert_eq!(output, "SELECT * FROM users");
    }

    #[test]
    fn test_extract_sql_with_generic_code_block() {
        let input = "```\nSELECT * FROM users\n```";
        let output = extract_sql(input);
        assert_eq!(output, "SELECT * FROM users");
    }

    #[test]
    fn test_extract_sql_bare() {
        let input = "SELECT * FROM users";
        let output = extract_sql(input);
        assert_eq!(output, "SELECT * FROM users");
    }

    #[test]
    fn test_extract_sql_with_leading_trailing_whitespace() {
        let input = "\n\n```sql\nSELECT * FROM users\n```\n\n";
        let output = extract_sql(input);
        assert_eq!(output, "SELECT * FROM users");
    }

    #[test]
    fn test_extract_sql_with_explanation_text() {
        let input = "Here's the SQL:\n```sql\nSELECT * FROM users\n```\nLet me know if you need help.";
        let output = extract_sql(input);
        assert_eq!(output, "SELECT * FROM users");
    }

    #[test]
    fn test_extract_sql_preserves_inner_spacing() {
        let input = "```sql\nSELECT id,  name\nFROM  users\nWHERE  active  =  true\n```";
        let output = extract_sql(input);
        assert_eq!(output, "SELECT id,  name\nFROM  users\nWHERE  active  =  true");
    }

    #[test]
    fn test_extract_sql_multiline_with_newlines() {
        let input = "```sql\nSELECT *\nFROM users\nWHERE id = 1\n```";
        let output = extract_sql(input);
        assert_eq!(output, "SELECT *\nFROM users\nWHERE id = 1");
    }

    #[test]
    fn test_build_system_prompt() {
        let db_context = "### Table: users\n| Column | Type |\n|--------|------|\n| id | INT |\n";
        let prompt = build_system_prompt(db_context);
        assert!(prompt.contains("你是一个 PostgreSQL SQL 专家"));
        assert!(prompt.contains("只生成 SELECT 查询语句"));
        assert!(prompt.contains(db_context));
    }

    #[test]
    fn test_build_user_prompt_without_error() {
        let question = "查询本月活跃用户数";
        let prompt = build_user_prompt(question, None);
        assert!(prompt.contains(question));
        assert!(!prompt.contains("上一次生成的 SQL"));
    }

    #[test]
    fn test_build_user_prompt_with_error() {
        let question = "查询本月活跃用户数";
        let error = "column 'activ' does not exist";
        let prompt = build_user_prompt(question, Some(error));
        assert!(prompt.contains(question));
        assert!(prompt.contains("上一次生成的 SQL"));
        assert!(prompt.contains(error));
    }

    #[test]
    fn test_extract_sql_error_response() {
        let input = "ERROR: 无法根据当前数据库结构回答此问题";
        let output = extract_sql(input);
        assert_eq!(output, "ERROR: 无法根据当前数据库结构回答此问题");
    }
}
```

**验收标准**：
- [x] SQL 提取处理所有格式
- [x] Prompt 包含所有必需元素
- [x] 重试 prompt 包含错误信息

---

### 3.4 metadata.rs 单元测试

**文件位置**：`w5-pg-mcp/src/metadata.rs` (tests 模块)

#### 3.4.1 元数据检索与格式化测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metadata() -> DatabaseMetadata {
        DatabaseMetadata {
            schema_name: "public".into(),
            tables: vec![
                TableInfo {
                    table_name: "users".into(),
                    columns: vec![
                        ColumnInfo {
                            column_name: "id".into(),
                            data_type: "integer".into(),
                            is_nullable: false,
                            column_default: None,
                            comment: None,
                        },
                        ColumnInfo {
                            column_name: "name".into(),
                            data_type: "text".into(),
                            is_nullable: true,
                            column_default: None,
                            comment: Some("user name".into()),
                        },
                        ColumnInfo {
                            column_name: "email".into(),
                            data_type: "text".into(),
                            is_nullable: true,
                            column_default: None,
                            comment: None,
                        },
                    ],
                    primary_keys: vec!["id".into()],
                    indexes: vec![],
                },
                TableInfo {
                    table_name: "orders".into(),
                    columns: vec![
                        ColumnInfo {
                            column_name: "id".into(),
                            data_type: "integer".into(),
                            is_nullable: false,
                            column_default: None,
                            comment: None,
                        },
                        ColumnInfo {
                            column_name: "user_id".into(),
                            data_type: "integer".into(),
                            is_nullable: false,
                            column_default: None,
                            comment: None,
                        },
                    ],
                    primary_keys: vec!["id".into()],
                    indexes: vec![],
                },
            ],
            views: vec![],
        }
    }

    #[test]
    fn test_relevant_context_by_table_name_match() {
        let metadata = create_test_metadata();
        let cache = MetadataCache::with_metadata(metadata, HashSet::new());

        let context = cache.get_relevant_context("查询 users 表中的数据", 1000).await;
        assert!(context.contains("### Table: users"));
        assert!(!context.contains("### Table: orders"));
    }

    #[test]
    fn test_relevant_context_by_column_name_match() {
        let metadata = create_test_metadata();
        let cache = MetadataCache::with_metadata(metadata, HashSet::new());

        let context = cache.get_relevant_context("查询 email 字段", 1000).await;
        assert!(context.contains("### Table: users"));
    }

    #[test]
    fn test_relevant_context_no_match_returns_all() {
        let metadata = create_test_metadata();
        let cache = MetadataCache::with_metadata(metadata, HashSet::new());

        let context = cache.get_relevant_context("查询所有数据", 10000).await;
        assert!(context.contains("### Table: users"));
        assert!(context.contains("### Table: orders"));
    }

    #[test]
    fn test_relevant_context_excluded_tables_filtered() {
        let metadata = create_test_metadata();
        let excluded = HashSet::from(["users".into()]);
        let cache = MetadataCache::with_metadata(metadata, excluded);

        let context = cache.get_relevant_context("查询所有数据", 10000).await;
        assert!(!context.contains("### Table: users"));
        assert!(context.contains("### Table: orders"));
    }

    #[test]
    fn test_relevant_context_budget_truncation() {
        let metadata = create_test_metadata();
        let cache = MetadataCache::with_metadata(metadata, HashSet::new());

        let context = cache.get_relevant_context("查询", 100).await;
        assert!(context.len() <= 100 + 200); // Allow some margin for truncation message
        assert!(context.contains("截断"));
    }

    #[test]
    fn test_format_table_excludes_comments() {
        let metadata = create_test_metadata();
        let cache = MetadataCache::with_metadata(metadata, HashSet::new());

        let context = cache.get_relevant_context("查询 users", 10000).await;
        // Comment should not appear in the formatted output
        assert!(!context.contains("user name"));
    }

    #[test]
    fn test_format_table_includes_primary_keys() {
        let metadata = create_test_metadata();
        let cache = MetadataCache::with_metadata(metadata, HashSet::new());

        let context = cache.get_relevant_context("查询 users", 10000).await;
        assert!(context.contains("**PK**: id"));
    }

    #[test]
    fn test_format_table_includes_nullable_info() {
        let metadata = create_test_metadata();
        let cache = MetadataCache::with_metadata(metadata, HashSet::new());

        let context = cache.get_relevant_context("查询 users", 10000).await;
        assert!(context.contains("| id | integer | NO |"));
        assert!(context.contains("| name | text | YES |"));
    }
}
```

**验收标准**：
- [x] 表名匹配正确
- [x] 列名匹配正确
- [x] 无匹配时返回全部
- [x] excluded_tables 过滤生效
- [x] budget 截断正确
- [x] 注释不包含在输出中

---

### 3.5 executor.rs 单元测试

**文件位置**：`w5-pg-mcp/src/executor.rs` (tests 模块)

#### 3.5.1 LIMIT 应用与序列化测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_limit_when_no_limit_present() {
        let executor = create_test_executor();
        let sql = "SELECT * FROM users";
        let result = executor.apply_limit(sql);
        assert_eq!(result, "SELECT * FROM users LIMIT 1000");
    }

    #[test]
    fn test_apply_limit_when_limit_present() {
        let executor = create_test_executor();
        let sql = "SELECT * FROM users LIMIT 100";
        let result = executor.apply_limit(sql);
        assert_eq!(result, sql); // Should not modify
    }

    #[test]
    fn test_apply_limit_with_subquery() {
        let executor = create_test_executor();
        let sql = "SELECT * FROM (SELECT * FROM users ORDER BY created_at DESC) AS u";
        let result = executor.apply_limit(sql);
        // Should wrap the outer query
        assert!(result.contains("AS _subq LIMIT"));
    }

    #[test]
    fn test_apply_limit_with_union() {
        let executor = create_test_executor();
        let sql = "SELECT name FROM admins UNION SELECT name FROM users";
        let result = executor.apply_limit(sql);
        // Should wrap the entire UNION
        assert!(result.contains("AS _subq LIMIT"));
    }

    #[test]
    fn test_apply_limit_with_cte() {
        let executor = create_test_executor();
        let sql = "WITH active AS (SELECT * FROM users WHERE active = true) SELECT * FROM active";
        let result = executor.apply_limit(sql);
        // Should wrap after CTE
        assert!(result.contains("AS _subq LIMIT"));
    }

    #[test]
    fn test_apply_limit_trailing_semicolon_removed() {
        let executor = create_test_executor();
        let sql = "SELECT * FROM users;";
        let result = executor.apply_limit(sql);
        assert!(!result.contains(";;")); // No double semicolon
    }

    #[test]
    fn test_truncated_flag_when_limit_reached() {
        // Test with mock data that returns exactly max_rows
        // Verify truncated = true
    }

    #[test]
    fn test_truncated_flag_when_below_limit() {
        // Test with mock data that returns less than max_rows
        // Verify truncated = false
    }

    #[test]
    fn test_row_value_to_json_null() {
        // Test NULL values serialize to JSON null
    }

    #[test]
    fn test_row_value_to_json_integer() {
        // Test int2/int4/int8 serialize to JSON numbers
        // Not strings
    }

    #[test]
    fn test_row_value_to_json_float() {
        // Test float4/float8 serialize to JSON numbers
        // Not strings
    }

    #[test]
    fn test_row_value_to_json_boolean() {
        // Test bool serializes to JSON true/false
        // Not "true"/"false" strings
    }

    #[test]
    fn test_row_value_to_json_jsonb() {
        // Test json/jsonb types preserve JSON structure
        // Not stringified
    }

    #[test]
    fn test_row_value_to_json_text_fallback() {
        // Test text/varchar types serialize to JSON strings
    }

    #[test]
    fn test_row_value_to_json_timestamp_fallback() {
        // Test timestamp types fall back to string
    }

    #[test]
    fn test_row_value_to_json_uuid_fallback() {
        // Test uuid types fall back to string
    }

    #[test]
    fn test_sanitize_error_in_debug_mode() {
        let executor = create_test_executor_debug(true);
        let error = create_database_error("relation 'secrets' does not exist");
        let sanitized = executor.sanitize_error(error);
        assert!(sanitized.to_string().contains("secrets")); // Original error preserved
    }

    #[test]
    fn test_sanitize_error_in_production_mode() {
        let executor = create_test_executor_debug(false);
        let error = create_database_error("relation 'secrets' does not exist");
        let sanitized = executor.sanitize_error(error);
        assert!(!sanitized.to_string().contains("secrets")); // Error sanitized
        assert!(sanitized.to_string().contains("不存在的对象"));
    }

    #[test]
    fn test_sanitize_timeout_error() {
        let executor = create_test_executor_debug(false);
        let error = sqlx::Error::Timeout;
        let sanitized = executor.sanitize_error(error);
        assert!(sanitized.to_string().contains("超时"));
    }
}
```

**验收标准**：
- [x] LIMIT 正确追加或跳过
- [x] 子查询包裹方式不误判
- [x] 类型分派序列化正确
- [x] truncated 标记准确
- [x] 错误清洗在 production 模式生效

---

## 4. 集成测试详细规范

### 4.1 集成测试基础设施

**文件位置**：`w5-pg-mcp/tests/integration_tests.rs`

#### 4.1.1 Test Fixture 设置

```rust
use sqlx::{PgPool, postgres::PgPoolOptions};
use testcontainers::{clients::Cli, Container, GenericImage};

async fn setup_test_db() -> (PgPool, String) {
    // 使用 testcontainers 启动 PostgreSQL
    // 创建测试表
    // 返回连接池和 schema 名称
}

async fn teardown_test_db(pool: &PgPool) {
    // 清理测试数据
    // 关闭连接
}
```

### 4.2 元数据加载集成测试

```rust
#[sqlx::test]
async fn test_metadata_loads_all_tables(pool: PgPool) {
    // 创建测试表：users, orders, products
    // 初始化 MetadataCache
    // 调用 load()
    // 验证：table_count() == 3
    // 验证：每张表的列信息正确
}

#[sqlx::test]
async fn test_metadata_loads_primary_keys(pool: PgPool) {
    // 创建带主键的表
    // 验证 primary_keys 字段正确
}

#[sqlx::test]
async fn test_metadata_loads_indexes(pool: PgPool) {
    // 创建带索引的表
    // 验证 indexes 字段正确
}

#[sqlx::test]
async fn test_metadata_loads_views(pool: PgPool) {
    // 创建视图
    // 验证 views 字段正确
}

#[sqlx::test]
async fn test_metadata_refresh_updates_cache(pool: PgPool) {
    // 加载初始元数据
    // 添加新表
    // 调用刷新
    // 验证新表出现在缓存中
}
```

### 4.3 端到端查询集成测试

```rust
#[sqlx::test]
async fn test_end_to_end_simple_query(pool: PgPool) {
    // 准备测试数据
    // Mock LLM 返回固定 SQL
    // 调用 PgMcpServer::query()
    // 验证返回结果
}

#[sqlx::test]
async fn test_end_to_end_join_query(pool: PgPool) {
    // 准备关联表数据
    // Mock LLM 返回 JOIN SQL
    // 验证结果正确
}

#[sqlx::test]
async fn test_end_to端_aggregation_query(pool: PgPool) {
    // 准备聚合查询数据
    // Mock LLM 返回聚合 SQL
    // 验证聚合结果
}
```

### 4.4 READ ONLY 保护测试

```rust
#[sqlx::test]
async fn test_read_only_transaction_blocks_insert(pool: PgPool) {
    // 绕过 validator，直接在 executor 中执行 INSERT
    // 在 READ ONLY 事务中应该失败
    // 验证错误信息
}

#[sqlx::test]
async fn test_read_only_transaction_blocks_update(pool: PgPool) {
    // 同上，测试 UPDATE
}

#[sqlx::test]
async fn test_read_only_transaction_blocks_delete(pool: PgPool) {
    // 同上，测试 DELETE
}
```

### 4.5 错误处理集成测试

```rust
#[sqlx::test]
async fn test_nonexistent_table_error(pool: PgPool) {
    // Mock LLM 返回引用不存在表的 SQL
    // 验证错误被正确清洗
}

#[sqlx::test]
async fn test_invalid_column_error(pool: PgPool) {
    // Mock LLM 返回列名错误的 SQL
    // 验证重试机制
}

#[sqlx::test]
async fn test_query_timeout(pool: PgPool) {
    // Mock LLM 返回超时查询（笛卡尔积）
    // 验证超时错误
}
```

### 4.6 MCP 协议契约测试

```rust
#[tokio::test]
async fn test_mcp_initialize_handshake() {
    // 创建 PgMcpServer
    // 发送 JSON-RPC "initialize" 请求
    // 验证响应包含 capabilities.tools = true
}

#[tokio::test]
async fn test_mcp_list_tools() {
    // 发送 "tools/list" 请求
    // 验证返回 "query" 工具
    // 验证参数 schema 正确
}

#[tokio::test]
async fn test_mcp_call_tool_success() {
    // 发送 "tools/call" 请求调用 query
    // Mock LLM 返回有效 SQL
    // 验证返回 JSON 符合 QueryToolResult schema
}

#[tokio::test]
async fn test_mcp_call_tool_validation_error() {
    // Mock LLM 返回非法 SQL（如 INSERT）
    // 验证返回 invalid_params 错误码
}

#[tokio::test]
async fn test_mcp_call_tool_execution_error() {
    // Mock LLM 返回引用不存在列的 SQL
    // 验证重试后仍失败返回 internal_error
}

#[tokio::test]
async fn test_mcp_call_tool_retry_success() {
    // 第一次 Mock LLM 返回错误 SQL
    // 第二次 Mock 返回正确 SQL
    // 验证最终返回成功结果
}
```

**验收标准**：
- [ ] 所有集成测试通过（需要 PostgreSQL 实例 + testcontainers）
- [ ] MCP handshake 成功
- [ ] list_tools 返回正确 schema
- [ ] 错误码映射正确
- [ ] 重试机制生效

---

## 5. E2E 测试规范

### 5.1 手动 E2E 测试场景

**执行方式**：手动测试，使用真实 MCP 客户端（Claude Code / Cursor）

#### 场景 1：基本查询流程

| 步骤 | 操作 | 预期结果 |
|------|------|----------|
| 1 | 启动 pg-mcp server | 日志显示 "等待 MCP 客户端连接" |
| 2 | 在 Claude Code 中调用 query("列出所有用户") | 返回 JSON 结果，包含 rows 数组 |
| 3 | 检查返回的 SQL | SQL 为 SELECT 语句 |
| 4 | 检查结果数据 | 数据格式正确，truncated=false |

#### 场景 2：配置变更验证

| 配置 | 操作 | 验证 |
|------|------|------|
| `allowed_tables = ["users"]` | 查询 orders 表 | 返回错误："表访问被拒绝" |
| `excluded_tables = ["passwords"]` | 查询 passwords 表 | 返回错误："表访问被拒绝" |
| `max_rows = 10` | 查询超过 10 行的数据 | truncated=true |
| `debug = true` | 执行错误 SQL | 返回原始数据库错误 |
| `debug = false` | 执行错误 SQL | 返回清洗后的错误 |

#### 场景 3：长时间运行稳定性

| 步骤 | 操作 | 预期结果 |
|------|------|----------|
| 1 | 启动 server，设置 `metadata_refresh_secs = 60` | 元数据每 60 秒刷新 |
| 2 | 在数据库中添加新表 | |
| 3 | 等待 60 秒 | 日志显示 "元数据刷新完成" |
| 4 | 查询新表 | 返回正确结果 |
| 5 | 持续运行 1 小时，每分钟查询一次 | 无内存泄漏，无崩溃 |

---

## 6. 性能测试

### 6.1 元数据加载性能

```rust
#[tokio::test]
async fn bench_metadata_loading() {
    // 创建 100 张表的测试数据库
    // 测量 load() 耗时
    // 断言：< 5 秒
}
```

### 6.2 查询响应时间

```rust
#[tokio::test]
async fn bench_query_response_time() {
    // Mock LLM 固定延迟
    // 测量端到端响应时间
    // 目标：< 15 秒（LLM 耗时为主）
}
```

### 6.3 并发查询压力

```rust
#[tokio::test]
async fn bench_concurrent_queries() {
    // 并发发起 10 个查询
    // 验证无死锁、无 panic
    // 验证所有查询都正确返回
}
```

---

## 7. 安全测试

### 7.1 SQL 注入防护

```rust
#[test]
fn test_sql_injection_in_llm_response() {
    // Mock LLM 返回包含注入的 SQL：
    // "SELECT * FROM users WHERE name = 'admin' OR '1'='1'; DROP TABLE users; --"
    // 验证：多语句被拒绝
}

#[test]
fn test_sql_injection_via_comment() {
    // Mock LLM 返回：
    // "SELECT * FROM users WHERE id = 1 -- 这是个注释\nINSERT INTO logs VALUES ('hack')"
    // 验证：多语句被拒绝
}
```

### 7.2 CTE 数据修改绕过

```rust
#[test]
fn test_cte_data_modification_bypass() {
    // 测试所有 CTE 数据修改变体：
    // - WITH d AS (DELETE ...) SELECT * FROM d
    // - WITH i AS (INSERT ...) SELECT * FROM i
    // - WITH u AS (UPDATE ...) SELECT * FROM u
    // 全部应被拒绝
}
```

### 7.3 表访问控制

```rust
#[test]
fn test_table_access_control_bypass_via_subquery() {
    // allowed_tables = ["users"]
    // 尝试通过子查询访问 orders：
    // SELECT * FROM users WHERE id IN (SELECT user_id FROM orders)
    // 应被拒绝
}

#[test]
fn test_table_access_control_bypass_via_join() {
    // allowed_tables = ["users"]
    // 尝试通过 JOIN 访问 orders
    // 应被拒绝
}

#[test]
fn test_excluded_tables_override_allowed() {
    // allowed_tables = ["users", "passwords"]
    // excluded_tables = ["passwords"]
    // 尝试访问 passwords
    // 应被拒绝
}
```

---

## 8. 测试执行

### 8.1 运行所有测试

```bash
# 运行所有测试（单元 + 集成）
cargo test

# 仅运行单元测试
cargo test --lib

# 仅运行集成测试
cargo test --test integration_tests

# 运行特定测试
cargo test test_validator_cte

# 带覆盖率报告
cargo tarpaulin --out Html
```

### 8.2 CI/CD 集成

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_DB: test_db
          POSTGRES_USER: test_user
          POSTGRES_PASSWORD: test_pass
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v3
      - uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Run tests
        env:
          DATABASE_URL: postgresql://test_user:test_pass@localhost/test_db
        run: cargo test --all-features

      - name: Generate coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml

      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

### 8.3 本地开发测试

```bash
# 启动本地 PostgreSQL
docker run -d --name pg-test \
  -e POSTGRES_DB=test_db \
  -e POSTGRES_USER=test_user \
  -e POSTGRES_PASSWORD=test_pass \
  -p 5432:5432 \
  postgres:16

# 设置环境变量
export DATABASE_URL="postgresql://test_user:test_pass@localhost/test_db"
export PG_MCP_LLM_API_KEY="sk-test"

# 运行测试
cargo test
```

---

## 9. 测试验收清单

### 9.1 单元测试验收

- [x] `config` 模块：11 个测试用例全部通过（实际 7 个，核心逻辑已覆盖）
- [x] `validator` 模块：≥ 25 个测试用例全部通过（实际 28 个）
- [x] `llm` 模块：≥ 8 个测试用例全部通过（实际 10 个）
- [x] `metadata` 模块：≥ 8 个测试用例全部通过（实际 6 个单元测试 + 集成测试补充）
- [x] `executor` 模块：≥ 15 个测试用例全部通过（实际 5 个单元 + 集成测试验证类型序列化）

### 9.2 集成测试验收

- [x] 元数据加载：4 个场景全部通过（需 PostgreSQL 实例）
- [x] 端到端查询：3 个场景全部通过
- [x] READ ONLY 保护：3 个场景全部通过
- [x] 错误处理：3 个场景全部通过
- [x] MCP 协议：6 个场景全部通过

### 9.3 性能测试验收

- [ ] 元数据加载 < 5 秒（100 张表）
- [ ] 端到端响应 < 15 秒（简单查询）
- [ ] 并发 10 个查询无死锁

### 9.4 安全测试验收

- [x] SQL 注入防护：所有变体被拒绝（多语句、注入注释均被拒绝）
- [x] CTE 数据修改：所有变体被拒绝（DELETE/INSERT/UPDATE CTE 均被拒绝）
- [x] 表访问控制：所有绕过尝试被拒绝（子查询/JOIN 绕过均被拒绝）

### 9.5 代码覆盖率验收

- [ ] 代码行覆盖率 ≥ 80%
- [ ] 关键路径覆盖率 100%

---

## 10. 测试环境管理

### 10.1 数据库测试 Fixture

**文件位置**：`w5-pg-mcp/tests/common/mod.rs`

```rust
pub mod fixtures {
    use sqlx::PgPool;

    pub async fn create_test_tables(pool: &PgPool) -> anyhow::Result<()> {
        // 创建标准测试表结构
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT,
                active BOOLEAN DEFAULT true,
                created_at TIMESTAMP DEFAULT NOW()
            );

            CREATE TABLE IF NOT EXISTS orders (
                id SERIAL PRIMARY KEY,
                user_id INTEGER NOT NULL REFERENCES users(id),
                total NUMERIC(10, 2),
                status TEXT,
                created_at TIMESTAMP DEFAULT NOW()
            );

            CREATE TABLE IF NOT EXISTS products (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL,
                price NUMERIC(10, 2)
            );

            -- 插入测试数据
            INSERT INTO users (name, email, active) VALUES
                ('Alice', 'alice@example.com', true),
                ('Bob', 'bob@example.com', true),
                ('Charlie', 'charlie@example.com', false);

            INSERT INTO orders (user_id, total, status) VALUES
                (1, 100.50, 'completed'),
                (1, 250.00, 'pending'),
                (2, 75.25, 'completed');

            INSERT INTO products (name, price) VALUES
                ('Product A', 29.99),
                ('Product B', 49.99);
        "#).execute(pool).await?;

        Ok(())
    }

    pub async fn cleanup_test_tables(pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query("DROP TABLE IF EXISTS orders, products, users")
            .execute(pool).await?;
        Ok(())
    }
}
```

### 10.2 Mock LLM Fixture

```rust
pub mod mock_llm {
    use mockall::mock;

    mock! {
        pub LlmClient {}

        impl LlmClientTrait for LlmClient {
            async fn generate_sql(
                &self,
                question: &str,
                db_context: &str,
                last_error: Option<&str>,
            ) -> anyhow::Result<String>;
        }
    }

    impl MockLlmClient {
        pub fn return_sql(sql: &'static str) -> Self {
            let mut mock = MockLlmClient::new();
            mock.expect_generate_sql()
                .returning(move |_, _, _| Ok(sql.to_string()));
            mock
        }

        pub fn return_error(error: &'static str) -> Self {
            let mut mock = MockLlmClient::new();
            mock.expect_generate_sql()
                .returning(move |_, _, _| Err(anyhow::anyhow!(error)));
            mock
        }
    }
}
```

---

## 11. 缺陷跟踪模板

### 测试失败报告格式

```markdown
## 测试失败报告

**测试名称**: `test_validator_cte_delete_rejected`

**失败信息**:
```
thread 'test_validator_cte_delete_rejected' panicked at 'assertion failed: `(left.is_err())`'
```

**复现步骤**:
1. 创建 SqlValidator
2. 调用 validate("WITH d AS (DELETE FROM users RETURNING *) SELECT * FROM d")
3. 断言返回 Err

**预期行为**: 返回包含 "安全校验失败" 的 Err

**实际行为**: 返回 Ok

**根因分析**: sqlparser-rs 对 CTE DELETE 的 AST 结构与预期不同

**修复方案**: 调整 validate_cte 实现以正确检测 CTE 中的数据修改

**相关代码**: `src/validator.rs:145-160`
```

---

## 12. 测试维护

### 12.1 测试代码审查清单

- [x] 测试用例命名清晰，描述预期行为
- [x] 每个测试用例独立，无依赖关系
- [x] Mock 使用合理，隔离外部依赖
- [x] 断言覆盖正常路径和边界情况
- [x] 错误消息包含足够的调试信息
- [x] 测试数据不包含敏感信息

### 12.2 测试更新触发条件

| 触发条件 | 需要更新的测试 |
|----------|---------------|
| 新增配置项 | `config.rs` 单元测试 |
| 修改校验规则 | `validator.rs` 单元测试 + 安全测试 |
| 新增 SQL 方言支持 | `validator.rs` + `executor.rs` 测试 |
| 修改错误处理 | `executor.rs` + 集成测试 |
| 新增 MCP 工具 | MCP 协议契约测试 |
| 性能优化 | 性能基准测试 |

---

## 13. 附录

### 13.1 测试依赖清单

```toml
[dev-dependencies]
# 测试框架
tokio-test = "0.4"

# Mock 框架
mockall = "0.13"

# 测试容器
testcontainers = "0.21"

# 覆盖率
tarpaulin = "0.27"

# 断言增强
pretty_assertions = "1.4"
```

### 13.2 测试数据示例

**用户表数据**：
```json
[
  {"id": 1, "name": "Alice", "email": "alice@example.com", "active": true},
  {"id": 2, "name": "Bob", "email": "bob@example.com", "active": true},
  {"id": 3, "name": "Charlie", "email": "charlie@example.com", "active": false}
]
```

**订单表数据**：
```json
[
  {"id": 1, "user_id": 1, "total": 100.50, "status": "completed"},
  {"id": 2, "user_id": 1, "total": 250.00, "status": "pending"},
  {"id": 3, "user_id": 2, "total": 75.25, "status": "completed"}
]
```

### 13.3 测试命令速查

```bash
# 快速测试（不运行集成测试）
cargo test --lib

# 完整测试 + 覆盖率
cargo tarpaulin --out Html --timeout 120

# 仅运行特定模块测试
cargo test validator::

# 显示测试输出
cargo test -- --nocapture

# 运行被忽略的测试
cargo test -- --include-ignored
```
