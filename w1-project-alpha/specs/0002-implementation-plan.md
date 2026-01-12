# Project Alpha - 实现计划文档

## 1. 项目概述

本文档基于 `0001-spec.md` 需求文档，详细规划 Project Alpha 项目的实现步骤。项目采用前后端分离架构，分阶段实现，确保每个阶段都有可交付的成果。

## 2. 项目结构

```
project-alpha/
├── backend/              # Rust 后端
│   ├── src/
│   ├── migrations/
│   ├── tests/
│   └── Cargo.toml
├── frontend/            # React 前端
│   ├── src/
│   ├── public/
│   └── package.json
└── specs/               # 文档
    ├── 0001-spec.md
    └── 0002-implementation-plan.md
```

## 3. 后端实现计划

### 3.1 Phase 1: 基础架构搭建（预计 2-3 天）

#### 3.1.1 项目初始化
1. 创建 Rust 项目
   ```bash
   cargo new project-alpha-backend
   cd project-alpha-backend
   ```

2. 配置 `Cargo.toml`
   - 添加依赖：axum, sqlx, tokio, serde, chrono, uuid, dotenvy, thiserror, anyhow, tracing
   - 配置工作空间（如需要）

3. 创建项目目录结构
   ```
   src/
   ├── main.rs
   ├── config/
   ├── handlers/
   ├── models/
   ├── repositories/
   ├── services/
   ├── routes/
   └── utils/
   ```

#### 3.1.2 数据库迁移
1. 创建迁移文件 `migrations/20231201000001_initial_schema.sql`
   - 创建 tickets 表
   - 创建 tags 表
   - 创建 ticket_tags 关联表
   - 创建索引（GIN 索引用于搜索，关联表索引）
   - 启用 PostgreSQL 扩展（uuid-ossp, pg_trgm）

2. 配置 SQLx 迁移
   - 在 `main.rs` 中集成迁移功能
   - 应用启动时自动运行迁移

#### 3.1.3 配置管理
1. 创建 `src/config/mod.rs`
   - 定义配置结构体（数据库连接、服务器端口等）
   - 从环境变量加载配置
   - 提供默认值

2. 创建 `.env.example` 文件
   - 数据库连接字符串
   - 服务器端口
   - 日志级别

#### 3.1.4 数据模型
1. 创建 `src/models/ticket.rs`
   - Ticket 结构体
   - TicketWithTags 结构体
   - CreateTicketRequest 结构体
   - UpdateTicketRequest 结构体
   - 实现 Serialize/Deserialize

2. 创建 `src/models/tag.rs`
   - Tag 结构体
   - CreateTagRequest 结构体
   - UpdateTagRequest 结构体
   - 实现 Serialize/Deserialize

3. 创建 `src/models/mod.rs`
   - 导出所有模型

#### 3.1.5 错误处理
1. 创建 `src/utils/error.rs`
   - 定义 AppError 枚举
   - 实现 From trait 用于错误转换
   - 实现 IntoResponse trait 用于 HTTP 响应
   - 定义 Result 类型别名

#### 3.1.6 主应用框架
1. 创建 `src/main.rs`
   - 初始化日志系统
   - 加载配置
   - 连接数据库
   - 运行迁移
   - 设置 Axum 路由
   - 启动 HTTP 服务器

**交付物**:
- ✅ 项目结构完整
- ✅ 数据库迁移文件
- ✅ 配置管理系统
- ✅ 数据模型定义
- ✅ 错误处理框架
- ✅ 应用可以启动（无功能）

### 3.2 Phase 2: 核心功能实现（预计 3-4 天）

#### 3.2.1 数据访问层（Repository）
1. 创建 `src/repositories/ticket_repository.rs`
   - `find_all(query)` - 支持标签、搜索、完成状态筛选
   - `find_by_id(id)` - 获取单个 Ticket（包含标签）
   - `create(data)` - 创建 Ticket（支持关联标签）
   - `update(id, data)` - 更新 Ticket（显式更新 updated_at）
   - `delete(id)` - 删除 Ticket
   - `toggle_completed(id)` - 切换完成状态
   - `add_tag(ticket_id, tag_id)` - 添加标签关联
   - `remove_tag(ticket_id, tag_id)` - 移除标签关联

2. 创建 `src/repositories/tag_repository.rs`
   - `find_all()` - 获取所有标签
   - `find_by_id(id)` - 获取单个标签
   - `find_by_name(name)` - 按名称查找（用于唯一性检查）
   - `create(data)` - 创建标签
   - `update(id, data)` - 更新标签
   - `delete(id)` - 删除标签

3. 创建 `src/repositories/mod.rs`
   - 导出所有 Repository
   - 创建 Repository 管理器（统一管理数据库连接）

**技术要点**:
- 使用 SQLx 进行类型安全的数据库查询
- 使用事务处理 Ticket 创建时的标签关联
- 使用动态 SQL 构建灵活的查询条件
- 使用 GIN 索引优化搜索性能

#### 3.2.2 API 处理器（Handlers）
1. 创建 `src/handlers/ticket_handler.rs`
   - `get_tickets` - GET /api/tickets
   - `get_ticket` - GET /api/tickets/:id
   - `create_ticket` - POST /api/tickets
   - `update_ticket` - PUT /api/tickets/:id
   - `delete_ticket` - DELETE /api/tickets/:id
   - `toggle_ticket_completed` - PATCH /api/tickets/:id/toggle
   - `add_tag_to_ticket` - POST /api/tickets/:ticket_id/tags/:tag_id
   - `remove_tag_from_ticket` - DELETE /api/tickets/:ticket_id/tags/:tag_id

2. 创建 `src/handlers/tag_handler.rs`
   - `get_tags` - GET /api/tags
   - `get_tag` - GET /api/tags/:id
   - `create_tag` - POST /api/tags
   - `update_tag` - PUT /api/tags/:id
   - `delete_tag` - DELETE /api/tags/:id

3. 创建 `src/handlers/mod.rs`
   - 导出所有处理器

**技术要点**:
- 使用 Axum 的 extractors 提取路径参数和查询参数
- 使用 JSON extractor 处理请求体
- 统一错误处理，返回标准错误响应
- 使用状态管理传递 Repository 实例

#### 3.2.3 路由配置
1. 创建 `src/routes/mod.rs`
   - 定义所有 API 路由
   - 配置中间件（CORS、日志等）
   - 集成错误处理

**路由结构**:
```
/api/tickets
  GET    /api/tickets              # 获取列表
  POST   /api/tickets              # 创建
  GET    /api/tickets/:id          # 获取单个
  PUT    /api/tickets/:id          # 更新
  DELETE /api/tickets/:id          # 删除
  PATCH  /api/tickets/:id/toggle   # 切换完成状态
  POST   /api/tickets/:ticket_id/tags/:tag_id    # 添加标签
  DELETE /api/tickets/:ticket_id/tags/:tag_id    # 移除标签

/api/tags
  GET    /api/tags                 # 获取列表
  POST   /api/tags                 # 创建
  GET    /api/tags/:id             # 获取单个
  PUT    /api/tags/:id             # 更新
  DELETE /api/tags/:id             # 删除
```

#### 3.2.4 时间戳管理
1. 在 Repository 层实现时间戳更新
   - 创建时：使用 `chrono::Utc::now()` 设置 created_at 和 updated_at
   - 更新时：显式更新 updated_at 字段
   - 切换完成状态时：更新 updated_at 字段

**交付物**:
- ✅ 完整的 Ticket CRUD 功能
- ✅ 完整的 Tag CRUD 功能
- ✅ Ticket-Tag 关联管理
- ✅ 查询和筛选功能
- ✅ 所有 API 端点实现
- ✅ 错误处理完善

### 3.3 Phase 3: 测试与优化（预计 1-2 天）

#### 3.3.1 API 测试
1. 创建集成测试
   - 使用 REST Client 或 Postman 测试所有端点
   - 测试正常流程
   - 测试错误情况
   - 测试边界条件

2. 创建测试数据
   - 编写 seed.sql 文件
   - 包含示例 Ticket 和 Tag 数据

#### 3.3.2 性能优化
1. 数据库查询优化
   - 检查慢查询
   - 优化索引使用
   - 优化 JOIN 查询

2. 代码优化
   - 代码审查
   - 重构重复代码
   - 添加必要的注释

**交付物**:
- ✅ API 测试文档
- ✅ 性能测试报告
- ✅ 代码优化完成

## 4. 前端实现计划

### 4.1 Phase 1: 基础架构搭建（预计 2-3 天）

#### 4.1.1 项目初始化
1. 创建 React + TypeScript 项目
   ```bash
   cd project-alpha
   npm create vite@latest frontend -- --template react-ts
   cd frontend
   ```

2. 安装依赖
   ```bash
   # UI 框架和样式
   npm install react react-dom
   npm install -D tailwindcss postcss autoprefixer
   npm install -D @types/react @types/react-dom
   
   # UI 组件库
   npm install class-variance-authority clsx tailwind-merge
   npm install lucide-react @radix-ui/react-dialog @radix-ui/react-dropdown-menu @radix-ui/react-select @radix-ui/react-toast
   
   # 状态管理
   npm install zustand
   
   # 路由
   npm install react-router-dom
   npm install -D @types/react-router-dom
   
   # HTTP 客户端
   npm install axios
   
   # 表单处理
   npm install react-hook-form @hookform/resolvers zod
   
   # 工具库
   npm install date-fns
   ```

3. 配置 Tailwind CSS
   ```bash
   npx tailwindcss init -p
   ```

4. 更新 `tailwind.config.js`
   - 配置内容路径
   - 配置自定义颜色系统
   - 配置主题扩展

5. 更新 `src/index.css`
   - 添加 Tailwind 指令
   - 配置 CSS 变量（支持深色模式）

#### 4.1.2 项目结构
```bash
mkdir -p src/{components,pages,hooks,services,stores,types,utils,lib}
```

#### 4.1.3 类型定义
1. 创建 `src/types/index.ts`
   - Ticket 接口
   - Tag 接口
   - 请求/响应类型
   - 查询参数类型

#### 4.1.4 API 客户端
1. 创建 `src/services/api.ts`
   - 配置 Axios 实例
   - Ticket API 方法
   - Tag API 方法
   - 错误处理拦截器

#### 4.1.5 状态管理
1. 创建 `src/stores/ticketStore.ts`
   - Tickets 状态
   - Loading 状态
   - Error 状态
   - CRUD 操作
   - 筛选和搜索

2. 创建 `src/stores/tagStore.ts`
   - Tags 状态
   - Loading 状态
   - Error 状态
   - CRUD 操作

3. 创建 `src/stores/uiStore.ts`
   - UI 状态（侧边栏、对话框等）
   - 筛选状态
   - 搜索状态

4. 创建 `src/stores/index.ts`
   - 导出所有 stores

#### 4.1.6 工具函数
1. 创建 `src/lib/utils.ts`
   - cn 函数（合并 Tailwind 类名）

**交付物**:
- ✅ 项目结构完整
- ✅ 依赖安装完成
- ✅ Tailwind CSS 配置完成
- ✅ 类型定义完成
- ✅ API 客户端完成
- ✅ 状态管理框架完成

### 4.2 Phase 2: 组件开发（预计 4-5 天）

#### 4.2.1 基础 UI 组件
1. 创建 `src/lib/utils.ts`
   - cn 函数（已创建）

2. 创建 `src/components/ui/button.tsx`
   - 按钮组件（多种变体）
   - 使用 class-variance-authority

3. 创建 `src/components/ui/input.tsx`
   - 输入框组件

4. 创建 `src/components/ui/card.tsx`
   - 卡片组件

5. 创建 `src/components/ui/label.tsx`
   - 标签组件

6. 创建 `src/components/ui/select.tsx`
   - 选择器组件（基于 Radix UI）

7. 创建 `src/components/ui/dialog.tsx`
   - 对话框组件（基于 Radix UI）

8. 创建 `src/components/ui/toast.tsx`
   - Toast 通知组件（基于 Radix UI）

9. 创建 `src/components/ui/badge.tsx`
   - 徽章组件（用于标签显示）

10. 创建 `src/components/ui/checkbox.tsx`
    - 复选框组件

#### 4.2.2 业务组件
1. 创建 `src/components/TagBadge.tsx`
   - 标签徽章组件
   - 显示标签名称和颜色

2. 创建 `src/components/TicketCard.tsx`
   - Ticket 卡片组件
   - 显示标题、描述、标签、完成状态
   - 操作按钮（编辑、删除、切换完成状态）

3. 创建 `src/components/TicketForm.tsx`
   - Ticket 表单组件
   - 使用 React Hook Form + Zod 验证
   - 支持创建和编辑模式

4. 创建 `src/components/TagSelector.tsx`
   - 标签选择器组件
   - 多选支持
   - 显示标签颜色

5. 创建 `src/components/SearchBar.tsx`
   - 搜索栏组件
   - 防抖处理（300ms）

6. 创建 `src/components/FilterPanel.tsx`
   - 筛选面板组件
   - 完成状态筛选
   - 标签筛选

7. 创建 `src/components/TagManager.tsx`
   - 标签管理器组件
   - 标签列表
   - 创建/编辑/删除标签

8. 创建 `src/components/Sidebar.tsx`
   - 侧边栏组件
   - 标签列表
   - 创建标签按钮

**交付物**:
- ✅ 所有基础 UI 组件
- ✅ 所有业务组件
- ✅ 组件样式统一
- ✅ 组件可复用

### 4.3 Phase 3: 页面开发（预计 2-3 天）

#### 4.3.1 路由配置
1. 创建 `src/App.tsx`
   - 配置 React Router
   - 定义路由结构

2. 创建路由结构
   ```
   /                    # 主页面（Ticket 列表）
   /tickets/:id         # Ticket 详情页（可选）
   ```

#### 4.3.2 主页面
1. 创建 `src/pages/TicketsPage.tsx`
   - 布局：侧边栏 + 主内容区
   - 集成 SearchBar
   - 集成 FilterPanel
   - 集成 TicketCard 列表
   - 集成 TicketForm（对话框）
   - 集成 TagManager（对话框）

2. 功能实现
   - 加载 Ticket 列表
   - 搜索功能
   - 筛选功能
   - 创建 Ticket
   - 编辑 Ticket
   - 删除 Ticket
   - 切换完成状态
   - 管理标签关联

#### 4.3.3 状态同步
1. 实现 Store 与组件的集成
   - 使用 Zustand hooks
   - 响应式更新
   - 错误处理
   - 加载状态

2. 实现筛选和搜索
   - 实时搜索（防抖）
   - 筛选状态管理
   - URL 参数同步（可选）

**交付物**:
- ✅ 主页面完成
- ✅ 所有功能集成
- ✅ 路由配置完成
- ✅ 状态管理完善

### 4.4 Phase 4: 优化与完善（预计 1-2 天）

#### 4.4.1 用户体验优化
1. 加载状态
   - 骨架屏
   - 加载指示器

2. 错误处理
   - 友好的错误提示
   - Toast 通知

3. 成功反馈
   - 操作成功提示
   - 自动刷新列表

#### 4.4.2 响应式设计
1. 移动端适配
   - 响应式布局
   - 移动端菜单

2. 侧边栏优化
   - 可折叠
   - 移动端自动隐藏

#### 4.4.3 性能优化
1. 代码分割
   - 路由级别的代码分割
   - 组件懒加载

2. 状态优化
   - 避免不必要的重渲染
   - 使用 React.memo

**交付物**:
- ✅ 用户体验优化完成
- ✅ 响应式设计完成
- ✅ 性能优化完成

## 5. 测试计划

### 5.1 后端测试
1. **单元测试**
   - Repository 层测试
   - 工具函数测试

2. **集成测试**
   - API 端点测试
   - 数据库操作测试

3. **端到端测试**
   - 完整业务流程测试

### 5.2 前端测试
1. **组件测试**
   - 使用 React Testing Library
   - 测试用户交互

2. **集成测试**
   - 页面功能测试
   - Store 集成测试

3. **E2E 测试**（可选）
   - 使用 Playwright 或 Cypress
   - 完整用户流程测试

## 6. 部署计划

### 6.1 开发环境
1. **后端**
   - 本地运行 PostgreSQL
   - 使用 `cargo run` 启动服务器
   - 端口：3000

2. **前端**
   - 使用 `npm run dev` 启动开发服务器
   - 端口：5173（Vite 默认）

### 6.2 生产环境
1. **后端部署**
   - 编译为可执行文件
   - 使用 systemd 或 Docker 管理
   - 配置环境变量
   - 配置数据库连接池

2. **前端部署**
   - 构建生产版本：`npm run build`
   - 部署到静态文件服务器（Nginx、Vercel、Netlify 等）
   - 配置 API 代理

3. **数据库**
   - 使用 PostgreSQL 16+
   - 配置备份策略
   - 配置连接池

## 7. 开发时间估算

### 7.1 后端开发
- Phase 1: 基础架构搭建 - 2-3 天
- Phase 2: 核心功能实现 - 3-4 天
- Phase 3: 测试与优化 - 1-2 天
- **总计**: 6-9 天

### 7.2 前端开发
- Phase 1: 基础架构搭建 - 2-3 天
- Phase 2: 组件开发 - 4-5 天
- Phase 3: 页面开发 - 2-3 天
- Phase 4: 优化与完善 - 1-2 天
- **总计**: 9-13 天

### 7.3 总体时间
- **后端**: 6-9 天
- **前端**: 9-13 天
- **测试**: 2-3 天
- **部署**: 1-2 天
- **总计**: 18-27 天（约 3-4 周）

## 8. 开发顺序建议

### 8.1 推荐顺序
1. **第一阶段**: 后端 Phase 1 + Phase 2（完成所有 API）
2. **第二阶段**: 前端 Phase 1（搭建基础架构）
3. **第三阶段**: 前端 Phase 2（开发组件）
4. **第四阶段**: 前端 Phase 3（开发页面，集成后端 API）
5. **第五阶段**: 测试和优化
6. **第六阶段**: 部署

### 8.2 并行开发
- 后端 Phase 1 完成后，可以开始前端 Phase 1
- 后端 API 完成后，前端可以开始集成测试

## 9. 关键里程碑

### 9.1 Milestone 1: 后端 API 完成
- ✅ 所有 API 端点实现
- ✅ 数据库迁移完成
- ✅ API 测试通过

### 9.2 Milestone 2: 前端基础完成
- ✅ 项目结构搭建
- ✅ 状态管理完成
- ✅ API 客户端完成

### 9.3 Milestone 3: UI 组件完成
- ✅ 所有基础组件完成
- ✅ 所有业务组件完成

### 9.4 Milestone 4: 功能集成完成
- ✅ 主页面完成
- ✅ 所有功能可用
- ✅ 前后端联调通过

### 9.5 Milestone 5: 项目完成
- ✅ 测试通过
- ✅ 性能优化完成
- ✅ 部署完成

## 10. 风险与应对

### 10.1 技术风险
1. **数据库性能问题**
   - 风险：大量数据时查询变慢
   - 应对：优化索引，使用分页，添加缓存

2. **前端状态管理复杂**
   - 风险：状态管理混乱
   - 应对：使用 Zustand，清晰的 Store 结构

3. **API 集成问题**
   - 风险：前后端接口不一致
   - 应对：使用 TypeScript 类型，API 文档

### 10.2 时间风险
1. **开发时间超期**
   - 风险：功能复杂导致延期
   - 应对：分阶段交付，优先核心功能

2. **测试时间不足**
   - 风险：测试不充分
   - 应对：边开发边测试，自动化测试

## 11. 质量保证

### 11.1 代码质量
- 使用 Rust Clippy 检查代码
- 使用 ESLint 检查前端代码
- 代码审查
- 统一代码风格

### 11.2 文档质量
- API 文档（使用 OpenAPI/Swagger，可选）
- 代码注释
- README 文档
- 部署文档

### 11.3 测试覆盖
- 单元测试覆盖率 > 70%
- 集成测试覆盖主要流程
- E2E 测试覆盖关键路径

## 12. 后续优化（可选）

### 12.1 功能扩展
- 用户认证系统
- Ticket 优先级
- Ticket 截止日期
- 数据导出

### 12.2 技术优化
- 前端状态持久化
- 离线支持（PWA）
- 实时更新（WebSocket）
- 移动端应用

## 13. 总结

本实现计划将项目分为多个阶段，每个阶段都有明确的交付物和目标。建议按照计划逐步实施，确保每个阶段的质量，最终交付一个功能完整、性能良好的 Ticket 管理系统。

**关键成功因素**:
1. 严格按照需求文档实现
2. 每个阶段完成后进行评审
3. 及时沟通和调整计划
4. 注重代码质量和测试
5. 保持文档更新
