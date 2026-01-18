/**
 * useMetadata Hook
 * 加载和刷新数据库元数据
 */
import { useState, useCallback } from 'react';
import { message } from 'antd';
import type { DatabaseMetadata } from '../services/types';
import * as api from '../services/api';

export function useMetadata(databaseId: string | null) {
  const [metadata, setMetadata] = useState<DatabaseMetadata | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 加载元数据（优先从缓存）
  const loadMetadata = useCallback(async () => {
    if (!databaseId) {
      setMetadata(null);
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const data = await api.getDatabaseMetadata(databaseId);
      setMetadata(data);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '加载元数据失败';
      setError(errorMsg);
      message.error(errorMsg);
    } finally {
      setLoading(false);
    }
  }, [databaseId]);

  // 刷新元数据（强制重新提取）
  const refreshMetadata = useCallback(async () => {
    if (!databaseId) {
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const data = await api.refreshMetadata(databaseId);
      setMetadata(data);
      message.success('元数据刷新成功');
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '刷新元数据失败';
      setError(errorMsg);
      message.error(errorMsg);
    } finally {
      setLoading(false);
    }
  }, [databaseId]);

  return {
    metadata,
    loading,
    error,
    loadMetadata,
    refreshMetadata,
  };
}
