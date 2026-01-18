/**
 * LoadingIndicator 组件
 * 查询执行时显示的加载指示器
 */
import React from 'react';
import { Spin, Button } from 'antd';
import { StopOutlined } from '@ant-design/icons';

interface LoadingIndicatorProps {
  loading: boolean;
  onCancel?: () => void;
  message?: string;
}

export const LoadingIndicator: React.FC<LoadingIndicatorProps> = ({
  loading,
  onCancel,
  message = '查询执行中...',
}) => {
  if (!loading) return null;

  return (
    <div
      style={{
        position: 'absolute',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        background: 'rgba(255, 255, 255, 0.8)',
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'center',
        alignItems: 'center',
        zIndex: 1000,
      }}
    >
      <Spin size="large" />
      <div style={{ marginTop: 16, marginBottom: 8 }}>{message}</div>
      {onCancel && (
        <Button icon={<StopOutlined />} onClick={onCancel} size="small">
          取消查询
        </Button>
      )}
    </div>
  );
};
