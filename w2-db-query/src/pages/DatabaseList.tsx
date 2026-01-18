/**
 * DatabaseList 页面
 * 显示所有数据库连接列表
 */
import React, { useState } from "react";
import { Button, Row, Col, Empty } from "antd";
import { PlusOutlined } from "@ant-design/icons";
import { DatabaseCard } from "../components/DatabaseCard";
import { AddDatabaseDialog } from "../components/AddDatabaseDialog";
import { useDatabases } from "../hooks/useDatabases";
import type { DatabaseConnection, AddDatabaseRequest } from "../services/types";

export const DatabaseList: React.FC = () => {
  const { databases, loading, addDatabase, updateDatabase, deleteDatabase } =
    useDatabases();
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editing, setEditing] = useState<DatabaseConnection | null>(null);

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
  };

  const handleDialogOk = async (values: AddDatabaseRequest) => {
    if (editing) {
      await updateDatabase({
        id: editing.id,
        ...values,
      });
    } else {
      await addDatabase(values);
    }
    setDialogOpen(false);
    setEditing(null);
  };

  return (
    <div style={{ padding: 24 }}>
      <div
        style={{
          marginBottom: 16,
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <h1>数据库连接</h1>
        <Button type="primary" icon={<PlusOutlined />} onClick={handleAdd}>
          添加数据库连接
        </Button>
      </div>

      {databases.length === 0 ? (
        <Empty description="还没有添加任何数据库连接" />
      ) : (
        <Row gutter={[16, 16]}>
          {databases.map((db) => (
            <Col key={db.id} xs={24} sm={12} md={8} lg={6}>
              <DatabaseCard
                database={db}
                onEdit={handleEdit}
                onDelete={handleDelete}
              />
            </Col>
          ))}
        </Row>
      )}

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
    </div>
  );
};
