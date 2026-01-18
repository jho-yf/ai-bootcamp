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
    await page.waitForSelector('.ant-layout', { timeout: 10000 });
    await page.waitForLoadState('domcontentloaded');
    await page.waitForTimeout(2000); // 等待数据库列表加载并自动选中第一个
  });

  test('应该显示数据库名称和刷新按钮', async ({ page }) => {
    // 等待页面加载和数据库自动选中
    await page.waitForTimeout(3000);

    // 检查数据库名称显示（在顶部工具栏）
    const dbName = page.locator('span').filter({ hasText: /测试数据库/i }).first();
    await expect(dbName).toBeVisible({ timeout: 10000 });

    // 检查刷新按钮
    const refreshButton = page.getByRole('button').filter({ hasText: /刷新元数据/i }).first();
    await expect(refreshButton).toBeVisible({ timeout: 5000 });
  });

  test('应该显示刷新元数据按钮', async ({ page }) => {
    // 等待页面加载和数据库自动选中
    await page.waitForTimeout(3000);

    // 刷新按钮应该可见（因为默认选中了第一个数据库）
    const refreshButton = page.getByRole('button').filter({ hasText: /刷新元数据/i }).first();
    await expect(refreshButton).toBeVisible({ timeout: 10000 });
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
    // 先清空数据库列表（通过mock返回空数组）
    await page.addInitScript(() => {
      if (window.__TAURI__) {
        const originalInvoke = window.__TAURI__.core.invoke;
        window.__TAURI__.core.invoke = async (cmd: string) => {
          if (cmd === 'list_databases') {
            return [];
          }
          return originalInvoke(cmd);
        };
      }
    });

    // 重新加载页面
    await page.reload();
    await page.waitForTimeout(2000);

    // 检查空状态提示
    const emptyState = page.locator('text=/暂无数据库连接/i').first();
    await expect(emptyState).toBeVisible({ timeout: 5000 });
  });
});
