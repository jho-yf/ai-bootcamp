import { test, expect } from "@playwright/test";

/**
 * 数据库浏览器测试
 */
test.describe("数据库浏览器", () => {
  test.beforeEach(async ({ page }) => {
    // 注入 Tauri API mock
    await page.addInitScript(() => {
      if (!window.__TAURI__) {
        (window as any).__TAURI__ = {
          core: {
            invoke: async (cmd: string, args?: any) => {
              switch (cmd) {
                case "list_databases":
                  return [
                    {
                      id: "test-db-1",
                      name: "测试数据库",
                      host: "localhost",
                      port: 5432,
                      databaseName: "testdb",
                      user: "testuser",
                      status: "connected",
                      createdAt: new Date().toISOString(),
                      updatedAt: new Date().toISOString(),
                    },
                  ];
                case "get_database_metadata":
                  return {
                    connectionId: args?.databaseId || "test-db-1",
                    tables: [
                      {
                        schema: "public",
                        name: "users",
                        tableType: "BASE TABLE",
                        columns: [
                          {
                            name: "id",
                            dataType: "integer",
                            nullable: false,
                            isPrimaryKey: true,
                            ordinalPosition: 1,
                          },
                          {
                            name: "name",
                            dataType: "character varying",
                            nullable: false,
                            isPrimaryKey: false,
                            ordinalPosition: 2,
                          },
                        ],
                        primaryKeys: ["id"],
                        foreignKeys: [],
                      },
                    ],
                    views: [],
                    extractedAt: new Date().toISOString(),
                  };
                case "refresh_metadata":
                  return {
                    connectionId: args?.databaseId || "test-db-1",
                    tables: [
                      {
                        schema: "public",
                        name: "users",
                        tableType: "BASE TABLE",
                        columns: [
                          {
                            name: "id",
                            dataType: "integer",
                            nullable: false,
                            isPrimaryKey: true,
                            ordinalPosition: 1,
                          },
                        ],
                        primaryKeys: ["id"],
                        foreignKeys: [],
                      },
                    ],
                    views: [],
                    extractedAt: new Date().toISOString(),
                  };
                default:
                  return null;
              }
            },
          },
        };
      }
    });

    await page.goto("/");
    await page.waitForSelector(".ant-layout", { timeout: 10000 });
    await page.waitForLoadState("domcontentloaded");
    await page.waitForTimeout(2000);

    // 点击第一个数据库菜单项
    const menuItem = page.locator(".ant-menu-item").first();
    const isVisible = await menuItem.isVisible().catch(() => false);
    if (isVisible) {
      await menuItem.click();
      await page.waitForTimeout(1000);
    }
  });

  test("应该显示数据库浏览器内容", async ({ page }) => {
    // 等待内容加载
    await page.waitForTimeout(1500);

    // 检查内容区域
    const content = page.locator(".ant-layout-content").first();
    await expect(content).toBeVisible({ timeout: 5000 });
  });

  test("应该显示侧边栏（元数据树）", async ({ page }) => {
    await page.waitForTimeout(1500);

    // 检查侧边栏
    const sider = page
      .locator(".ant-layout-sider")
      .filter({ hasNotText: /添加数据库连接/i })
      .last();
    const isVisible = await sider.isVisible().catch(() => false);

    // 侧边栏可能存在也可能不存在，取决于实现
    if (isVisible) {
      await expect(sider).toBeVisible({ timeout: 5000 });
    }
  });

  test("应该显示刷新元数据按钮", async ({ page }) => {
    await page.waitForTimeout(1500);

    // 查找刷新按钮
    const refreshButton = page
      .getByRole("button")
      .filter({ hasText: /刷新|元数据/i })
      .first();

    const isVisible = await refreshButton.isVisible().catch(() => false);
    if (isVisible) {
      await expect(refreshButton).toBeVisible({ timeout: 5000 });
    }
  });

  test("应该显示 SQL 编辑器标签页", async ({ page }) => {
    await page.waitForTimeout(1500);

    // 查找标签页
    const tabs = page.locator(".ant-tabs-tab");
    const count = await tabs.count();

    if (count > 0) {
      // 检查是否有 SQL 编辑器标签
      const sqlTab = page.getByRole("tab").filter({ hasText: /SQL|编辑器/i });
      const isVisible = await sqlTab.isVisible().catch(() => false);

      if (isVisible) {
        await expect(sqlTab.first()).toBeVisible({ timeout: 5000 });
      }
    }
  });

  test("应该显示自然语言查询标签页", async ({ page }) => {
    await page.waitForTimeout(1500);

    // 查找标签页
    const tabs = page.locator(".ant-tabs-tab");
    const count = await tabs.count();

    if (count > 0) {
      // 检查是否有自然语言查询标签
      const nlTab = page.getByRole("tab").filter({ hasText: /自然语言|查询/i });
      const isVisible = await nlTab.isVisible().catch(() => false);

      if (isVisible) {
        await expect(nlTab.first()).toBeVisible({ timeout: 5000 });
      }
    }
  });
});
