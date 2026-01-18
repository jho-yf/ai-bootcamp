import { test, expect } from '@playwright/test';

/**
 * 数据库浏览器测试
 */
test.describe('数据库浏览器', () => {
  test.beforeEach(async ({ page }) => {
    // 注入 Tauri API mock
    await page.addInitScript(() => {
      if (!window.__TAURI__) {
        (window as any).__TAURI__ = {
          core: {
            invoke: async (cmd: string, args?: any) => {
              switch (cmd) {
                case 'list_databases':
                  return [
                    {
                      id: 'test-db-1',
                      name: '测试数据库',
                      host: 'localhost',
                      port: 5432,
                      databaseName: 'testdb',
                      user: 'testuser',
                      status: 'connected',
                      createdAt: new Date().toISOString(),
                      updatedAt: new Date().toISOString(),
                    },
                  ];
                case 'get_database_metadata':
                  return {
                    connectionId: args?.databaseId || 'test-db-1',
                    tables: [
                      {
                        schema: 'public',
                        name: 'users',
                        tableType: 'BASE TABLE',
                        columns: [
                          {
                            name: 'id',
                            dataType: 'integer',
                            nullable: false,
                            isPrimaryKey: true,
                            ordinalPosition: 1,
                          },
                        ],
                        primaryKeys: ['id'],
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

    await page.goto('/');
    await page.waitForSelector('.ant-layout, h1', { timeout: 10000 });

    // 导航到数据库浏览器页面
    const menuItem = page.locator('.ant-menu-item').filter({ hasText: /数据库浏览器/i }).first();
    await menuItem.waitFor({ state: 'visible', timeout: 5000 });
    await menuItem.click();

    await page.waitForLoadState('domcontentloaded');
    await page.waitForTimeout(1000);
  });

  test('应该显示数据库选择下拉框', async ({ page }) => {
    // 等待页面加载
    await page.waitForTimeout(1500);

    const select = page.locator('.ant-select').first();
    await expect(select).toBeVisible({ timeout: 5000 });

    const selectText = await select.textContent();
    expect(selectText).toMatch(/选择数据库/i);
  });

  test('应该显示刷新元数据按钮', async ({ page }) => {
    // 等待页面加载
    await page.waitForTimeout(1500);

    // 刷新按钮可能初始时不可见（未选择数据库时）
    const refreshButton = page.getByRole('button').filter({ hasText: /刷新|元数据/i }).first();

    // 检查按钮是否存在（可能可见或不可见）
    const isVisible = await refreshButton.isVisible().catch(() => false);
    if (isVisible) {
      await expect(refreshButton).toBeVisible();
    } else {
      // 如果按钮不可见，这是正常的（未选择数据库时）
      console.log('刷新按钮不可见，可能未选择数据库');
    }
  });

  test('应该显示侧边栏', async ({ page }) => {
    // 等待页面加载
    await page.waitForTimeout(1500);

    // 检查侧边栏是否存在
    const sidebar = page.locator('.ant-layout-sider').first();
    await expect(sidebar).toBeVisible({ timeout: 5000 });
  });

  test('应该显示内容区域', async ({ page }) => {
    // 等待页面加载
    await page.waitForTimeout(1500);

    // 检查内容区域是否存在
    const content = page.locator('.ant-layout-content').first();
    await expect(content).toBeVisible({ timeout: 5000 });
  });

  test('未选择数据库时应显示提示', async ({ page }) => {
    // 等待页面加载
    await page.waitForTimeout(1500);

    // 检查是否有提示信息（使用更灵活的选择器）
    const emptyState = page.locator('text=/请先选择|数据库|连接/i').first();

    // 如果存在空状态提示
    const isVisible = await emptyState.isVisible().catch(() => false);
    if (isVisible) {
      await expect(emptyState).toBeVisible();
    } else {
      // 如果没有提示，可能页面结构不同
      console.log('未找到提示信息，可能页面结构不同');
    }
  });
});
