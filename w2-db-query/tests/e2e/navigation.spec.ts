import { test, expect } from '@playwright/test';

/**
 * 导航测试
 */
test.describe('应用导航', () => {
  test.beforeEach(async ({ page }) => {
    // 注入 Tauri API mock
    await page.addInitScript(() => {
      if (!window.__TAURI__) {
        (window as any).__TAURI__ = {
          core: {
            invoke: async (cmd: string) => {
              if (cmd === 'list_databases') {
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
              }
              return null;
            },
          },
        };
      }
    });

    await page.goto('/');
    await page.waitForSelector('.ant-layout', { timeout: 10000 });
    await page.waitForLoadState('domcontentloaded');
    await page.waitForTimeout(2000); // 等待数据库列表加载并自动选中
  });

  test('应该显示数据库列表菜单', async ({ page }) => {
    // 等待菜单加载和数据库列表加载
    await page.waitForSelector('.ant-menu', { timeout: 10000 });
    await page.waitForTimeout(2000);

    // 检查是否有数据库菜单项（菜单项可能包含图标和文本）
    const menuItem = page.locator('.ant-menu-item').filter({ hasText: /测试数据库/i }).first();
    await expect(menuItem).toBeVisible({ timeout: 10000 });
  });

  test('应该能够选择数据库', async ({ page }) => {
    // 等待菜单加载和数据库列表加载
    await page.waitForSelector('.ant-menu', { timeout: 10000 });
    await page.waitForTimeout(2000);

    // 点击数据库菜单项
    const menuItem = page.locator('.ant-menu-item').filter({ hasText: /测试数据库/i }).first();
    await menuItem.waitFor({ state: 'visible', timeout: 10000 });
    await menuItem.click();

    // 等待页面切换
    await page.waitForTimeout(2000);

    // 检查数据库浏览器是否显示（数据库名称在顶部工具栏）
    const dbName = page.locator('span').filter({ hasText: /测试数据库/i }).first();
    await expect(dbName).toBeVisible({ timeout: 10000 });
  });

  test('菜单项应该高亮当前选中的数据库', async ({ page }) => {
    // 等待菜单加载和数据库列表加载
    await page.waitForSelector('.ant-menu', { timeout: 10000 });
    await page.waitForTimeout(2000);

    // 默认应该选中第一个数据库
    const menuItem = page.locator('.ant-menu-item').filter({ hasText: /测试数据库/i }).first();
    await menuItem.waitFor({ state: 'visible', timeout: 10000 });

    // 等待选中状态应用
    await page.waitForTimeout(1000);

    // 检查是否被选中
    const itemClass = await menuItem.getAttribute('class');
    expect(itemClass).toContain('ant-menu-item-selected');
  });
});
