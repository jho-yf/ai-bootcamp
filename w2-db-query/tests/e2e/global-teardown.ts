import { FullConfig } from "@playwright/test";

/**
 * 全局测试清理
 * 在测试结束后执行
 */
async function globalTeardown(config: FullConfig) {
  // 可以在这里执行全局清理操作
  console.log("🧹 Playwright E2E 测试全局清理...");
}

export default globalTeardown;
