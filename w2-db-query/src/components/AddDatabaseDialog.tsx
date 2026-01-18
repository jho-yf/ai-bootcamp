/**
 * AddDatabaseDialog 组件
 * 添加/编辑数据库连接的表单对话框
 */
import React, { useEffect, useState } from 'react';
import { Modal, Form, Input, InputNumber, Button, Space, message } from 'antd';
import { ThunderboltOutlined } from '@ant-design/icons';
import type { DatabaseConnection, AddDatabaseRequest } from '../services/types';
import * as api from '../services/api';

interface AddDatabaseDialogProps {
  open: boolean;
  editing?: DatabaseConnection | null;
  onCancel: () => void;
  onOk: (values: AddDatabaseRequest) => Promise<void>;
  loading?: boolean;
}

export const AddDatabaseDialog: React.FC<AddDatabaseDialogProps> = ({
  open,
  editing,
  onCancel,
  onOk,
  loading = false,
}) => {
  const [form] = Form.useForm();
  const [testing, setTesting] = useState(false);

  useEffect(() => {
    if (editing) {
      form.setFieldsValue({
        name: editing.name,
        host: editing.host,
        port: editing.port,
        databaseName: editing.databaseName,
        user: editing.user,
        password: '', // 不显示密码
      });
    } else {
      form.resetFields();
    }
    // 重置测试状态
    setTesting(false);
  }, [editing, form, open]);

  const handleTestConnection = async () => {
    try {
      // 验证必填字段（除了 name）
      const values = await form.validateFields(['host', 'port', 'databaseName', 'user', 'password']);

      setTesting(true);

      const result = await api.testConnection({
        host: values.host,
        port: values.port,
        databaseName: values.databaseName,
        user: values.user,
        password: values.password,
      });

      if (result) {
        message.success('连接测试成功！');
      } else {
        message.error('连接测试失败');
      }
    } catch (error) {
      // 错误已在 api.ts 中处理并显示 message
    } finally {
      setTesting(false);
    }
  };

  const handleOk = async () => {
    try {
      const values = await form.validateFields();
      await onOk(values);
      form.resetFields();
    } catch (error) {
      // 表单验证失败
    }
  };

  const handleCancel = () => {
    form.resetFields();
    setTesting(false);
    onCancel();
  };

  return (
    <Modal
      title={editing ? '编辑数据库连接' : '添加数据库连接'}
      open={open}
      onOk={handleOk}
      onCancel={handleCancel}
      confirmLoading={loading}
      width={600}
      okText="确定"
      cancelText="取消"
      footer={
        <Space>
          <Button onClick={handleCancel}>取消</Button>
          <Button
            icon={<ThunderboltOutlined />}
            onClick={handleTestConnection}
            loading={testing}
            disabled={loading}
          >
            测试连接
          </Button>
          <Button type="primary" onClick={handleOk} loading={loading}>
            确定
          </Button>
        </Space>
      }
    >
      <Form
        form={form}
        layout="vertical"
        initialValues={{
          port: 5432,
        }}
      >
        <Form.Item
          name="name"
          label="连接名称"
          rules={[{ required: true, message: '请输入连接名称' }]}
        >
          <Input placeholder="例如：生产数据库" />
        </Form.Item>

        <Form.Item
          name="host"
          label="主机地址"
          rules={[{ required: true, message: '请输入主机地址' }]}
        >
          <Input placeholder="localhost 或 IP 地址" />
        </Form.Item>

        <Form.Item
          name="port"
          label="端口"
          rules={[
            { required: true, message: '请输入端口' },
            { type: 'number', min: 1, max: 65535, message: '端口必须在 1-65535 之间' },
          ]}
        >
          <InputNumber style={{ width: '100%' }} placeholder="5432" />
        </Form.Item>

        <Form.Item
          name="databaseName"
          label="数据库名"
          rules={[{ required: true, message: '请输入数据库名' }]}
        >
          <Input placeholder="数据库名称" />
        </Form.Item>

        <Form.Item
          name="user"
          label="用户名"
          rules={[{ required: true, message: '请输入用户名' }]}
        >
          <Input placeholder="数据库用户名" />
        </Form.Item>

        <Form.Item
          name="password"
          label="密码"
          rules={[{ required: true, message: '请输入密码' }]}
        >
          <Input.Password placeholder="数据库密码" />
        </Form.Item>
      </Form>
    </Modal>
  );
};
