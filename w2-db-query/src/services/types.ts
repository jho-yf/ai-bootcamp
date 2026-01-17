/**
 * TypeScript 类型定义
 * 映射后端 Rust 数据模型，确保前后端类型一致
 */

// 数据库连接状态
export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'failed';

// 数据库连接
export interface DatabaseConnection {
  id: string;
  name: string;
  host: string;
  port: number;
  databaseName: string;
  user: string;
  // password 不从后端返回
  status: ConnectionStatus;
  createdAt: string; // ISO 8601 格式
  updatedAt: string;
}

// 列信息
export interface ColumnInfo {
  name: string;
  dataType: string;
  nullable: boolean;
  defaultValue?: string;
  isPrimaryKey: boolean;
  ordinalPosition: number;
}

// 外键信息
export interface ForeignKeyInfo {
  constraintName: string;
  columnName: string;
  referencedTable: string;
  referencedColumn: string;
}

// 表信息
export interface TableInfo {
  schema: string;
  name: string;
  tableType: string;
  columns: ColumnInfo[];
  primaryKeys: string[];
  foreignKeys: ForeignKeyInfo[];
}

// 视图信息
export interface ViewInfo {
  schema: string;
  name: string;
  columns: ColumnInfo[];
  definition?: string;
}

// 数据库元数据
export interface DatabaseMetadata {
  connectionId: string;
  tables: TableInfo[];
  views: ViewInfo[];
  extractedAt: string;
}

// 查询结果
export interface QueryResult {
  columns: string[];
  rows: Record<string, any>[];
  total: number;
  execTimeMs: number;
  sql: string;
  truncated: boolean;
}

// 自然语言查询响应
export interface NLQueryResponse {
  generatedSql: string;
  result: QueryResult;
}

// 添加数据库请求
export interface AddDatabaseRequest {
  name: string;
  host: string;
  port: number;
  databaseName: string;
  user: string;
  password: string;
}

// 更新数据库请求
export interface UpdateDatabaseRequest {
  id: string;
  name?: string;
  host?: string;
  port?: number;
  databaseName?: string;
  user?: string;
  password?: string;
}

// 执行查询请求
export interface RunQueryRequest {
  databaseId: string;
  sql: string;
}

// 自然语言查询请求
export interface RunNLQueryRequest {
  databaseId: string;
  prompt: string;
}
