import { chromium, FullConfig } from '@playwright/test';

/**
 * 全局测试设置
 * 在测试开始前执行
 */
async function globalSetup(config: FullConfig) {
  // 可以在这里执行全局初始化操作
  // 例如：启动测试数据库、清理测试数据等
  console.log('🧪 Playwright E2E 测试全局设置...');
}

export default globalSetup;
