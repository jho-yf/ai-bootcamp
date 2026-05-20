# GenSlides 设计规格

## 1. 架构概览

采用**前后端分离**架构：

- **前端**：Vue 3.5 + Vite 8 + Pinia 3（SPA，纯静态部署）
- **后端**：Next.js 16.2（App Router，API Routes）
- **共享**：TypeScript 类型定义包

**最新依赖版本：**
- `next`: 16.2.6
- `vue`: 3.5.34
- `vite`: 8.0.13
- `pinia`: 3.0.4
- `@google/genai`: 2.4.0
- `tailwindcss`: 4.3.0

### 分层架构

```
┌─────────────────────────────────────────────────────────────┐
│                      前端 (Vue 3)                           │
│  Views → Components → Stores (Pinia) → Composables (API)   │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ HTTP/SSE
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   后端 (Next.js App Router)                  │
│  API Routes → Services → { Storage, AI }                   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      数据存储                               │
│  ./genslides/<slug>/outline.yaml + images/                 │
└─────────────────────────────────────────────────────────────┘
```

### 依赖方向

- 前端 → 后端 API（HTTP/SSE）
- 后端 Routes → Services → { Storage, AI }
- 禁止反向依赖和跨层调用

---

## 2. 目录结构

```
genslides/
├── frontend/                          # 前端项目（Vue 3 + Vite）
│   ├── src/
│   │   ├── views/                     # 页面组件
│   │   ├── components/                # UI 组件
│   │   │   ├── layout/               # 布局组件
│   │   │   ├── preview/              # 预览组件
│   │   │   ├── editor/               # 编辑组件
│   │   │   ├── style/                # 风格组件
│   │   │   └── common/               # 通用组件
│   │   ├── stores/                    # Pinia 状态管理
│   │   ├── composables/               # 组合式函数
│   │   └── types/                     # 类型定义
│   └── vite.config.ts
│
├── backend/                           # 后端项目（Next.js App Router）
│   ├── app/
│   │   └── api/
│   │       └── ppt/
│   │           ├── route.ts           # GET 列表 / POST 新建
│   │           └── [slug]/
│   │               ├── route.ts       # GET 详情 / PUT 更新 / DELETE 删除
│   │               ├── slides/
│   │               │   ├── route.ts   # GET 列表 / POST 新建 / PUT 排序
│   │               │   └── [sid]/
│   │               │       └── route.ts # PUT 更新 / DELETE 删除
│   │               ├── generate/
│   │               │   ├── [sid]/
│   │               │   │   └── route.ts # POST 生成单张（SSE）
│   │               │   ├── batch/
│   │               │   │   └── route.ts # POST 批量生成（SSE）
│   │               │   └── style/
│   │               │       └── route.ts # POST 生成候选 / PUT 选择风格
│   │               └── images/
│   │                   └── [filename]/
│   │                       └── route.ts # GET 图片文件
│   ├── src/
│   │   ├── lib/
│   │   │   ├── services/              # 业务逻辑
│   │   │   ├── storage/               # 文件存储
│   │   │   └── ai/                    # AI 模型调用
│   │   ├── middleware/                 # 中间件
│   │   └── types/                     # 类型定义
│   └── genslides/                     # 本地数据目录
│
└── shared/                            # 共享类型包
    └── src/types/
```

---

## 3. 类型定义

### 3.1 核心类型（shared/src/types/）

```typescript
// 前后端共享的类型定义

interface Slide {
  sid: string;
  content: string;
  images: SlideImage[];
  activeImageIndex: number;
}

interface SlideImage {
  filename: string;     // blake3(prompt).jpg
  textHash: string;     // 生成时的 prompt 哈希
  createdAt: string;
  cost: number;
}

interface Presentation {
  slug: string;
  title: string;
  style: StyleConfig;
  slides: Slide[];
  totalCost: number;
}

interface StyleConfig {
  prompt: string;
  candidates: string[];
  referenceImage: string | null;
}
```

### 3.2 前端类型（frontend/src/types/）

```typescript
interface PresentationSummary {
  slug: string;
  title: string;
  slideCount: number;
  totalCost: number;
  createdAt: string;
  updatedAt: string;
}

interface GenerateProgress {
  status: 'generating' | 'complete' | 'error';
  progress?: number;
  image?: SlideImage;
  error?: string;
}

interface BatchGenerateProgress extends GenerateProgress {
  sid: string;
}
```

### 3.3 后端错误类型（backend/src/types/）

```typescript
class AppError extends Error {
  code: string;
  statusCode: number;
}

const ErrorCodes = {
  SLIDE_NOT_FOUND: 'SLIDE_NOT_FOUND',
  PRESENTATION_NOT_FOUND: 'PRESENTATION_NOT_FOUND',
  GENERATION_FAILED: 'GENERATION_FAILED',
  INVALID_REQUEST: 'INVALID_REQUEST'
} as const;
```

---

## 4. API 接口定义

### 4.1 演示文稿管理

| 方法 | 路径 | 功能 | 请求体 | 响应 |
|------|------|------|--------|------|
| GET | `/api/ppt` | 获取列表 | - | `{ presentations: PresentationSummary[] }` |
| POST | `/api/ppt` | 新建 | `{ slug, title }` | `{ slug, title }` |
| GET | `/api/ppt/:slug` | 获取详情 | - | `Presentation` |
| PUT | `/api/ppt/:slug` | 更新标题 | `{ title }` | `{ slug, title }` |
| DELETE | `/api/ppt/:slug` | 删除 | - | 204 |

### 4.2 Slide 管理

| 方法 | 路径 | 功能 | 请求体 | 响应 |
|------|------|------|--------|------|
| GET | `/api/ppt/:slug/slides` | 获取演示文稿数据 | - | `{ title, style, slides, totalCost }` |
| POST | `/api/ppt/:slug/slides` | 新建 slide | `{ content, index? }` | `{ slide: Slide }` |
| PUT | `/api/ppt/:slug/slides/:sid` | 更新 slide | `{ content?, activeImageIndex? }` | `{ slide, hasMatchingImage }` |
| DELETE | `/api/ppt/:slug/slides/:sid` | 删除 slide | - | 204 |
| PUT | `/api/ppt/:slug/slides` | 批量排序 | `{ orderedSids: string[] }` | `{ slides: Slide[] }` |

### 4.3 图片生成

| 方法 | 路径 | 功能 | 请求体 | 响应 |
|------|------|------|--------|------|
| POST | `/api/ppt/:slug/generate/:sid` | 生成单张 | - | SSE: `progress` → `complete` |
| POST | `/api/ppt/:slug/generate/batch` | 批量生成 | `{ sids: string[] }` | SSE: 多个 `complete` → `done` |
| POST | `/api/ppt/:slug/generate/style` | 生成风格候选 | `{ prompt }` | `{ candidates, prompt }` |
| PUT | `/api/ppt/:slug/generate/style` | 选择风格 | `{ referenceImage }` | `{ style: StyleConfig }` |

### 4.4 图片文件

| 方法 | 路径 | 功能 | 响应 |
|------|------|------|------|
| GET | `/api/ppt/:slug/images/:filename` | 获取图片 | `image/jpeg` 二进制流 |

### 4.5 SSE 事件格式

**单张生成：**
```
event: progress
data: {"status": "generating", "progress": 0.5}

event: complete
data: {"image": {"filename": "...", "textHash": "...", "cost": 0.02}}
```

**批量生成：**
```
event: complete
data: {"sid": "slide1", "image": {"filename": "...", "cost": 0.02}}

event: done
data: {"totalCost": 0.06}
```

---

## 5. 模块职责

### 5.1 后端 API 层（Next.js App Router）

| 模块 | 职责 | 关键方法 |
|------|------|----------|
| `app/api/ppt/route.ts` | 演示文稿列表/新建 | `GET()`, `POST()` |
| `app/api/ppt/[slug]/route.ts` | 演示文稿详情/更新/删除 | `GET()`, `PUT()`, `DELETE()` |
| `app/api/ppt/[slug]/slides/route.ts` | Slide 列表/新建/排序 | `GET()`, `POST()`, `PUT()` |
| `app/api/ppt/[slug]/slides/[sid]/route.ts` | Slide 更新/删除 | `PUT()`, `DELETE()` |
| `app/api/ppt/[slug]/generate/[sid]/route.ts` | 单张生成（SSE） | `POST()` |
| `app/api/ppt/[slug]/generate/batch/route.ts` | 批量生成（SSE） | `POST()` |
| `app/api/ppt/[slug]/generate/style/route.ts` | 风格生成/选择 | `POST()`, `PUT()` |
| `app/api/ppt/[slug]/images/[filename]/route.ts` | 图片文件 | `GET()` |

### 5.2 后端存储层

| 模块 | 职责 | 关键方法 |
|------|------|----------|
| `PresentationRepo` | 演示文稿目录管理 | `list()`, `create()`, `delete()`, `exists()` |
| `OutlineRepo` | outline.yaml 读写 | `read()`, `write()`, `init()` |
| `ImageRepo` | 图片文件读写 | `save(slug, prompt, buffer)`, `read(slug, filename)`, `delete(slug, filename)` |

### 5.2 后端业务层

| 模块 | 职责 | 关键方法 |
|------|------|----------|
| `PresentationService` | 演示文稿 CRUD | `list()`, `create()`, `get()`, `update()`, `delete()` |
| `SlideService` | Slide 业务逻辑 | `createSlide()`, `updateSlide()`, `deleteSlide()`, `reorderSlides()` |
| `GenerateService` | 图片生成业务 | `generateSlideImage()`, `batchGenerate()`, `computePromptHash()` |
| `StyleService` | 风格管理业务 | `generateCandidates()`, `selectStyle()`, `needsGuide()` |

### 5.3 后端 AI 层

| 模块 | 职责 | 关键方法 |
|------|------|----------|
| `ImageGenerator` | AI 模型调用封装 | `generate()`, `generateMultiple()` |

**SDK**: `@google/genai` v2.4.0（官方 Google Generative AI SDK）
**文档**: https://ai.google.dev/gemini-api/docs
**仓库**: https://github.com/googleapis/js-genai

### 5.4 前端状态管理

| Store | 职责 | 关键状态/方法 |
|-------|------|---------------|
| `usePresentationStore` | 演示文稿状态 | `presentation`, `selectedSid`, `loadPresentation()`, `generateImage()` |
| `useUiStore` | UI 状态 | `error`, `success`, `isLoading` |

### 5.5 前端组合式函数

| Composable | 职责 | 关键方法 |
|------------|------|----------|
| `useApi` | API 调用封装 | `get()`, `post()`, `put()`, `del()` |
| `useSse` | SSE 事件处理 | `onMessage()`, `onError()`, `close()` |

---

## 6. 关键交互流程

### 6.1 首次打开 — 风格引导

```
用户打开 /[slug]
    │
    ▼
Vue Router → EditorView.vue
    │
    ▼
loadPresentation(slug) → GET /api/ppt/:slug
    │
    ▼
检查 style.referenceImage === null ?
    ├─ Yes → 打开 StyleGuidePopup
    │          │
    │          ▼ 用户输入 prompt
    │          ▼ POST /generate/style { prompt }
    │          ▼ 展示 4 张候选图
    │          ▼ 用户选择一张
    │          ▼ PUT /generate/style { referenceImage }
    │          ▼ 关闭弹窗
    │
    └─ No → 直接进入主界面
```

### 6.2 编辑文本 → 手动生成

```
用户编辑 slide 文本
    │
    ▼
updateSlideContent(sid, content) → PUT /slides/:sid { content }
    │
    ▼
返回 { slide, hasMatchingImage: false }
    │
    ▼
显示 "生成新图片" 按钮
    │
    ▼
用户点击 → generateImage(sid)
    │
    ▼
POST /generate/:sid (SSE)
    │
    ▼
接收 progress → 更新 loading
    │
    ▼
接收 complete → 更新预览图和缩略图
```

### 6.3 全屏播放

```
用户点击 "播放"
    │
    ▼
startFullscreen() → Fullscreen API
    │
    ▼
启动定时器（默认 5s）
    │
    ▼
每 5s → playingIndex++ → 展示下一张
    │
    ▼
最后一张 → 循环回第一张
    │
    ▼
ESC / 点击 → stopFullscreen()
```

---

## 7. 图片命名与缓存

### 7.1 命名规则

```
<blake3(prompt)>.jpg
```

- 使用 prompt（slide content）的 blake3 哈希作为文件名
- 相同 prompt 生成的图片会覆盖旧文件，天然去重
- 简洁明了，无冗余信息

### 7.2 缓存策略

- 浏览器端：`Cache-Control: max-age=31536000, immutable`
- 全屏播放：预加载相邻 slide 图片

---

## 8. 技术决策

| 决策 | 选择 | 版本 | 理由 |
|------|------|------|------|
| 前端框架 | Vue 3 | 3.5.34 | 轻量、易用、中文文档友好 |
| 构建工具 | Vite | 8.0.13 | 极速 HMR，原生 ESM |
| 状态管理 | Pinia | 3.0.4 | Vue 官方推荐，简洁 API |
| 后端框架 | Next.js | 16.2.6 | App Router、API Routes、TypeScript 优先 |
| UI 样式 | Tailwind CSS | 4.3.0 | CSS-first 配置，原子化样式 |
| AI SDK | @google/genai | 2.4.0 | 官方 SDK，支持 Nano Banana Pro |
| 流式响应 | SSE | - | 轻量级实时推送，无需 WebSocket |
| 数据格式 | YAML | - | 可读性好，适合配置文件 |

---

## 9. 环境变量

### 前端 (.env.development)

```bash
VITE_API_BASE_URL=http://localhost:3000
```

### 后端 (.env.local)

```bash
GOOGLE_AI_API_KEY=your_api_key_here
GOOGLE_AI_MODEL=gemini-3-pro-image-preview
DATA_DIR=./genslides
DEFAULT_PLAY_INTERVAL=5
```
