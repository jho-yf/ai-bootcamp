# GenSlides ASCII 架构图

## 1. 前端架构

```
  ┌─────────────────────────────────────────────────────────────────────────┐
  │                            Router (vue-router 4.5)                      │
  │                     ┌──────────────┬──────────────┐                     │
  │                     │   "/"        │  "/:slug"     │                     │
  │                     │  HomeView    │  EditorView   │                     │
  │                     └──────┬───────┴──────┬───────┘                     │
  └────────────────────────────┼──────────────┼─────────────────────────────┘
                               │              │
          ┌────────────────────┘              └─────────────────────┐
          │                                                         │
          ▼                                                         ▼
  ┌───────────────────────┐                ┌─────────────────────────────────────┐
  │      HomeView         │                │           EditorView                │
  │  ┌─────────────────┐  │                │  ┌───────────────────────────────┐  │
  │  │   AppHeader     │  │                │  │         AppHeader             │  │
  │  └─────────────────┘  │                │  │   (editableTitle, slug)       │  │
  │  ┌─────────────────┐  │                │  │   (cost in actions slot)      │  │
  │  │ ConfirmDialog   │  │                │  └───────────────────────────────┘  │
  │  │ (create/delete) │  │                │  ┌──────────────┐ ┌──────────────┐  │
  │  └─────────────────┘  │                │  │ StyleThumbnail│ │  SlideList   │  │
  └───────────────────────┘                │  └──────────────┘ └──────┬───────┘  │
                                           │                    ┌────┴────┐      │
                                           │                    │SlideCard│      │
                                           │                    └─────────┘      │
                                           │  ┌──────────────────────────────┐  │
                                           │  │       ImagePreview           │  │
                                           │  └──────────────────────────────┘  │
                                           │  ┌──────────────────────────────┐  │
                                           │  │       ThumbnailBar           │  │
                                           │  └──────────────────────────────┘  │
                                           │  ┌──────────────────────────────┐  │
                                           │  │     FullscreenPlayer         │  │
                                           │  └──────────────────────────────┘  │
                                           │  ┌───────────────┐                 │
                                           │  │SlideEditPopup │                 │
                                           │  └───────────────┘                 │
                                           │  ┌──────────────┐┌──────────────┐  │
                                           │  │StyleGuidePopup││StyleDetailPop│  │
                                           │  └───────┬──────┘└──────────────┘  │
                                           │          │                         │
                                           │  ┌───────▼──────┐                  │
                                           │  │StyleSelector  │                  │
                                           │  └──────────────┘                  │
                                           └─────────────────────────────────────┘
          │                                                         │
          │                   依赖方向: Views → Components           │
          ▼                                                         ▼
  ┌─────────────────────────────────────────────────────────────────────────┐
  │                        Stores (Pinia 3.0)                               │
  │  ┌─────────────────────────────────┐  ┌──────────────────────────────┐  │
  │  │   usePresentationStore          │  │       useUiStore             │  │
  │  │                                 │  │                              │  │
  │  │  状态:                           │  │  状态:                        │  │
  │  │   presentation                  │  │   error                      │  │
  │  │   presentations                 │  │   showStyleGuide             │  │
  │  │   selectedSid                   │  │                              │  │
  │  │   generatingSids                │  │  方法:                        │  │
  │  │   isLoading / error             │  │   setError / clearError      │  │
  │  │                                 │  │   openStyleGuide             │  │
  │  │  计算属性:                        │  │   closeStyleGuide            │  │
  │  │   selectedSlide                 │  └──────────────────────────────┘  │
  │  │   currentImage                  │                                     │
  │  │                                 │                                     │
  │  │  方法:                           │                                     │
  │  │   loadPresentations             │                                     │
  │  │   createPresentation            │                                     │
  │  │   loadPresentation              │                                     │
  │  │   updateTitle / delete          │                                     │
  │  │   addSlide / updateSlide        │                                     │
  │  │   deleteSlide / reorderSlides   │                                     │
  │  │   generateImage (SSE/POST)      │                                     │
  │  │   batchGenerate (SSE/POST)      │                                     │
  │  │   generateStyleCandidates       │                                     │
  │  │   selectStyle / cleanup         │                                     │
  │  └────────────────┬────────────────┘                                     │
  └───────────────────┼──────────────────────────────────────────────────────┘
                      │
                      │              依赖方向: Stores → Composables
                      ▼
  ┌─────────────────────────────────────────────────────────────────────────┐
  │                      Composables                                        │
  │  ┌─────────────────────────────┐  ┌─────────────────────────────────┐  │
  │  │        useApi               │  │          useSse                 │  │
  │  │                             │  │                                 │  │
  │  │  BASE_URL = '/api/ppt'      │  │  fetch + POST + ReadableStream │  │
  │  │  get<T>(path)               │  │  connect(url)                   │  │
  │  │  post<T>(path, body)        │  │  on(event, callback)            │  │
  │  │  put<T>(path, body)         │  │  close()                        │  │
  │  │  del<T>(path)               │  │                                 │  │
  │  │                             │  │  事件:                           │  │
  │  │  内部: request<T>()         │  │   complete / progress / error   │  │
  │  │   fetch + JSON 解析          │  │   done (batch)                  │  │
  │  │   错误处理 + 204 空响应       │  │                                 │  │
  │  └─────────────────────────────┘  └─────────────────────────────────┘  │
  └─────────────────────────────────────────────────────────────────────────┘
                      │
                      │              依赖方向: Composables → Backend API
                      ▼
              ┌───────────────────┐
              │   /api/ppt/*      │
              │   (后端 REST API)  │
              └───────────────────┘
```

## 2. 前端技术栈

```
  ┌─────────────────────────────────────────────────────────────────────────┐
  │                        前端技术栈                                        │
  ├─────────────────────────────────────────────────────────────────────────┤
  │                                                                         │
  │  ┌─── 核心框架 ──────────────────────────────────────────────────────┐  │
  │  │   Vue 3.5.34              渐进式 JavaScript 框架                    │  │
  │  │   ├── Composition API     <script setup> 语法糖                    │  │
  │  │   ├── reactivity          ref / reactive / computed               │  │
  │  │   └── Teleport            弹窗组件渲染到 body                      │  │
  │  │                                                                   │  │
  │  │   Vue Router 4.5.0        客户端路由                               │  │
  │  │   ├── createWebHistory    HTML5 History 模式                      │  │
  │  │   └── 动态路由             /:slug 参数路由                         │  │
  │  │                                                                   │  │
  │  │   Pinia 3.0.4             状态管理                                 │  │
  │  │   ├── defineStore         组合式 Store 定义                        │  │
  │  │   └── storeToRefs         响应式解构                               │  │
  │  └───────────────────────────────────────────────────────────────────┘  │
  │                                                                         │
  │  ┌─── 构建工具 ──────────────────────────────────────────────────────┐  │
  │  │   Vite 8.0.13             下一代前端构建工具                        │  │
  │  │   ├── @vitejs/plugin-vue  6.0.7   Vue SFC 编译插件                 │  │
  │  │   ├── HMR                 热模块替换                               │  │
  │  │   └── ESBuild             预构建依赖                               │  │
  │  │                                                                   │  │
  │  │   TypeScript 5.8.3        类型安全                                 │  │
  │  │   ├── vue-tsc 2.2.8       Vue SFC 类型检查                        │  │
  │  │   └── strict: true        严格模式                                 │  │
  │  └───────────────────────────────────────────────────────────────────┘  │
  │                                                                         │
  │  ┌─── 样式方案 ──────────────────────────────────────────────────────┐  │
  │  │   Tailwind CSS 4.3.0      原子化 CSS 框架                          │  │
  │  │   ├── @tailwindcss/vite   4.3.0   Vite 集成插件                   │  │
  │  │   ├── CSS-first 配置      @theme 设计令牌                          │  │
  │  │   └── 响应式工具类         sm: / md: / lg: 前缀                    │  │
  │  └───────────────────────────────────────────────────────────────────┘  │
  │                                                                         │
  │  ┌─── 无额外运行时库 ────────────────────────────────────────────────┐  │
  │  │   HTTP 请求:     原生 fetch API (composables/useApi.ts)           │  │
  │  │   SSE 流式:      fetch POST + ReadableStream (composables/useSse.ts) │
  │  │   UI 组件库:     无 (手写 Tailwind 组件)                           │  │
  │  │   动画库:        Vue 内置 <Transition>                            │  │
  │  └───────────────────────────────────────────────────────────────────┘  │
  └─────────────────────────────────────────────────────────────────────────┘
```

## 3. 后端架构

```
  ┌─────────────────────────────────────────────────────────────────────────┐
  │                    Next.js 16.2 App Router 路由层                       │
  │  app/api/ppt/                                                           │
  │  ├── route.ts                     GET  列表 / POST 新建                 │
  │  └── [slug]/                                                              │
  │      ├── route.ts                 GET 详情 / PUT 改标题 / DELETE 删除    │
  │      ├── slides/route.ts          POST 新建slide / PUT 排序             │
  │      ├── slides/[sid]/route.ts    PUT 编辑 / DELETE 删除                 │
  │      ├── generate/[sid]/route.ts  POST 单张生成 (SSE 流式)              │
  │      ├── generate/batch/route.ts  POST 批量生成 (SSE 流式)              │
  │      ├── generate/style/route.ts  POST 生成候选 / PUT 选择 / DELETE 清理│
  │      └── images/[filename]/route.ts GET 获取图片                        │
  ├─────────────────────────────────────────────────────────────────────────┤
  │                          ▼  Routes → Services                           │
  │  ┌───────────────────────────────────────────────────────────────────┐  │
  │  │  PresentationService    SlideService    GenerateService   StyleService│
  │  │  (CRUD + slug 生成)     (Slide CRUD)   (AI 图片编排)     (风格生成) │
  │  └────────────┬──────────┴───────┬────────┴───────┬───────────┴──────┘  │
  │               └──────────────────┴────────────────┴──────────────────┘  │
  │                          ▼  Services → {Storage, AI}                    │
  │  ┌──────────────────────┐  ┌──────────────────────────────────────┐    │
  │  │     Storage 层        │  │         AI 层                        │    │
  │  │  PresentationRepo     │  │  ImageGenerator                      │    │
  │  │  OutlineRepo (YAML)   │  │  @google/genai 2.4                   │    │
  │  │  ImageRepo (JPEG)     │  │  gemini-3-pro-image-preview          │    │
  │  │  原子写入 + 内存队列   │  │  proxy (undici ProxyAgent)           │    │
  │  └──────────────────────┘  └──────────────────────────────────────┘    │
  └─────────────────────────────────────────────────────────────────────────┘
```

## 4. 后端技术栈

```
  ┌─────────────────────────────────────────────────────────────────────────┐
  │                        后端技术栈                                        │
  ├─────────────────────────────────────────────────────────────────────────┤
  │  ┌─── 核心框架 ──────────────────────────────────────────────────────┐  │
  │  │   Next.js 16.2.6          Web 框架 (App Router + Route Handlers)  │  │
  │  │   React 19.0.0            (Next.js 依赖，API 路由不使用渲染)       │  │
  │  └───────────────────────────────────────────────────────────────────┘  │
  │  ┌─── AI / 数据 ────────────────────────────────────────────────────┐  │
  │  │   @google/genai 2.4.0     Google Generative AI SDK               │  │
  │  │   hash-wasm 4.12.0        Blake3 哈希 (prompt → 图片缓存键)       │  │
  │  │   yaml 2.9.0              YAML 解析/序列化                        │  │
  │  └───────────────────────────────────────────────────────────────────┘  │
  │  ┌─── 网络 ─────────────────────────────────────────────────────────┐  │
  │  │   undici 6.25.0           ProxyAgent 代理支持                     │  │
  │  │   TypeScript 5.7.0        类型安全 (strict: true)                 │  │
  │  └───────────────────────────────────────────────────────────────────┘  │
  │  ┌─── 存储 ─────────────────────────────────────────────────────────┐  │
  │  │   文件系统 (fs/promises)  无数据库/ORM                             │  │
  │  │   原子写入: .tmp → rename  内存写入队列串行化                      │  │
  │  │   blake3 哈希文件名        内容寻址缓存                            │  │
  │  └───────────────────────────────────────────────────────────────────┘  │
  └─────────────────────────────────────────────────────────────────────────┘
```

## 5. 数据流图

```
  ┌─────────────────────────────────────────────────────────────────────────┐
  │                                                                         │
  │  示例一: 新建 Slide                                                      │
  │  用户 → SlideList('+新建') → EditorView → addSlide() →                   │
  │  presentationStore → useApi.post() →  HTTP  ──────────────────────►    │
  │  → Route POST → SlideService → OutlineRepo → outline.yaml              │
  │  返回 {slide} ─────────────────────────────────────────────────────►    │
  │  → store.slides.splice() → Vue 响应式 → UI 重新渲染                      │
  │                                                                         │
  │  示例二: 生成图片 (SSE 流式)                                              │
  │  用户 → 双击Slide选中 → 点击"保存并生成" →                                │
  │  presentationStore.generateImage(slug, sid) →                           │
  │  generatingSids.add(sid) → useSse.fetch(POST) → HTTP POST ────────►   │
  │  → Route POST → ReadableStream →                                        │
  │  → GenerateService: buildPrompt → blake3 → 检查缓存 →                   │
  │  → imageGenerator.generate(prompt, refImage) → Google AI API            │
  │  → imageRepo.save() → slideService.addImageToSlide()                   │
  │  → outlineRepo.write() → event:complete ──────────────────────────►    │
  │  → store: images.push() + activeImageIndex → generatingSids.delete()   │
  │  → Vue 响应式 → ImagePreview + ThumbnailBar + SlideCard 更新            │
  │                                                                         │
  │  示例三: 风格生成                                                        │
  │  用户 → StyleGuidePopup → 输入prompt → 生成候选 →                        │
  │  POST /generate/style → StyleService → 删除旧候选 →                     │
  │  blake3(prompt) → ImageGenerator.generate() → AI API →                  │
  │  ImageRepo.save() → 返回 candidates → UI 显示 StyleSelector            │
  │  → 用户选择 → PUT /generate/style → 删除未选中图片 →                     │
  │  data.style.referenceImage = filename → outlineRepo.write()             │
  │  → presentation.value.style 更新 → StyleThumbnail 更新                  │
  │                                                                         │
  └─────────────────────────────────────────────────────────────────────────┘
```
