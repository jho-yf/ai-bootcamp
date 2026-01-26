# 数据模型：数据库查询工具

**Feature**: 001-db-query  
**Date**: 2026-01-17  
**Source**: 基于 spec.md 的关键实体和功能需求

## 模型概览

本应用包含以下核心数据模型：

1. **DatabaseConnection** - 数据库连接配置
2. **DatabaseMetadata** - 数据库元数据（表、视图、列）
3. **TableInfo / ViewInfo** - 表和视图信息
4. **ColumnInfo** - 列信息
5. **SQLQuery** - SQL 查询
6. **QueryResult** - 查询结果
7. **NaturalLanguageQuery** - 自然语言查询

---

## 1. DatabaseConnection（数据库连接）

### 用途

表示一个 PostgreSQL 数据库的连接配置。用户可以添加多个数据库连接，每个连接包含主机、端口、凭据等信息。

### Rust 类型定义

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseConnection {
    /// 唯一标识符（UUID）
    pub id: String,
    
    /// 用户自定义的连接名称（如 "生产数据库"、"测试环境"）
    pub name: String,
    
    /// 数据库主机地址
    pub host: String,
    
    /// 端口号（默认 5432）
    pub port: u16,
    
    /// 数据库名称
    pub database_name: String,
    
    /// 用户名
    pub user: String,
    
    /// 密码（存储时加密）
    #[serde(skip_serializing)]
    pub password: String,
    
    /// 连接状态
    pub status: ConnectionStatus,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    /// 未连接
    Disconnected,
    /// 连接中
    Connecting,
    /// 已连接
    Connected,
    /// 连接失败
    Failed,
}
```

### TypeScript 类型定义

```typescript
export interface DatabaseConnection {
  id: string;
  name: string;
  host: string;
  port: number;
  databaseName: string;
  user: string;
  // password 不从后端返回
  status: 'disconnected' | 'connecting' | 'connected' | 'failed';
  createdAt: string; // ISO 8601 格式
  updatedAt: string;
}
```

### 验证规则
- `id`: 必须是有效的 UUID
- `name`: 非空字符串，长度 1-100
- `host`: 非空字符串，可以是 IP 或域名
- `port`: 1-65535 之间的整数
- `database_name`: 非空字符串，符合 PostgreSQL 数据库名规则
- `user`: 非空字符串
- `password`: 非空字符串（存储时使用 AES-256 加密）

### 状态转换
```
Disconnected -> Connecting -> Connected
                           -> Failed
Connected -> Disconnected (手动断开或连接丢失)
```

---

## 2. DatabaseMetadata（数据库元数据）

### 用途
表示某个数据库的完整结构信息，包括所有表、视图、列、约束等。从 PostgreSQL 提取后缓存在本地 SQLite。

### Rust 类型定义

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseMetadata {
    /// 对应的数据库连接 ID
    pub connection_id: String,
    
    /// 所有表
    pub tables: Vec<TableInfo>,
    
    /// 所有视图
    pub views: Vec<ViewInfo>,
    
    /// 元数据提取时间
    pub extracted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableInfo {
    /// Schema 名称（通常是 "public"）
    pub schema: String,
    
    /// 表名
    pub name: String,
    
    /// 表类型（"table" 或 "view"）
    pub table_type: String,
    
    /// 所有列
    pub columns: Vec<ColumnInfo>,
    
    /// 主键列名列表
    pub primary_keys: Vec<String>,
    
    /// 外键约束
    pub foreign_keys: Vec<ForeignKeyInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewInfo {
    pub schema: String,
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    /// 视图定义 SQL（可选）
    pub definition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnInfo {
    /// 列名
    pub name: String,
    
    /// 数据类型（如 "integer", "varchar", "timestamp"）
    pub data_type: String,
    
    /// 是否可空
    pub nullable: bool,
    
    /// 默认值（可选）
    pub default_value: Option<String>,
    
    /// 是否为主键
    pub is_primary_key: bool,
    
    /// 列位置（用于排序）
    pub ordinal_position: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForeignKeyInfo {
    /// 外键名称
    pub constraint_name: String,
    
    /// 本表列名
    pub column_name: String,
    
    /// 引用的表名
    pub referenced_table: String,
    
    /// 引用的列名
    pub referenced_column: String,
}
```

### TypeScript 类型定义

```typescript
export interface DatabaseMetadata {
  connectionId: string;
  tables: TableInfo[];
  views: ViewInfo[];
  extractedAt: string;
}

export interface TableInfo {
  schema: string;
  name: string;
  tableType: string;
  columns: ColumnInfo[];
  primaryKeys: string[];
  foreignKeys: ForeignKeyInfo[];
}

export interface ViewInfo {
  schema: string;
  name: string;
  columns: ColumnInfo[];
  definition?: string;
}

export interface ColumnInfo {
  name: string;
  dataType: string;
  nullable: boolean;
  defaultValue?: string;
  isPrimaryKey: boolean;
  ordinalPosition: number;
}

export interface ForeignKeyInfo {
  constraintName: string;
  columnName: string;
  referencedTable: string;
  referencedColumn: string;
}
```

### 提取逻辑
1. 查询 `information_schema.tables` 获取所有表和视图
2. 对每个表查询 `information_schema.columns` 获取列信息
3. 查询 `information_schema.table_constraints` 和 `information_schema.key_column_usage` 获取约束
4. 使用 LLM 将原始数据转换为规范化 JSON 格式
5. 存储到 SQLite `metadata` 表

---

## 3. SQLQuery（SQL 查询）

### 用途
表示用户编写或 AI 生成的 SQL 查询语句。

### Rust 类型定义

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SQLQuery {
    /// 查询 ID（用于历史记录）
    pub id: String,
    
    /// 对应的数据库连接 ID
    pub connection_id: String,
    
    /// 原始 SQL 语句
    pub sql: String,
    
    /// 解析后的 SQL（添加 LIMIT 后）
    pub parsed_sql: Option<String>,
    
    /// 查询来源
    pub source: QuerySource,
    
    /// 执行状态
    pub status: QueryStatus,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuerySource {
    /// 用户手动输入
    Manual,
    /// AI 生成
    AI,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QueryStatus {
    /// 待执行
    Pending,
    /// 执行中
    Running,
    /// 执行成功
    Success,
    /// 执行失败
    Failed,
}
```

### TypeScript 类型定义

```typescript
export interface SQLQuery {
  id: string;
  connectionId: string;
  sql: string;
  parsedSql?: string;
  source: 'manual' | 'ai';
  status: 'pending' | 'running' | 'success' | 'failed';
  createdAt: string;
}
```

---

## 4. QueryResult（查询结果）

### 用途
表示 SQL 查询执行后返回的结果集，包括列名、行数据、执行时间等。

### Rust 类型定义

```rust
use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    /// 列名列表
    pub columns: Vec<String>,
    
    /// 行数据（每行是一个 HashMap，key 是列名，value 是列值）
    pub rows: Vec<HashMap<String, Value>>,
    
    /// 总行数
    pub total: usize,
    
    /// 执行时间（毫秒）
    pub exec_time_ms: u64,
    
    /// 执行的 SQL（包含自动添加的 LIMIT）
    pub sql: String,
    
    /// 是否被截断（超过 LIMIT）
    pub truncated: bool,
}
```

### TypeScript 类型定义

```typescript
export interface QueryResult {
  columns: string[];
  rows: Record<string, any>[];
  total: number;
  execTimeMs: number;
  sql: string;
  truncated: boolean;
}
```

### 示例数据

```json
{
  "columns": ["id", "name", "email", "createdAt"],
  "rows": [
    {
      "id": 1,
      "name": "张三",
      "email": "zhangsan@example.com",
      "createdAt": "2026-01-15T10:30:00Z"
    },
    {
      "id": 2,
      "name": "李四",
      "email": "lisi@example.com",
      "createdAt": "2026-01-16T14:20:00Z"
    }
  ],
  "total": 2,
  "execTimeMs": 45,
  "sql": "SELECT * FROM users LIMIT 100",
  "truncated": false
}
```

### 数据转换规则
- PostgreSQL 类型 → JSON Value：
  - `integer`, `bigint` → `Number`
  - `varchar`, `text` → `String`
  - `boolean` → `Boolean`
  - `timestamp`, `date` → `String` (ISO 8601 格式)
  - `json`, `jsonb` → `Object` (解析后的 JSON)
  - `NULL` → `null`

---

## 5. NaturalLanguageQuery（自然语言查询）

### 用途
表示用户的自然语言查询描述和 AI 生成的 SQL 结果。

### Rust 类型定义

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NaturalLanguageQuery {
    /// 查询 ID
    pub id: String,
    
    /// 对应的数据库连接 ID
    pub connection_id: String,
    
    /// 用户输入的自然语言描述
    pub prompt: String,
    
    /// AI 生成的 SQL 查询
    pub generated_sql: Option<String>,
    
    /// 生成状态
    pub status: GenerationStatus,
    
    /// 错误消息（如果生成失败）
    pub error_message: Option<String>,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GenerationStatus {
    /// 生成中
    Generating,
    /// 生成成功
    Success,
    /// 生成失败
    Failed,
}
```

### TypeScript 类型定义

```typescript
export interface NaturalLanguageQuery {
  id: string;
  connectionId: string;
  prompt: string;
  generatedSql?: string;
  status: 'generating' | 'success' | 'failed';
  errorMessage?: string;
  createdAt: string;
}
```

### 示例

```json
{
  "id": "nlq-123",
  "connectionId": "db-1",
  "prompt": "查询所有年龄大于30岁且活跃状态的用户姓名和邮箱",
  "generatedSql": "SELECT name, email FROM users WHERE age > 30 AND status = 'active' LIMIT 100",
  "status": "success",
  "errorMessage": null,
  "createdAt": "2026-01-17T15:30:00Z"
}
```

---

## 数据流图

```
用户输入连接信息
    ↓
DatabaseConnection 创建
    ↓
连接 PostgreSQL
    ↓
提取 information_schema
    ↓
LLM 转换为规范 JSON
    ↓
DatabaseMetadata 存储到 SQLite
    ↓
用户编写 SQL 或输入自然语言
    ↓
SQLQuery 或 NaturalLanguageQuery
    ↓
SQL 解析和 LIMIT 注入
    ↓
执行查询
    ↓
QueryResult 返回前端
    ↓
表格展示
```

---

## 持久化存储

### SQLite Schema（本地缓存）

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
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 元数据表（JSON 格式）
CREATE TABLE metadata (
    connection_id TEXT PRIMARY KEY,
    metadata_json TEXT NOT NULL,
    extracted_at TEXT NOT NULL,
    FOREIGN KEY (connection_id) REFERENCES connections(id) ON DELETE CASCADE
);

-- 查询历史（统一表，包含 SQL 查询和自然语言查询）
CREATE TABLE query_history (
    id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    query_type TEXT NOT NULL,          -- 'sql' 或 'natural_language'
    sql TEXT,                           -- SQL 查询语句（手动输入或 AI 生成）
    prompt TEXT,                        -- 自然语言描述（仅 natural_language 类型）
    executed_at TEXT NOT NULL,          -- 执行/创建时间
    exec_time_ms INTEGER,               -- 执行时间（毫秒），可能为 NULL
    status TEXT NOT NULL,               -- 'success', 'failed', 'pending' 等
    FOREIGN KEY (connection_id) REFERENCES connections(id) ON DELETE CASCADE
);

-- 索引：按连接和时间查询历史
CREATE INDEX idx_query_history_connection_time 
ON query_history(connection_id, executed_at DESC);

-- 索引：按查询类型过滤
CREATE INDEX idx_query_history_type 
ON query_history(query_type);
```

### 查询历史表使用说明

**query_history 表设计理念**: 使用 `query_type` 字段统一管理所有类型的查询历史，避免表结构重复。

#### 普通 SQL 查询记录示例

```sql
INSERT INTO query_history (
    id, 
    connection_id, 
    query_type, 
    sql, 
    prompt, 
    executed_at, 
    exec_time_ms, 
    status
) VALUES (
    'query-123',
    'db-1',
    'sql',                                  -- 普通 SQL 查询
    'SELECT * FROM users WHERE age > 30',
    NULL,                                   -- 无自然语言描述
    '2026-01-17T15:30:00Z',
    45,
    'success'
);
```

#### 自然语言查询记录示例

```sql
INSERT INTO query_history (
    id, 
    connection_id, 
    query_type, 
    sql, 
    prompt, 
    executed_at, 
    exec_time_ms, 
    status
) VALUES (
    'query-124',
    'db-1',
    'natural_language',                     -- 自然语言查询
    'SELECT name, email FROM users WHERE age > 30 LIMIT 100',  -- AI 生成的 SQL
    '查询所有年龄大于30岁的用户姓名和邮箱',    -- 用户的自然语言输入
    '2026-01-17T15:35:00Z',
    120,
    'success'
);
```

#### 查询示例

```sql
-- 获取所有 SQL 查询历史
SELECT * FROM query_history 
WHERE connection_id = 'db-1' AND query_type = 'sql'
ORDER BY executed_at DESC;

-- 获取所有自然语言查询历史
SELECT * FROM query_history 
WHERE connection_id = 'db-1' AND query_type = 'natural_language'
ORDER BY executed_at DESC;

-- 获取最近 10 条查询（不区分类型）
SELECT * FROM query_history 
WHERE connection_id = 'db-1'
ORDER BY executed_at DESC 
LIMIT 10;
```

---

## 总结

所有数据模型均符合项目章程要求：
- ✅ Rust 类型使用 `#[serde(rename_all = "camelCase")]` 确保 JSON 输出为 camelCase
- ✅ TypeScript 类型与 Rust 类型完全对应，保证类型安全
- ✅ 所有字段都有明确的类型标注
- ✅ 中文注释和文档
- ✅ 使用标准库和 serde 进行序列化

**下一步**: 生成 API 契约文档（Tauri Commands）
