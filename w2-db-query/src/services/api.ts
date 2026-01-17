/**
 * Tauri API 封装
 * 统一封装所有 Tauri invoke 调用，提供类型安全的 API 接口
 */
import { invoke } from '@tauri-apps/api/tauri';
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
 * 获取所有数据库连接
 */
export async function listDatabases(): Promise<DatabaseConnection[]> {
  return await invoke<DatabaseConnection[]>('list_databases');
}

/**
 * 添加新数据库连接
 */
export async function addDatabase(
  request: AddDatabaseRequest
): Promise<DatabaseConnection> {
  return await invoke<DatabaseConnection>('add_database', request);
}

/**
 * 更新数据库连接
 */
export async function updateDatabase(
  request: UpdateDatabaseRequest
): Promise<DatabaseConnection> {
  return await invoke<DatabaseConnection>('update_database', request);
}

/**
 * 删除数据库连接
 */
export async function deleteDatabase(databaseId: string): Promise<void> {
  return await invoke('delete_database', { databaseId });
}

/**
 * 测试数据库连接
 */
export async function testConnection(
  request: Omit<AddDatabaseRequest, 'name'>
): Promise<boolean> {
  return await invoke<boolean>('test_connection', request);
}

/**
 * 获取数据库元数据
 */
export async function getDatabaseMetadata(
  databaseId: string
): Promise<DatabaseMetadata> {
  return await invoke<DatabaseMetadata>('get_database_metadata', {
    databaseId,
  });
}

/**
 * 刷新数据库元数据
 */
export async function refreshMetadata(
  databaseId: string
): Promise<DatabaseMetadata> {
  return await invoke<DatabaseMetadata>('refresh_metadata', { databaseId });
}

/**
 * 执行 SQL 查询
 */
export async function runSqlQuery(
  request: RunQueryRequest
): Promise<QueryResult> {
  return await invoke<QueryResult>('run_sql_query', request);
}

/**
 * 取消正在执行的查询
 */
export async function cancelQuery(databaseId: string): Promise<void> {
  return await invoke('cancel_query', { databaseId });
}

/**
 * 自然语言生成并执行查询
 */
export async function runNLQuery(
  request: RunNLQueryRequest
): Promise<NLQueryResponse> {
  return await invoke<NLQueryResponse>('run_nl_query', request);
}

/**
 * 仅生成 SQL（不执行）
 */
export async function generateSqlFromNL(
  request: RunNLQueryRequest
): Promise<string> {
  return await invoke<string>('generate_sql_from_nl', request);
}
