/**
 * MetadataTree 组件
 * 使用 Ant Design Tree 显示数据库表和视图的树形结构
 */
import React, { useMemo } from 'react';
import { Tree, Spin, Empty } from 'antd';
import { TableOutlined, EyeOutlined } from '@ant-design/icons';
import type { DatabaseMetadata, TableInfo, ViewInfo } from '../services/types';

interface MetadataTreeProps {
  metadata: DatabaseMetadata | null;
  loading?: boolean;
  onTableSelect?: (table: TableInfo) => void;
  onViewSelect?: (view: ViewInfo) => void;
  onTableDoubleClick?: (table: TableInfo) => void;
  onViewDoubleClick?: (view: ViewInfo) => void;
}

export const MetadataTree: React.FC<MetadataTreeProps> = ({
  metadata,
  loading = false,
  onTableSelect,
  onViewSelect,
  onTableDoubleClick,
  onViewDoubleClick,
}) => {
  const { treeData, defaultExpandedKeys } = useMemo(() => {
    if (!metadata) return { treeData: [], defaultExpandedKeys: [] };

    const tables: any[] = metadata.tables.map((table) => ({
      title: (
        <span style={{ display: 'flex', alignItems: 'center', minWidth: 0 }}>
          <TableOutlined style={{ marginRight: 8, flexShrink: 0 }} />
          <span style={{
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
            flex: 1,
            minWidth: 0
          }}>
            {table.name}
          </span>
        </span>
      ),
      key: `table-${table.schema}-${table.name}`,
      isLeaf: false,
      children: table.columns.map((col) => ({
        title: (
          <span style={{ display: 'flex', alignItems: 'center', minWidth: 0 }}>
            <span style={{
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
              flex: 1,
              minWidth: 0
            }}>
              {col.name} ({col.dataType})
            </span>
            {col.isPrimaryKey && <span style={{ color: '#1890ff', marginLeft: 8, flexShrink: 0 }}>PK</span>}
            {!col.nullable && <span style={{ color: '#ff4d4f', marginLeft: 8, flexShrink: 0 }}>NOT NULL</span>}
          </span>
        ),
        key: `col-${table.schema}-${table.name}-${col.name}`,
        isLeaf: true,
      })),
      table,
    }));

    const views: any[] = metadata.views.map((view) => ({
      title: (
        <span style={{ display: 'flex', alignItems: 'center', minWidth: 0 }}>
          <EyeOutlined style={{ marginRight: 8, flexShrink: 0 }} />
          <span style={{
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
            flex: 1,
            minWidth: 0
          }}>
            {view.name}
          </span>
        </span>
      ),
      key: `view-${view.schema}-${view.name}`,
      isLeaf: false,
      children: view.columns.map((col) => ({
        title: (
          <span style={{
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
            display: 'block'
          }}>
            {col.name} ({col.dataType})
          </span>
        ),
        key: `col-${view.schema}-${view.name}-${col.name}`,
        isLeaf: true,
      })),
      view,
    }));

    const data = [
      {
        title: '表',
        key: 'tables',
        children: tables,
      },
      {
        title: '视图',
        key: 'views',
        children: views,
      },
    ];

    // 默认展开到表层级：只展开 "表" 和 "视图" 节点，不展开具体的表和视图
    const expandedKeys = ['tables', 'views'];

    return { treeData: data, defaultExpandedKeys: expandedKeys };
  }, [metadata]);

  const handleSelect = (_selectedKeys: React.Key[], info: any) => {
    const node = info.node;
    if (node.table) {
      onTableSelect?.(node.table);
    } else if (node.view) {
      onViewSelect?.(node.view);
    }
  };

  const handleDoubleClick = (_event: React.MouseEvent, node: any) => {
    if (node.table) {
      onTableDoubleClick?.(node.table);
    } else if (node.view) {
      onViewDoubleClick?.(node.view);
    }
  };

  if (loading) {
    return (
      <div style={{ textAlign: 'center', padding: 50 }}>
        <Spin size="large" />
      </div>
    );
  }

  if (!metadata) {
    return <Empty description="请先选择数据库连接" />;
  }

  return (
    <Tree
      showLine
      defaultExpandedKeys={defaultExpandedKeys}
      treeData={treeData}
      onSelect={handleSelect}
      onDoubleClick={handleDoubleClick}
      style={{ padding: 16, background: '#fff' }}
    />
  );
};
