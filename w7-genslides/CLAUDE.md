# GenSlides — CLAUDE.md

## 项目概述

GenSlides 是一个本地运行的 AI 图片幻灯片生成工具。用户输入文字内容，使用 Google Nano Banana Pro 模型生成视觉风格统一的图片，以走马灯形式全屏播放。

技术栈：TypeScript 6 + Next.js 16 (App Router) + Tailwind CSS v4 + Zustand 5 + @google/genai

## 设计文档

- PRD: `specs/w7/001-genslides-prd.md`
- 设计规格: `specs/w7/002-genslides-design.md`

## 目录结构与职责边界

```
src/
├── app/                    # Next.js App Router（仅 HTTP 层）
│   ├── api/
│   │   └── presentations/  # 所有 API 统一嵌套在 /api/presentations 下
│   │       ├── route.ts    # GET 列表 / POST 新建
│   │       └── [slug]/     # 单个演示文稿及其子资源
│   │           ├── slides/
│   │           ├── generate/
│   │           └── images/
│   ├── [slug]/             # 编辑页（URL: /my-talk）
│   └── page.tsx            # 首页（演示文稿列表）
├── components/             # React UI 组件（纯展示 + Zustand store 调用）
├── lib/                    # 后端业务逻辑（不依赖 Next.js）
│   ├── ai/                 # AI 模型调用封装
│   └── services/           # 业务逻辑编排
├── storage/                # 存储层（仅文件 I/O）
└── types/                  # 共享类型定义
```

### 严格分层规则

- **API 层**（`app/api/`）：只做 HTTP 解析和响应。不包含业务逻辑。调用 service 层。
- **Service 层**（`lib/services/`）：业务逻辑编排。可调用 storage 层和 ai 层。不依赖 Next.js。
- **Storage 层**（`storage/`）：纯文件 I/O 操作。不包含业务逻辑。不依赖其他业务模块。
- **AI 层**（`lib/ai/`）：封装 @google/genai 调用。不包含业务逻辑。
- **Components**：通过 Zustand store 与后端交互，不直接调用 fetch。UI 状态由 store 管理。

依赖方向：API → Service → { Storage, AI }。禁止反向依赖和跨层调用。

## TypeScript 最佳实践

- **严格模式**：`tsconfig.json` 开启 `strict: true`，禁止隐式 any
- **接口优先**：所有数据结构用 `interface` 定义，放在 `src/types/` 下
- **枚举用 const**：用 `as const` 对象代替 `enum`
- **错误类型**：自定义 `AppError` 类继承 `Error`，携带 `code` 和 `statusCode`
- **空值处理**：使用 optional chaining 和 nullish coalescing，避免 `!` 非空断言
- **类型推导**：优先依赖类型推导，仅在必要时显式标注（函数参数、返回值）
- **避免类型断言**：用 type guard 代替 `as` 断言

## Next.js 16 最佳实践

- **Server Components 默认**：所有组件默认为 Server Component，仅交互组件加 `"use client"`
- **`"use client"` 最小化**：只在需要 hooks、事件处理、浏览器 API 的组件上加 `"use client"`
- **API Route 结构**：
  - 每个 route.ts 导出标准 HTTP 方法函数（`GET`, `POST`, `PUT`, `DELETE`）
  - 使用 `NextRequest` / `NextResponse` 类型
  - 参数校验使用 TypeScript 类型守卫，不引入额外验证库
- **数据获取**：Server Component 中直接调用 service 层，不走 API
- **环境变量**：敏感变量只通过 `process.env` 在 server 端访问，不暴露到客户端
- **动态路由**：使用 `params` promise 获取路由参数（Next.js 16 async params）

## Tailwind CSS v4 最佳实践

- **CSS-first 配置**：在 `app.css` 中使用 `@theme` 定义设计令牌，不使用 `tailwind.config.ts`
- **不自定义 CSS 类**：直接在 JSX 中使用 Tailwind 工具类，不创建 `@apply` 封装
- **响应式**：使用 `sm:`, `md:`, `lg:` 前缀处理不同屏幕
- **暗色模式**：使用 `dark:` 前缀（如需要）
- **组件变体**：使用条件类名拼接（`className` 条件表达式），不引入 cva 等库

## 架构原则

### SOLID

- **单一职责**：每个文件一个类/函数/组件。service 按领域拆分（presentation、slide、generate、style）
- **开闭原则**：AI 层通过接口抽象，替换模型不需要修改 service 层
- **里氏替换**：storage 层的 repo 可被 mock 替换用于测试
- **接口隔离**：API 的请求/响应类型按接口定义，不使用大而全的通用类型
- **依赖反转**：service 通过构造函数接收 repo 和 generator 实例（DI）

### YAGNI

- 不实现 PRD 中 Non-Goals 列出的功能
- Phase 1 只实现 MVP，不提前搭建 Phase 2/3 的框架
- 不引入不需要的库：不使用 form 库、UI 组件库、ORM 等

### KISS

- 状态管理仅使用 Zustand，不引入 Redux 或 Context 复合模式
- 数据存储使用 YAML 文件，不引入数据库
- API 使用 REST，不引入 GraphQL 或 tRPC
- 文件命名直接反映功能，不使用抽象的通用名

## 并发处理

- **批量图片生成**：使用 `Promise.all` 并行调用 AI 生成接口，第一个 slide 串行生成作为 base image
- **竞态保护**：前端 Zustand store 中维护 `generatingSids` Set，防止同一 slide 重复触发生成
- **文件写入**：使用原子写入（先写临时文件再 rename），避免并发写入导致文件损坏
- **SSE 流式响应**：批量生成使用 Server-Sent Events 逐个推送完成结果，不等待全部完成
- **文件锁**：outline.yaml 读写通过内存队列串行化，避免并发读写冲突

## 错误处理

### 后端错误处理

- **自定义错误类**：
  ```
  class AppError extends Error {
    constructor(public code: string, public statusCode: number, message: string)
  }
  ```
- **Service 层**：抛出 `AppError`，携带业务语义的 code（如 `SLIDE_NOT_FOUND`、`GENERATION_FAILED`）
- **API 层**：统一 try-catch，将 `AppError` 转换为标准 JSON 错误响应
- **AI 调用**：捕获 SDK 错误，包装为 `AppError` 再向上传播
- **文件操作**：使用 `try-catch` 处理文件不存在、权限错误等

### 前端错误处理

- **API 调用失败**：Zustand store 中维护 `error` 状态，UI 层展示 toast 通知
- **生成失败**：在 slide 上标记 error 状态，提供重试按钮
- **全局错误边界**：Next.js `error.tsx` 捕获渲染错误

### 日志处理

- **后端日志**：使用 `console.log` / `console.error`，结构化输出 JSON 格式
  - 请求日志：`[API] POST /api/[slug]/generate/[sid]` + 耗时
  - 业务日志：`[Service] Generating image for slide: slide1` + 关键参数
  - 错误日志：`[Error] code=GENERATION_FAILED slug=my-talk sid=slide1` + stack trace
- **前端日志**：开发环境使用 `console.log`，生产环境不输出调试日志
- **不引入日志框架**：项目规模小，`console` 足够

## 代码风格

- **不写注释**：通过清晰的命名表达意图，只在 WHY 不明显时加一行注释
- **不写多行 docstring**
- **文件组织**：同一文件的代码按使用顺序排列，被调用的函数在调用者之前
- **命名约定**：
  - 组件文件：PascalCase（`SlideCard.tsx`）
  - 工具/服务文件：kebab-case（`slide-service.ts`）
  - 类型文件：kebab-case（`slide.ts`）
  - CSS 类：Tailwind 工具类直接写在 JSX 中
  - 环境变量：UPPER_SNAKE_CASE（`GOOGLE_AI_API_KEY`）
  - API 路由参数：kebab-case（`slug`, `sid`）
