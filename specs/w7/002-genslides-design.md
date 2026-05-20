# GenSlides 设计规格

## 1. 项目目录结构

```
genslides/
├── package.json
├── next.config.ts                     # Next.js 16 配置
├── tsconfig.json                      # TypeScript 6.0
├── app.css                            # Tailwind CSS v4 入口（CSS-first 配置）
├── .env                               # 环境变量（GOOGLE_AI_API_KEY 等）
├── .env.example                       # 环境变量模板
│
├── genslides/                         # 本地数据存储根目录
│   └── <slug>/
│       ├── outline.yaml
│       └── images/
│           ├── style_candidate_xxx.jpg
│           └── <sid>_<x>_<hash>.jpg
│
├── src/
│   ├── app/                           # Next.js App Router
│   │   ├── layout.tsx                 # 根布局
│   │   ├── page.tsx                   # 主页（演示文稿列表）
│   │   ├── [slug]/
│   │   │   └── page.tsx               # 单个演示文稿编辑页（URL: /my-talk）
│   │   └── api/
│   │       └── presentations/
│   │           ├── route.ts           # GET 列表 / POST 新建
│   │           └── [slug]/
│   │               ├── route.ts       # GET 详情 / PUT 更新 / DELETE 删除
│   │               ├── slides/
│   │               │   ├── route.ts   # GET 列表 / POST 新建 / PUT 排序
│   │               │   └── [sid]/
│   │               │       └── route.ts # PUT 更新 / DELETE 删除
│   │               ├── generate/
│   │               │   ├── [sid]/
│   │               │   │   └── route.ts # POST 生成单张
│   │               │   ├── batch/
│   │               │   │   └── route.ts # POST 批量生成
│   │               │   └── style/
│   │               │       └── route.ts # POST 生成候选 / PUT 选择风格
│   │               └── images/
│   │                   └── [filename]/
│   │                       └── route.ts # GET 图片文件
│   │
│   ├── components/                    # UI 组件
│   │   ├── layout/
│   │   │   ├── AppHeader.tsx          # 顶部 Logo + 标题栏
│   │   │   ├── SlideList.tsx          # 左侧 Slide 列表
│   │   │   └── SlideCard.tsx          # 单个 Slide 卡片
│   │   ├── preview/
│   │   │   ├── ImagePreview.tsx       # 右侧图片预览区
│   │   │   ├── ThumbnailBar.tsx       # 底部缩略图栏
│   │   │   ├── GenerateButton.tsx     # "生成新图片"按钮
│   │   │   └── FullscreenPlayer.tsx   # 全屏走马灯播放
│   │   ├── editor/
│   │   │   └── SlideEditor.tsx        # Slide 文本编辑器
│   │   ├── style/
│   │   │   ├── StyleGuidePopup.tsx    # 风格引导弹窗
│   │   │   └── StyleSelector.tsx      # 4 选 1 风格选择器
│   │   └── common/
│   │       ├── ConfirmDialog.tsx      # 确认对话框
│   │       └── CostDisplay.tsx        # 成本展示组件
│   │
│   ├── lib/                           # 后端业务逻辑（不依赖 Next.js）
│   │   ├── ai/
│   │   │   └── image-generator.ts     # AI 图片生成（封装 @google/genai）
│   │   ├── services/
│   │   │   ├── presentation-service.ts # 演示文稿 CRUD（slug 管理）
│   │   │   ├── slide-service.ts       # Slide 业务逻辑（CRUD、排序）
│   │   │   ├── generate-service.ts    # 图片生成业务（单张、批量、风格）
│   │   │   └── style-service.ts       # 风格管理业务
│   │   └── hash.ts                    # blake3 哈希工具
│   │
│   ├── storage/                       # 存储层（仅负责文件 I/O）
│   │   ├── presentation-repo.ts       # 演示文稿目录管理（list/create/delete slug）
│   │   ├── outline-repo.ts            # outline.yaml 读写
│   │   └── image-repo.ts              # 图片文件读写
│   │
│   └── types/                         # 类型定义
│       ├── slide.ts                   # Slide 相关类型
│       ├── style.ts                   # Style 相关类型
│       └── api.ts                     # API 请求/响应类型
```

### 分层职责

| 层级 | 目录 | 职责 | 依赖 |
|------|------|------|------|
| **API 层** | `src/app/api/` | HTTP 请求解析、参数校验、响应格式化 | services |
| **业务层** | `src/lib/services/` | 业务逻辑编排、规则校验 | storage, ai |
| **存储层** | `src/storage/` | 文件系统读写，不包含业务逻辑 | 无外部依赖 |
| **AI 层** | `src/lib/ai/` | 封装模型调用，处理图片生成 | @google/genai |

### 环境变量

通过 `.env` 文件配置，Next.js 内置 dotenv 支持：

```bash
# .env.example

# Google AI API Key（必需）
GOOGLE_AI_API_KEY=your_api_key_here

# 图片生成模型（可选，默认 gemini-3-pro-image-preview）
# 可选值：gemini-3-pro-image-preview (Nano Banana Pro)、gemini-3.1-flash-image-preview (Nano Banana 2)
GOOGLE_AI_MODEL=gemini-3-pro-image-preview

# 数据存储根目录（可选，默认 ./genslides）
DATA_DIR=./genslides

# 全屏播放默认间隔秒数（可选，默认 5）
DEFAULT_PLAY_INTERVAL=5
```

---

## 2. 类型定义

### 2.1 核心数据类型

```typescript
// src/types/slide.ts

interface Slide {
  sid: string;          // 唯一标识，如 "slide1"
  content: string;      // 文字内容（即 prompt）
  images: SlideImage[]; // 该 slide 关联的所有图片
  activeImageIndex: number; // 当前选中的图片索引
}

interface SlideImage {
  filename: string;     // 文件名：<sid>_<x>_<hash>.jpg
  textHash: string;     // 生成时文字内容的 blake3 哈希
  createdAt: string;    // ISO 时间戳
  cost: number;         // 本次生成成本（USD）
}

interface Presentation {
  slug: string;
  title: string;
  style: StyleConfig;
  slides: Slide[];
  totalCost: number;    // 总生成成本
}

// src/types/style.ts

interface StyleConfig {
  prompt: string;                  // 风格描述文字
  candidates: string[];            // 4 张候选图片路径
  referenceImage: string | null;   // 用户选中的风格图片
}

// src/types/api.ts

// API 请求/响应类型定义见下方各接口
```

---

## 3. API 接口定义

### 3.1 演示文稿管理（Presentation CRUD）

#### `GET /api/presentations` — 获取演示文稿列表

**Response 200:**
```typescript
interface ListPresentationsResponse {
  presentations: PresentationSummary[];
}

interface PresentationSummary {
  slug: string;
  title: string;
  slideCount: number;
  totalCost: number;
  createdAt: string;     // ISO 时间戳
  updatedAt: string;     // ISO 时间戳
}
```

#### `POST /api/presentations` — 新建演示文稿

**Request Body:**
```typescript
interface CreatePresentationRequest {
  slug: string;          // URL 友好的标识符，如 "my-talk"
  title: string;         // 演示文稿标题
}
```

**Response 201:**
```typescript
interface CreatePresentationResponse {
  slug: string;
  title: string;
}
```

#### `GET /api/presentations/[slug]` — 获取演示文稿详情

**Response 200:**
```typescript
interface GetPresentationResponse {
  slug: string;
  title: string;
  style: StyleConfig;
  slides: Slide[];
  totalCost: number;
  createdAt: string;
  updatedAt: string;
}
```

#### `PUT /api/presentations/[slug]` — 更新演示文稿

**Request Body:**
```typescript
interface UpdatePresentationRequest {
  title?: string;        // 更新标题
}
```

**Response 200:**
```typescript
interface UpdatePresentationResponse {
  slug: string;
  title: string;
}
```

#### `DELETE /api/presentations/[slug]` — 删除演示文稿

删除整个 slug 目录，包括所有 slides 和图片。

**Response 204**（无内容）

---

### 3.2 Slide 管理

#### `GET /api/presentations/[slug]/slides` — 获取演示文稿数据

**Response 200:**
```typescript
interface GetSlidesResponse {
  title: string;
  style: StyleConfig;
  slides: Slide[];
  totalCost: number;
}
```

#### `POST /api/presentations/[slug]/slides` — 新建 Slide

**Request Body:**
```typescript
interface CreateSlideRequest {
  content: string;   // slide 文字内容
  index?: number;    // 插入位置，不传则追加到末尾
}
```

**Response 201:**
```typescript
interface CreateSlideResponse {
  slide: Slide;      // 新建的 slide（含 sid）
}
```

#### `PUT /api/presentations/[slug]/slides/[sid]` — 更新 Slide

**Request Body:**
```typescript
interface UpdateSlideRequest {
  content?: string;    // 更新文字内容
  activeImageIndex?: number; // 切换选中的图片
}
```

**Response 200:**
```typescript
interface UpdateSlideResponse {
  slide: Slide;
  hasMatchingImage: boolean; // 是否已有匹配当前文本哈希的图片
}
```

#### `DELETE /api/presentations/[slug]/slides/[sid]` — 删除 Slide

**Response 204**（无内容）

#### `PUT /api/presentations/[slug]/slides` — 批量更新排序

**Request Body:**
```typescript
interface ReorderSlidesRequest {
  orderedSids: string[];  // 新的 sid 排序
}
```

**Response 200:**
```typescript
interface ReorderSlidesResponse {
  slides: Slide[];  // 排序后的 slides
}
```

---

### 3.3 图片生成

#### `POST /api/presentations/[slug]/generate/[sid]` — 生成单张 Slide 图片

**Request Body:**
```typescript
interface GenerateImageRequest {
  // 无需额外参数，sid 和 content 从 outline.yaml 读取
}
```

**Response 200（流式 SSE）:**
```
event: progress
data: {"status": "generating", "progress": 0.5}

event: complete
data: {"image": {"filename": "slide1_1_a1b2c3.jpg", "textHash": "a1b2c3", "cost": 0.02}}
```

**逻辑：**
1. 读取 sid 对应的 content，计算 blake3 哈希
2. 检查是否已存在相同哈希的图片，有则直接返回
3. 调用 AI 生成图片，传入 content + style reference
4. 保存图片到 `images/`，更新 outline.yaml
5. 流式返回进度和结果

#### `POST /api/presentations/[slug]/generate/batch` — 批量并行生成

**Request Body:**
```typescript
interface BatchGenerateRequest {
  sids: string[];       // 需要生成的 slide IDs
}
```

**Response 200（流式 SSE）:**
```
event: progress
data: {"sid": "slide1", "status": "generating", "progress": 0.3}

event: complete
data: {"sid": "slide1", "image": {"filename": "...", "cost": 0.02}}

event: complete
data: {"sid": "slide2", "image": {"filename": "...", "cost": 0.02}}

event: done
data: {"totalCost": 0.06}
```

**逻辑：**
1. 第一个 slide 先串行生成（作为 base image）
2. 后续 slides 并行生成，参考 style image + base image
3. 每个 slide 完成后推送 SSE 事件

#### `POST /api/presentations/[slug]/generate/style` — 生成风格候选图片

**Request Body:**
```typescript
interface GenerateStyleRequest {
  prompt: string;       // 风格描述文字
}
```

**Response 200:**
```typescript
interface GenerateStyleResponse {
  candidates: string[]; // 4 张候选图片路径
  prompt: string;       // 保存的 prompt
}
```

#### `PUT /api/presentations/[slug]/generate/style` — 选择风格图片

**Request Body:**
```typescript
interface SelectStyleRequest {
  referenceImage: string; // 选中的候选图片路径
}
```

**Response 200:**
```typescript
interface SelectStyleResponse {
  style: StyleConfig;
}
```

---

### 3.4 图片文件

#### `GET /api/presentations/[slug]/images/[filename]` — 获取图片

**Response 200:** 返回 `image/jpeg` 二进制流

**Response 404:** 图片不存在

---

## 4. 核心模块设计

### 4.1 存储层

#### presentation-repo.ts

```typescript
// 职责：演示文稿目录级别的管理

class PresentationRepo {
  // 列出所有演示文稿
  async list(): Promise<PresentationSummary[]>

  // 检查 slug 是否存在
  async exists(slug: string): Promise<boolean>

  // 创建新的演示文稿目录和 outline.yaml
  async create(slug: string, title: string): Promise<void>

  // 删除整个 slug 目录（含所有图片）
  async delete(slug: string): Promise<void>

  // 获取元信息（不读取完整 outline）
  async getMeta(slug: string): Promise<PresentationSummary>
}
```

#### outline-repo.ts

```typescript
// 职责：outline.yaml 文件的读写，不包含业务逻辑

class OutlineRepo {
  // 读取 outline.yaml，解析为 Presentation 结构
  async read(slug: string): Promise<Presentation>

  // 将 Presentation 写回 outline.yaml
  async write(slug: string, data: Presentation): Promise<void>

  // 检查 slug 目录是否存在
  async exists(slug: string): Promise<boolean>

  // 初始化一个新的演示文稿目录
  async init(slug: string, title: string): Promise<void>
}
```

#### image-repo.ts

```typescript
// 职责：图片文件的读写

class ImageRepo {
  // 保存图片 buffer 到文件，返回文件名
  async save(slug: string, sid: string, index: number, textHash: string, buffer: Buffer): Promise<string>

  // 读取图片文件
  async read(slug: string, filename: string): Promise<Buffer>

  // 删除图片
  async delete(slug: string, filename: string): Promise<void>

  // 列出某个 slide 的所有图片
  async listBySid(slug: string, sid: string): Promise<string[]>
}
```

### 4.2 业务层

#### presentation-service.ts

```typescript
class PresentationService {
  constructor(
    private presentationRepo: PresentationRepo,
  ) {}

  // 获取演示文稿列表
  async list(): Promise<PresentationSummary[]>

  // 新建演示文稿
  async create(slug: string, title: string): Promise<{ slug: string; title: string }>

  // 获取演示文稿详情
  async get(slug: string): Promise<Presentation>

  // 更新标题
  async update(slug: string, title: string): Promise<void>

  // 删除演示文稿
  async delete(slug: string): Promise<void>
}
```

#### slide-service.ts

```typescript
class SlideService {
  constructor(
    private outlineRepo: OutlineRepo,
    private imageRepo: ImageRepo,
  ) {}

  // 获取演示文稿完整数据（含图片列表）
  async getPresentation(slug: string): Promise<Presentation>

  // 新建 slide
  async createSlide(slug: string, content: string, index?: number): Promise<Slide>

  // 更新 slide 内容，返回是否有匹配图片
  async updateSlide(slug: string, sid: string, content: string): Promise<{ slide: Slide; hasMatchingImage: boolean }>

  // 删除 slide（同时删除关联图片）
  async deleteSlide(slug: string, sid: string): Promise<void>

  // 重新排序
  async reorderSlides(slug: string, orderedSids: string[]): Promise<Slide[]>

  // 计算总成本
  async getTotalCost(slug: string): Promise<number>
}
```

#### generate-service.ts

```typescript
class GenerateService {
  constructor(
    private outlineRepo: OutlineRepo,
    private imageRepo: ImageRepo,
    private imageGenerator: ImageGenerator,
  ) {}

  // 生成单张 slide 图片
  async generateSlideImage(slug: string, sid: string): AsyncGenerator<GenerateProgress>

  // 批量并行生成
  async batchGenerate(slug: string, sids: string[]): AsyncGenerator<BatchGenerateProgress>

  // 计算文本 blake3 哈希
  computeTextHash(content: string): string

  // 检查是否已有匹配哈希的图片
  findMatchingImage(slide: Slide, textHash: string): SlideImage | null
}
```

#### style-service.ts

```typescript
class StyleService {
  constructor(
    private outlineRepo: OutlineRepo,
    private imageGenerator: ImageGenerator,
  ) {}

  // 生成 4 张风格候选
  async generateCandidates(slug: string, prompt: string): Promise<string[]>

  // 选择风格图片
  async selectStyle(slug: string, referenceImage: string): Promise<StyleConfig>

  // 获取当前风格
  async getStyle(slug: string): Promise<StyleConfig>

  // 是否需要引导（无 referenceImage）
  needsGuide(style: StyleConfig): boolean
}
```

### 4.3 AI 层

使用 `@google/genai` SDK，基于 Nano Banana Pro（`gemini-3-pro-image-preview`）生成图片。

#### image-generator.ts

```typescript
import { GoogleGenAI } from "@google/genai";

interface GenerateOptions {
  prompt: string;
  referenceImage?: Buffer;    // 风格参考图（base64 编码传入）
}

interface GenerateResult {
  images: Buffer[];           // 生成的图片 buffer
  cost: number;               // 本次消耗（USD）
}

class ImageGenerator {
  private ai: GoogleGenAI;
  private model: string;

  constructor() {
    this.ai = new GoogleGenAI({ apiKey: process.env.GOOGLE_AI_API_KEY });
    this.model = process.env.GOOGLE_AI_MODEL || "gemini-3-pro-image-preview";
  }

  // 生成单张 slide 图片
  async generate(options: GenerateOptions): Promise<GenerateResult> {
    const contents = this.buildContents(options.prompt, options.referenceImage);
    const response = await this.ai.models.generateContent({
      model: this.model,
      contents,
      config: {
        responseModalities: ["TEXT", "IMAGE"],
        imageConfig: {
          aspectRatio: "16:9",
          imageSize: "2K",
        },
      },
    });
    return this.extractImages(response);
  }

  // 生成多张风格候选图片（并行调用多次）
  async generateMultiple(prompt: string, count: number): Promise<GenerateResult> {
    const promises = Array.from({ length: count }, () =>
      this.ai.models.generateContent({
        model: this.model,
        contents: [{ text: prompt }],
        config: {
          responseModalities: ["IMAGE"],
          imageConfig: {
            aspectRatio: "16:9",
            imageSize: "1K",
          },
        },
      })
    );
    const responses = await Promise.all(promises);
    const allImages: Buffer[] = [];
    let totalCost = 0;
    for (const response of responses) {
      const result = this.extractImages(response);
      allImages.push(...result.images);
      totalCost += result.cost;
    }
    return { images: allImages, cost: totalCost };
  }

  // 构建请求内容（文本 + 可选参考图片）
  private buildContents(prompt: string, referenceImage?: Buffer) {
    const parts: any[] = [{ text: prompt }];
    if (referenceImage) {
      parts.push({
        inlineData: {
          mimeType: "image/jpeg",
          data: referenceImage.toString("base64"),
        },
      });
    }
    return parts;
  }

  // 从响应中提取图片
  private extractImages(response: any): GenerateResult {
    const images: Buffer[] = [];
    for (const part of response.candidates[0].content.parts) {
      if (part.inlineData) {
        images.push(Buffer.from(part.inlineData.data, "base64"));
      }
    }
    // 估算成本（基于 usageMetadata）
    const cost = this.estimateCost(response.usageMetadata);
    return { images, cost };
  }

  // 根据模型 token 用量估算成本
  private estimateCost(usage: any): number {
    if (!usage) return 0;
    // Nano Banana Pro: $0.0025/1K input tokens, $0.01/1K output tokens
    const inputCost = (usage.promptTokenCount || 0) * 0.0000025;
    const outputCost = (usage.candidatesTokenCount || 0) * 0.00001;
    return inputCost + outputCost;
  }
}
```

**关键设计决策：**

| 决策 | 说明 |
|------|------|
| 模型选择 | 默认 `gemini-3-pro-image-preview`（Nano Banana Pro），适合专业素材制作 |
| 宽高比 | 16:9，标准幻灯片比例 |
| 分辨率 | Slide 图片 2K，风格候选 1K（节省成本） |
| 参考图传递 | 通过 `inlineData` 将风格图 base64 编码后传入，模型自动参考风格 |
| 多图生成 | 风格候选通过 `Promise.all` 并行调用，每次调用生成 1 张 |
| 成本计算 | 基于 `usageMetadata` 中的 token 用量估算 |

---

## 5. 前端状态管理

使用 Zustand 管理全局状态，避免 prop drilling。

### 5.1 Store 结构

```typescript
interface PresentationStore {
  // 数据
  slug: string;
  title: string;
  slides: Slide[];
  style: StyleConfig;
  totalCost: number;

  // UI 状态
  selectedSid: string | null;
  generatingSids: Set<string>;        // 正在生成的 slide 集合
  isStyleGuideOpen: boolean;          // 风格引导弹窗
  isFullscreen: boolean;              // 全屏播放
  playingIndex: number;               // 播放当前索引

  // Actions
  loadPresentation: (slug: string) => Promise<void>;
  selectSlide: (sid: string) => void;
  createSlide: (content: string) => Promise<void>;
  updateSlideContent: (sid: string, content: string) => Promise<{ hasMatchingImage: boolean }>;
  deleteSlide: (sid: string) => Promise<void>;
  reorderSlides: (orderedSids: string[]) => Promise<void>;
  generateImage: (sid: string) => Promise<void>;
  batchGenerate: (sids: string[]) => Promise<void>;
  generateStyle: (prompt: string) => Promise<string[]>;
  selectStyle: (image: string) => Promise<void>;
  startFullscreen: () => void;
  stopFullscreen: () => void;
}
```

### 5.2 前端页面组件树

```
[slug]/page.tsx
├── AppHeader                          # Logo + 标题
│   ├── Logo
│   └── EditableTitle
├── MainLayout (flex row)
│   ├── SlideList                      # 左侧面板
│   │   ├── SlideCard[] (Draggable)    # 可拖拽卡片
│   │   └── NewSlideButton
│   └── PreviewPanel                   # 右侧面板
│       ├── ImagePreview               # 主预览
│       ├── GenerateButton             # 生成按钮（条件显示）
│       ├── ThumbnailBar               # 缩略图
│       └── PlayButton                 # 播放
├── StyleGuidePopup                    # 风格引导弹窗
│   ├── PromptInput
│   ├── StyleSelector (2x2 grid)
│   └── ConfirmButton
└── FullscreenPlayer                   # 全屏播放覆盖层
```

---

## 6. 关键交互流程

### 6.1 首次打开 — 风格引导

```
用户打开 /[slug]
    │
    ▼
GET /api/presentations/[slug]/slides
    │
    ▼
style.referenceImage === null ?
    ├─ Yes → 打开 StyleGuidePopup
    │          │
    │          ▼
    │     用户输入 prompt
    │          │
    │          ▼
    │     POST /api/presentations/[slug]/generate/style { prompt }
    │          │
    │          ▼
    │     展示 4 张候选图
    │          │
    │          ▼
    │     用户选择一张
    │          │
    │          ▼
    │     PUT /api/presentations/[slug]/generate/style { referenceImage }
    │          │
    │          ▼
    │     关闭弹窗，进入主界面
    │
    └─ No → 直接进入主界面
```

### 6.2 编辑文本 → 检测 → 手动生成

```
用户编辑 slide 文本
    │
    ▼
PUT /api/presentations/[slug]/slides/[sid] { content }
    │
    ▼
返回 { slide, hasMatchingImage: false }
    │
    ▼
前端显示 "生成新图片" 按钮
    │
    ▼
用户点击按钮
    │
    ▼
POST /api/presentations/[slug]/generate/[sid]
    │  (SSE 流式)
    ▼
接收 progress 事件 → 更新 loading 状态
    │
    ▼
接收 complete 事件 → 更新预览图和缩略图栏
```

### 6.3 全屏播放

```
用户点击 "播放" 按钮
    │
    ▼
请求 Fullscreen API 进入全屏
    │
    ▼
设置 playingIndex = selectedSlideIndex
    │
    ▼
启动定时器（默认 5s）
    │
    ▼
每 5s → playingIndex++ → 展示下一张 slide 的 activeImage
    │
    ▼
最后一张后循环回到第一张
    │
    ▼
ESC / 按 → 退出全屏，清除定时器
```

---

## 7. 图片命名与缓存策略

### 7.1 命名规则

```
<sid>_<序号>_<blake3(content)>.jpg
```

- `sid`：slide 唯一标识
- `序号`：同一 slide 的第几张图（从 1 递增）
- `blake3(content)`：生成时文字内容的 blake3 哈希（16 位十六进制）

前端通过比较当前 content 的哈希与已有图片的 textHash，判断是否需要显示"生成新图片"按钮。

### 7.2 缓存策略

- 图片通过 `GET /api/presentations/[slug]/images/[filename]` 加载
- 浏览器端设置 `Cache-Control: max-age=31536000, immutable`（文件名含哈希，内容不会变）
- 全屏播放时预加载相邻 slide 的图片（`<link rel="preload">`）
