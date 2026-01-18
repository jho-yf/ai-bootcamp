import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { DatabaseCard } from "../DatabaseCard";
import type { DatabaseConnection } from "../../services/types";

const mockDatabase: DatabaseConnection = {
  id: "test-1",
  name: "测试数据库",
  host: "localhost",
  port: 5432,
  databaseName: "testdb",
  user: "testuser",
  password: "testpass",
  status: "connected",
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
};

describe("DatabaseCard", () => {
  it("应该显示数据库名称", () => {
    render(<DatabaseCard database={mockDatabase} />);
    expect(screen.getByText("测试数据库")).toBeInTheDocument();
  });

  it("应该显示连接信息", () => {
    render(<DatabaseCard database={mockDatabase} />);
    expect(screen.getByText(/localhost:5432/i)).toBeInTheDocument();
    expect(screen.getByText(/testdb/i)).toBeInTheDocument();
    expect(screen.getByText(/testuser/i)).toBeInTheDocument();
  });

  it("应该显示连接状态", () => {
    render(<DatabaseCard database={mockDatabase} />);
    expect(screen.getByText(/已连接/i)).toBeInTheDocument();
  });

  it("应该在点击时调用 onClick", async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();
    render(<DatabaseCard database={mockDatabase} onClick={onClick} />);

    const card = screen.getByText("测试数据库").closest(".ant-card");
    if (card) {
      await user.click(card);
      expect(onClick).toHaveBeenCalledWith(mockDatabase);
    }
  });

  it("应该在点击编辑按钮时调用 onEdit", async () => {
    const user = userEvent.setup();
    const onEdit = vi.fn();
    render(<DatabaseCard database={mockDatabase} onEdit={onEdit} />);

    const editButton = screen.getByText(/编辑/i);
    await user.click(editButton);
    expect(onEdit).toHaveBeenCalledWith(mockDatabase);
  });

  it("应该根据状态显示不同的状态文本", () => {
    const { rerender } = render(
      <DatabaseCard database={{ ...mockDatabase, status: "failed" }} />
    );
    expect(screen.getByText(/连接失败/i)).toBeInTheDocument();

    rerender(
      <DatabaseCard database={{ ...mockDatabase, status: "connecting" }} />
    );
    expect(screen.getByText(/连接中/i)).toBeInTheDocument();
  });
});
