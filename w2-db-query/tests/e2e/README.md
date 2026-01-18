# E2E 测试说明

## 运行测试

### 前置条件

1. **启动开发服务器**（在另一个终端）：
   ```bash
   npm run dev
   ```

2. **运行测试**：
   ```bash
   # 运行所有测试
   npm run test:e2e

   # UI 模式（推荐用于调试）
   npm run test:e2e:ui

   # 调试模式
   npm run test:e2e:debug

   # 有头模式（显示浏览器）
   npm run test:e2e:headed
   ```

## 测试覆盖

- ✅ 数据库列表页面
- ✅ 导航功能
- ✅ SQL 查询编辑器
- ✅ 数据库浏览器

## 注意事项

- 测试使用 Tauri API mock，不需要真实的后端连接
- 确保开发服务器在 `http://localhost:1420` 运行
- 测试会自动注入 mock，模拟 Tauri invoke 调用
