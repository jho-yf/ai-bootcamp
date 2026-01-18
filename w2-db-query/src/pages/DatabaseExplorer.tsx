/**
 * DatabaseExplorer 页面
 * 数据库结构浏览器：侧边栏显示元数据树，右侧显示详细信息或 SQL 查询
 */
import React, { useState } from 'react';
import { Layout, Button, Space, Card, message } from 'antd';
import { ReloadOutlined, PlayCircleOutlined, ClearOutlined } from '@ant-design/icons';
import { MetadataTree } from '../components/MetadataTree';
import { SqlEditor } from '../components/SqlEditor';
import { QueryResultTable } from '../components/QueryResultTable';
import { LoadingIndicator } from '../components/LoadingIndicator';
import { useDatabases } from '../hooks/useDatabases';
import { useMetadata } from '../hooks/useMetadata';
import { useQuery } from '../hooks/useQuery';
import type { TableInfo, ViewInfo, RunQueryRequest } from '../services/types';

const { Sider, Content } = Layout;

interface DatabaseExplorerProps {
  selectedDbId: string;
}

export const DatabaseExplorer: React.FC<DatabaseExplorerProps> = ({
  selectedDbId,
}) => {
  const { databases } = useDatabases();
  const { metadata, loading, loadMetadata, refreshMetadata } = useMetadata(selectedDbId);
  const { result, loading: queryLoading, error, executeQuery, cancelQuery, clearResult } = useQuery();
  const [selectedTable, setSelectedTable] = useState<TableInfo | null>(null);
  const [sql, setSql] = useState('SELECT * FROM ');

  const currentDatabase = databases.find(db => db.id === selectedDbId);

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

  // 监听 Ctrl+Enter 快捷键
  React.useEffect(() => {
    const handleExecute = () => {
      if (selectedDbId && sql.trim()) {
        handleRunQuery();
      }
    };

    window.addEventListener('execute-query', handleExecute);
    return () => {
      window.removeEventListener('execute-query', handleExecute);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedDbId, sql]);

  // 当选择表时，自动填充 SQL
  React.useEffect(() => {
    if (selectedTable) {
      setSql(`SELECT * FROM ${selectedTable.schema ? `${selectedTable.schema}.` : ''}${selectedTable.name} LIMIT 100;`);
    }
  }, [selectedTable]);

  const handleTableSelect = (table: TableInfo) => {
    setSelectedTable(table);
  };

  const handleViewSelect = (view: ViewInfo) => {
    setSelectedTable(null);
    // 当选择视图时，自动填充 SQL
    setSql(`SELECT * FROM ${view.schema ? `${view.schema}.` : ''}${view.name} LIMIT 100;`);
  };

  const handleTableDoubleClick = async (table: TableInfo) => {
    // 双击表时，填充 SQL 并执行查询
    const sqlQuery = `SELECT * FROM ${table.schema ? `${table.schema}.` : ''}${table.name} LIMIT 100;`;
    setSql(sqlQuery);
    setSelectedTable(table);

    // 执行查询
    try {
      const request: RunQueryRequest = {
        databaseId: selectedDbId,
        sql: sqlQuery,
      };
      await executeQuery(request);
    } catch (err) {
      // 错误已在 useQuery 中处理
    }
  };

  const handleViewDoubleClick = async (view: ViewInfo) => {
    // 双击视图时，填充 SQL 并执行查询
    const sqlQuery = `SELECT * FROM ${view.schema ? `${view.schema}.` : ''}${view.name} LIMIT 100;`;
    setSql(sqlQuery);
    setSelectedTable(null);

    // 执行查询
    try {
      const request: RunQueryRequest = {
        databaseId: selectedDbId,
        sql: sqlQuery,
      };
      await executeQuery(request);
    } catch (err) {
      // 错误已在 useQuery 中处理
    }
  };

  // 当 selectedDbId 变化时，重置状态并加载元数据
  React.useEffect(() => {
    setSelectedTable(null);
    clearResult();
    if (selectedDbId) {
      loadMetadata();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedDbId]);


  return (
    <Layout style={{ height: '100vh' }}>
      <div style={{ padding: 16, borderBottom: '1px solid #f0f0f0', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Space>
          <span style={{ fontWeight: 500, fontSize: 16 }}>
            {currentDatabase?.name || '数据库'}
          </span>
          <Button
            icon={<ReloadOutlined />}
            onClick={() => refreshMetadata()}
            loading={loading}
            size="small"
          >
            刷新元数据
          </Button>
        </Space>
      </div>

      <Layout>
        <Sider
          width={300}
          style={{
            background: '#fff',
            borderRight: '1px solid #f0f0f0',
            overflow: 'auto',
            height: 'calc(100vh - 73px)' // 减去顶部工具栏的高度
          }}
        >
          <MetadataTree
            metadata={metadata}
            loading={loading}
            onTableSelect={handleTableSelect}
            onViewSelect={handleViewSelect}
            onTableDoubleClick={handleTableDoubleClick}
            onViewDoubleClick={handleViewDoubleClick}
          />
        </Sider>

        <Content style={{ display: 'flex', flexDirection: 'column', overflow: 'hidden', height: 'calc(100vh - 73px)' }}>
          <div style={{
            display: 'flex',
            flexDirection: 'column',
            height: '100%',
            overflow: 'hidden',
            minHeight: 0
          }}>
            <div style={{
              padding: 16,
              borderBottom: '1px solid #f0f0f0',
              flexShrink: 0
            }}>
              <Space>
                <Button
                  type="primary"
                  icon={<PlayCircleOutlined />}
                  onClick={handleRunQuery}
                  loading={queryLoading}
                  disabled={!sql.trim()}
                >
                  执行查询 (Ctrl+Enter)
                </Button>
                <Button icon={<ClearOutlined />} onClick={clearResult} disabled={!result}>
                  清除结果
                </Button>
              </Space>
            </div>
            <div style={{
              flex: '1 1 0',
              minHeight: 0,
              borderBottom: '1px solid #f0f0f0',
              position: 'relative',
              display: 'flex',
              flexDirection: 'column'
            }}>
              <div style={{ flex: 1, minHeight: 0 }}>
                <SqlEditor value={sql} onChange={(value) => setSql(value || '')} height="100%" />
              </div>
              <LoadingIndicator loading={queryLoading} onCancel={handleCancel} />
            </div>
            <div style={{
              flex: '1 1 0',
              minHeight: 0,
              overflow: 'auto',
              padding: 16
            }}>
              {error && (
                <Card style={{ marginBottom: 16, borderColor: '#ff4d4f' }}>
                  <div style={{ color: '#ff4d4f' }}>错误: {error}</div>
                </Card>
              )}
              <QueryResultTable result={result} loading={queryLoading} />
            </div>
          </div>
        </Content>
      </Layout>
    </Layout>
  );
};
