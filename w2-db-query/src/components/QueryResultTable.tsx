/**
 * QueryResultTable 组件
 * 使用 Ant Design Table 显示查询结果
 */
import React, { useMemo } from 'react';
import { Table, Empty, Tag } from 'antd';
import type { QueryResult } from '../services/types';
import type { ColumnsType } from 'antd/es/table';

interface QueryResultTableProps {
  result: QueryResult | null;
  loading?: boolean;
}

export const QueryResultTable: React.FC<QueryResultTableProps> = ({
  result,
  loading = false,
}) => {
  const columns: ColumnsType<any> = useMemo(() => {
    if (!result || result.columns.length === 0) return [];

    return result.columns.map((colName) => ({
      title: colName,
      dataIndex: colName,
      key: colName,
      ellipsis: true,
      width: 150,
      render: (value: any) => {
        if (value === null || value === undefined) {
          return <span style={{ color: '#999' }}>NULL</span>;
        }
        if (typeof value === 'boolean') {
          return <Tag color={value ? 'success' : 'default'}>{value ? 'true' : 'false'}</Tag>;
        }
        if (typeof value === 'number') {
          return <span>{value.toLocaleString()}</span>;
        }
        if (typeof value === 'string' && value.length > 100) {
          return (
            <span title={value}>
              {value.substring(0, 100)}...
            </span>
          );
        }
        return <span>{String(value)}</span>;
      },
    }));
  }, [result]);

  const dataSource = useMemo(() => {
    if (!result) return [];
    return result.rows.map((row, index) => ({
      key: index,
      ...row,
    }));
  }, [result]);

  if (!result) {
    return (
      <div style={{ padding: 50, textAlign: 'center' }}>
        <Empty description="暂无查询结果" />
      </div>
    );
  }

  return (
    <div>
      <div style={{ marginBottom: 16, padding: '8px 16px', background: '#f5f5f5', borderRadius: 4 }}>
        <span>共 {result.total} 行</span>
        {result.truncated && (
          <Tag color="warning" style={{ marginLeft: 8 }}>
            结果已截断（LIMIT 100）
          </Tag>
        )}
        <span style={{ marginLeft: 16, color: '#666' }}>
          执行时间: {result.execTimeMs}ms
        </span>
      </div>
      <Table
        columns={columns}
        dataSource={dataSource}
        loading={loading}
        scroll={{ x: 'max-content', y: 400 }}
        pagination={{
          pageSize: 50,
          showSizeChanger: true,
          showTotal: (total) => `共 ${total} 条`,
        }}
        size="small"
      />
    </div>
  );
};
