/**
 * useDatabases Hook
 * 封装数据库连接的 CRUD 操作
 */
import { useState, useEffect, useCallback } from 'react';
import { message } from 'antd';
import type { DatabaseConnection, AddDatabaseRequest, UpdateDatabaseRequest } from '../services/types';
import * as api from '../services/api';

export function useDatabases() {
  const [databases, setDatabases] = useState<DatabaseConnection[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 加载所有数据库连接
  const loadDatabases = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.listDatabases();
      setDatabases(data);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '加载数据库列表失败';
      setError(errorMsg);
      message.error(errorMsg);
    } finally {
      setLoading(false);
    }
  }, []);

  // 添加数据库连接
  const addDatabase = useCallback(async (request: AddDatabaseRequest) => {
    setLoading(true);
    setError(null);
    try {
      const newDb = await api.addDatabase(request);
      setDatabases((prev) => [...prev, newDb]);
      message.success('数据库连接添加成功');
      return newDb;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '添加数据库连接失败';
      setError(errorMsg);
      message.error(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  // 更新数据库连接
  const updateDatabase = useCallback(async (request: UpdateDatabaseRequest) => {
    setLoading(true);
    setError(null);
    try {
      const updated = await api.updateDatabase(request);
      setDatabases((prev) =>
        prev.map((db) => (db.id === updated.id ? updated : db))
      );
      message.success('数据库连接更新成功');
      return updated;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '更新数据库连接失败';
      setError(errorMsg);
      message.error(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  // 删除数据库连接
  const deleteDatabase = useCallback(async (databaseId: string) => {
    setLoading(true);
    setError(null);
    try {
      await api.deleteDatabase(databaseId);
      setDatabases((prev) => prev.filter((db) => db.id !== databaseId));
      message.success('数据库连接删除成功');
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '删除数据库连接失败';
      setError(errorMsg);
      message.error(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  // 测试连接
  const testConnection = useCallback(async (request: Omit<AddDatabaseRequest, 'name'>) => {
    setLoading(true);
    setError(null);
    try {
      const result = await api.testConnection(request);
      if (result) {
        message.success('连接测试成功');
      }
      return result;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '连接测试失败';
      setError(errorMsg);
      message.error(errorMsg);
      return false;
    } finally {
      setLoading(false);
    }
  }, []);

  // 初始化时加载
  useEffect(() => {
    loadDatabases();
  }, [loadDatabases]);

  return {
    databases,
    loading,
    error,
    loadDatabases,
    addDatabase,
    updateDatabase,
    deleteDatabase,
    testConnection,
  };
}
