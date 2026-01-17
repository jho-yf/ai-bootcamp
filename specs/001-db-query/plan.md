# 实施计划：数据库查询工具

**Branch**: `001-db-query` | **Date**: 2026-01-17 | **Spec**: [spec.md](./spec.md)  
**Input**: 功能规范来自 `/specs/001-db-query/spec.md`

## 概述

一个跨平台桌面应用，用于连接和查询 PostgreSQL 数据库。核心功能包括：
- 数据库连接管理和元数据自动提取
- 带语法高亮的 SQL 编辑器和查询执行
- AI 驱动的自然语言到 SQL 的查询生成

技术方案：使用 Tauri 构建桌面应用框架，Rust 后端处理数据库连接和查询执行，React + Refine 前端提供现代化用户界面，SQLite 本地缓存元数据，OpenAI API 提供自然语言查询能力。

## 技术上下文

**Language/Version**: Rust 1.75+ (后端), TypeScript 5.0+ (前端)  
**Primary Dependencies**: 
- 后端: Tauri 1.5+, tokio-postgres 0.7+, sqlparser-rs 0.38+, async-openai (OpenAI SDK), rusqlite (SQLite 客户端), serde 1.0+ (JSON 序列化)
- 前端: React 18+, Refine 5, Ant Design 5+, Monaco Editor, Tailwind CSS 3+, @tauri-apps/api (前端桥接)

**Storage**: 
- PostgreSQL (外部用户数据库，通过 tokio-postgres 连接)
- SQLite (本地元数据缓存和配置存储，路径: `./db_query.db`)

**Testing**: cargo test (Rust 单元测试), vitest (前端单元测试)  
**Target Platform**: 桌面应用（Windows 10+, macOS 11+, Linux）  
**Project Type**: Tauri 桌面应用（前后端分离架构）  
**Performance Goals**: 
- 数据库连接和元数据提取 < 30秒
- 查询执行和结果显示 < 2秒（1000行内）
- 应用启动和缓存加载 < 3秒

**Constraints**: 
- 单个查询结果自动限制为 100 行（LIMIT 100）
- 支持最多 100 列 × 100 行的结果表格流畅渲染
- 查询结果必须使用 camelCase JSON 格式
- 本地 SQLite 数据库用于持久化存储

**Scale/Scope**: 
- 支持多个 PostgreSQL 数据库连接
- 每个数据库可能包含数百个表和视图
- 元数据缓存可能达到数 MB
- 单用户桌面应用（无并发用户）

## 章程检查

*GATE: 必须在 Phase 0 研究前通过。Phase 1 设计后重新检查。*

### ✅ 通过的原则

- **I. Rust 标准样式**: 所有 Rust 代码使用 `cargo fmt`，CI 强制执行 ✅
- **II. TypeScript 前端**: 前端使用 TypeScript strict 模式，无 JavaScript 文件 ✅
- **III. 严格类型安全**: 所有 Rust 函数和 TypeScript 组件都有完整类型标注 ✅
- **IV. CamelCase JSON**: 使用 `#[serde(rename_all = "camelCase")]` 序列化所有响应 ✅
- **V. 开放访问**: 应用层无身份验证，数据库连接使用标准凭据 ✅
- **VI. 中文优先文档**: 所有文档、注释、错误消息以中文为主 ✅

### 技术栈合规性

- **后端**: Rust + Tauri + tokio-postgres + sqlparser-rs + OpenAI SDK + SQLite ✅
- **前端**: React 18+ + Refine 5 + Tailwind CSS + Ant Design + Monaco Editor ✅

### 代码质量门禁

**Rust**:
- `cargo fmt --check` - 格式化检查
- `cargo clippy -- -D warnings` - 严格代码检查
- `cargo build --release` - 发布构建验证
- `cargo test` - 单元测试（如有）

**TypeScript**:
- `prettier --check` - 格式化检查
- `eslint` - 零警告
- `tsc --noEmit` - 类型检查
- `npm run build` - 生产构建

### 查询安全合规性

- 自动为无 LIMIT 的 SELECT 查询追加 `LIMIT 100` ✅
- 使用 sqlparser-rs 解析和验证查询语法 ✅
- 默认拒绝 DDL 语句（CREATE/DROP/ALTER）✅

## 项目结构

### 文档（本功能）

```text
specs/001-db-query/
├── plan.md              # 本文件（/speckit.plan 命令输出）
├── research.md          # Phase 0 输出（/speckit.plan 命令）
├── data-model.md        # Phase 1 输出（/speckit.plan 命令）
├── quickstart.md        # Phase 1 输出（/speckit.plan 命令）
├── contracts/           # Phase 1 输出（/speckit.plan 命令）
│   └── tauri-commands.md
└── tasks.md             # Phase 2 输出（/speckit.tasks 命令 - 非 /speckit.plan 创建）
```

### 源代码（仓库根目录）

本项目是 Tauri 桌面应用，采用前后端分离架构：

```text
w2-db-query/
├── src-tauri/               # Rust 后端（Tauri 应用）
│   ├── src/
│   │   ├── main.rs          # Tauri 应用入口
│   │   ├── commands/        # Tauri command 函数
│   │   │   ├── mod.rs
│   │   │   ├── database.rs  # 数据库连接管理
│   │   │   ├── metadata.rs  # 元数据提取
│   │   │   ├── query.rs     # SQL 查询执行
│   │   │   └── ai.rs        # AI 自然语言查询
│   │   ├── models/          # 数据模型（DatabaseInfo, QueryResult 等）
│   │   │   ├── mod.rs
│   │   │   ├── database.rs
│   │   │   ├── metadata.rs
│   │   │   └── query.rs
│   │   ├── services/        # 业务逻辑服务
│   │   │   ├── mod.rs
│   │   │   ├── postgres_service.rs  # PostgreSQL 连接和查询
│   │   │   ├── metadata_service.rs  # 元数据提取和转换
│   │   │   ├── cache_service.rs     # SQLite 缓存管理
│   │   │   ├── query_parser.rs      # SQL 解析和 LIMIT 注入
│   │   │   └── ai_service.rs        # OpenAI API 集成
│   │   └── utils/           # 工具函数
│   │       ├── mod.rs
│   │       ├── error.rs     # 错误处理
│   │       └── config.rs    # 配置管理（环境变量）
│   ├── Cargo.toml           # Rust 依赖
│   └── tauri.conf.json      # Tauri 配置
│
├── src/                     # React 前端
│   ├── main.tsx             # 前端入口
│   ├── App.tsx              # 根组件
│   ├── pages/               # 页面组件
│   │   ├── DatabaseList.tsx      # 数据库连接列表
│   │   ├── QueryEditor.tsx       # SQL 编辑器和查询界面
│   │   └── DatabaseExplorer.tsx  # 数据库结构浏览器
│   ├── components/          # 可复用组件
│   │   ├── DatabaseCard.tsx      # 数据库连接卡片
│   │   ├── MetadataTree.tsx      # 元数据树形视图
│   │   ├── SqlEditor.tsx         # Monaco Editor 包装器
│   │   ├── QueryResultTable.tsx  # 查询结果表格
│   │   ├── NLQueryInput.tsx      # 自然语言查询输入
│   │   └── LoadingIndicator.tsx # 加载指示器
│   ├── services/            # 前端服务（Tauri API 调用）
│   │   ├── api.ts           # Tauri invoke 封装
│   │   └── types.ts         # TypeScript 类型定义
│   ├── hooks/               # 自定义 React Hooks
│   │   ├── useDatabases.ts
│   │   ├── useMetadata.ts
│   │   └── useQuery.ts
│   ├── stores/              # 状态管理（Zustand 或 Context）
│   │   ├── databaseStore.ts
│   │   └── queryStore.ts
│   └── styles/              # 样式文件
│       └── globals.css      # Tailwind CSS 全局样式
│
├── package.json             # 前端依赖
├── tsconfig.json            # TypeScript 配置
├── tailwind.config.js       # Tailwind CSS 配置
├── vite.config.ts           # Vite 构建配置
└── README.md                # 项目说明
```

**结构决策**: 选择 Tauri 桌面应用架构，因为需求明确为跨平台桌面应用。前端使用 React + Refine 快速构建 CRUD 界面，后端使用 Rust 保证性能和安全性。Tauri 的 command 模式提供清晰的前后端通信接口。

## 复杂度跟踪

> 本项目完全符合章程，无违规需要说明。

| 违规项 | 为何需要 | 拒绝的简化方案及原因 |
|--------|----------|---------------------|
| 无 | 无 | 无 |
