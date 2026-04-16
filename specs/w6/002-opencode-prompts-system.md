# OpenCode System Prompt 与工具 Prompt 架构详解

> 本文档基于 `vendors/opencode/` 代码库的深入分析，系统性地介绍 OpenCode 的 prompt 架构设计——包括系统提示词（system prompt）的选择与组装、工具（tool）描述的定义与注册、智能体（agent）提示词的设计，以及各组件之间的协作关系。

---

## 目录

1. [架构概览](#1-架构概览)
2. [系统提示词（System Prompt）](#2-系统提示词system-prompt)
   - 2.1 [按 Provider 分类的提示词模板](#21-按-provider-分类的提示词模板)
   - 2.2 [提示词选择逻辑](#22-提示词选择逻辑)
   - 2.3 [各 Provider 提示词详解](#23-各-provider-提示词详解)
3. [动态系统提示词组装](#3-动态系统提示词组装)
   - 3.1 [环境信息注入](#31-环境信息注入)
   - 3.2 [技能（Skills）列表注入](#32-技能skills列表注入)
   - 3.3 [用户指令（Instructions）加载](#33-用户指令instructions加载)
4. [提示词最终组装流程](#4-提示词最终组装流程)
   - 4.1 [LLM.stream() 中的系统消息构建](#41-llmstream-中的系统消息构建)
   - 4.2 [runLoop 中的动态组装](#42-runloop-中的动态组装)
   - 4.3 [完整组装流程图](#43-完整组装流程图)
5. [工具（Tool）Prompt 架构](#5-工具tool-prompt-架构)
   - 5.1 [工具定义模式：.ts + .txt 配对](#51-工具定义模式-ts--txt-配对)
   - 5.2 [工具注册与过滤](#52-工具注册与过滤)
   - 5.3 [各工具描述详解](#53-各工具描述详解)
6. [智能体（Agent）Prompt 架构](#6-智能体agent-prompt-架构)
   - 6.1 [内置 Agent 及其提示词](#61-内置-agent-及其提示词)
   - 6.2 [Agent 动态生成](#62-agent-动态生成)
7. [Plan 模式提示词系统](#7-plan-模式提示词系统)
   - 7.1 [Plan 模式激活](#71-plan-模式激活)
   - 7.2 [Plan 到 Build 切换](#72-plan-到-build-切换)
   - 7.3 [最大步数限制](#73-最大步数限制)
8. [Provider 消息转换层](#8-provider-消息转换层)
9. [插件系统对 Prompt 的影响](#9-插件系统对-prompt-的影响)
10. [关键设计模式总结](#10-关键设计模式总结)

---

## 1. 架构概览

OpenCode 是一个基于 TypeScript 的 AI 编码助手 CLI 工具，使用 Vercel AI SDK（`ai` 包）与各种 LLM 提供商通信。其 prompt 架构采用**分层组合（Layered Composition）**模式，核心目录结构如下：

```
packages/opencode/src/
├── session/
│   ├── system.ts          # SystemPrompt 命名空间：选择 provider 提示词、构建环境信息
│   ├── prompt.ts          # 主会话提示词循环：组装系统提示词、解析工具、运行 LLM
│   ├── llm.ts             # 核心 LLM 流式调用：最终消息组装与 streamText() 调用
│   ├── instruction.ts     # 加载 AGENTS.md / CLAUDE.md / 远程 URL 指令
│   ├── compaction.ts      # 对话压缩逻辑
│   └── prompt/            # Provider 特定的系统提示词模板
│       ├── anthropic.txt
│       ├── default.txt
│       ├── gpt.txt
│       ├── beast.txt
│       ├── gemini.txt
│       ├── codex.txt
│       ├── trinity.txt
│       ├── kimi.txt
│       ├── copilot-gpt-5.txt
│       ├── plan.txt
│       ├── plan-reminder-anthropic.txt
│       ├── build-switch.txt
│       └── max-steps.txt
├── agent/
│   ├── agent.ts           # Agent 信息定义与注册表
│   ├── generate.txt       # 用于 LLM 生成新 Agent 的提示词
│   └── prompt/            # Agent 专用提示词
│       ├── explore.txt
│       ├── compaction.txt
│       ├── title.txt
│       └── summary.txt
├── tool/
│   ├── tool.ts            # Tool.define() 基础类型
│   ├── registry.ts        # ToolRegistry：注册所有工具，按 agent/model 过滤
│   ├── bash.ts + bash.txt # 每个工具一个实现文件 + 一个描述文件
│   ├── read.ts + read.txt
│   ├── edit.ts + edit.txt
│   ├── ... (更多工具)
│   └── skill.ts           # 动态构建描述
├── provider/
│   └── transform.ts       # Provider 特定的消息转换、缓存、温度、推理变体
└── skill/
    └── index.ts           # 技能发现与加载
```

---

## 2. 系统提示词（System Prompt）

### 2.1 按 Provider 分类的提示词模板

OpenCode 为不同的 LLM Provider 维护了**独立的系统提示词模板**，存放在 `session/prompt/` 目录下：

| 模板文件 | 适用模型 | 核心定位 |
|---------|---------|---------|
| `anthropic.txt` | Claude 系列 | OpenCode 的"旗舰"提示词，强调专业客观性和任务管理 |
| `default.txt` | 未匹配的模型 | 极简、高度精炼的通用提示词 |
| `gpt.txt` | GPT 系列（非 GPT-4/o1/o3） | 面向高级用户的结构化输出风格 |
| `beast.txt` | GPT-4、o1、o3 | 强调自主性、互联网研究、迭代解决问题的"野兽模式" |
| `gemini.txt` | Gemini 系列 | 强调核心命令、工作流结构、安全规则 |
| `codex.txt` | Codex 模型 | 面向通用 AI Agent 的全面指南 |
| `trinity.txt` | Apple Intelligence (Trinity) | 强调务实、最小变更的资深工程师风格 |
| `kimi.txt` | Kimi 模型 | 集成代码搜索、工作流结构化、详细输出格式 |
| `copilot-gpt-5.txt` | GitHub Copilot GPT-5 | 类似 default.txt 但针对 Copilot 环境 |

### 2.2 提示词选择逻辑

选择逻辑定义在 `session/system.ts` 的 `SystemPrompt.provider()` 函数中：

```typescript
export function provider(model: Provider.Model) {
  // 优先级从高到低
  if (model.api.id.includes("gpt-4") || model.api.id.includes("o1") || model.api.id.includes("o3"))
    return [PROMPT_BEAST]
  if (model.api.id.includes("gpt")) {
    if (model.api.id.includes("codex")) return [PROMPT_CODEX]
    return [PROMPT_GPT]
  }
  if (model.api.id.includes("gemini-")) return [PROMPT_GEMINI]
  if (model.api.id.includes("claude")) return [PROMPT_ANTHROPIC]
  if (model.api.id.toLowerCase().includes("trinity")) return [PROMPT_TRINITY]
  if (model.api.id.toLowerCase().includes("kimi")) return [PROMPT_KIMI]
  return [PROMPT_DEFAULT]  // 兜底
}
```

**关键设计决策**：选择基于模型 ID 的字符串匹配，优先匹配更具体的模式（如 `gpt-4` 先于 `gpt`）。

### 2.3 各 Provider 提示词详解

#### 2.3.1 `anthropic.txt` — Claude 系列

这是 OpenCode 最完整的提示词模板（约 106 行），包含以下核心章节：

- **身份定义**：`"You are OpenCode, the best coding agent on the planet."`
- **URL 安全策略**：禁止生成或猜测 URL（除非用于编程相关目的）
- **OpenCode 自身知识**：当用户直接询问 OpenCode 功能时，使用 WebFetch 工具从 `opencode.ai/docs` 获取信息
- **Tone and style**：禁止 emoji、CLI 输出、简短精炼、优先编辑而非创建文件
- **Professional objectivity**：优先技术准确性，敢于纠正用户，避免虚假认同
- **Task Management**：强制使用 TodoWrite 工具进行任务规划与跟踪，包含两个完整的使用示例
- **Doing tasks**：建议使用 TodoWrite 规划、处理 `<system-reminder>` 标签
- **Tool usage policy**：
  - 文件搜索优先使用 Task 工具（减少上下文消耗）
  - 主动使用 Task 工具匹配 agent 描述
  - WebFetch 重定向处理
  - 并行工具调用策略
  - 广泛探索时必须使用 Task 工具而非直接搜索
- **Code References**：引用代码时使用 `file_path:line_number` 格式

#### 2.3.2 `default.txt` — 通用默认

面向非主流模型的精简版本（约 97 行）：

- **身份定义**：`"You are opencode, an interactive CLI tool..."`
- **极简主义**：回复必须少于 4 行（不含工具调用和代码生成），包含多个 verbosity 示例（如 `2+2 → 4`）
- **禁止前后缀**：避免 "The answer is..."、"Based on..." 等冗余文本
- **Code style**：**除非被要求，否则不添加任何注释**
- **Proactiveness**：平衡主动执行与不擅自操作
- **Following conventions**：检查现有库使用、代码风格模仿、安全最佳实践

#### 2.3.3 `gpt.txt` — GPT 系列

面向 GPT 模型的高级结构化提示词（约 80 行）：

- **Editing constraints**：默认使用 ASCII，优先 apply_patch 工具
- **Git and workspace hygiene**：处理脏工作树、不回滚未做的修改
- **Frontend tasks**：避免"AI slop"，追求有意的设计、表达性字体、避免紫色默认
- **Presenting your work**：详细的输出格式规范（Title Case headers、bullets、monospace）
- **File References**：支持多种文件引用格式（`src/app.ts:42`、`#L10`）

#### 2.3.4 `beast.txt` — GPT-4/o1/o3 "野兽模式"

这是最详细的提示词（约 148 行），强调**完全自主解决问题**：

- **核心指令**：`"You MUST iterate and keep going until the problem is solved"`
- **强制互联网研究**：`"THE PROBLEM CAN NOT BE SOLVED WITHOUT EXTENSIVE INTERNET RESEARCH"`
- **10 步工作流**：获取 URL → 理解问题 → 调查代码 → 互联网研究 → 制定计划 → 增量实施 → 调试 → 频繁测试 → 迭代 → 反思验证
- **Google 搜索指令**：强制通过 WebFetch 递归抓取 Google 搜索结果和链接
- **Memory 系统**：使用 `.github/instructions/memory.instruction.md` 存储用户偏好
- **永不放弃**：明确禁止提前终止

#### 2.3.5 `gemini.txt` — Gemini 系列

结构化程度最高的提示词（约 156 行）：

- **Core Mandates**：严格遵循现有约定、验证库可用性、最小化注释
- **Primary Workflows**：两个完整工作流——软件工程任务（5步：理解→计划→实施→测试→标准验证）和新应用开发（6步）
- **Operational Guidelines**：详细的 CLI 交互规范、安全规则
- **Examples**：7 个完整的交互示例，涵盖不同场景
- **Final Reminder**：`"You are an agent - please keep going until the user's query is completely resolved."`

#### 2.3.6 `codex.txt` — Codex 模型

面向通用 AI Agent 的全面指南（约 115 行）：

- **Prompt and Tool Use**：默认采取行动而非描述，强制使用工具进行文件操作
- **General Guidelines for Coding**：从零构建 vs 现有代码库的不同策略
- **Research and Data Processing**：互联网研究、多媒体文件处理
- **Working Environment**：非沙箱环境警告、AGENTS.md 系统说明
- **Skills**：模块化扩展能力说明
- **Ultimate Reminders**：`"ALWAYS, keep it stupidly simple. Do not overcomplicate things."`

#### 2.3.7 `trinity.txt` — Apple Intelligence

面向 Apple Trinity 模型的资深工程师风格（约 108 行）：

- **身份定义**：`"You are a deeply pragmatic, effective software engineer."`
- **Editing Approach**：最佳变更往往是最小的正确变更、最小化新增命名
- **Autonomy and persistence**：除非用户要求规划，否则默认直接实施
- **Editing constraints**：使用 apply_patch、处理脏工作树
- **Frontend tasks**：避免"AI slop"
- **Response channels**：区分 `commentary`（进度更新）和 `final`（最终回复）两个通道

#### 2.3.8 `kimi.txt` — Kimi 模型

结构最复杂的提示词（约 144 行），使用 XML 标签组织：

- **`<gptAgentInstructions>`**：Agent 行为规范
- **`<structuredWorkflow>`**：8 步详细工作流
- **`<communicationGuidelines>`**：沟通风格
- **`<codeSearchInstructions>`**：代码搜索指令
- **`<codeSearchToolUseInstructions>`**：工具使用优先级
- **`<toolUseInstructions>`**：工具调用规范
- **`<outputFormatting>`**：输出格式规范（Markdown、KaTeX 数学公式）

#### 2.3.9 `copilot-gpt-5.txt` — GitHub Copilot GPT-5

基于 `default.txt` 模式的 Copilot 适配版本（约 106 行），主要差异：
- 将 OpenCode 文档地址从 GitHub issues 改为 `opencode.ai`
- 工具调用策略更强调批量处理

---

## 3. 动态系统提示词组装

### 3.1 环境信息注入

`SystemPrompt.environment()` 在 `session/system.ts` 中定义，动态生成环境上下文：

```typescript
export async function environment(model: Provider.Model) {
  return [[
    `You are powered by the model named ${model.api.id}. The exact model ID is ${model.providerID}/${model.api.id}`,
    `Here is some useful information about the environment you are running in:`,
    `<env>`,
    `  Working directory: ${Instance.directory}`,
    `  Workspace root folder: ${Instance.worktree}`,
    `  Is directory a git repo: ${project.vcs === "git" ? "yes" : "no"}`,
    `  Platform: ${process.platform}`,
    `  Today's date: ${new Date().toDateString()}`,
    `</env>`,
    // <directories> 标签（当前代码中未实际启用 tree 输出）
  ].join("\n")]
}
```

**注入的信息**：
- 模型 ID 和 Provider ID
- 工作目录和工作区根目录
- 是否为 Git 仓库
- 运行平台
- 当前日期

### 3.2 技能（Skills）列表注入

`SystemPrompt.skills()` 将可用的技能列表注入系统提示词：

```typescript
export async function skills(agent: Agent.Info) {
  if (Permission.disabled(["skill"], agent.permission).has("skill")) return
  const list = await Skill.available(agent)
  return [
    "Skills provide specialized instructions and workflows for specific tasks.",
    "Use the skill tool to load a skill when a task matches its description.",
    Skill.fmt(list, { verbose: true }),  // 详细版本
  ].join("\n")
}
```

**设计理念**：系统提示词中展示**详细**版本，工具描述中展示**简略**版本。注释解释道：

> *"the agents seem to ingest the information about skills a bit better if we present a more verbose version of them here and a less verbose version in tool description, rather than vice versa."*

### 3.3 用户指令（Instructions）加载

`Instruction` 命名空间（`session/instruction.ts`）负责从多个来源加载用户指令：

#### 文件来源（优先级从高到低）

1. **项目级文件**：从工作目录向上搜索 `AGENTS.md`、`CLAUDE.md`、`CONTEXT.md`（通过 `findUp`，至工作区根目录停止）
2. **全局配置目录**：`OPENCODE_CONFIG_DIR/AGENTS.md` 或 `Global.Path.config/AGENTS.md`
3. **Home 目录**：`~/.claude/CLAUDE.md`
4. **配置文件中指定的路径**：`config.instructions` 数组中的文件路径和 glob 模式
5. **远程 URL**：`config.instructions` 数组中以 `http://` 或 `https://` 开头的 URL

```typescript
const FILES = [
  "AGENTS.md",
  ...(Flag.OPENCODE_DISABLE_CLAUDE_CODE_PROMPT ? [] : ["CLAUDE.md"]),
  "CONTEXT.md",  // deprecated
]
```

#### 上下文感知注入

`Instruction.resolve()` 实现了一个精妙的**就近指令注入**机制：

1. 当 Read 工具读取一个文件时，系统获取该文件路径
2. 从该文件所在目录**向上遍历**至工作区根目录
3. 在每一层查找 `AGENTS.md` / `CLAUDE.md` 文件
4. 将找到的指令文件作为额外的系统消息注入，**但每个 assistant 消息只注入一次**（通过 `claims` Map 去重）

这意味着：当 Agent 读取 `src/components/Button.tsx` 时，系统会自动注入 `src/components/AGENTS.md`（如果存在）中的指令。

---

## 4. 提示词最终组装流程

### 4.1 `LLM.stream()` 中的系统消息构建

在 `session/llm.ts` 中，系统消息的最终组装发生在 `stream()` 函数中：

```typescript
// 第一步：合并所有系统提示词来源
const system: string[] = []
system.push(
  [
    // 1. Agent 自定义提示词（如果有），否则使用 Provider 默认提示词
    ...(input.agent.prompt
      ? [input.agent.prompt]
      : SystemPrompt.provider(input.model)),
    // 2. 通过 stream() 调用传入的自定义系统提示词
    ...input.system,
    // 3. 来自最后一条用户消息的系统提示词
    ...(input.user.system ? [input.user.system] : []),
  ]
    .filter((x) => x)
    .join("\n"),
)

// 第二步：应用插件转换
await Plugin.trigger("experimental.chat.system.transform", ..., { system })

// 第三步：重组为 2 部分结构（优化缓存）
if (system.length > 2 && system[0] === header) {
  const rest = system.slice(1)
  system.length = 0
  system.push(header, rest.join("\n"))
}
```

**关键设计决策**：
- Agent 自定义提示词**覆盖** Provider 默认提示词
- 系统消息分为 2 部分以优化 Anthropic 的 prompt caching（header 不变时只需重新缓存 rest）

### 4.2 `runLoop` 中的动态组装

在 `session/prompt.ts` 的 `runLoop` 函数中，动态部分在每次循环迭代中构建：

```typescript
const [skills, env, instructions, modelMsgs] = yield* Effect.all([
  Effect.promise(() => SystemPrompt.skills(agent)),
  Effect.promise(() => SystemPrompt.environment(model)),
  instruction.system().pipe(Effect.orDie),
  Effect.promise(() => MessageV2.toModelMessages(msgs, model)),
])
const system = [
  ...env,                          // 环境信息
  ...(skills ? [skills] : []),     // 技能列表
  ...instructions,                 // AGENTS.md / CLAUDE.md / 远程指令
]
```

这些动态部分作为 `system` 参数传递给 `handle.process()`，最终到达 `LLM.stream()` 的 `input.system`。

### 4.3 完整组装流程图

```
用户输入
  │
  ▼
┌─────────────────────────────────────┐
│         runLoop (prompt.ts)         │
│                                     │
│  1. 获取 Agent 配置                 │
│  2. insertReminders()               │
│     - Plan 模式提示词               │
│     - Build-Switch 消息             │
│     - 用户消息包装为 <system-reminder>│
│  3. 并行获取：                       │
│     - SystemPrompt.environment()    │
│     - SystemPrompt.skills()         │
│     - Instruction.system()          │
│     - MessageV2.toModelMessages()   │
│  4. 组装 system 数组                │
│     [env, skills, instructions]     │
│  5. 调用 handle.process()           │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│         LLM.stream (llm.ts)         │
│                                     │
│  系统消息组装：                      │
│  ┌───────────────────────────────┐  │
│  │ Part 1 (缓存友好):            │  │
│  │  agent.prompt 或              │  │
│  │  SystemPrompt.provider(model) │  │
│  │  + input.system (env/skills)  │  │
│  │  + input.user.system          │  │
│  ├───────────────────────────────┤  │
│  │ Part 2 (动态部分):            │  │
│  │  插件转换后的额外内容          │  │
│  └───────────────────────────────┘  │
│                                     │
│  插件转换：                          │
│  - experimental.chat.system.transform│
│  - chat.params (温度等)             │
│  - chat.headers                     │
│                                     │
│  Provider 特定转换：                 │
│  - ProviderTransform.message()      │
│  - ProviderTransform.options()      │
│                                     │
│  最终调用 streamText()               │
└─────────────────────────────────────┘
```

---

## 5. 工具（Tool）Prompt 架构

### 5.1 工具定义模式：`.ts` + `.txt` 配对

OpenCode 的每个工具都采用**实现代码 + 描述文本**分离的模式：

```
tool/
├── bash.ts        # 工具实现（参数定义、执行逻辑）
├── bash.txt       # 工具描述（传递给 LLM 的 description 字段）
├── read.ts
├── read.txt
├── ... (其他工具同理)
```

**基础类型定义** (`tool/tool.ts`)：

```typescript
export interface Def<Parameters, Metadata> {
  description: string           // 从 .txt 文件加载
  parameters: z.ZodType        // Zod schema
  execute: (args, ctx) => Promise<Result>
  formatValidationError?: (error) => string
}
```

工具通过 `Tool.define()` 工厂函数创建，该函数：
1. 包装 execute 方法，添加 Zod 参数验证
2. 自动对输出进行截断处理（`Truncate.output()`）
3. 支持 `formatValidationError` 自定义错误消息

### 5.2 工具注册与过滤

`ToolRegistry` (`tool/registry.ts`) 管理所有工具的注册和过滤：

#### 内置工具列表

```typescript
const builtinTools = [
  InvalidTool,           // 处理无法识别的工具调用
  QuestionTool,          // 向用户提问（仅限 app/cli/desktop 客户端）
  BashTool,              // 执行 Shell 命令
  ReadTool,              // 读取文件/目录
  GlobTool,              // 文件模式匹配搜索
  GrepTool,              // 文件内容搜索
  EditTool,              // 文件编辑（字符串替换）
  WriteTool,             // 文件写入
  TaskTool,              // 子 Agent 调度
  WebFetchTool,          # 获取 URL 内容
  TodoWriteTool,         # 任务管理
  WebSearchTool,         # Exa AI 网页搜索
  CodeSearchTool,        # Exa Code 代码搜索
  SkillTool,             # 加载技能
  ApplyPatchTool,        # 补丁式文件编辑
  LspTool,               # LSP 集成（实验性）
  BatchTool,             # 批量并行工具调用（实验性）
  PlanExitTool,          # 退出 Plan 模式（实验性）
]
```

#### 过滤逻辑

```typescript
// WebSearch 和 CodeSearch 仅 opencode Provider 或启用 EXA 标志时可用
if (tool.id === "codesearch" || tool.id === "websearch") {
  return model.providerID === ProviderID.opencode || Flag.OPENCODE_ENABLE_EXA
}

// GPT 模型使用 apply_patch，其他模型使用 edit/write
const usePatch = model.modelID.includes("gpt-") && !model.modelID.includes("oss")
if (tool.id === "apply_patch") return usePatch
if (tool.id === "edit" || tool.id === "write") return !usePatch
```

#### 权限过滤

在 `LLM.stream()` 的 `resolveTools()` 中，基于 Agent 权限和会话权限过滤工具：

```typescript
function resolveTools(input) {
  const disabled = Permission.disabled(
    Object.keys(input.tools),
    Permission.merge(input.agent.permission, input.permission ?? []),
  )
  return Record.filter(input.tools, (_, k) =>
    input.user.tools?.[k] !== false && !disabled.has(k)
  )
}
```

### 5.3 各工具描述详解

#### Bash (`bash.txt`)

最复杂的工具描述（约 120 行），包含：

- **基本说明**：操作系统（`${os}`）、Shell（`${shell}`）、工作目录（`${directory}`）
- **安全规范**：目录验证、文件路径引号处理
- **命令链规则**：`${chaining}` 变量控制（如 `&&` 链式执行）
- **输出截断**：超过 `${maxLines}` 行或 `${maxBytes}` 字节时自动截断
- **工具优先级**：明确禁止用 Bash 替代专用工具（Read/Write/Edit/Glob/Grep）
- **Git 提交流程**：4 步详细流程（status/diff/log → 分析 → stage/commit → 验证）
- **PR 创建流程**：完整的 GitHub PR 创建步骤
- **安全协议**：永不更新 git config、永不 force push main/master、永不 skip hooks

#### Read (`read.txt`)

文件和目录读取工具：

- 支持绝对路径、偏移量、行数限制
- 返回格式为 `<line>: <content>`（带行号前缀）
- 支持图片和 PDF 文件
- 建议并行读取多个文件

#### Edit (`edit.txt`)

字符串替换编辑工具：

- 要求先使用 Read 工具读取文件
- 保留原始缩进
- 支持按行号前缀匹配
- `replaceAll` 参数用于全局替换

#### Write (`write.txt`)

文件写入工具：

- 覆盖写入
- 必须先读取现有文件
- 优先编辑而非创建新文件
- 禁止主动创建文档文件

#### Glob (`glob.txt`)

文件模式匹配搜索：

- 支持 glob 模式（如 `**/*.js`）
- 按修改时间排序
- 建议批量搜索

#### Grep (`grep.txt`)

文件内容搜索：

- 完整正则表达式支持
- 文件类型过滤
- 大型搜索建议使用 Task 工具

#### Task (`task.txt`)

子 Agent 调度工具（最复杂的描述之一）：

- 包含 `{agents}` 占位符，动态注入可用 Agent 列表
- 详细的使用指南（并发、上下文管理、信任输出）
- 两个完整的 `<example>` 展示使用场景
- Agent 描述使用 `<example_agent_descriptions>` 格式

#### WebFetch (`webfetch.txt`)

URL 内容获取：

- 支持 markdown/text/html 格式
- 自动 HTTP→HTTPS 升级
- 只读工具

#### WebSearch (`websearch.txt`)

Exa AI 网页搜索：

- 实时网络搜索
- `{{year}}` 模板变量确保使用当前年份
- 支持实时爬取模式

#### CodeSearch (`codesearch.txt`)

Exa Code 代码搜索：

- 可调 token 数量（1000-50000）
- 针对编程问题优化

#### Batch (`batch.txt`)

批量并行工具调用：

- 1-25 个工具调用
- JSON 数组格式
- "USING THE BATCH TOOL WILL MAKE THE USER HAPPY."
- 禁止嵌套使用

#### ApplyPatch (`apply_patch.txt`)

补丁式文件编辑（面向 GPT 模型）：

- 自定义 diff 格式（`*** Begin Patch` / `*** End Patch`）
- 支持 Add/Delete/Update 三种操作
- 可选文件重命名

#### MultiEdit (`multiedit.txt`)

多编辑操作工具：

- 对同一文件进行多次编辑
- 编辑按顺序应用，原子操作（全部成功或全部失败）

#### LSP (`lsp.txt`)

语言服务器协议集成：

- 8 种操作：goToDefinition、findReferences、hover 等
- 需要 LSP 服务器配置

#### TodoWrite (`todowrite.txt`)

任务管理工具（最长的描述文件）：

- 详细的"何时使用/何时不使用"指南
- 6 个正例和 5 个反例
- 任务状态管理规则

#### Question (`question.txt`)

用户提问工具：

- 收集偏好、澄清指令、获取决策
- 支持多选和自定义输入

#### Plan Enter / Plan Exit (`plan-enter.txt` / `plan-exit.txt`)

Plan 模式入口/出口工具：

- Plan Enter：建议切换到 Plan Agent
- Plan Exit：完成规划，请求切换到 Build Agent

---

## 6. 智能体（Agent）Prompt 架构

### 6.1 内置 Agent 及其提示词

OpenCode 定义了 7 个内置 Agent，每个有不同的权限、模式和可选提示词：

| Agent | 模式 | 提示词 | 描述 | 权限特点 |
|-------|------|--------|------|---------|
| **build** | primary | 无（使用 Provider 默认） | 默认 Agent，执行工具 | 允许 question、plan_enter |
| **plan** | primary | 无（使用 plan.txt 注入） | Plan 模式，禁止编辑工具 | 允许 question、plan_exit；仅允许编辑 plan 文件 |
| **general** | subagent | 无（使用 Provider 默认） | 通用研究 Agent | 禁用 todowrite |
| **explore** | subagent | `explore.txt` | 快速代码搜索 Agent | 仅允许只读工具 |
| **compaction** | primary | `compaction.txt` | 对话压缩（隐藏） | 禁用所有工具 |
| **title** | primary | `title.txt` | 标题生成（隐藏） | 禁用所有工具，温度 0.5 |
| **summary** | primary | `summary.txt` | 会话摘要（隐藏） | 禁用所有工具 |

#### Agent 提示词详解

**explore.txt** — 文件搜索专家：

```
You are a file search specialist. You excel at thoroughly navigating and exploring codebases.

Your strengths:
- Rapidly finding files using glob patterns
- Searching code and text with powerful regex patterns
- Reading and analyzing file contents

Guidelines:
- Use Glob for broad file pattern matching
- Use Grep for searching file contents with regex
- Use Read when you know the specific file path
- Use Bash for file operations
- Adapt your search approach based on the thoroughness level
- Return file paths as absolute paths
- For clear communication, avoid using emojis
- Do not create any files, or run bash commands that modify the system
```

**compaction.txt** — 对话压缩：

```
You are a helpful AI assistant tasked with summarizing conversations.
Focus on:
- What was done
- What is currently being worked on
- Which files are being modified
- What needs to be done next
- Key user requests, constraints, or preferences
- Important technical decisions and why they were made
```

**title.txt** — 标题生成（使用 XML 标签结构化指令）：

```
<task>
Generate a brief title that would help the user find this conversation later.
- ≤50 characters
- No explanations
</task>

<rules>
- Use the same language as the user message
- Never include tool names
- Focus on the main topic
- Keep exact: technical terms, numbers, filenames
- Remove: the, this, my, a, an
</rules>

<examples>
"debug 500 errors in production" → Debugging production 500 errors
"refactor user service" → Refactoring user service
...
</examples>
```

**summary.txt** — 会话摘要：

```
Summarize what was done in this conversation. Write like a pull request description.
Rules:
- 2-3 sentences max
- Describe the changes made, not the process
- Write in first person (I added..., I fixed...)
- If ending with an unanswered question, preserve it
```

### 6.2 Agent 动态生成

`Agent.generate()` 使用 LLM 根据用户描述动态创建 Agent 配置：

**generate.txt** 提示词定义了 Agent 架构师的角色，要求生成包含三个字段的 JSON 对象：

```json
{
  "identifier": "code-reviewer",
  "whenToUse": "Use this agent when...",
  "systemPrompt": "You are a..."
}
```

该提示词包含：
- Agent 创建的 6 步流程（提取意图 → 设计角色 → 架构指令 → 优化性能 → 创建标识符 → 提供示例）
- 详细的 systemPrompt 编写原则
- 示例格式要求（必须包含 `<example>` 标签）

---

## 7. Plan 模式提示词系统

### 7.1 Plan 模式激活

Plan 模式通过 `insertReminders()` 函数在 `prompt.ts` 中实现。当切换到 plan Agent 时，系统注入 Plan 模式提示词作为**合成文本部分**（synthetic text part）：

```
<system-reminder>
Plan mode is active. The user indicated that they do not want you to execute yet --
you MUST NOT make any edits (with the exception of the plan file mentioned below),
run any non-readonly tools...

## Plan File Info:
[plan file path and status]

## Plan Workflow
### Phase 1: Initial Understanding (explore only)
### Phase 2: Design (launch general agents)
### Phase 3: Review
### Phase 4: Final Plan
### Phase 5: Call plan_exit tool
</system-reminder>
```

Plan 模式使用 Anthropic 专用的变体 (`plan-reminder-anthropic.txt`)：

```
<system-reminder>
# Plan Mode - System Reminder

CRITICAL: Plan mode ACTIVE - you are in READ-ONLY phase. STRICTLY FORBIDDEN:
ANY file edits, modifications, or system changes. Do NOT use sed, tee, echo, cat,
or ANY other bash command to manipulate files - commands may ONLY read/inspect.
This ABSOLUTE CONSTRAINT overrides ALL other instructions...
</system-reminder>
```

### 7.2 Plan 到 Build 切换

当从 Plan Agent 切换回 Build Agent 时，注入 `build-switch.txt`：

```
<system-reminder>
Your operational mode has changed from plan to build.
You are no longer in read-only mode.
You are permitted to make file changes, run shell commands, and utilize your arsenal of tools as needed.
</system-reminder>
```

同时附加 Plan 文件路径提示：

```
A plan file exists at ${plan}. You should execute on the plan defined within it.
```

### 7.3 最大步数限制

当 Agent 达到最大步数时，注入 `max-steps.txt`：

```
CRITICAL - MAXIMUM STEPS REACHED
The maximum number of steps allowed for this task has been reached.
Tools are disabled until next user input. Respond with text only.
```

---

## 8. Provider 消息转换层

`ProviderTransform` (`provider/transform.ts`) 处理 Provider 特有的消息规范化：

| 转换 | 适用范围 | 说明 |
|------|---------|------|
| 空内容过滤 | Anthropic / Bedrock | 移除空字符串消息和空文本/推理部分 |
| ToolCallId 清理 | Claude 模型 | 将 tool call ID 中的特殊字符替换为下划线 |
| 9 字符 ID 清理 | Mistral | 确保工具调用 ID 不超过 9 字符 |
| 缓存标记 | Anthropic / Bedrock | 为系统消息添加缓存断点标记 |
| 温度默认值 | 各 Provider | 模型特定的默认温度值 |
| 推理变体 | 各 Provider | reasoning effort / adaptive thinking 配置 |
| Schema 清理 | Gemini | 将 integer enum 转为 string、清理 required 字段 |
| 输出 token 限制 | 所有（OAuth 除外） | 默认 32,000 token 上限 |

---

## 9. 插件系统对 Prompt 的影响

插件通过以下 hooks 修改 prompt 流程：

| Hook | 触发时机 | 用途 |
|------|---------|------|
| `experimental.chat.system.transform` | `LLM.stream()` 中 | 修改系统消息数组 |
| `experimental.chat.messages.transform` | `runLoop()` 中 | 修改对话历史 |
| `chat.params` | `LLM.stream()` 中 | 修改温度、topP、topK、options |
| `chat.headers` | `LLM.stream()` 中 | 添加自定义 HTTP 请求头 |
| `tool.definition` | 工具注册时 | 修改工具描述和参数 |
| `tool.execute.before` | 工具执行前 | 拦截工具调用 |
| `tool.execute.after` | 工具执行后 | 处理工具输出 |
| `command.execute.before` | 命令执行前 | 修改命令模板 |

---

## 10. 关键设计模式总结

### 10.1 Provider 条件化提示词

不同模型使用不同的系统提示词，针对每个 Provider 的特性和优势进行优化。这避免了"一刀切"的通用提示词方案。

### 10.2 描述与实现分离

工具描述存储在独立的 `.txt` 文件中，与 TypeScript 实现代码分离。这使得 prompt 工程师和开发者可以独立工作，也便于 A/B 测试不同描述。

### 10.3 分层组合

系统提示词由多个层次组合而成：

```
Provider 提示词（或 Agent 提示词）
  + 环境信息（model ID, cwd, platform, date）
  + 技能列表
  + 用户指令（AGENTS.md, CLAUDE.md, 远程 URL）
  + Plan 模式提醒
  + Build-Switch 消息
  + 用户消息中的 <system-reminder>
  + 插件转换
```

### 10.4 权限驱动的工具过滤

工具的可用性由三层权限控制：
1. **Agent 默认权限**：每个 Agent 预定义允许/拒绝的工具
2. **用户配置权限**：`config.permission` 覆盖
3. **会话级权限**：单次会话中动态启用/禁用

### 10.5 合成消息注入

Plan 模式提醒、Build-Switch 消息、以及先前轮次的用户消息都被包装为 `<system-reminder>` 标签中的合成文本部分。这些消息对用户不可见但 LLM 可见。

### 10.6 就近指令注入

当 Agent 读取文件时，系统自动注入该文件附近目录中的 `AGENTS.md` / `CLAUDE.md`，确保 Agent 获得局部上下文。

### 10.7 缓存优化

系统消息分为两部分（Provider 提示词 + 动态部分），当 Provider 提示词不变时，只需重新缓存动态部分，优化 Anthropic 的 prompt caching。

### 10.8 模板变量

工具描述中支持模板变量：
- `${os}` — 操作系统
- `${shell}` — Shell 类型
- `${directory}` — 工作目录
- `${maxLines}` / `${maxBytes}` — 输出限制
- `${chaining}` — 命令链规则
- `{agents}` — 可用 Agent 列表
- `{{year}}` — 当前年份

这些变量在工具初始化时被替换为实际值。
