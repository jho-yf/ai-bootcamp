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
              if (cmd === 'list_databases') return [];
              return null;
            },
          },
        };
      }
    });

    await page.goto('/');
    await page.waitForSelector('.ant-layout, h1', { timeout: 10000 });
    await page.waitForLoadState('domcontentloaded');
    await page.waitForTimeout(1000);
  });

  test('应该能够导航到数据库连接页面', async ({ page }) => {
    // 等待菜单加载
    await page.waitForSelector('.ant-menu', { timeout: 5000 });

    // 点击数据库连接菜单项（使用更灵活的选择器）
    const menuItem = page.locator('.ant-menu-item').filter({ hasText: /数据库连接/i }).first();
    await menuItem.waitFor({ state: 'visible', timeout: 5000 });
    await menuItem.click();

    // 等待页面切换
    await page.waitForTimeout(500);

    // 检查页面标题
    const heading = page.locator('h1').first();
    await expect(heading).toBeVisible({ timeout: 5000 });
    const headingText = await heading.textContent();
    expect(headingText).toMatch(/数据库|连接/i);
  });

  test('应该能够导航到数据库浏览器页面', async ({ page }) => {
    // 等待菜单加载
    await page.waitForSelector('.ant-menu', { timeout: 5000 });

    // 点击数据库浏览器菜单项
    const menuItem = page.locator('.ant-menu-item').filter({ hasText: /数据库浏览器/i }).first();
    await menuItem.waitFor({ state: 'visible', timeout: 5000 });
    await menuItem.click();

    // 等待页面切换
    await page.waitForTimeout(500);

    // 检查页面内容（使用更灵活的选择器）
    const selectText = page.locator('text=/选择数据库/i').first();
    await expect(selectText).toBeVisible({ timeout: 5000 });
  });

  test('应该能够导航到 SQL 查询页面', async ({ page }) => {
    // 等待菜单加载
    await page.waitForSelector('.ant-menu', { timeout: 5000 });

    // 点击 SQL 查询菜单项
    const menuItem = page.locator('.ant-menu-item').filter({ hasText: /SQL|查询/i }).first();
    await menuItem.waitFor({ state: 'visible', timeout: 5000 });
    await menuItem.click();

    // 等待页面切换
    await page.waitForTimeout(500);

    // 检查页面内容
    const selectText = page.locator('text=/选择数据库/i').first();
    await expect(selectText).toBeVisible({ timeout: 5000 });
  });

  test('菜单项应该高亮当前页面', async ({ page }) => {
    // 等待菜单加载
    await page.waitForSelector('.ant-menu', { timeout: 5000 });

    // 默认应该在数据库连接页面
    const dbConnectionItem = page.locator('.ant-menu-item').filter({ hasText: /数据库连接/i }).first();
    await dbConnectionItem.waitFor({ state: 'visible', timeout: 5000 });

    // 检查是否被选中
    const dbItemClass = await dbConnectionItem.getAttribute('class');
    expect(dbItemClass).toContain('ant-menu-item-selected');

    // 切换到其他页面
    const sqlQueryItem = page.locator('.ant-menu-item').filter({ hasText: /SQL|查询/i }).first();
    await sqlQueryItem.waitFor({ state: 'visible', timeout: 5000 });
    await sqlQueryItem.click();

    // 等待切换完成
    await page.waitForTimeout(500);

    // SQL 查询应该被选中
    const sqlItemClass = await sqlQueryItem.getAttribute('class');
    expect(sqlItemClass).toContain('ant-menu-item-selected');
  });
});
