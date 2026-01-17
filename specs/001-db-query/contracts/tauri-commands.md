# Tauri Command API 契约

**Feature**: 001-db-query  
**Date**: 2026-01-17  
**Protocol**: Tauri Command Pattern (前端通过 `invoke` 调用后端 Rust 函数)

## API 概览

本应用通过 Tauri 的 Command 模式提供前后端通信。所有 API 都是异步的，返回 `Result<T, String>`。

### API 列表

| Command | 功能 | 优先级 |
|---------|------|--------|
| `list_databases` | 获取所有数据库连接 | P1 |
| `add_database` | 添加新数据库连接 | P1 |
| `update_database` | 更新数据库连接 | P1 |
| `delete_database` | 删除数据库连接 | P1 |
| `test_connection` | 测试数据库连接 | P1 |
| `get_database_metadata` | 获取数据库元数据 | P1 |
| `refresh_metadata` | 刷新数据库元数据 | P1 |
| `run_sql_query` | 执行 SQL 查询 | P2 |
| `cancel_query` | 取消正在执行的查询 | P2 |
| `run_nl_query` | 自然语言生成并执行查询 | P3 |
| `generate_sql_from_nl` | 仅生成 SQL（不执行）| P3 |

---

## 数据库连接管理 API

### 1. `list_databases` - 获取所有数据库连接

#### 功能描述
返回用户已配置的所有数据库连接列表（不包含密码）。

#### 请求参数
无参数

#### 返回值
```rust
Result<Vec<DatabaseConnection>, String>
```

#### 前端调用示例
```typescript
import { invoke } from '@tauri-apps/api/tauri';
import type { DatabaseConnection } from './types';

const databases = await invoke<DatabaseConnection[]>('list_databases');
```

#### 返回示例
```json
[
  {
    "id": "db-1",
    "name": "生产数据库",
    "host": "localhost",
    "port": 5432,
    "databaseName": "myapp_prod",
    "user": "admin",
    "status": "connected",
    "createdAt": "2026-01-15T10:00:00Z",
    "updatedAt": "2026-01-17T08:30:00Z"
  }
]
```

#### 错误场景
- SQLite 读取失败: `"无法加载数据库连接列表"`

---

### 2. `add_database` - 添加新数据库连接

#### 功能描述
添加新的 PostgreSQL 数据库连接配置，自动尝试连接并提取元数据。

#### 请求参数
```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddDatabaseRequest {
    name: String,
    host: String,
    port: u16,
    database_name: String,
    user: String,
    password: String,
}
```

#### 返回值
```rust
Result<DatabaseConnection, String>
```

#### 前端调用示例
```typescript
const newDb = await invoke<DatabaseConnection>('add_database', {
  name: '测试数据库',
  host: 'localhost',
  port: 5432,
  databaseName: 'test_db',
  user: 'postgres',
  password: 'secret123'
});
```

#### 返回示例
```json
{
  "id": "db-2",
  "name": "测试数据库",
  "host": "localhost",
  "port": 5432,
  "databaseName": "test_db",
  "user": "postgres",
  "status": "connected",
  "createdAt": "2026-01-17T15:30:00Z",
  "updatedAt": "2026-01-17T15:30:00Z"
}
```

#### 错误场景
- 连接失败: `"无法连接到数据库：[错误详情]"`
- 参数验证失败: `"数据库名称不能为空"`
- 重复名称: `"已存在同名的数据库连接"`

---

### 3. `update_database` - 更新数据库连接

#### 功能描述
更新现有数据库连接的配置（除了 ID）。

#### 请求参数
```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateDatabaseRequest {
    id: String,
    name: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    database_name: Option<String>,
    user: Option<String>,
    password: Option<String>,
}
```

#### 返回值
```rust
Result<DatabaseConnection, String>
```

#### 前端调用示例
```typescript
const updated = await invoke<DatabaseConnection>('update_database', {
  id: 'db-1',
  name: '新名称',
  host: 'newhost.com'
});
```

#### 错误场景
- 连接不存在: `"数据库连接不存在"`
- 更新失败: `"无法更新数据库连接：[错误详情]"`

---

### 4. `delete_database` - 删除数据库连接

#### 功能描述
删除指定的数据库连接及其缓存的元数据。

#### 请求参数
```rust
database_id: String
```

#### 返回值
```rust
Result<(), String>
```

#### 前端调用示例
```typescript
await invoke('delete_database', { databaseId: 'db-1' });
```

#### 错误场景
- 连接不存在: `"数据库连接不存在"`
- 删除失败: `"无法删除数据库连接"`

---

### 5. `test_connection` - 测试数据库连接

#### 功能描述
测试指定数据库连接是否可用（不保存连接）。

#### 请求参数
```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestConnectionRequest {
    host: String,
    port: u16,
    database_name: String,
    user: String,
    password: String,
}
```

#### 返回值
```rust
Result<bool, String>
```

#### 前端调用示例
```typescript
const isConnected = await invoke<boolean>('test_connection', {
  host: 'localhost',
  port: 5432,
  databaseName: 'test_db',
  user: 'postgres',
  password: 'secret123'
});
```

#### 返回示例
```json
true
```

#### 错误场景
- 连接失败: `"连接失败：[错误详情]"`

---

## 元数据管理 API

### 6. `get_database_metadata` - 获取数据库元数据

#### 功能描述
获取指定数据库的完整元数据（表、视图、列、约束）。优先返回缓存，如无缓存则自动提取。

#### 请求参数
```rust
database_id: String
```

#### 返回值
```rust
Result<DatabaseMetadata, String>
```

#### 前端调用示例
```typescript
const metadata = await invoke<DatabaseMetadata>('get_database_metadata', {
  databaseId: 'db-1'
});
```

#### 返回示例
```json
{
  "connectionId": "db-1",
  "tables": [
    {
      "schema": "public",
      "name": "users",
      "tableType": "table",
      "columns": [
        {
          "name": "id",
          "dataType": "integer",
          "nullable": false,
          "isPrimaryKey": true,
          "ordinalPosition": 1
        },
        {
          "name": "name",
          "dataType": "varchar",
          "nullable": false,
          "isPrimaryKey": false,
          "ordinalPosition": 2
        }
      ],
      "primaryKeys": ["id"],
      "foreignKeys": []
    }
  ],
  "views": [],
  "extractedAt": "2026-01-17T10:00:00Z"
}
```

#### 错误场景
- 数据库不存在: `"数据库连接不存在"`
- 提取失败: `"无法提取元数据：[错误详情]"`
- 权限不足: `"数据库用户没有读取元数据的权限"`

---

### 7. `refresh_metadata` - 刷新数据库元数据

#### 功能描述
强制重新从数据库提取最新元数据，更新缓存。

#### 请求参数
```rust
database_id: String
```

#### 返回值
```rust
Result<DatabaseMetadata, String>
```

#### 前端调用示例
```typescript
const metadata = await invoke<DatabaseMetadata>('refresh_metadata', {
  databaseId: 'db-1'
});
```

#### 错误场景
- 与 `get_database_metadata` 相同

---

## 查询执行 API

### 8. `run_sql_query` - 执行 SQL 查询

#### 功能描述
执行用户提供的 SQL 查询，自动添加 LIMIT 100（如需要），返回结果。

#### 请求参数
```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunQueryRequest {
    database_id: String,
    sql: String,
}
```

#### 返回值
```rust
Result<QueryResult, String>
```

#### 前端调用示例
```typescript
const result = await invoke<QueryResult>('run_sql_query', {
  databaseId: 'db-1',
  sql: 'SELECT * FROM users WHERE age > 30'
});
```

#### 返回示例
```json
{
  "columns": ["id", "name", "email", "age"],
  "rows": [
    {
      "id": 1,
      "name": "张三",
      "email": "zhangsan@example.com",
      "age": 35
    }
  ],
  "total": 1,
  "execTimeMs": 45,
  "sql": "SELECT * FROM users WHERE age > 30 LIMIT 100",
  "truncated": false
}
```

#### 错误场景
- 语法错误: `"SQL 语法错误：[错误详情]"`
- 权限错误: `"没有权限执行此查询"`
- 连接错误: `"数据库连接已断开"`
- DDL 阻止: `"不允许执行 DDL 语句（CREATE/DROP/ALTER）"`

---

### 9. `cancel_query` - 取消正在执行的查询

#### 功能描述
取消当前正在执行的查询（如果支持）。

#### 请求参数
```rust
database_id: String
```

#### 返回值
```rust
Result<(), String>
```

#### 前端调用示例
```typescript
await invoke('cancel_query', { databaseId: 'db-1' });
```

#### 错误场景
- 无正在执行的查询: `"没有正在执行的查询"`
- 取消失败: `"无法取消查询"`

---

## AI 查询 API

### 10. `run_nl_query` - 自然语言查询并执行

#### 功能描述
将用户的自然语言描述通过 AI 转换为 SQL，并自动执行查询，返回结果。

#### 请求参数
```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunNLQueryRequest {
    database_id: String,
    prompt: String,
}
```

#### 返回值
```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NLQueryResponse {
    generated_sql: String,
    result: QueryResult,
}

Result<NLQueryResponse, String>
```

#### 前端调用示例
```typescript
const response = await invoke<NLQueryResponse>('run_nl_query', {
  databaseId: 'db-1',
  prompt: '查询所有年龄大于30岁的用户姓名和邮箱'
});

console.log('生成的 SQL:', response.generatedSql);
console.log('查询结果:', response.result);
```

#### 返回示例
```json
{
  "generatedSql": "SELECT name, email FROM users WHERE age > 30 LIMIT 100",
  "result": {
    "columns": ["name", "email"],
    "rows": [
      {
        "name": "张三",
        "email": "zhangsan@example.com"
      }
    ],
    "total": 1,
    "execTimeMs": 50,
    "sql": "SELECT name, email FROM users WHERE age > 30 LIMIT 100",
    "truncated": false
  }
}
```

#### 错误场景
- AI 生成失败: `"无法生成 SQL 查询：[错误详情]"`
- 生成的 SQL 无效: `"AI 生成的 SQL 语法错误"`
- 执行失败: 与 `run_sql_query` 相同

---

### 11. `generate_sql_from_nl` - 仅生成 SQL（不执行）

#### 功能描述
将自然语言转换为 SQL，但不执行查询。用户可以审查后手动执行。

#### 请求参数
```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenerateSQLRequest {
    database_id: String,
    prompt: String,
}
```

#### 返回值
```rust
Result<String, String>
```

#### 前端调用示例
```typescript
const generatedSql = await invoke<string>('generate_sql_from_nl', {
  databaseId: 'db-1',
  prompt: '查询所有用户'
});

// 将生成的 SQL 显示在编辑器中供用户审查
setSqlCode(generatedSql);
```

#### 返回示例
```json
"SELECT * FROM users LIMIT 100"
```

#### 错误场景
- AI 生成失败: `"无法生成 SQL 查询：[错误详情]"`
- 网络错误: `"无法连接到 OpenAI API"`
- API 限额: `"OpenAI API 配额已用尽"`

---

## 错误处理规范

所有 Command 返回 `Result<T, String>`，错误消息使用中文，遵循以下格式：

```rust
Err(format!("错误类型：{}", 详细信息))
```

### 错误类型

| 错误类型 | 示例消息 |
|---------|---------|
| 连接错误 | `"数据库连接失败：连接超时"` |
| 语法错误 | `"SQL 语法错误：第 3 行缺少分号"` |
| 权限错误 | `"权限不足：没有 SELECT 权限"` |
| 网络错误 | `"网络错误：无法连接到 OpenAI API"` |
| 参数错误 | `"参数错误：数据库 ID 不能为空"` |
| 资源不存在 | `"数据库连接不存在"` |

---

## 性能要求

| API | 预期响应时间 |
|-----|-------------|
| `list_databases` | < 100ms（从 SQLite 读取）|
| `add_database` | < 30s（包含元数据提取）|
| `get_database_metadata` | < 200ms（缓存），< 10s（首次提取）|
| `run_sql_query` | < 2s（1000行内）|
| `run_nl_query` | < 10s（包含 AI 生成时间）|

---

## 安全性

- 密码存储使用 AES-256 加密
- 所有 DDL 语句默认拒绝执行
- 查询自动添加 LIMIT 100 防止过大结果集
- 错误消息不泄露敏感信息（如数据库密码）

---

## 总结

所有 Tauri Command API 已定义完成，符合：
- ✅ CamelCase JSON 格式（serde rename_all）
- ✅ 完整类型定义（Rust + TypeScript）
- ✅ 中文错误消息
- ✅ 清晰的请求/响应示例

**下一步**: 创建 quickstart.md 快速开始指南
