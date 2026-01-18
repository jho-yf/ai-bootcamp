/**
 * DatabaseCard 组件
 * 显示单个数据库连接卡片
 */
import React from 'react';
import { Card, Tag, Button, Space, Popconfirm } from 'antd';
import { DatabaseOutlined, DeleteOutlined, EditOutlined } from '@ant-design/icons';
import type { DatabaseConnection } from '../services/types';

interface DatabaseCardProps {
  database: DatabaseConnection;
  onEdit?: (db: DatabaseConnection) => void;
  onDelete?: (id: string) => void;
  onClick?: (db: DatabaseConnection) => void;
}

export const DatabaseCard: React.FC<DatabaseCardProps> = ({
  database,
  onEdit,
  onDelete,
  onClick,
}) => {
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'connected':
        return 'success';
      case 'connecting':
        return 'processing';
      case 'failed':
        return 'error';
      default:
        return 'default';
    }
  };

  const getStatusText = (status: string) => {
    switch (status) {
      case 'connected':
        return '已连接';
      case 'connecting':
        return '连接中';
      case 'failed':
        return '连接失败';
      default:
        return '未连接';
    }
  };

  return (
    <Card
      hoverable
      style={{ marginBottom: 16 }}
      onClick={() => onClick?.(database)}
      actions={[
        <Button
          key="edit"
          type="text"
          icon={<EditOutlined />}
          onClick={(e) => {
            e.stopPropagation();
            onEdit?.(database);
          }}
        >
          编辑
        </Button>,
        <Popconfirm
          key="delete"
          title="确定要删除这个数据库连接吗？"
          onConfirm={() => {
            onDelete?.(database.id);
          }}
        >
          <Button
            type="text"
            danger
            icon={<DeleteOutlined />}
            onClick={(e) => e.stopPropagation()}
          >
            删除
          </Button>
        </Popconfirm>,
      ]}
    >
      <Card.Meta
        avatar={<DatabaseOutlined style={{ fontSize: 24 }} />}
        title={database.name}
        description={
          <Space direction="vertical" size="small" style={{ width: '100%' }}>
            <div>
              <Tag color={getStatusColor(database.status)}>
                {getStatusText(database.status)}
              </Tag>
            </div>
            <div style={{ fontSize: 12, color: '#666' }}>
              {database.host}:{database.port} / {database.databaseName}
            </div>
            <div style={{ fontSize: 12, color: '#999' }}>
              用户: {database.user}
            </div>
          </Space>
        }
      />
    </Card>
  );
};
