# 技术研究：数据库查询工具

**Feature**: 001-db-query  
**Date**: 2026-01-17  
**Purpose**: 解决技术选型、集成模式和最佳实践的研究决策

## 研究领域概览

本文档记录关键技术决策的研究结果，包括：
1. Tauri 应用架构和前后端通信
2. PostgreSQL 元数据提取策略
3. SQL 解析和查询安全
4. OpenAI API 集成模式
5. SQLite 本地缓存设计
6. Monaco Editor 集成方案

---

## 1. Tauri 应用架构

### 决策
采用 Tauri 1.5+ 的标准 Command 模式，使用 `#[tauri::command]` 定义异步 API，前端通过 `@tauri-apps/api` 的 `invoke` 函数调用。

### 理由
- **轻量级**: Tauri 使用系统原生 WebView，比 Electron 更小（~3MB vs ~100MB）
- **安全**: Rust 后端提供内存安全，Command 模式提供明确的前后端边界
- **异步支持**: `async fn` 直接支持，适合数据库 I/O 密集操作
- **跨平台**: 一次构建支持 Windows、macOS、Linux

### 考虑的替代方案
- **Electron**: 体积大，内存占用高，不符合桌面应用轻量级需求
- **纯 Web 应用**: 无法访问本地 SQLite 文件系统，不符合离线缓存需求
- **原生开发（Qt/C++）**: 开发效率低，不如 Tauri 的 Web 技术栈快速

### 最佳实践
- 所有 Command 函数返回 `Result<T, String>` 统一错误处理
- 使用 `State<T>` 管理全局状态（如数据库连接池）
- 前端调用示例：
  ```typescript
  import { invoke } from '@tauri-apps/api/tauri';
  
  const result = await invoke<QueryResult>('run_sql_query', {
    databaseId: 'db-1',
    sql: 'SELECT * FROM users'
  });
  ```

---

## 2. PostgreSQL 元数据提取

### 决策
使用 PostgreSQL `information_schema` 标准视图提取元数据，包括：
- `information_schema.tables` - 所有表和视图
- `information_schema.columns` - 列信息（名称、类型、是否可空）
- `information_schema.table_constraints` - 主键和外键约束
- `information_schema.key_column_usage` - 约束关联的列

### 理由
- **标准化**: `information_schema` 是 SQL 标准，PostgreSQL 完全支持
- **完整性**: 包含类型、约束、默认值等完整信息
- **查询性能**: 系统视图已优化，查询速度快
- **兼容性**: 未来扩展到其他数据库（MySQL、SQL Server）时接口一致

### 示例查询
```sql
-- 获取所有表
SELECT table_schema, table_name, table_type 
FROM information_schema.tables 
WHERE table_schema NOT IN ('pg_catalog', 'information_schema');

-- 获取表的列信息
SELECT column_name, data_type, is_nullable, column_default
FROM information_schema.columns
WHERE table_schema = 'public' AND table_name = 'users'
ORDER BY ordinal_position;
```

### 考虑的替代方案
- **pg_catalog 系统表**: 更底层但不跨数据库，查询复杂度高
- **SHOW TABLES 命令**: 非标准 SQL，信息不完整
- **直接查询表数据**: 性能差，无法获取结构信息

### 元数据转换为 JSON
使用 LLM（OpenAI）将原始元数据规范化为统一 JSON 格式：
```json
{
  "tables": [
    {
      "name": "users",
      "type": "table",
      "columns": [
        {
          "name": "id",
          "dataType": "integer",
          "nullable": false,
          "isPrimaryKey": true
        }
      ]
    }
  ]
}
```

---

## 3. SQL 解析和查询安全

### 决策
使用 `sqlparser-rs` crate 解析 SQL 语句，检测并自动添加 LIMIT 子句。

### 理由
- **准确解析**: sqlparser-rs 支持 PostgreSQL 方言，正确处理复杂 SQL
- **AST 操作**: 可以修改语法树后重新生成 SQL（添加 LIMIT）
- **语法验证**: 在执行前捕获语法错误，避免数据库往返
- **安全检查**: 检测 DDL 语句（CREATE/DROP/ALTER）并可选择性拒绝

### 实现逻辑
```rust
use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

fn inject_limit(sql: &str) -> Result<String, String> {
    let dialect = PostgreSqlDialect {};
    let ast = Parser::parse_sql(&dialect, sql)
        .map_err(|e| format!("SQL 语法错误: {}", e))?;
    
    if let Some(Statement::Query(mut query)) = ast.get(0).cloned() {
        if query.limit.is_none() {
            query.limit = Some(sqlparser::ast::Expr::Value(
                sqlparser::ast::Value::Number("100".to_string(), false)
            ));
        }
        return Ok(query.to_string());
    }
    
    Ok(sql.to_string())
}
```

### 考虑的替代方案
- **正则表达式**: 无法处理复杂 SQL（子查询、CTE），容易出错
- **字符串拼接**: 不安全，可能破坏原始 SQL 语义
- **不添加 LIMIT**: 性能风险，可能返回数百万行数据

---

## 4. OpenAI API 集成

### 决策
使用 `async-openai` crate（Rust OpenAI 官方 SDK），调用 `gpt-4` 或 `gpt-3.5-turbo` 模型，将自然语言转换为 SQL。

### 理由
- **官方 SDK**: 维护良好，支持最新 API 特性
- **异步**: 基于 tokio，与 Tauri 和 tokio-postgres 无缝集成
- **类型安全**: 完整的 Rust 类型定义

### Prompt 设计
将数据库元数据作为上下文注入 Prompt：

```rust
let prompt = format!(
    "你是一个 SQL 专家。根据以下 PostgreSQL 数据库结构，将用户的自然语言查询转换为 SQL 查询。\n\n\
    数据库结构：\n{}\n\n\
    用户查询：{}\n\n\
    请只返回 SQL 查询语句，不要包含解释或其他文本。",
    metadata_json,
    user_prompt
);
```

### 配置
- API Key: 从环境变量 `OPENAI_API_KEY` 读取
- 超时: 30 秒（LLM 推理时间）
- 错误处理: 网络错误、API 限额、无效响应的友好提示

### 考虑的替代方案
- **本地 LLM（Llama/Mistral）**: 质量不如 GPT-4，部署复杂度高
- **规则引擎**: 无法处理灵活的自然语言，维护成本高
- **Claude API**: 可作为备选，但 OpenAI 文档更完善

---

## 5. SQLite 本地缓存

### 决策
使用 `rusqlite` crate 管理本地 SQLite 数据库 (`./db_query.db`)，存储：
- 数据库连接配置（加密密码）
- 元数据缓存（表、列、约束）
- 查询历史（可选）

### Schema 设计
```sql
-- 数据库连接表
CREATE TABLE connections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER NOT NULL,
    database_name TEXT NOT NULL,
    user TEXT NOT NULL,
    password_encrypted TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 元数据表（JSON 格式存储）
CREATE TABLE metadata (
    connection_id TEXT PRIMARY KEY,
    metadata_json TEXT NOT NULL,
    extracted_at INTEGER NOT NULL,
    FOREIGN KEY (connection_id) REFERENCES connections(id)
);

-- 查询历史（可选）
CREATE TABLE query_history (
    id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    sql TEXT NOT NULL,
    executed_at INTEGER NOT NULL,
    exec_time_ms INTEGER,
    FOREIGN KEY (connection_id) REFERENCES connections(id)
);
```

### 密码加密
使用操作系统 Keyring（通过 `keyring` crate）或简单的 AES-256 加密存储密码。

### 理由
- **离线可用**: 缓存元数据后无需重新连接数据库
- **快速启动**: 应用启动时直接从 SQLite 加载配置
- **可移植**: 单个 `.db` 文件，易于备份和迁移

### 考虑的替代方案
- **JSON 文件**: 不支持查询，并发访问需要锁
- **内存缓存**: 应用关闭后丢失，不符合持久化需求
- **远程数据库**: 桌面应用无需网络依赖

---

## 6. Monaco Editor 集成

### 决策
使用 `@monaco-editor/react` 包集成 Monaco Editor，配置 SQL 语法高亮和自动完成。

### 理由
- **专业编辑器**: VS Code 同款编辑器，用户体验极佳
- **SQL 支持**: 内置 SQL 语法高亮和基础自动完成
- **可扩展**: 支持自定义 IntelliSense（基于元数据的列名提示）
- **轻量级**: React 集成简单，性能优异

### 配置示例
```typescript
import Editor from '@monaco-editor/react';

<Editor
  height="400px"
  language="sql"
  theme="vs-dark"
  value={sqlCode}
  onChange={(value) => setSqlCode(value || '')}
  options={{
    minimap: { enabled: false },
    fontSize: 14,
    lineNumbers: 'on',
    automaticLayout: true,
  }}
/>
```

### 自定义自动完成
可以注册自定义 CompletionProvider，提供表名和列名提示：
```typescript
monaco.languages.registerCompletionItemProvider('sql', {
  provideCompletionItems: (model, position) => {
    return {
      suggestions: metadata.tables.map(table => ({
        label: table.name,
        kind: monaco.languages.CompletionItemKind.Table,
        insertText: table.name,
      }))
    };
  }
});
```

### 考虑的替代方案
- **CodeMirror**: 功能相近但社区不如 Monaco 活跃
- **简单 textarea**: 无语法高亮，用户体验差
- **ace-editor**: 较旧，TypeScript 支持不完善

---

## 7. 错误处理和用户反馈

### 决策
使用统一的错误类型和中文错误消息，前端通过 Ant Design 的 `message` 和 `notification` 组件显示。

### 错误分类
```rust
#[derive(Debug)]
pub enum AppError {
    DatabaseConnection(String),  // 数据库连接错误
    QueryExecution(String),      // 查询执行错误
    MetadataExtraction(String),  // 元数据提取错误
    AIService(String),           // AI 服务错误
    CacheStorage(String),        // 缓存存储错误
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppError::DatabaseConnection(msg) => {
                write!(f, "数据库连接失败：{}", msg)
            }
            // ... 其他错误类型
        }
    }
}
```

### 前端错误显示
```typescript
try {
  const result = await invoke('run_sql_query', { databaseId, sql });
} catch (error) {
  message.error(error as string);
}
```

---

## 研究总结

所有关键技术决策已完成研究并记录。主要亮点：

1. **Tauri 架构**: 轻量级、安全、跨平台，完美契合桌面应用需求
2. **PostgreSQL 元数据**: 使用标准 `information_schema`，易于扩展到其他数据库
3. **SQL 解析**: sqlparser-rs 提供准确解析和安全的 LIMIT 注入
4. **OpenAI 集成**: 官方 SDK + 元数据上下文 = 高质量自然语言查询
5. **SQLite 缓存**: 单文件持久化，快速启动，离线可用
6. **Monaco Editor**: 专业编辑体验，支持自定义自动完成

所有选择均符合项目章程要求（Rust、TypeScript、类型安全、camelCase JSON）。

**下一步**: Phase 1 - 数据模型和 API 契约设计
