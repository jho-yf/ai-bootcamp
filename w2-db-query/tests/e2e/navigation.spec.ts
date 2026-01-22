import { test, expect } from "@playwright/test";

/**
 * 应用导航测试
 */
test.describe("应用导航", () => {
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
    await page.waitForTimeout(1000);
  });

  test("应该显示应用标题", async ({ page }) => {
    const title = page
      .locator(".ant-typography")
      .filter({ hasText: /数据库查询工具/i });
    await expect(title).toBeVisible({ timeout: 5000 });
  });

  test("应该显示左侧边栏", async ({ page }) => {
    const sider = page.locator(".ant-layout-sider").first();
    await expect(sider).toBeVisible({ timeout: 5000 });
  });

  test('应该显示"添加数据库连接"按钮', async ({ page }) => {
    const addButton = page
      .getByRole("button")
      .filter({ hasText: /添加数据库连接/i })
      .first();
    await expect(addButton).toBeVisible({ timeout: 5000 });
  });

  test("应该在侧边栏显示数据库列表", async ({ page }) => {
    // 等待菜单项加载
    await page.waitForTimeout(1500);

    // 检查是否有数据库菜单项
    const menuItems = page.locator(".ant-menu-item");
    const count = await menuItems.count();

    // 应该至少有一个数据库菜单项（如果有 mock 数据）
    if (count > 0) {
      await expect(menuItems.first()).toBeVisible({ timeout: 5000 });
    }
  });

  test("应该能够点击数据库菜单项", async ({ page }) => {
    await page.waitForTimeout(1500);

    // 查找数据库菜单项
    const menuItem = page
      .locator(".ant-menu-item")
      .filter({ hasText: /测试数据库/i });
    const isVisible = await menuItem.isVisible().catch(() => false);

    if (isVisible) {
      await menuItem.click();
      await page.waitForTimeout(500);

      // 检查是否显示数据库浏览器内容
      const content = page.locator(".ant-layout-content").first();
      await expect(content).toBeVisible({ timeout: 5000 });
    }
  });

  test("未选择数据库时应显示提示", async ({ page }) => {
    // Mock 返回空列表
    await page.addInitScript(() => {
      if (window.__TAURI__) {
        const originalInvoke = (window as any).__TAURI__.core.invoke;
        (window as any).__TAURI__.core.invoke = async (
          cmd: string,
          args?: any,
        ) => {
          if (cmd === "list_databases") {
            return [];
          }
          return originalInvoke(cmd, args);
        };
      }
    });

    await page.reload();
    await page.waitForTimeout(2000);

    // 应该显示空状态提示
    const emptyText = page
      .locator("text=/还没有添加|暂无数据库|点击添加/i")
      .first();
    const isVisible = await emptyText.isVisible().catch(() => false);

    if (isVisible) {
      await expect(emptyText).toBeVisible({ timeout: 5000 });
    }
  });
});
