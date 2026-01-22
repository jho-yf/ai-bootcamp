# Playwright MCP Server 配置说明

## 概述

本项目已配置 Playwright MCP Server，允许通过 Cursor IDE 使用浏览器自动化功能。

## 配置位置

### Cursor IDE 配置

配置文件位于：`~/.cursor/mcp.json`

```json
{
  "mcpServers": {
    "playwright": {
      "command": "npx",
      "args": ["-y", "@playwright/mcp@latest"],
      "env": {}
    }
  }
}
```

### 项目配置

项目级别的 Playwright 配置位于：`.playwright-mcp.config.json`

## 安装和设置

### 1. 安装 Playwright 浏览器

```bash
cd w2-db-query
npx playwright install
```

这会安装 Chromium、Firefox 和 WebKit 浏览器。

### 2. 验证配置

重启 Cursor IDE 后，MCP server 应该会自动启动。你可以在 Cursor 中看到 Playwright 相关的工具。

### 3. 使用 Playwright MCP Server

在 Cursor 中，你可以使用以下功能：

- **浏览器自动化**：打开网页、点击元素、填写表单等
- **页面截图**：捕获网页截图
- **页面交互**：模拟用户操作
- **页面内容提取**：获取页面文本、链接等信息

## 高级配置

### 自定义浏览器选项

编辑 `.playwright-mcp.config.json`：

```json
{
  "browser": {
    "browserName": "chromium",  // 可选: chromium, firefox, webkit
    "headless": false,          // true = 无头模式
    "executablePath": "/path/to/chrome"  // 可选：自定义浏览器路径
  },
  "contextOptions": {
    "viewport": {
      "width": 1280,
      "height": 720
    },
    "userAgent": "Custom User Agent"  // 可选
  }
}
```

### 使用配置文件启动

```bash
npx @playwright/mcp@latest --config .playwright-mcp.config.json
```

### 环境变量

可以通过环境变量配置：

```bash
export PLAYWRIGHT_BROWSERS_PATH=/custom/path
```

## 故障排除

### MCP Server 未启动

1. 检查 `~/.cursor/mcp.json` 文件是否存在且格式正确
2. 重启 Cursor IDE
3. 检查 Node.js 版本（需要 18+）：`node --version`

### 浏览器未安装

运行：
```bash
npx playwright install
```

### 权限问题

确保有执行权限：
```bash
chmod +x node_modules/.bin/playwright
```

## 相关资源

- [Playwright MCP Server GitHub](https://github.com/microsoft/playwright-mcp)
- [MCP 文档](https://modelcontextprotocol.io/)
- [Playwright 文档](https://playwright.dev/)

## 注意事项

- Playwright MCP Server 主要用于浏览器自动化测试和网页交互
- 在无头模式下运行时，不会显示浏览器窗口
- 确保网络连接正常，以便访问外部网页
- 某些网站可能有反爬虫机制，可能需要额外的配置
