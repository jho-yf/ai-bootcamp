/**
 * Tauri API 封装
 * 统一封装所有 Tauri invoke 调用，提供类型安全的 API 接口
 */
import { invoke } from '@tauri-apps/api/core';
import { message } from 'antd';
import type {
  DatabaseConnection,
  DatabaseMetadata,
  QueryResult,
  NLQueryResponse,
  AddDatabaseRequest,
  UpdateDatabaseRequest,
  RunQueryRequest,
  RunNLQueryRequest,
} from './types';

/**
 * 统一的错误处理包装器
 */
async function handleInvoke<T>(
  command: string,
  args?: Record<string, any>
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    const errorMsg = typeof error === 'string' ? error : '操作失败';
    message.error(errorMsg);
    throw new Error(errorMsg);
  }
}

/**
 * 获取所有数据库连接
 */
export async function listDatabases(): Promise<DatabaseConnection[]> {
  return handleInvoke<DatabaseConnection[]>('list_databases');
}

/**
 * 添加新数据库连接
 */
export async function addDatabase(
  request: AddDatabaseRequest
): Promise<DatabaseConnection> {
  return handleInvoke<DatabaseConnection>('add_database', { request });
}

/**
 * 更新数据库连接
 */
export async function updateDatabase(
  request: UpdateDatabaseRequest
): Promise<DatabaseConnection> {
  return handleInvoke<DatabaseConnection>('update_database', { request });
}

/**
 * 删除数据库连接
 */
export async function deleteDatabase(databaseId: string): Promise<void> {
  return handleInvoke('delete_database', { databaseId });
}

/**
 * 测试数据库连接
 */
export async function testConnection(
  request: Omit<AddDatabaseRequest, 'name'>
): Promise<boolean> {
  return handleInvoke<boolean>('test_connection', { request });
}

/**
 * 获取数据库元数据
 */
export async function getDatabaseMetadata(
  databaseId: string
): Promise<DatabaseMetadata> {
  return handleInvoke<DatabaseMetadata>('get_database_metadata', {
    databaseId,
  });
}

/**
 * 刷新数据库元数据
 */
export async function refreshMetadata(
  databaseId: string
): Promise<DatabaseMetadata> {
  return handleInvoke<DatabaseMetadata>('refresh_metadata', { databaseId });
}

/**
 * 执行 SQL 查询
 */
export async function runSqlQuery(
  request: RunQueryRequest
): Promise<QueryResult> {
  return handleInvoke<QueryResult>('run_sql_query', { request });
}

/**
 * 取消正在执行的查询
 */
export async function cancelQuery(databaseId: string): Promise<void> {
  return handleInvoke('cancel_query', { databaseId });
}

/**
 * 自然语言生成并执行查询
 */
export async function runNLQuery(
  request: RunNLQueryRequest
): Promise<NLQueryResponse> {
  return handleInvoke<NLQueryResponse>('run_nl_query', { request });
}

/**
 * 仅生成 SQL（不执行）
 */
export async function generateSqlFromNL(
  request: RunNLQueryRequest
): Promise<string> {
  return handleInvoke<string>('generate_sql_from_nl', { request });
}
