# w2-db-query 项目章程

<!--
同步影响报告 (SYNC IMPACT REPORT)
==================
版本变化: NEW → 1.0.0
修改的原则: 初始创建
新增章节:
  - 核心原则 (6条原则)
  - 技术栈要求
  - 开发工作流
  - 治理规则
删除章节: 无
需要更新的模板:
  ✅ plan-template.md - 已审查，兼容
  ✅ spec-template.md - 已审查，兼容
  ✅ tasks-template.md - 已审查，兼容
后续待办事项: 无
-->

## 核心原则 (Core Principles)

### I. Rust 标准样式 (Rust Standard Style)

所有 Rust 代码必须遵循 The Standard Style（官方 Rust 格式化标准，通过 `rustfmt` 实现）。
此原则为**不可协商**，适用于所有后端代码、Tauri 应用代码和基于 Rust 的工具。

**理由 (Rationale)**: 统一的代码格式消除了无意义的争论，提高代码审查效率，确保所有团队成
员能够立即阅读和理解代码库。Rust 社区标准经过实战检验并被广泛采用。

**执行 (Enforcement)**: 所有 Rust 代码在合并前必须通过 `cargo fmt --check`。CI 管道必
须强制执行此检查。

### II. TypeScript 前端 (TypeScript Frontend)

前端必须使用启用严格模式的 TypeScript 实现。源代码中不允许使用 JavaScript 
(.js/.jsx) 文件（配置文件除外）。

**理由**: TypeScript 提供编译时类型安全，在运行前捕获错误，改善 IDE 支持，并作为组件接
口和数据结构的活文档。

**要求**:
- tsconfig.json 中必须设置 `strict: true`
- 除非在代码注释中明确说明理由，否则不允许使用 `any` 类型
- 所有 React 组件必须有类型化的 props 接口

### III. 严格类型安全 (Strict Type Safety)

前端和后端必须在整个代码库中保持严格的类型标注。此原则适用于：
- 所有 Rust 函数签名（参数和返回类型）
- 所有 TypeScript 函数、组件和数据结构
- 数据库模式定义
- API 契约定义

**理由**: 显式类型作为编译器强制的契约，将运行时错误减少 70-90%，支持自信重构，并提供
新团队成员能快速理解的自文档化代码。

**不合规**: 缺少适当类型标注的代码将在代码审查中被拒绝。TypeScript 中不允许隐式 
`any`，Rust 中不允许无类型的函数参数。

### IV. CamelCase JSON 序列化

后端生成的所有 JSON 数据必须使用 camelCase 命名约定作为键名。这适用于：
- API 响应
- WebSocket 消息
- 配置文件
- 通过 API 暴露的数据库查询结果

**理由**: JavaScript/TypeScript 前端自然使用 camelCase 作为对象属性。在 JSON 中使用 
camelCase 消除了大小写转换的认知开销，减少了命名不一致导致的错误风险。

**实现**:
- 在 Rust 结构体上使用 serde 的 `#[serde(rename_all = "camelCase")]`
- 数据库列名内部可以使用 snake_case，但在 JSON 序列化前必须转换为 camelCase

### V. 开放访问（无身份验证） (Open Access - No Authentication)

应用程序不得实现身份验证或授权机制。任何用户都可以无限制访问所有功能和数据。

**理由**: 这是一个用于个人或团队本地使用的桌面应用程序，数据库 URL 和凭据在连接级别管
理。身份验证会增加复杂性而不提供安全价值，因为用户已经直接向其数据库进行身份验证。

**范围**: 此原则仅适用于应用程序层。数据库连接仍必须使用适当的身份验证来连接到 
PostgreSQL 或其他数据库系统。

### VI. 中文优先文档 (Chinese-First Documentation)

项目的所有文档必须以中文为主要语言，英文为辅助语言。这适用于：
- 代码注释（复杂逻辑和公共 API）
- README 和使用指南
- API 文档
- 开发者文档
- 提交消息（可选，但鼓励）

**理由**: 项目团队主要使用中文交流。中文优先的文档降低了理解障碍，提高了团队协作效率，
确保所有成员都能快速理解系统。

**实现指南**:
- 关键技术术语和 API 名称保留英文
- 复杂的业务逻辑和架构决策使用中文解释
- 代码示例中的变量名可以使用英文（遵循语言习惯）
- 对外开放的部分提供双语版本

## 技术栈要求 (Technology Stack Requirements)

以下技术选择是强制性的，必须遵循：

### 后端技术栈 (Backend Stack)
- **运行时**: Rust（最新稳定版）
- **应用框架**: Tauri（桌面应用包装器）
- **数据库客户端**: tokio-postgres（异步 PostgreSQL 客户端）
- **SQL 解析**: sqlparser-rs（用于查询分析和操作）
- **LLM 集成**: OpenAI SDK for Rust
- **本地存储**: SQLite（用于元数据缓存）

### 前端技术栈 (Frontend Stack)
- **UI 框架**: React 18+
- **管理框架**: Refine 5（用于 CRUD 和数据管理）
- **样式**: Tailwind CSS（实用优先的 CSS）
- **组件库**: Ant Design（用于一致的 UI 组件）
- **SQL 编辑器**: Monaco Editor（VS Code 的编辑器组件）

**理由**: 选择这些技术是因为：
- Rust 为数据库操作提供内存安全和性能
- Tauri 支持跨平台桌面应用，二进制文件体积小
- Refine 5 加速 CRUD 界面开发
- Monaco Editor 提供专业的 SQL 编辑体验，支持语法高亮

## 开发工作流 (Development Workflow)

### 代码质量门禁 (Code Quality Gates)

所有代码在合并前必须通过以下检查：

1. **Rust 代码**:
   - `cargo fmt --check`（格式化）
   - `cargo clippy -- -D warnings`（代码检查）
   - `cargo build --release`（编译）
   - `cargo test`（单元测试，如果存在）

2. **TypeScript 代码**:
   - `prettier --check`（格式化）
   - `eslint` 零警告
   - `tsc --noEmit`（类型检查）
   - `npm run build`（生产构建）

### 查询安全 (Query Safety)

LLM 生成或用户输入的 SQL 查询在执行前必须进行分析：
- 如果没有 `LIMIT` 子句，自动追加 `LIMIT 100`
- 使用 sqlparser-rs 解析查询以验证语法
- 拒绝 DDL 语句（CREATE、DROP、ALTER），除非在设置中明确启用

### 元数据管理 (Metadata Management)

数据库元数据（表、视图、列、类型）必须：
- 从 PostgreSQL 系统目录中提取
- 使用 LLM 转换为 JSON 格式以进行结构规范化
- 缓存在本地 SQLite 数据库中以便重用
- 根据用户请求或连接更改时刷新

## 治理 (Governance)

### 修订流程 (Amendment Process)

本章程可以通过以下方式修订：
1. 提交带有理由的文档化提案
2. 对现有代码进行影响分析
3. 团队审查和批准
4. 受影响代码的迁移计划

### 合规性 (Compliance)

- 所有 Pull Request 必须审查章程合规性
- 违规行为必须以书面形式说明理由并在合并前获得批准
- 本文档优先于所有其他编码标准或实践

### 版本控制策略 (Versioning Policy)

版本格式: MAJOR.MINOR.PATCH
- **MAJOR**: 核心原则或技术栈的破坏性变更
- **MINOR**: 添加新原则或扩展现有原则
- **PATCH**: 澄清、措辞改进、错误修正

**版本**: 1.0.0 | **批准日期**: 2026-01-17 | **最后修订**: 2026-01-17
