/**
 * Playwright 测试设置
 */
import { test as base } from "@playwright/test";
import { mockTauriInvoke } from "./mock-tauri";

// 扩展测试上下文
export const test = base.extend({
  // 可以添加自定义 fixtures
  page: async ({ page }, use) => {
    // 在页面加载前注入 Tauri mock
    await page.addInitScript(() => {
      // 注入 mock 脚本
      if (typeof window !== "undefined" && !window.__TAURI__) {
        (window as any).__TAURI__ = {
          core: {
            invoke: async (cmd: string, args?: any) => {
              console.log("[Mock Tauri] invoke:", cmd, args);

              // Mock 响应
              switch (cmd) {
                case "list_databases":
                  return [];
                case "test_connection":
                  return true;
                case "add_database":
                  return {
                    id: "test-db-new",
                    name: args?.name || "新数据库",
                    host: args?.host || "localhost",
                    port: args?.port || 5432,
                    databaseName: args?.databaseName || "testdb",
                    user: args?.user || "testuser",
                    status: "connected",
                    createdAt: new Date().toISOString(),
                    updatedAt: new Date().toISOString(),
                  };
                default:
                  return null;
              }
            },
          },
        };
      }
    });

    await use(page);
  },
});

export { expect } from "@playwright/test";
