# GenSlides — AI 图片幻灯片生成器 PRD

## 1. 产品概述

### 1.1 产品定义

GenSlides 是一个本地运行的 AI 图片幻灯片生成工具。用户输入文字内容，系统使用 Google Nano Banana Pro 模型生成视觉风格统一的图片，并以走马灯形式全屏播放，形成完整的幻灯片演示。

### 1.2 目标用户

- 需要快速制作视觉演示文稿的开发者与内容创作者
- 希望通过 AI 自动生成高质量幻灯片的个人用户

### 1.3 核心价值

- 零设计门槛：文字输入即可生成专业级视觉幻灯片
- 风格统一：所有 slides 共享同一视觉风格，确保演示一致性
- 本地运行：无需云端部署，所有数据和处理均在本地完成
- 低成本可控：实时展示生成成本，帮助用户控制消耗

---

## 2. 功能需求

### 2.1 幻灯片管理（左侧面板）

#### 2.1.1 Logo 与标题

- 顶部展示应用 Logo "GenSlides"
- 标题栏显示当前演示文稿标题，支持双击编辑（如"我的演示文稿"）

#### 2.1.2 Slides 列表

- 以垂直卡片列表展示所有 slides
- 每个 slide 卡片展示：
  - 唯一标识 sid（如 Slide1、Slide2...）
  - 当前使用的缩略图预览
- 支持点击选中，选中后右侧预览区同步展示该 slide 内容
- 列表末尾提供"新建 Slide"按钮，hover 时显示
- **拖拽排序**：支持拖拽调整 slide 在列表中的顺序，拖拽时有视觉占位反馈
- 支持删除 slide

#### 2.1.3 Slide 内容编辑

- 选中 slide 后，可编辑该 slide 的文字内容
- 文字内容即为传给图片生成模型的 prompt 描述
- 文字内容变更后，系统检测是否存在对应文本哈希的图片：
  - **已有匹配图片**：直接展示该图片
  - **无匹配图片**：在主预览区下方显示"生成新图片"按钮，用户点击后触发生成（非自动触发，避免误操作消耗成本）

### 2.2 预览与播放（右侧面板）

#### 2.2.1 图片预览区

- 展示当前选中 slide 的最新生成图片
- 若 slide 尚无图片，展示空白占位状态
- 图片以幻灯片比例（16:9）展示
- 当文本内容变更且无匹配哈希图片时，预览区下方显示"生成新图片"按钮

#### 2.2.2 缩略图栏

- 预览区底部水平排列该 slide 生成的所有历史图片缩略图
- 点击缩略图可切换预览区展示的图片
- 当前选中图片以高亮边框标识
- 可将某张缩略图设为当前 slide 的展示图片

#### 2.2.3 全屏播放

- 右上角提供"播放"按钮
- 点击后从当前选中的 slide 开始全屏播放
- 播放模式为走马灯（自动轮播），按 slide 列表顺序依次展示
- 播放控制：
  - 自动切换间隔可配置（默认 5 秒）
  - 支持键盘左右箭头手动翻页
  - ESC 退出全屏播放

### 2.3 风格设定（首次引导）

#### 2.3.1 风格图片引导弹窗

- 应用首次打开时，检测 `outline.yaml` 中是否已有风格图片
- **无风格图片**：弹出引导弹窗（Popup），流程如下：
  1. 弹窗中提供文本输入框，用户输入风格描述（如"极简科技风，深蓝渐变背景，白色无衬线字体"）
  2. 点击"生成"，调用 Nano Banana Pro 一次性生成 **4 张风格候选图片**
  3. 4 张图片以 2×2 网格展示在弹窗中
  4. 用户点击选择其中一张作为全局风格参考
  5. 选中后保存到 `outline.yaml` 的 `style.reference_image` 字段，弹窗关闭
- **已有风格图片**：跳过弹窗，直接进入主界面
- 用户可在主界面随时重新打开风格设置，更换风格图片

#### 2.3.2 生成引擎

- 使用 Google Nano Banana Pro 模型生成图片
- 通过 Google AI SDK 调用
- 每个 slide 的文字内容作为 prompt 传入模型

#### 2.3.3 视觉风格统一

- 用户选中的风格图片作为全局参考，保存到 `outline.yaml` 的 `style.reference_image`
- 所有 slides 的图片生成均参考该风格图片，确保视觉一致性
- 第一个 slide 的图片作为基准图（base image），后续 slides 同时参考风格图片和基准图生成

#### 2.3.4 并行生成

- Slides 之间图片生成互不影响，支持并行处理
- 基准图（第一个 slide 的图片）需先生成完成
- 生成过程中展示 loading 状态和进度指示

#### 2.3.5 成本展示

- 实时统计并展示当前所有 slides 的图片生成总成本
- 每次生成操作前展示预估成本，确认后执行

### 2.4 数据存储

#### 2.4.1 文件结构

```
./genslides/<slug>/
├── outline.yaml          # 包含所有 slides 的文字内容与元数据
└── images/
    └── <sid>_<x>_<text_blake3_hash>.jpg  # slide 图片文件
```

- `outline.yaml`：存储所有 slides 的文字内容、顺序、风格设定等
- 图片命名规则：`<slide_id>_<序号>_<文字内容的blake3哈希>.jpg`
- 同一 slide 生成多次产生多张图片，通过哈希区分版本

#### 2.4.2 Outline 数据结构

```yaml
title: 我的演示文稿
style:
  prompt: "极简科技风，深蓝背景，白色文字"
  candidates:
    - "images/style_candidate_a1b2.jpg"
    - "images/style_candidate_c3d4.jpg"
    - "images/style_candidate_e5f6.jpg"
    - "images/style_candidate_g7h8.jpg"
  reference_image: "images/style_candidate_c3d4.jpg"  # 用户从 4 张候选中选择的风格图片路径
slides:
  - sid: slide1
    content: "TypeScript 的核心优势"
  - sid: slide2
    content: "Tailwind CSS 设计哲学"
```

---

## 3. 技术方案

### 3.1 技术栈

| 层级 | 技术选型 |
|------|---------|
| 后端框架 | TypeScript 6 + Next.js 16 (App Router) |
| AI SDK | @google/genai 2.4 (Nano Banana Pro) |
| 前端样式 | Tailwind CSS v4（CSS-first 配置） |
| 状态管理 | Zustand 5 |
| 数据格式 | YAML (outline.yaml) |
| 图片哈希 | blake3 |

### 3.2 架构设计

- **单页应用**：Next.js 16 全栈应用，所有功能在单个页面内完成
- **API Routes**：Next.js App Router API Routes 处理图片生成请求
- **本地文件系统**：直接读写本地 `./genslides/` 目录
- **流式响应**：图片生成使用流式传输，前端实时展示生成进度

### 3.3 核心 API

| API | 方法 | 功能 |
|-----|------|------|
| `/api/presentations` | GET | 获取演示文稿列表 |
| `/api/presentations` | POST | 新建演示文稿 |
| `/api/presentations/[slug]` | GET | 获取演示文稿详情 |
| `/api/presentations/[slug]` | PUT | 更新演示文稿 |
| `/api/presentations/[slug]` | DELETE | 删除演示文稿 |
| `/api/presentations/[slug]/slides` | GET | 获取指定演示文稿的所有 slides 数据 |
| `/api/presentations/[slug]/slides` | POST | 新建 slide |
| `/api/presentations/[slug]/slides/[sid]` | PUT | 更新 slide 内容 |
| `/api/presentations/[slug]/slides/[sid]` | DELETE | 删除 slide |
| `/api/presentations/[slug]/generate/[sid]` | POST | 为指定 slide 生成图片 |
| `/api/presentations/[slug]/generate/batch` | POST | 批量并行生成所有 slides 图片 |
| `/api/presentations/[slug]/generate/style` | POST | 根据描述生成 4 张风格候选图片 |
| `/api/presentations/[slug]/images/[filename]` | GET | 获取图片文件 |

---

## 4. 交互设计

### 4.1 页面布局

```
┌─────────────────────────────────────────────────┐
│  [Logo: GenSlides]     [演示文稿标题（可编辑）]    │
├──────────────┬──────────────────────────────────┤
│  Slide 列表   │        图片预览区                  │
│  ┌──────────┐│                                    │
│  │ Slide 1  ││      [当前选中 slide 的图片]         │
│  ├──────────┤│                                    │
│  │ Slide 2  ││                                    │
│  ├──────────┤│  ┌───┐ ┌───┐ ┌───┐ ┌───┐         │
│  │ Slide 3  ││  │ 1 │ │ 2 │ │ 3 │ │ 4 │  [播放]  │
│  │   ...    ││  └───┘ └───┘ └───┘ └───┘         │
│  [+ 新建]   ││      缩略图栏                      │
└──────────────┴──────────────────────────────────┘
```

### 4.2 核心交互流程

1. **首次打开** → 检测风格图片，无则弹出引导弹窗 → 输入描述 → 生成 4 张候选 → 选择风格
2. **添加 Slide** → 点击"新建"，输入文字内容
3. **生成图片** → 文本变更后点击"生成新图片"按钮
4. **预览选择** → 点击缩略图切换不同版本
5. **全屏播放** → 点击"播放"，走马灯自动轮播
6. **拖拽排序** → 在左侧列表中拖拽 slide 调整播放顺序

### 4.3 状态管理

- 当前选中的 slide sid
- 各 slide 的图片生成状态（idle / generating / done / error）
- 风格设定（描述文字 + 参考图片）
- 播放状态（播放中 / 暂停 / 当前展示的 slide index）

---

## 5. 非功能需求

### 5.1 性能

- 图片生成支持并行，slides 之间互不阻塞
- 缩略图使用懒加载，避免大量图片同时渲染
- 全屏播放使用图片预加载，确保切换流畅无闪烁

### 5.2 本地运行

- 整个应用无需外部服务器，完全运行在本地
- 数据存储在本地文件系统，无需数据库
- 仅需网络连接 Google AI API 进行图片生成

### 5.3 成本控制

- 每次生成前展示预估 token 消耗和费用
- 页面底部常驻展示当前演示文稿的总生成成本
- 支持设置单次生成预算上限

---

## 6. Non-Goals（明确排除）

1. **不支持导出 PPTX**：本产品聚焦于本地预览和播放，不做文件格式导出
2. **不做云端部署**：仅作为本地工具运行
3. **不做多人协作**：单用户本地使用场景
4. **不做视频导出**：仅支持实时播放，不导出视频文件
5. **不使用现成的 slides 库**：自行实现播放和展示逻辑，保持轻量

---

## 7. 里程碑规划

### Phase 1：MVP

- 基础左右分栏布局
- Slide CRUD 操作
- 单张图片生成与预览
- 全屏走马灯播放

### Phase 2：体验增强

- 视觉风格参考图上传
- 缩略图版本管理
- 批量并行生成
- 成本统计面板

### Phase 3：优化打磨

- 生成进度实时推送
- 图片预加载与缓存
- 拖拽排序
- 播放间隔配置
