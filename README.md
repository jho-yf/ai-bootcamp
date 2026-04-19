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
├── specs/                # 各项目的规格说明和设计文档
├── docs/                 # 通用文档
└── Makefile              # 根目录构建脚本
```

## 子项目介绍

### w1-project-alpha
全栈演示项目，包含：
- **Backend**: Rust + Axum + SQLx + PostgreSQL
- **Frontend**: React + TypeScript + Vite

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
Tauri + React 数据库查询工具，支持：
- Playwright E2E 测试
- 数据库查询与可视化

### w3-raflow
Tauri 工作流工具，探索 Rust + React 的应用场景。

### w5-pg-mcp
PostgreSQL Model Context Protocol 服务器，用 Rust 实现：
- LLM 驱动的自然语言到 SQL 转换
- Schema 自动发现
- SQL 验证与安全检查

### w6-opencode-logging
OpenCode CLI 的 LLM 日志记录插件。

## 技术栈

- **语言**: Rust, TypeScript, SQL
- **后端**: Axum, Tauri, SQLx
- **前端**: React, Vite, TailwindCSS
- **数据库**: PostgreSQL
- **工具**: Playwright, pre-commit

## 开发

项目使用 pre-commit 钩子确保代码质量：
```bash
pre-commit install
```

## 文档

- `instructions.md` - 开发任务说明
- `specs/w*/` - 各项目的详细规格说明
- `docs/` - 通用设计文档
