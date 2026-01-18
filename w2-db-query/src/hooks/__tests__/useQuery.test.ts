import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { useQuery } from "../useQuery";
import * as api from "../../services/api";

// Mock API
vi.mock("../../services/api", () => ({
  runSqlQuery: vi.fn(),
  generateSqlFromNL: vi.fn(),
  runNLQuery: vi.fn(),
  cancelQuery: vi.fn(),
}));

// Mock antd message
vi.mock("antd", async () => {
  const actual = await vi.importActual("antd");
  return {
    ...actual,
    message: {
      success: vi.fn(),
      error: vi.fn(),
      warning: vi.fn(),
      info: vi.fn(),
    },
  };
});

describe("useQuery", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("应该初始化时没有结果", () => {
    const { result } = renderHook(() => useQuery());
    expect(result.current.result).toBeNull();
    expect(result.current.loading).toBe(false);
    expect(result.current.error).toBeNull();
    expect(result.current.generatedSql).toBeNull();
  });

  it("应该能够执行 SQL 查询", async () => {
    const mockResult = {
      columns: ["id", "name"],
      rows: [{ id: 1, name: "Alice" }],
      total: 1,
      execTimeMs: 50,
      sql: "SELECT * FROM users",
      truncated: false,
    };

    vi.mocked(api.runSqlQuery).mockResolvedValue(mockResult);

    const { result } = renderHook(() => useQuery());

    await act(async () => {
      await result.current.executeQuery({
        databaseId: "test-db",
        sql: "SELECT * FROM users",
      });
    });

    await waitFor(() => {
      expect(result.current.result).toEqual(mockResult);
      expect(result.current.loading).toBe(false);
    });
  });

  it("应该在查询失败时设置错误", async () => {
    const error = new Error("查询失败");
    vi.mocked(api.runSqlQuery).mockRejectedValue(error);

    const { result } = renderHook(() => useQuery());

    await act(async () => {
      try {
        await result.current.executeQuery({
          databaseId: "test-db",
          sql: "SELECT * FROM users",
        });
      } catch (e) {
        // 预期的错误
      }
    });

    await waitFor(() => {
      expect(result.current.error).toBe("查询失败");
      expect(result.current.loading).toBe(false);
    });
  });

  it("应该能够生成 SQL", async () => {
    const mockSql = "SELECT * FROM users LIMIT 100";
    vi.mocked(api.generateSqlFromNL).mockResolvedValue(mockSql);

    const { result } = renderHook(() => useQuery());

    await act(async () => {
      await result.current.generateSqlFromNL({
        databaseId: "test-db",
        prompt: "查询所有用户",
      });
    });

    await waitFor(() => {
      expect(result.current.generatedSql).toBe(mockSql);
      expect(result.current.loading).toBe(false);
    });
  });

  it("应该能够清除结果", () => {
    const { result } = renderHook(() => useQuery());

    act(() => {
      result.current.clearResult();
    });

    expect(result.current.result).toBeNull();
    expect(result.current.error).toBeNull();
    expect(result.current.generatedSql).toBeNull();
  });
});
