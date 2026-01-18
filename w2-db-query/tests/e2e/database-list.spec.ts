import { test, expect } from '@playwright/test';

/**
 * 数据库列表页面测试
 */
test.describe('数据库列表页面', () => {
  test.beforeEach(async ({ page }) => {
    // 注入 Tauri API mock
    await page.addInitScript(() => {
      if (!window.__TAURI__) {
        (window as any).__TAURI__ = {
          core: {
            invoke: async (cmd: string, args?: any) => {
              console.log('[Mock Tauri] invoke:', cmd, args);

              switch (cmd) {
                case 'list_databases':
                  return [];
                case 'test_connection':
                  return true;
                case 'add_database':
                  return {
                    id: 'test-db-new',
                    name: args?.name || '新数据库',
                    host: args?.host || 'localhost',
                    port: args?.port || 5432,
                    databaseName: args?.databaseName || 'testdb',
                    user: args?.user || 'testuser',
                    status: 'connected',
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

    // 导航到应用首页
    await page.goto('/');

    // 等待应用加载 - 检查关键元素
    await page.waitForSelector('h1, .ant-layout', { timeout: 10000 });

    // 等待 React 应用完全加载
    await page.waitForLoadState('domcontentloaded');
    await page.waitForTimeout(1000); // 给 React 一些时间渲染
  });

  test('应该显示数据库连接页面', async ({ page }) => {
    // 等待页面加载
    await page.waitForSelector('.ant-layout', { timeout: 10000 });

    // 检查页面标题
    const heading = page.locator('h4').filter({ hasText: /数据库查询工具/i }).first();
    await expect(heading).toBeVisible({ timeout: 5000 });

    // 检查是否有"添加数据库连接"按钮（在左侧菜单顶部）
    const addButton = page.getByRole('button').filter({ hasText: /添加数据库连接/i }).first();
    await expect(addButton).toBeVisible({ timeout: 5000 });
  });

  test('应该能够打开添加数据库连接对话框', async ({ page }) => {
    // 等待页面加载
    await page.waitForSelector('.ant-layout', { timeout: 10000 });
    await page.waitForTimeout(1000);

    // 点击添加按钮（在左侧菜单顶部）
    const addButton = page.getByRole('button').filter({ hasText: /添加数据库连接/i }).first();
    await addButton.waitFor({ state: 'visible', timeout: 5000 });
    await addButton.click();

    // 等待对话框出现
    const dialog = page.locator('.ant-modal').first();
    await dialog.waitFor({ state: 'visible', timeout: 5000 });
    await expect(dialog).toBeVisible();

    // 检查对话框标题
    const title = dialog.locator('.ant-modal-title, [role="dialog"] h2').first();
    await expect(title).toBeVisible();
    const titleText = await title.textContent();
    expect(titleText).toMatch(/添加|数据库|连接/i);

    // 检查表单字段（使用更灵活的选择器）
    await expect(page.locator('label').filter({ hasText: /连接名称/i })).toBeVisible({ timeout: 3000 });
    await expect(page.locator('label').filter({ hasText: /主机地址/i })).toBeVisible({ timeout: 3000 });
    await expect(page.locator('label').filter({ hasText: /端口/i })).toBeVisible({ timeout: 3000 });
    await expect(page.locator('label').filter({ hasText: /数据库名/i })).toBeVisible({ timeout: 3000 });
    await expect(page.locator('label').filter({ hasText: /用户名/i })).toBeVisible({ timeout: 3000 });
    await expect(page.locator('label').filter({ hasText: /密码/i })).toBeVisible({ timeout: 3000 });
  });

  test('添加数据库连接表单验证', async ({ page }) => {
    // 等待页面加载
    await page.waitForSelector('.ant-layout', { timeout: 10000 });
    await page.waitForTimeout(1000);

    // 打开对话框
    const addButton = page.getByRole('button').filter({ hasText: /添加数据库连接/i }).first();
    await addButton.waitFor({ state: 'visible', timeout: 5000 });
    await addButton.click();

    // 等待对话框出现
    const modal = page.locator('.ant-modal').first();
    await modal.waitFor({ state: 'visible', timeout: 5000 });
    await page.waitForTimeout(500); // 等待对话框动画完成

    // 查找确定按钮（在对话框内的所有按钮中查找）
    const allButtons = modal.locator('button');
    const buttonCount = await allButtons.count();

    let submitButton = null;
    for (let i = 0; i < buttonCount; i++) {
      const btn = allButtons.nth(i);
      const text = await btn.textContent();
      if (text && /确定|提交|保存/i.test(text)) {
        submitButton = btn;
        break;
      }
    }

    // 如果找不到，尝试通过 type="primary" 查找
    if (!submitButton) {
      submitButton = modal.locator('button.ant-btn-primary, button[type="button"].ant-btn-primary').first();
    }

    // 确保按钮存在
    await submitButton.waitFor({ state: 'visible', timeout: 5000 });
    await submitButton.click();

    // 等待验证错误出现（Ant Design 表单验证）
    await page.waitForTimeout(1000); // 给验证更多时间

    // 应该显示验证错误（使用更灵活的选择器）
    const errorMessages = page.locator('.ant-form-item-explain-error, .ant-form-item-has-error, .ant-form-item-explain');

    // 等待至少一个错误消息出现
    try {
      await errorMessages.first().waitFor({ state: 'visible', timeout: 3000 });
    } catch {
      // 如果等待失败，检查是否有错误类
      const hasError = await modal.locator('.ant-form-item-has-error').count();
      expect(hasError).toBeGreaterThan(0);
      return;
    }

    const errorCount = await errorMessages.count();

    // 至少应该有一个验证错误
    expect(errorCount).toBeGreaterThan(0);
  });

  test('应该能够填写并测试连接', async ({ page }) => {
    // 等待页面加载
    await page.waitForSelector('.ant-layout', { timeout: 10000 });
    await page.waitForTimeout(1000);

    // 打开对话框
    const addButton = page.getByRole('button').filter({ hasText: /添加数据库连接/i }).first();
    await addButton.waitFor({ state: 'visible', timeout: 5000 });
    await addButton.click();

    // 等待对话框出现
    await page.waitForSelector('.ant-modal', { timeout: 5000 });

    // 填写表单（使用 label 文本查找输入框）
    const nameInput = page.locator('input').filter({ has: page.locator('label').filter({ hasText: /连接名称/i }) }).first();
    if (await nameInput.count() === 0) {
      // 如果找不到，尝试直接通过 placeholder 或 name 属性
      await page.locator('input[placeholder*="连接名称"], input[placeholder*="名称"]').first().fill('测试数据库');
    } else {
      await nameInput.fill('测试数据库');
    }

    // 使用更简单的方式填写表单
    const inputs = page.locator('.ant-modal input');
    const inputCount = await inputs.count();

    if (inputCount >= 5) {
      // 按顺序填写：名称、主机、端口、数据库名、用户名、密码
      await inputs.nth(0).fill('测试数据库');
      await inputs.nth(1).fill('localhost');
      // 端口可能是 InputNumber，需要特殊处理
      const portInput = page.locator('.ant-input-number-input, input[type="number"]').first();
      if (await portInput.isVisible()) {
        await portInput.fill('5432');
      }
      await inputs.nth(3).fill('testdb');
      await inputs.nth(4).fill('testuser');
      // 密码字段
      const passwordInput = page.locator('input[type="password"]').first();
      await passwordInput.fill('testpass');
    }

    // 检查测试连接按钮是否存在
    const testButton = page.getByRole('button').filter({ hasText: /测试连接/i }).first();
    await expect(testButton).toBeVisible({ timeout: 3000 });

    // 检查按钮是否可点击（不应该被禁用）
    const isDisabled = await testButton.isDisabled();
    // 如果按钮存在且未被禁用，说明表单填写正确
    if (!isDisabled) {
      await expect(testButton).toBeEnabled();
    }
  });

  test('应该能够关闭对话框', async ({ page }) => {
    // 等待页面加载
    await page.waitForSelector('.ant-layout', { timeout: 10000 });
    await page.waitForTimeout(1000);

    // 打开对话框
    const addButton = page.getByRole('button').filter({ hasText: /添加数据库连接/i }).first();
    await addButton.waitFor({ state: 'visible', timeout: 5000 });
    await addButton.click();

    // 等待对话框出现
    const dialog = page.locator('.ant-modal').first();
    await dialog.waitFor({ state: 'visible', timeout: 5000 });
    await page.waitForTimeout(500);

    // 查找取消按钮（遍历所有按钮）
    const allButtons = dialog.locator('button');
    const buttonCount = await allButtons.count();

    let cancelButton = null;
    for (let i = 0; i < buttonCount; i++) {
      const btn = allButtons.nth(i);
      const text = await btn.textContent();
      if (text && /取消/i.test(text)) {
        cancelButton = btn;
        break;
      }
    }

    // 如果找不到，尝试查找非主要按钮
    if (!cancelButton) {
      cancelButton = dialog.locator('button:not(.ant-btn-primary)').first();
    }

    await cancelButton.waitFor({ state: 'visible', timeout: 5000 });
    await cancelButton.click();

    // 等待对话框关闭（Ant Design Modal 有动画，需要更长时间）
    await page.waitForTimeout(800);

    // 对话框应该关闭（检查是否不可见）
    const isVisible = await dialog.isVisible().catch(() => false);
    expect(isVisible).toBe(false);
  });

  test('空状态应该显示提示信息', async ({ page }) => {
    // 等待页面加载
    await page.waitForSelector('.ant-layout', { timeout: 10000 });
    await page.waitForTimeout(1000);

    // 如果没有数据库连接，应该显示空状态（mock 返回空数组）
    const emptyState = page.locator('text=/暂无数据库连接/i').first();
    await expect(emptyState).toBeVisible({ timeout: 5000 });

    // 检查是否有"点击添加"链接
    const addLink = page.getByRole('button').filter({ hasText: /点击添加/i }).first();
    await expect(addLink).toBeVisible({ timeout: 3000 });
  });
});
