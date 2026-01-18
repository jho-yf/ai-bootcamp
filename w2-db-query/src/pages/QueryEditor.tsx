/**
 * QueryEditor 页面
 * SQL 编辑器和查询结果展示
 */
import React, { useState, useEffect } from 'react';
import { Layout, Button, Select, Space, Card, message } from 'antd';
import { PlayCircleOutlined, ClearOutlined } from '@ant-design/icons';
import { SqlEditor } from '../components/SqlEditor';
import { QueryResultTable } from '../components/QueryResultTable';
import { LoadingIndicator } from '../components/LoadingIndicator';
import { useDatabases } from '../hooks/useDatabases';
import { useQuery } from '../hooks/useQuery';
import type { RunQueryRequest } from '../services/types';

const { Content } = Layout;

export const QueryEditor: React.FC = () => {
  const { databases } = useDatabases();
  const { result, loading, error, executeQuery, cancelQuery, clearResult } = useQuery();
  const [selectedDbId, setSelectedDbId] = useState<string | null>(null);
  const [sql, setSql] = useState('SELECT * FROM ');

  const handleRunQuery = async () => {
    if (!selectedDbId) {
      message.warning('请先选择数据库连接');
      return;
    }

    if (!sql.trim()) {
      message.warning('请输入 SQL 查询');
      return;
    }

    try {
      const request: RunQueryRequest = {
        databaseId: selectedDbId,
        sql: sql.trim(),
      };
      await executeQuery(request);
    } catch (err) {
      // 错误已在 useQuery 中处理
    }
  };

  const handleCancel = () => {
    if (selectedDbId) {
      cancelQuery(selectedDbId);
    }
  };

  // 默认选中第一个数据库
  useEffect(() => {
    if (databases.length > 0 && !selectedDbId) {
      setSelectedDbId(databases[0].id);
    }
  }, [databases, selectedDbId]);

  // 监听 Ctrl+Enter 快捷键
  useEffect(() => {
    const handleExecute = () => {
      if (selectedDbId && sql.trim()) {
        handleRunQuery();
      }
    };

    window.addEventListener('execute-query', handleExecute);
    return () => {
      window.removeEventListener('execute-query', handleExecute);
    };
  }, [selectedDbId, sql]);

  return (
    <Layout style={{ height: '100vh', position: 'relative' }}>
      <div style={{ padding: 16, borderBottom: '1px solid #f0f0f0' }}>
        <Space>
          <Select
            placeholder="选择数据库"
            style={{ width: 300 }}
            value={selectedDbId}
            onChange={setSelectedDbId}
            options={databases.map((db) => ({
              label: db.name,
              value: db.id,
            }))}
          />
          <Button
            type="primary"
            icon={<PlayCircleOutlined />}
            onClick={handleRunQuery}
            loading={loading}
            disabled={!selectedDbId || !sql.trim()}
          >
            执行查询 (Ctrl+Enter)
          </Button>
          <Button icon={<ClearOutlined />} onClick={clearResult} disabled={!result}>
            清除结果
          </Button>
        </Space>
      </div>

      <Content style={{ display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        <div style={{ flex: 1, borderBottom: '1px solid #f0f0f0', position: 'relative' }}>
          <SqlEditor value={sql} onChange={(value) => setSql(value || '')} height="100%" />
          <LoadingIndicator loading={loading} onCancel={handleCancel} />
        </div>

        <div style={{ flex: 1, overflow: 'auto', padding: 16 }}>
          {error && (
            <Card style={{ marginBottom: 16, borderColor: '#ff4d4f' }}>
              <div style={{ color: '#ff4d4f' }}>错误: {error}</div>
            </Card>
          )}
          <QueryResultTable result={result} loading={loading} />
        </div>
      </Content>
    </Layout>
  );
};
