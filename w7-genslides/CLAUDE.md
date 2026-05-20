# GenSlides — CLAUDE.md

## 项目概述

GenSlides 是一个本地运行的 AI 图片幻灯片生成工具。用户输入文字内容，使用 Google Nano Banana Pro 模型生成视觉风格统一的图片，以走马灯形式全屏播放。

**架构：前后端分离**
- **前端**：Vue 3.5 + Vite 8 + Pinia 3（SPA，纯静态部署）
- **后端**：Next.js 16.2（App Router，API Routes）
- **共享**：TypeScript 类型定义包

**技术栈**：TypeScript + Vue 3.5 + Tailwind CSS 4.3 + Next.js 16.2 + @google/genai 2.4

## 设计文档

- PRD: `specs/w7/001-genslides-prd.md`
- 设计规格: `specs/w7/002-genslides-design.md`

## 目录结构与职责边界

```
w7-genslides/
├── frontend/                    # 前端项目（Vue 3 + Vite）
│   ├── src/
│   │   ├── views/              # 页面组件
│   │   ├── components/         # UI 组件
│   │   ├── stores/             # Pinia 状态管理
│   │   ├── composables/        # 组合式函数
│   │   └── types/              # 类型定义
│   └── vite.config.ts
│
├── backend/                     # 后端项目（Next.js App Router）
│   ├── app/
│   │   └── api/
│   │       └── ppt/            # 所有 API 统一在 /api/ppt 下
│   │           ├── route.ts    # GET 列表 / POST 新建
│   │           └── [slug]/     # 单个演示文稿及其子资源
│   ├── src/
│   │   ├── lib/
│   │   │   ├── services/       # 业务逻辑
│   │   │   ├── storage/        # 文件存储
│   │   │   └── ai/             # AI 模型调用
│   │   ├── middleware/          # 中间件
│   │   └── types/              # 类型定义
│   └── genslides/              # 本地数据目录
│
└── shared/                      # 共享类型包
    └── src/types/
```

### 严格分层规则

- **前端 Views**：页面级组件，组合子组件，调用 stores
- **前端 Components**：UI 组件，纯展示逻辑，通过 stores 与后端交互
- **前端 Stores**：全局状态管理（Pinia），调用 composables
- **前端 Composables**：可复用逻辑（API 调用、SSE），不包含业务逻辑
- **后端 Routes**：HTTP 请求解析、参数校验、响应格式化
- **后端 Services**：业务逻辑编排，可调用 storage 和 ai
- **后端 Storage**：纯文件 I/O 操作，不包含业务逻辑
- **后端 AI**：封装 `@google/genai` v2.4.0 调用，不包含业务逻辑
  - SDK: https://github.com/googleapis/js-genai
  - 文档: https://ai.google.dev/gemini-api/docs

依赖方向：Views → Components → Stores → Composables → Backend API
后端：Routes → Services → { Storage, AI }
禁止反向依赖和跨层调用。

## TypeScript 最佳实践

### 通用

- **严格模式**：`tsconfig.json` 开启 `strict: true`，禁止隐式 any
- **接口优先**：所有数据结构用 `interface` 定义
- **枚举用 const**：用 `as const` 对象代替 `enum`
- **空值处理**：使用 optional chaining 和 nullish coalescing，避免 `!` 非空断言
- **类型推导**：优先依赖类型推导，仅在必要时显式标注
- **避免类型断言**：用 type guard 代替 `as` 断言

### 前端（Vue 3）

- **`<script setup>` 语法糖**：使用 Composition API + `<script setup>` 简洁语法
- **ref vs reactive**：基本类型用 `ref`，对象用 `reactive`
- **computed 缓存**：派生状态必须用 `computed`，不要手动计算
- **props 类型**：使用 `defineProps<{ ... }>()` 类型声明
- **emit 类型**：使用 `defineEmits<{ ... }>()` 类型声明
- **模板引用**：使用 `ref()` 获取 DOM 元素引用

### 后端（Express）

- **路由分离**：每个路由模块独立文件，使用 `Router()` 导出
- **中间件**：错误处理、日志等中间件独立文件
- **异步处理**：使用 async/await，不要混用 Promise 和回调

## Vue 3 最佳实践

- **组合式函数**：将可复用逻辑抽取到 `composables/` 目录
- **状态管理**：使用 Pinia，按领域拆分 stores
- **组件通信**：props down, events up，避免 prop drilling
- **列表渲染**：使用 `v-for` 时必须提供 `:key`
- **条件渲染**：优先使用 `v-if`，频繁切换用 `v-show`
- **生命周期**：使用 `onMounted`、`onUnmounted` 等组合式 API
- **事件处理**：使用 `@event` 语法，内联箭头函数
- **样式绑定**：使用 `:class` 和 `:style` 动态绑定

## Tailwind CSS v4 最佳实践

- **CSS-first 配置**：在 `style.css` 中使用 `@theme` 定义设计令牌
- **不自定义 CSS 类**：直接在模板中使用 Tailwind 工具类
- **响应式**：使用 `sm:`, `md:`, `lg:` 前缀处理不同屏幕
- **暗色模式**：使用 `dark:` 前缀（如需要）
- **组件变体**：使用条件类名拼接（`:class` 条件表达式）

## 架构原则

### SOLID

- **单一职责**：每个文件一个类/函数/组件。service 按领域拆分
- **开闭原则**：AI 层通过接口抽象，替换模型不需要修改 service 层
- **里氏替换**：storage 层的 repo 可被 mock 替换用于测试
- **接口隔离**：API 的请求/响应类型按接口定义，不使用大而全的通用类型
- **依赖反转**：service 通过构造函数接收 repo 和 generator 实例（DI）

### YAGNI

- 不实现 PRD 中 Non-Goals 列出的功能
- Phase 1 只实现 MVP，不提前搭建 Phase 2/3 的框架
- 不引入不需要的库：不使用 form 库、UI 组件库、ORM 等

### KISS

- 状态管理仅使用 Pinia，不引入 Redux 或 Context 复合模式
- 数据存储使用 YAML 文件，不引入数据库
- API 使用 REST，不引入 GraphQL 或 tRPC
- 文件命名直接反映功能，不使用抽象的通用名

## 代码组织

### 文件命名

- **组件文件**：PascalCase（`SlideCard.vue`）
- **工具/服务文件**：kebab-case（`slide-service.ts`）
- **类型文件**：kebab-case（`slide.ts`）
- **CSS 类**：Tailwind 工具类直接写在模板中
- **环境变量**：UPPER_SNAKE_CASE（`GOOGLE_AI_API_KEY`）
- **API 路由参数**：kebab-case（`slug`, `sid`）

### 文件组织

- 同一文件的代码按使用顺序排列，被调用的函数在调用者之前
- 组件内部：props → composables → computed → methods → lifecycle
- 服务内部：构造函数 → 公共方法 → 私有方法

### 类型定义

- 共享类型放在 `shared/src/types/`
- 前端特有类型放在 `frontend/src/types/`
- 后端特有类型放在 `backend/src/types/`
- 使用 `import type` 导入类型

## 并发处理

### 后端

- **批量图片生成**：使用 `Promise.all` 并行调用 AI 生成接口，第一个 slide 串行生成作为 base image
- **文件写入**：使用原子写入（先写临时文件再 rename），避免并发写入导致文件损坏
- **文件锁**：outline.yaml 读写通过内存队列串行化，避免并发读写冲突

### 前端

- **竞态保护**：Pinia store 中维护 `generatingSids` Set，防止同一 slide 重复触发生成
- **SSE 流式响应**：批量生成使用 Server-Sent Events 逐个推送完成结果，不等待全部完成
- **请求取消**：使用 AbortController 取消未完成的请求

## 错误处理

### 后端错误处理

- **自定义错误类**：
  ```typescript
  class AppError extends Error {
    constructor(
      public code: string,
      public statusCode: number,
      message: string
    ) {
      super(message);
      this.name = 'AppError';
    }
  }
  ```
- **Service 层**：抛出 `AppError`，携带业务语义的 code（如 `SLIDE_NOT_FOUND`、`GENERATION_FAILED`）
- **Route 层**：统一 try-catch，将 `AppError` 转换为标准 JSON 错误响应
- **AI 调用**：捕获 SDK 错误，包装为 `AppError` 再向上传播
- **文件操作**：使用 `try-catch` 处理文件不存在、权限错误等

### 前端错误处理

- **API 调用失败**：Pinia store 中维护 `error` 状态，UI 层展示 toast 通知
- **生成失败**：在 slide 上标记 error 状态，提供重试按钮
- **全局错误**：Vue `app.config.errorHandler` 捕获渲染错误

## 日志处理

### 后端日志

- 使用 `console.log` / `console.error`，结构化输出 JSON 格式
- 请求日志：`[API] POST /api/ppt/:slug/generate/:sid` + 耗时
- 业务日志：`[Service] Generating image for slide: slide1` + 关键参数
- 错误日志：`[Error] code=GENERATION_FAILED slug=my-talk sid=slide1` + stack trace

### 前端日志

- 开发环境使用 `console.log`
- 生产环境不输出调试日志
- 不引入日志框架，项目规模小，`console` 足够

## 代码风格

- **不写注释**：通过清晰的命名表达意图，只在 WHY 不明显时加一行注释
- **不写多行 docstring**
- **不写重复代码**：抽取到 composable 或 service
- **不提前优化**：先让代码工作，再优化性能
- **不做防御性编程**：信任内部调用，只在系统边界做校验
