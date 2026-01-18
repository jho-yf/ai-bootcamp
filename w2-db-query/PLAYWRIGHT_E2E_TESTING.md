# Playwright E2E 测试指南

## 概述

本项目使用 Playwright 进行端到端（E2E）测试，覆盖前端的主要功能和用户交互。

## 测试覆盖

### ✅ 已实现的测试

1. **数据库列表页面** (`database-list.spec.ts`)
   - 显示数据库连接页面
   - 打开添加数据库连接对话框
   - 表单验证
   - 填写并测试连接
   - 关闭对话框
   - 空状态提示

2. **导航功能** (`navigation.spec.ts`)
   - 导航到数据库连接页面
   - 导航到数据库浏览器页面
   - 导航到 SQL 查询页面
   - 菜单项高亮

3. **SQL 查询编辑器** (`query-editor.spec.ts`)
   - 显示 SQL 编辑器
   - 输入 SQL 查询
   - 显示执行查询按钮
   - 按钮禁用状态
   - 数据库选择下拉框
   - 清除结果按钮

4. **数据库浏览器** (`database-explorer.spec.ts`)
   - 数据库选择下拉框
   - 刷新元数据按钮
   - 侧边栏显示
   - 内容区域显示
   - 未选择数据库时的提示

## 运行测试

### 前置条件

1. **安装依赖**：
   ```bash
   npm install
   ```

2. **启动开发服务器**（在另一个终端）：
   ```bash
   npm run dev
   ```
   确保服务器运行在 `http://localhost:1420`

### 运行命令

```bash
# 运行所有测试
npm run test:e2e

# UI 模式（推荐用于调试）
npm run test:e2e:ui

# 调试模式
npm run test:e2e:debug

# 有头模式（显示浏览器窗口）
npm run test:e2e:headed

# 运行特定测试文件
npm run test:e2e tests/e2e/database-list.spec.ts

# 运行特定测试用例
npm run test:e2e tests/e2e/database-list.spec.ts:53
```

## 测试架构

### Tauri API Mock

由于 Tauri 应用在浏览器环境中运行，测试使用 mock 来模拟 Tauri API 调用：

- `window.__TAURI__.core.invoke()` 被 mock，返回预设的响应
- 所有测试文件在 `beforeEach` 中注入 mock
- Mock 响应包括：
  - `list_databases`: 返回测试数据库列表
  - `test_connection`: 返回连接测试结果
  - `add_database`: 返回新创建的数据库连接
  - `get_database_metadata`: 返回数据库元数据
  - `run_sql_query`: 返回查询结果

### 测试配置

- **配置文件**: `playwright.config.ts`
- **测试目录**: `tests/e2e/`
- **全局设置**: `tests/e2e/global-setup.ts`
- **全局清理**: `tests/e2e/global-teardown.ts`

## 编写新测试

### 基本结构

```typescript
import { test, expect } from '@playwright/test';

test.describe('功能名称', () => {
  test.beforeEach(async ({ page }) => {
    // 注入 Tauri mock
    await page.addInitScript(() => {
      if (!window.__TAURI__) {
        (window as any).__TAURI__ = {
          core: {
            invoke: async (cmd: string, args?: any) => {
              // Mock 响应
              return null;
            },
          },
        };
      }
    });
    
    await page.goto('/');
    await page.waitForSelector('.ant-layout', { timeout: 10000 });
  });

  test('测试用例描述', async ({ page }) => {
    // 测试代码
  });
});
```

### 最佳实践

1. **使用灵活的选择器**：Ant Design 组件可能使用不同的类名，使用文本匹配更可靠
2. **添加适当的等待**：React 应用需要时间渲染，使用 `waitForTimeout` 或 `waitForSelector`
3. **处理异步操作**：表单提交、对话框打开等操作需要等待
4. **错误处理**：使用 `try-catch` 或 `.catch()` 处理可能失败的操作

## 故障排除

### 测试失败常见原因

1. **开发服务器未启动**：确保 `npm run dev` 正在运行
2. **选择器超时**：增加 `timeout` 或使用更灵活的选择器
3. **异步操作未完成**：添加适当的等待时间
4. **Monaco Editor 交互**：Monaco Editor 结构复杂，可能需要特殊处理

### 调试技巧

1. **使用 UI 模式**：`npm run test:e2e:ui` 可以逐步执行测试
2. **查看截图和视频**：测试失败时会自动生成截图和视频
3. **使用 Playwright Inspector**：`npm run test:e2e:debug` 打开调试器
4. **查看 trace**：使用 `npx playwright show-trace <trace-file>` 查看详细执行过程

## CI/CD 集成

在 CI 环境中运行测试：

```yaml
- name: Install dependencies
  run: npm install

- name: Install Playwright browsers
  run: npx playwright install --with-deps chromium

- name: Run E2E tests
  run: npm run test:e2e
  env:
    CI: true
```

## 相关文档

- [Playwright 官方文档](https://playwright.dev/)
- [项目测试 README](./tests/e2e/README.md)
