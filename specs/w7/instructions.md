# Instructions

## Gemini 探究

帮我研究一下市面上关于使用 AI 进行 slides 生成的工具，尤其是 Manus 和 NotebookLLM 的 slides 生成功能，探索其实现的原理。另外，探索如果使用 google 最新推出的 nano banana pro 来做 slides 生成。
思考：根据文本生成图片，把所有图片以幻灯片的形式连接起来播放，就构成了 slides，类似 NotebookLLM 的 slides 生成功能。
要求：图片的视觉风格统一，用户可以提供一个视觉风格图片或者文字描述

## PRD 生成

根据 @specs/w7/原型图.png 的内容，仔细阅读并思考，生成一个 @specs/w7/001-genslides-prd.md 的 PRD 输出为中文。
要求：
1. 这个 APP 是一个本地运行的单页 app，使用 nano banana pro 生成图片 slides, 可以以走马灯的形式全屏播放
2. 后端使用 TypeScript + Next.js 实现，前端使用 TypeScript + Tailwind CSS 实现
   
## PRD 补充

 @specs/w7/001-genslides-prd.md 需要修改：

 1. 对于侧边栏 slide, 可以通过托拉拽来调整位置
 2. 文本内容变化后，如果图片中没有对应的文本 hash 的图片，在主图片区域下放一个按钮，用户点击可以生成新的图片
 3. outline.yaml 中需要保存用户选择的风格图片。当第一次打开时，如果没有风格图片，需要有一个 popup，用户可以输入一段文字，生成四张风格图片，让用户选择，用户选中的作为 slides 风格，后续生成的图片都参考这个风格。风格的 prompt 和图片均要保存到 outline.yaml 中。

## Design Spec 生成

根据 @specs/w7/001-genslides-prd.md 和 @specs/w7/原型图.png  的内容，生成一个 design spec 输出到 @specs/w7/002-genslides-design.md 中，输出为中文。注意：
1. 所有前端所需的 API 接口要定义清楚
2. 整体项目的目录结构和代码层次清晰
3. 后端代码 API/业务/存储需要保持清晰的边界

## 目录结构生成

根据 @specs/w7/002-genslides-design.md 中的目录结构，在 @w7-genslides 目录下生成对应的目录结构，先不要生成代码。然后生成一个 CLAUDE.md 文件，内容充分考虑：
1. TypeScript 语言的 Best Practice
2. Next.js 框架的 Best Practice
3. Tailwind CSS v4 的 Best Practice
4. Vue 3 的 Best Practice
5. 架构设计遵循的原则：SOLID / YAGNI / KISS
6. 代码的组织结构
7. 并发处理
8. 错误处理和日志处理

## 代码生成

根据 @specs/w7/002-genslides-design.md 和 @specs/w7/原型图.png，启动 frontend 和 backend 两个 agent 分别撰写前端代码和后端代码，代码在 @w7-genslides 目录下。

## Slide 风格

使用浅黄色 / 红褐色的水彩画风格，走可爱卡通路线，主要角色是一个可爱的动物，类似 1 的风格。
