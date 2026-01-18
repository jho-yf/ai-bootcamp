import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright 测试配置
 * 用于测试 Tauri 应用的前端界面
 *
 * 注意：Tauri 应用需要先启动才能测试
 * 运行测试前，请先启动应用：npm run tauri dev
 */
export default defineConfig({
  testDir: './tests/e2e',

  // 全局设置和清理
  globalSetup: './tests/e2e/global-setup.ts',
  globalTeardown: './tests/e2e/global-teardown.ts',

  // 测试超时时间
  timeout: 30 * 1000,
  expect: {
    timeout: 5000,
  },

  // 并行执行设置
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,

  // 报告设置
  reporter: [
    ['html', { outputFolder: 'playwright-report' }],
    ['list'],
  ],

  // 共享设置
  use: {
    baseURL: 'http://localhost:1420',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },

  // 测试项目配置
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    // 可以添加更多浏览器
    // {
    //   name: 'firefox',
    //   use: { ...devices['Desktop Firefox'] },
    // },
    // {
    //   name: 'webkit',
    //   use: { ...devices['Desktop Safari'] },
    // },
  ],

  // Web Server 配置：启动 Vite 开发服务器用于测试
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:1420',
    reuseExistingServer: true, // 始终重用现有服务器
    timeout: 120 * 1000,
    stdout: 'ignore',
    stderr: 'pipe',
  },
});
