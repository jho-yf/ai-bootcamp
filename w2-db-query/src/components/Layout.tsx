/**
 * Layout 组件
 * 全局布局：顶部导航栏、左侧数据库列表菜单、右侧内容区域
 */
import React, { useState, useEffect } from 'react';
import { Layout, Menu, Typography, Button, Space } from 'antd';
import { DatabaseOutlined, PlusOutlined, EditOutlined, DeleteOutlined } from '@ant-design/icons';
import { DatabaseExplorer } from '../pages/DatabaseExplorer';
import { AddDatabaseDialog } from './AddDatabaseDialog';
import { useDatabases } from '../hooks/useDatabases';
import type { DatabaseConnection, AddDatabaseRequest } from '../services/types';

const { Header, Content, Sider } = Layout;
const { Title } = Typography;

// 数据库菜单项组件
interface DatabaseMenuItemProps {
  db: DatabaseConnection;
  onEdit: (db: DatabaseConnection) => void;
  onDelete: (id: string) => void;
}

const DatabaseMenuItem: React.FC<DatabaseMenuItemProps> = ({ db, onEdit, onDelete }) => {
  const [isHovered, setIsHovered] = useState(false);

  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        width: '100%',
      }}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      <span style={{ flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
        {db.name}
      </span>
      <Space
        className="db-actions"
        size={4}
        style={{
          opacity: isHovered ? 1 : 0,
          transition: 'opacity 0.2s',
          marginLeft: 8,
          pointerEvents: isHovered ? 'auto' : 'none',
        }}
      >
        <Button
          type="text"
          size="small"
          icon={<EditOutlined />}
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            onEdit(db);
          }}
          onMouseDown={(e) => {
            e.preventDefault();
            e.stopPropagation();
          }}
          title="编辑"
          style={{
            padding: '0 4px',
            height: '20px',
            width: '20px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        />
        <Button
          type="text"
          size="small"
          icon={<DeleteOutlined />}
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            if (window.confirm(`确定要删除数据库连接 "${db.name}" 吗？`)) {
              onDelete(db.id);
            }
          }}
          onMouseDown={(e) => {
            e.preventDefault();
            e.stopPropagation();
          }}
          title="删除"
          danger
          style={{
            padding: '0 4px',
            height: '20px',
            width: '20px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        />
      </Space>
    </div>
  );
};

export const AppLayout: React.FC = () => {
  const { databases, loading, addDatabase, updateDatabase, deleteDatabase } = useDatabases();
  const [selectedDbId, setSelectedDbId] = useState<string | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editing, setEditing] = useState<DatabaseConnection | null>(null);

  // 默认选中第一个数据库
  useEffect(() => {
    if (databases.length > 0 && !selectedDbId) {
      setSelectedDbId(databases[0].id);
    }
  }, [databases, selectedDbId]);

  const handleAdd = () => {
    setEditing(null);
    setDialogOpen(true);
  };

  const handleEdit = (db: DatabaseConnection) => {
    setEditing(db);
    setDialogOpen(true);
  };

  const handleDelete = async (id: string) => {
    await deleteDatabase(id);
    // 如果删除的是当前选中的数据库，切换到第一个
    if (id === selectedDbId) {
      const remaining = databases.filter(db => db.id !== id);
      if (remaining.length > 0) {
        setSelectedDbId(remaining[0].id);
      } else {
        setSelectedDbId(null);
      }
    }
  };

  const handleDialogOk = async (values: AddDatabaseRequest) => {
    if (editing) {
      await updateDatabase({
        id: editing.id,
        ...values,
      });
    } else {
      const newDb = await addDatabase(values);
      // 添加后自动选中新数据库
      if (newDb) {
        setSelectedDbId(newDb.id);
      }
    }
    setDialogOpen(false);
    setEditing(null);
  };

  const menuItems = databases.map((db) => ({
    key: db.id,
    icon: <DatabaseOutlined />,
    label: <DatabaseMenuItem db={db} onEdit={handleEdit} onDelete={handleDelete} />,
  }));

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Header style={{ background: '#001529', padding: '0 24px', display: 'flex', alignItems: 'center' }}>
        <Title level={4} style={{ color: '#fff', margin: 0 }}>
          数据库查询工具
        </Title>
      </Header>
      <Layout>
        <Sider width={250} style={{ background: '#fff', borderRight: '1px solid #f0f0f0' }}>
          <div style={{ padding: 16, borderBottom: '1px solid #f0f0f0' }}>
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={handleAdd}
              block
            >
              添加数据库连接
            </Button>
          </div>
          <Menu
            mode="inline"
            selectedKeys={selectedDbId ? [selectedDbId] : []}
            items={menuItems}
            onClick={({ key, domEvent }) => {
              // 检查点击的是否是操作按钮
              const target = domEvent.target as HTMLElement;
              // 如果点击的是按钮或按钮内的图标，不处理菜单点击
              if (
                target.closest('.db-actions') ||
                target.closest('button') ||
                target.tagName === 'BUTTON' ||
                target.closest('svg') ||
                target.closest('.anticon')
              ) {
                return;
              }
              setSelectedDbId(key as string);
            }}
            style={{ height: 'calc(100vh - 113px)', borderRight: 0, overflow: 'auto' }}
          />
          {databases.length === 0 && (
            <div style={{ padding: 24, textAlign: 'center', color: '#999' }}>
              暂无数据库连接
              <br />
              <Button type="link" onClick={handleAdd} style={{ padding: 0, marginTop: 8 }}>
                点击添加
              </Button>
            </div>
          )}
        </Sider>
        <Content style={{ background: '#fff', marginLeft: 16 }}>
          {selectedDbId ? (
            <DatabaseExplorer
              selectedDbId={selectedDbId}
            />
          ) : (
            <div style={{ padding: 50, textAlign: 'center', color: '#999' }}>
              {databases.length === 0 ? (
                <>
                  <DatabaseOutlined style={{ fontSize: 64, marginBottom: 16 }} />
                  <div>还没有添加任何数据库连接</div>
                  <Button type="primary" icon={<PlusOutlined />} onClick={handleAdd} style={{ marginTop: 16 }}>
                    添加数据库连接
                  </Button>
                </>
              ) : (
                <div>请从左侧选择一个数据库</div>
              )}
            </div>
          )}
        </Content>
      </Layout>
      <AddDatabaseDialog
        open={dialogOpen}
        editing={editing}
        onCancel={() => {
          setDialogOpen(false);
          setEditing(null);
        }}
        onOk={handleDialogOk}
        loading={loading}
      />
    </Layout>
  );
};
