# AI Bootcamp

AI 训练营项目集合，包含多个实验性项目，涵盖 AI 工具集成、全栈开发、Rust 编程等主题。

## 项目结构

```
ai-bootcamp/
├── w1-project-alpha/     # 全栈项目 (Rust/Axum + React)
├── w2-db-query/          # Tauri 数据库查询工具
├── w3-raflow/            # Tauri 工作流工具
├── w5-pg-mcp/            # PostgreSQL MCP 服务器 (Rust)
├── w6-opencode-logging/  # OpenCode 日志插件
├── w7-genslides/         # AI 幻灯片生成工具 (Vue + Next.js)
├── specs/                # 各项目的规格说明和设计文档
├── docs/                 # 通用文档
└── Makefile              # 根目录构建脚本
```

## 子项目介绍

### w1-project-alpha
全栈演示项目：
- **语言**: Rust, TypeScript, SQL
- **后端**: Axum + SQLx + PostgreSQL
- **前端**: React + Vite + TailwindCSS

快速启动：
```bash
make w1-install    # 安装依赖
make w1-dev        # 启动开发环境
make w1-backend    # 仅启动后端
make w1-frontend   # 仅启动前端
make w1-backend-build   # 构建后端
make w1-frontend-build  # 构建前端
```

### w2-db-query
Tauri 数据库查询工具：
- **语言**: Rust, TypeScript
- **桌面框架**: Tauri
- **前端**: React + Vite
- **测试**: Playwright E2E

### w3-raflow
Tauri 工作流工具：
- **语言**: Rust, TypeScript
- **桌面框架**: Tauri
- **前端**: React + Vite

### w5-pg-mcp
PostgreSQL MCP 服务器：
- **语言**: Rust
- **协议**: Model Context Protocol
- **功能**: 自然语言转 SQL、Schema 自动发现、SQL 安全检查

### w6-opencode-logging
OpenCode CLI 日志插件：
- **语言**: TypeScript
- **功能**: LLM 调用日志记录

### w7-genslides
AI 幻灯片生成工具：
- **语言**: TypeScript
- **前端**: Vue 3 + Vite + Pinia + TailwindCSS
- **后端**: Next.js (App Router)
- **AI**: Google Nano Banana Pro 图片生成

## 开发

项目使用 pre-commit 钩子确保代码质量：
```bash
pre-commit install
```

## 文档

- `instructions.md` - 开发任务说明
- `specs/w*/` - 各项目的详细规格说明
- `docs/` - 通用设计文档
