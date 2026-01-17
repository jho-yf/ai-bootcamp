# 任务列表：数据库查询工具

**Input**: 设计文档来自 `/specs/001-db-query/`  
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/tauri-commands.md

**组织原则**: 任务按 3 个主要阶段组织，优化为快速交付可用产品。

## 格式：`[ID] [P?] [Story?] 描述`

- **[P]**: 可并行运行（不同文件，无依赖）
- **[Story]**: 映射到用户故事（US1-P1, US2-P2, US3-P3）
- 包含具体文件路径

## 路径约定

本项目使用 Tauri 桌面应用架构：
- **后端**: `w2-db-query/src-tauri/src/`
- **前端**: `w2-db-query/src/`
- **配置**: `w2-db-query/` 根目录

---

## Phase 1: 项目设置和基础设施

**目的**: 初始化 Tauri 项目并搭建核心基础设施

### 项目初始化

- [ ] T001 创建 w2-db-query 目录并初始化 Tauri 项目（使用 `npm create tauri-app@latest`）
- [ ] T002 [P] 配置后端 Cargo.toml 添加依赖：tauri, serde, tokio-postgres, rusqlite, sqlparser, async-openai, uuid, chrono
- [ ] T003 [P] 配置前端 package.json 添加依赖：@refinedev/core, @refinedev/antd, antd, @monaco-editor/react, @tauri-apps/api
- [ ] T004 [P] 创建 .env 文件模板配置 OPENAI_API_KEY
- [ ] T005 [P] 配置 TypeScript tsconfig.json 启用 strict 模式

### 后端基础结构

- [ ] T006 [P] 创建后端目录结构：src-tauri/src/{commands, models, services, utils}
- [ ] T007 [P] 实现错误处理 src-tauri/src/utils/error.rs 定义 AppError 枚举
- [ ] T008 [P] 实现配置管理 src-tauri/src/utils/config.rs 读取环境变量
- [ ] T009 实现 SQLite Schema 初始化 src-tauri/src/services/cache_service.rs 创建 connections 和 metadata 表

### 前端基础结构

- [ ] T010 [P] 创建前端目录结构：src/{pages, components, services, hooks, stores, styles}
- [ ] T011 [P] 配置 Tailwind CSS tailwind.config.js 和 globals.css
- [ ] T012 [P] 定义 TypeScript 类型 src/services/types.ts 映射后端数据模型
- [ ] T013 [P] 创建 Tauri API 封装 src/services/api.ts 统一 invoke 调用

**Checkpoint**: 项目结构就绪，可以启动 Tauri 开发服务器（`npm run tauri dev`）

---

## Phase 2: 核心功能 - 数据库连接和查询（MVP）

**目的**: 实现 P1（数据库连接管理）和 P2（SQL 查询执行）功能，构成可用的 MVP

### 2.1 数据模型（P1+P2 共享）

- [ ] T014 [P] [US1] 实现 DatabaseConnection 模型 src-tauri/src/models/database.rs
- [ ] T015 [P] [US1] 实现 DatabaseMetadata, TableInfo, ColumnInfo 模型 src-tauri/src/models/metadata.rs
- [ ] T016 [P] [US2] 实现 QueryResult 模型 src-tauri/src/models/query.rs

### 2.2 数据库连接服务（P1）

- [ ] T017 [US1] 实现 PostgreSQL 连接服务 src-tauri/src/services/postgres_service.rs：连接、断开、测试连接
- [ ] T018 [US1] 实现 SQLite 缓存服务 src-tauri/src/services/cache_service.rs：保存/加载连接配置（加密密码）
- [ ] T019 [US1] 实现元数据提取服务 src-tauri/src/services/metadata_service.rs：查询 information_schema 并解析

### 2.3 数据库连接管理 Commands（P1）

- [ ] T020 [P] [US1] 实现 list_databases command src-tauri/src/commands/database.rs
- [ ] T021 [P] [US1] 实现 add_database command src-tauri/src/commands/database.rs（含元数据自动提取）
- [ ] T022 [P] [US1] 实现 update_database command src-tauri/src/commands/database.rs
- [ ] T023 [P] [US1] 实现 delete_database command src-tauri/src/commands/database.rs
- [ ] T024 [P] [US1] 实现 test_connection command src-tauri/src/commands/database.rs

### 2.4 元数据管理 Commands（P1）

- [ ] T025 [P] [US1] 实现 get_database_metadata command src-tauri/src/commands/metadata.rs（优先返回缓存）
- [ ] T026 [P] [US1] 实现 refresh_metadata command src-tauri/src/commands/metadata.rs（强制重新提取）

### 2.5 前端 - 数据库连接管理界面（P1）

- [ ] T027 [P] [US1] 创建 DatabaseList 页面 src/pages/DatabaseList.tsx（显示所有连接）
- [ ] T028 [P] [US1] 创建 DatabaseCard 组件 src/components/DatabaseCard.tsx（单个连接卡片）
- [ ] T029 [US1] 创建添加连接表单对话框 src/components/AddDatabaseDialog.tsx（使用 Ant Design Form）
- [ ] T030 [US1] 创建 useDatabases Hook src/hooks/useDatabases.ts（封装数据库 CRUD 操作）

### 2.6 前端 - 元数据浏览界面（P1）

- [ ] T031 [P] [US1] 创建 DatabaseExplorer 页面 src/pages/DatabaseExplorer.tsx（主布局：侧边栏+内容区）
- [ ] T032 [US1] 创建 MetadataTree 组件 src/components/MetadataTree.tsx（使用 Ant Design Tree 显示表和视图）
- [ ] T033 [US1] 创建 useMetadata Hook src/hooks/useMetadata.ts（加载和刷新元数据）

**Checkpoint US1**: 此时可以添加数据库连接并浏览数据库结构（独立测试 P1 功能）

### 2.7 SQL 查询解析服务（P2）

- [ ] T034 [US2] 实现 SQL 解析器 src-tauri/src/services/query_parser.rs：使用 sqlparser-rs 解析并注入 LIMIT 100
- [ ] T035 [US2] 在 query_parser.rs 中添加 DDL 检测和拒绝逻辑

### 2.8 SQL 查询执行 Commands（P2）

- [ ] T036 [P] [US2] 实现 run_sql_query command src-tauri/src/commands/query.rs（解析、执行、返回结果）
- [ ] T037 [P] [US2] 实现 cancel_query command src-tauri/src/commands/query.rs（取消正在执行的查询）

### 2.9 前端 - SQL 编辑和查询界面（P2）

- [ ] T038 [US2] 创建 QueryEditor 页面 src/pages/QueryEditor.tsx（主布局：编辑器+结果表格）
- [ ] T039 [US2] 创建 SqlEditor 组件 src/components/SqlEditor.tsx（集成 Monaco Editor，配置 SQL 语法高亮）
- [ ] T040 [US2] 创建 QueryResultTable 组件 src/components/QueryResultTable.tsx（使用 Ant Design Table 显示结果）
- [ ] T041 [US2] 创建 LoadingIndicator 组件 src/components/LoadingIndicator.tsx（查询执行时显示）
- [ ] T042 [US2] 创建 useQuery Hook src/hooks/useQuery.ts（执行查询、管理加载状态）

### 2.10 前端 - 应用主框架和路由

- [ ] T043 更新 App.tsx src/App.tsx 配置路由：DatabaseList, DatabaseExplorer, QueryEditor
- [ ] T044 创建全局布局组件 src/components/Layout.tsx（顶部导航栏、侧边菜单）

### 2.11 集成和错误处理

- [ ] T045 在 src-tauri/src/main.rs 中注册所有 Tauri commands
- [ ] T046 实现前端全局错误处理 src/services/api.ts（捕获 invoke 错误并显示 Ant Design message）

**Checkpoint US2**: 此时可以执行 SQL 查询并查看结果表格（独立测试 P1+P2 完整功能）

---

## Phase 3: 高级功能 - AI 自然语言查询（P3）

**目的**: 实现自然语言到 SQL 的智能转换功能

### 3.1 AI 服务集成

- [ ] T047 [US3] 实现 OpenAI 服务 src-tauri/src/services/ai_service.rs：调用 OpenAI API，传递元数据上下文
- [ ] T048 [US3] 实现 Prompt 构建逻辑 ai_service.rs：将元数据格式化为 LLM 上下文

### 3.2 AI 查询 Commands

- [ ] T049 [P] [US3] 实现 generate_sql_from_nl command src-tauri/src/commands/ai.rs（仅生成 SQL，不执行）
- [ ] T050 [P] [US3] 实现 run_nl_query command src-tauri/src/commands/ai.rs（生成并执行 SQL）

### 3.3 前端 - 自然语言查询界面

- [ ] T051 [US3] 创建 NLQueryInput 组件 src/components/NLQueryInput.tsx（自然语言输入框和生成按钮）
- [ ] T052 [US3] 集成 NLQueryInput 到 QueryEditor 页面（标签切换：SQL 编辑器 / 自然语言输入）
- [ ] T053 [US3] 在 useQuery Hook 中添加自然语言查询支持

### 3.4 最终优化

- [ ] T054 [P] 添加查询历史记录功能：在 query_history 表中保存查询（区分 sql 和 natural_language 类型）
- [ ] T055 [P] 实现代码格式化：Rust `cargo fmt`，TypeScript `prettier`
- [ ] T056 运行 quickstart.md 中的验证场景，确保所有用户故事可用

**Checkpoint US3**: 完整应用就绪，支持自然语言查询（独立测试 P3 功能）

---

## 依赖关系和执行顺序

### Phase 依赖

- **Phase 1**: 无依赖 - 可立即开始
- **Phase 2**: 依赖 Phase 1 完成 - 阻塞所有核心功能
- **Phase 3**: 依赖 Phase 2 完成 - 需要基础查询功能

### 任务依赖（Phase 内）

#### Phase 1
- T001 必须最先执行（创建项目）
- T002-T005 可并行（配置依赖）
- T006-T009 可并行（后端结构，T009 依赖 SQLite 依赖已配置）
- T010-T013 可并行（前端结构）

#### Phase 2
- T014-T016 可并行（数据模型，无依赖）
- T017-T019 顺序执行（服务层有依赖关系）
- T020-T024 可并行（Commands，依赖 T017-T019）
- T025-T026 可并行（元数据 Commands，依赖 T019）
- T027-T030 依赖 T020-T026（前端依赖后端 API）
- T031-T033 依赖 T025-T026（元数据前端依赖元数据 API）
- T034-T035 顺序执行（查询解析）
- T036-T037 可并行（查询 Commands，依赖 T034-T035）
- T038-T042 依赖 T036-T037（查询前端依赖查询 API）
- T043-T044 依赖所有前端页面和组件
- T045-T046 最后执行（集成）

#### Phase 3
- T047-T048 顺序执行（AI 服务）
- T049-T050 可并行（AI Commands，依赖 T047-T048）
- T051-T053 依赖 T049-T050（前端依赖 AI API）
- T054-T056 可并行（最终优化）

### 并行执行机会

**Phase 1 并行组**:
```
并行组 A: T002, T003, T004, T005（配置文件）
并行组 B: T006, T007, T008, T009（后端基础，前提是 T002 完成）
并行组 C: T010, T011, T012, T013（前端基础，前提是 T003 完成）
```

**Phase 2 并行组**:
```
并行组 A: T014, T015, T016（数据模型）
并行组 B: T020, T021, T022, T023, T024（数据库 Commands，前提是 T017-T019 完成）
并行组 C: T025, T026（元数据 Commands）
并行组 D: T027, T028（数据库列表前端，前提是并行组 B 完成）
并行组 E: T036, T037（查询 Commands，前提是 T034-T035 完成）
```

**Phase 3 并行组**:
```
并行组 A: T049, T050（AI Commands，前提是 T047-T048 完成）
并行组 B: T054, T055, T056（最终优化）
```

---

## 实施策略

### MVP First（推荐）

1. **Phase 1** → 项目初始化完成
2. **Phase 2 至 T033** → 完成 US1（数据库连接和元数据浏览）
   - **停止并验证**: 测试添加连接、浏览表结构
   - 此时已有可演示的 MVP！
3. **Phase 2 至 T046** → 完成 US2（SQL 查询执行）
   - **停止并验证**: 测试执行查询、查看结果
   - 此时已有完整的数据库查询工具！
4. **Phase 3** → 完成 US3（AI 自然语言查询）
   - **停止并验证**: 测试自然语言生成 SQL
   - 完整产品交付！

### 增量交付

- **里程碑 1** (Phase 1): 项目框架就绪
- **里程碑 2** (Phase 2 US1): MVP - 数据库连接和浏览
- **里程碑 3** (Phase 2 US2): 核心功能 - SQL 查询执行
- **里程碑 4** (Phase 3): 完整产品 - AI 增强

### 团队协作（如果多人）

1. **Phase 1**: 团队一起完成基础设施
2. **Phase 2**: 
   - 开发者 A: US1 后端（T017-T026）
   - 开发者 B: US1 前端（T027-T033）
   - 完成后切换到 US2
3. **Phase 3**: 
   - 开发者 A: AI 后端（T047-T050）
   - 开发者 B: AI 前端（T051-T053）

---

## 任务统计

- **总任务数**: 56
- **Phase 1**: 13 任务（项目设置）
- **Phase 2**: 33 任务（核心功能 MVP）
  - US1 任务: 17 个
  - US2 任务: 13 个
  - 共享/集成: 3 个
- **Phase 3**: 10 任务（AI 高级功能）

- **可并行任务**: 26 个（标记 [P]）
- **顺序任务**: 30 个（有依赖关系）

---

## 备注

- [P] 任务 = 不同文件，无依赖，可并行执行
- [Story] 标签 = 映射到具体用户故事，支持独立测试
- 每个 Phase 完成后都有 Checkpoint，可以停下来验证功能
- 避免: 模糊任务、文件冲突、跨用户故事的强依赖
- 提交频率: 每完成 2-3 个相关任务或每个逻辑功能点提交一次
