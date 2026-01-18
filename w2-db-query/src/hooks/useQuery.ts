/**
 * useQuery Hook
 * 执行 SQL 查询并管理加载状态（包括自然语言查询）
 */
import { useState, useCallback } from "react";
import { message } from "antd";
import type {
  QueryResult,
  RunQueryRequest,
  RunNLQueryRequest,
  NLQueryResponse,
} from "../services/types";
import * as api from "../services/api";

export function useQuery() {
  const [result, setResult] = useState<QueryResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [generatedSql, setGeneratedSql] = useState<string | null>(null);

  // 执行 SQL 查询
  const executeQuery = useCallback(async (request: RunQueryRequest) => {
    setLoading(true);
    setError(null);
    setResult(null);
    setGeneratedSql(null);

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
      const errorMsg = err instanceof Error ? err.message : "查询执行失败";
      setError(errorMsg);
      message.error(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  // 从自然语言生成 SQL（不执行）
  const generateSqlFromNL = useCallback(async (request: RunNLQueryRequest) => {
    setLoading(true);
    setError(null);
    setGeneratedSql(null);

    try {
      const sql = await api.generateSqlFromNL(request);
      setGeneratedSql(sql);
      message.success("SQL 生成成功");
      return sql;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : "生成 SQL 失败";
      setError(errorMsg);
      message.error(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  // 执行自然语言查询（生成并执行）
  const executeNLQuery = useCallback(async (request: RunNLQueryRequest) => {
    setLoading(true);
    setError(null);
    setResult(null);

    try {
      const response: NLQueryResponse = await api.runNLQuery(request);
      setGeneratedSql(response.generatedSql);
      setResult(response.result);

      if (response.result.truncated) {
        message.warning(`查询结果已截断，仅显示前 ${response.result.total} 行`);
      } else {
        message.success(`查询成功，返回 ${response.result.total} 行`);
      }

      return response;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : "自然语言查询失败";
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
      message.info("查询已取消");
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : "取消查询失败";
      message.error(errorMsg);
    }
  }, []);

  // 清除结果
  const clearResult = useCallback(() => {
    setResult(null);
    setError(null);
    setGeneratedSql(null);
  }, []);

  return {
    result,
    loading,
    error,
    generatedSql,
    executeQuery,
    generateSqlFromNL,
    executeNLQuery,
    cancelQuery,
    clearResult,
  };
}
