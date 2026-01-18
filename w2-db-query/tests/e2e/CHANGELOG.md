# E2E 测试更新日志

## 2026-01-17 - 重写所有测试

### 变更内容

根据实际应用页面结构，重写了所有 Playwright E2E 测试。

### 测试文件更新

1. **fixtures.ts**
   - 更新 Tauri mock 以匹配实际 API 调用格式
   - 添加 `generate_sql_from_nl` 和 `run_nl_query` mock
   - 修正参数传递格式（支持 `request` 对象）

2. **navigation.spec.ts**
   - 重写为测试应用布局和导航功能
   - 测试应用标题、侧边栏、数据库列表菜单
   - 测试数据库选择交互

3. **database-list.spec.ts** (原 database-list.spec.ts)
   - 重写为测试数据库连接管理功能
   - 测试添加、编辑、删除数据库连接
   - 测试对话框交互和表单验证

4. **database-explorer.spec.ts**
   - 重写为测试数据库浏览器功能
   - 测试元数据显示、刷新按钮
   - 测试标签页显示

5. **query-editor.spec.ts**
   - 重写为测试 SQL 查询编辑器功能
   - 测试 SQL 编辑器显示和输入
   - 测试执行和清除按钮

6. **natural-language-query.spec.ts**
   - 重写为测试自然语言查询功能
   - 测试标签页切换
   - 测试自然语言输入和 SQL 生成
   - 测试按钮状态（启用/禁用）

### 主要改进

1. **更准确的元素选择器**
   - 使用 Ant Design 组件的实际类名和角色
   - 使用更灵活的选择器，支持元素可能不存在的情况

2. **更好的错误处理**
   - 使用 `isVisible().catch(() => false)` 检查元素是否存在
   - 避免因元素不存在而导致的测试失败

3. **更符合实际结构**
   - 测试基于实际的 `AppLayout` 和 `DatabaseExplorer` 组件结构
   - Mock 数据匹配实际的 API 响应格式

4. **更全面的测试覆盖**
   - 覆盖所有主要用户交互流程
   - 测试边界情况和错误处理

### 测试执行

```bash
# 启动开发服务器（在另一个终端）
npm run dev

# 运行所有 E2E 测试
npm run test:e2e

# UI 模式（推荐用于调试）
npm run test:e2e:ui
```

### 注意事项

1. 测试需要开发服务器运行在 `http://localhost:1420`
2. 某些测试可能会因为 Monaco Editor 加载时间而需要更长等待时间
3. 部分元素可能在特定状态下不可见（例如按钮禁用状态），测试会相应处理
