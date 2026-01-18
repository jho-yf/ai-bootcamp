# E2E 测试说明

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

5. **自然语言查询功能** (`natural-language-query.spec.ts`) - 新增
   - 显示标签页（SQL 编辑器和自然语言查询）
   - 切换到自然语言查询标签页
   - 显示自然语言查询按钮
   - 在顶部按钮区域显示自然语言查询按钮
   - 在 SQL 编辑器标签页禁用自然语言查询按钮
   - 输入自然语言查询
   - 显示生成的 SQL

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

# 运行自然语言查询测试
npm run test:e2e tests/e2e/natural-language-query.spec.ts
```

## 测试架构

### Tauri API Mock

由于 Tauri 应用在浏览器环境中运行，测试使用 mock 来模拟 Tauri API 调用：

- `window.__TAURI__.core.invoke()` 被 mock，返回预设的响应
- 所有测试文件在 `beforeEach` 中注入 mock
- Mock 响应包括：
  - `list_databases`: 返回测试数据库列表
  - `get_database_metadata`: 返回测试元数据
  - `run_sql_query`: 返回测试查询结果
  - `generate_sql_from_nl`: 返回生成的 SQL（新增）
  - `run_nl_query`: 返回生成的 SQL 和查询结果（新增）

### 测试文件结构

```
tests/e2e/
├── fixtures.ts              # Playwright fixtures 和 Tauri mock 工具
├── global-setup.ts          # 全局测试设置
├── global-teardown.ts       # 全局测试清理
├── helpers/                 # 测试辅助函数
│   ├── mock-tauri.ts        # Tauri API mock 工具
│   └── setup.ts             # 测试设置辅助函数
├── database-list.spec.ts    # 数据库列表页面测试
├── database-explorer.spec.ts # 数据库浏览器测试
├── query-editor.spec.ts     # SQL 查询编辑器测试
├── navigation.spec.ts       # 导航功能测试
└── natural-language-query.spec.ts # 自然语言查询测试（新增）
```

## 测试统计

- **测试文件数**: 5 个
- **测试用例总数**: 25+ 个测试场景
- **功能覆盖**: 所有主要用户界面和交互流程

## 注意事项

1. **Monaco Editor**: SQL 编辑器的 Monaco Editor 可能加载较慢，测试中使用了较长的等待时间
2. **异步操作**: 某些操作是异步的，需要适当的等待时间
3. **Mock 响应**: Mock 响应应该与实际的 Tauri API 响应格式一致
4. **浏览器兼容性**: 测试主要在 Chromium 浏览器中运行

## 持续集成

测试可以在 CI/CD 环境中运行：

```yaml
# GitHub Actions 示例
- name: Install Playwright Browsers
  run: npx playwright install --with-deps

- name: Run E2E tests
  run: npm run test:e2e
```
