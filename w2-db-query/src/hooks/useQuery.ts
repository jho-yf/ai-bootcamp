/**
 * useQuery Hook
 * 执行 SQL 查询并管理加载状态
 */
import { useState, useCallback } from 'react';
import { message } from 'antd';
import type { QueryResult, RunQueryRequest } from '../services/types';
import * as api from '../services/api';

export function useQuery() {
  const [result, setResult] = useState<QueryResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 执行 SQL 查询
  const executeQuery = useCallback(async (request: RunQueryRequest) => {
    setLoading(true);
    setError(null);
    setResult(null);

    try {
      const queryResult = await api.runSqlQuery(request);
      setResult(queryResult);

      if (queryResult.truncated) {
        message.warning(`查询结果已截断，仅显示前 ${queryResult.total} 行`);
      } else {
        message.success(`查询成功，返回 ${queryResult.total} 行`);
      }

      return queryResult;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '查询执行失败';
      setError(errorMsg);
      message.error(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  // 取消查询
  const cancelQuery = useCallback(async (databaseId: string) => {
    try {
      await api.cancelQuery(databaseId);
      setLoading(false);
      message.info('查询已取消');
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '取消查询失败';
      message.error(errorMsg);
    }
  }, []);

  // 清除结果
  const clearResult = useCallback(() => {
    setResult(null);
    setError(null);
  }, []);

  return {
    result,
    loading,
    error,
    executeQuery,
    cancelQuery,
    clearResult,
  };
}
