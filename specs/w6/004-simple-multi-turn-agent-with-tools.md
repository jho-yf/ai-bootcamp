# 004 - 构建简单的 Multi-Turn Agent with Tools

> 基于 OpenCode (`vendors/opencode/`) 代码库的深入分析，提炼其 coding agent 核心设计模式，为构建一个简单的 multi-turn agent with tools 提供架构蓝图。

---

## 目录

1. [OpenCode Agent 核心架构提炼](#1-opencode-agent-核心架构提炼)
2. [简化版架构设计](#2-简化版架构设计)
3. [核心组件详细设计](#3-核心组件详细设计)
4. [数据流与核心循环](#4-数据流与核心循环)
5. [关键设计决策与取舍](#5-关键设计决策与取舍)
6. [实现建议](#6-实现建议)

---

## 1. OpenCode Agent 核心架构提炼

### 1.1 Agent 不是对象，而是配置

OpenCode 的 Agent 并非一个有状态的对象（class），而是一个**声明式配置对象** `Agent.Info`，通过 Effect-ts 的 Service/Layer 模式提供服务。

**源码**: `packages/opencode/src/agent/agent.ts:28-53`

```typescript
// Agent 本质上是一组声明式配置
const Info = z.object({
  name: z.string(),
  description: z.string().optional(),
  mode: z.enum(["subagent", "primary", "all"]),  // 角色定位
  prompt: z.string().optional(),                   // 自定义系统提示词
  model: z.object({ modelID, providerID }).optional(),
  permission: Permission.Ruleset.zod,              // 工具权限规则
  steps: z.number().int().positive().optional(),   // 最大循环步数
  temperature: z.number().optional(),
  topP: z.number().optional(),
})
```

**核心洞察**：Agent 的"智能"不在于 Agent 对象本身，而在于：
1. **系统提示词**（定义行为准则）
2. **可用工具集**（定义能力边界）
3. **权限规则**（定义安全边界）
4. **运行循环**（定义执行流程）

### 1.2 工具的定义-注册-执行三层分离

OpenCode 的工具系统分为三个清晰的层次：

**第一层：定义** (`packages/opencode/src/tool/tool.ts:36-42`)

```typescript
interface Def<Parameters, Metadata> {
  id: string                    // 工具唯一标识
  description: string           // 传递给 LLM 的描述文本
  parameters: z.ZodType         // Zod schema 定义参数
  execute(args, ctx): Promise<ExecuteResult>  // 执行函数
}
```

**第二层：注册** (`packages/opencode/src/tool/registry.ts`)

ToolRegistry 管理所有工具实例，按 agent 权限和 model 能力过滤可用工具。

**第三层：解析与执行** (`packages/opencode/src/session/prompt.ts:357-527`)

`resolveTools()` 将 `Tool.Def` 转换为 Vercel AI SDK 的 `Tool` 对象，为每个工具创建 `Tool.Context` 闭包（包含权限检查、元数据回调等），然后传给 `streamText()`。

### 1.3 核心循环：while(true) + 流式处理

**源码**: `packages/opencode/src/session/prompt.ts:1312-1541`

```
┌─────────────────────────────────────────────────────┐
│  runLoop() = while (true) {                         │
│                                                     │
│    1. 加载消息历史                                   │
│    2. 检查退出条件                                   │
│       - 最后一条 assistant 消息完成（非 tool-calls） │
│       - 无待处理工具调用                             │
│       → break                                       │
│                                                     │
│    3. 处理待办任务                                   │
│       - 子任务 (subtask)                             │
│       - 上下文压缩 (compaction)                      │
│       - 上下文溢出检测                               │
│                                                     │
│    4. 解析工具集                                     │
│    5. 调用 LLM（流式）                               │
│    6. 处理流式事件                                   │
│       - text → 流式文本                              │
│       - tool-call → 执行工具                         │
│       - tool-result → 记录结果                       │
│       - finish → 检查是否需要继续                     │
│                                                     │
│    7. 根据处理结果决定：                              │
│       "continue" → 继续循环                          │
│       "stop"     → break                             │
│       "compact"  → 触发压缩后继续                     │
│  }                                                   │
└─────────────────────────────────────────────────────┘
```

**退出条件**（`prompt.ts:1352-1360`）：

循环在以下条件全部满足时退出：
1. 最后一条 assistant 消息有 `finish` 原因，且该原因不是 `"tool-calls"` 或 `"unknown"`
2. assistant 消息中没有待处理的工具调用
3. 最后一条用户消息的 ID < 最后一条 assistant 消息的 ID（即 assistant 已经响应了用户）

### 1.4 消息结构：Part-based 消息模型

OpenCode 使用 **Part-based 消息模型**，一条消息由多个类型化的 Part 组成：

**源码**: `packages/opencode/src/session/message-v2.ts`

```
Message
├── role: "user" | "assistant"
├── metadata (model, agent, cost, tokens, finish reason...)
└── parts: Part[]  ← 消息的核心内容
    ├── TextPart      { type: "text", text: "..." }
    ├── ToolPart      { type: "tool", state: "pending"|"running"|"completed"|"error", ... }
    ├── ReasoningPart { type: "reasoning", text: "..." }
    ├── FilePart      { type: "file", name, url, ... }
    └── ... (step-start, step-finish, snapshot, etc.)
```

**ToolPart 的状态机**：

```
pending  ──(tool-call event)──→  running  ──(tool-result)──→  completed
                                     │
                                     └──(error)──→  error
```

这种设计使得工具调用的每个阶段都可以独立持久化和流式更新。

### 1.5 工具调用端到端流程

```
用户消息
  │
  ▼
runLoop: while(true)
  │
  ├─→ resolveTools(): 将 Tool.Def[] 转为 AI SDK Tool Map
  │     为每个工具创建 execute 闭包：
  │     - 包装权限检查 (ctx.ask)
  │     - 包装元数据回调 (ctx.metadata)
  │     - 包装输出截断 (Truncate.output)
  │
  ├─→ llm.stream(): 调用 Vercel AI SDK streamText()
  │     参数: system[], messages[], tools, model, ...
  │
  ├─→ processor.handleEvent(): 处理流式事件
  │     │
  │     ├─ tool-input-start → 创建 pending ToolPart
  │     ├─ tool-call        → 更新为 running，执行 doom loop 检测
  │     │                      (同一工具相同参数连续 3 次 → 提示用户)
  │     ├─ [SDK 自动执行工具 execute 函数]
  │     │     │
  │     │     ├─ 权限检查 (ctx.ask)
  │     │     ├─ 实际执行 (如 Bash: spawn child process)
  │     │     ├─ 元数据实时更新 (ctx.metadata)
  │     │     └─ 返回 { title, output, metadata }
  │     │
  │     ├─ tool-result → 标记 completed
  │     └─ tool-error  → 标记 error
  │
  └─→ 根据 process 结果:
       "continue" → 消息历史中已有工具结果，继续循环
       "stop"     → break
       "compact"  → 压缩上下文后继续
```

### 1.6 提示词分层组合

系统提示词由多个层次动态组合（`llm.ts:99-124`）：

```
Part 1（缓存友好，很少变化）:
  ┌─ Agent 自定义 prompt 或 Provider 默认 prompt
  └─ 环境信息 (model ID, cwd, platform, date)

Part 2（动态部分，每次可能变化）:
  ├─ 技能列表 (skills)
  ├─ 用户指令 (AGENTS.md / CLAUDE.md)
  ├─ Plan 模式提醒
  └─ 插件注入内容
```

---

## 2. 简化版架构设计

去掉 OpenCode 中与"核心 Agent 功能"无关的复杂度（Effect-ts、插件系统、多 Provider、MCP、子 Agent、Plan 模式、压缩），保留核心骨架：

### 2.1 架构总览

```
┌──────────────────────────────────────────────────────┐
│                    SimpleAgent                        │
│                                                      │
│  ┌──────────┐   ┌──────────┐   ┌──────────────┐     │
│  │  Config   │   │  Tools   │   │  MessageStore│     │
│  │  - prompt │   │  - defs[]│   │  - messages[]│     │
│  │  - model  │   │  - exec()│   │  - append()  │     │
│  └──────────┘   └──────────┘   └──────────────┘     │
│                                                      │
│  ┌──────────────────────────────────────────────┐    │
│  │               runLoop()                       │    │
│  │  while (true) {                               │    │
│  │    messages → LLM → [text | tool_call] →     │    │
│  │    if tool_call: exec → append result → cont  │    │
│  │    if text only: break                        │    │
│  │  }                                            │    │
│  └──────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────┘
```

### 2.2 核心类型定义

```typescript
// ============ 消息类型 ============

interface Message {
  role: "system" | "user" | "assistant" | "tool"
  content: string
  // 仅 tool 角色使用
  toolCallId?: string
  toolName?: string
}

// LLM 返回的 assistant 消息可能包含工具调用
interface AssistantMessage {
  role: "assistant"
  content: string | null          // 文本回复（可能为空）
  toolCalls?: ToolCall[]          // 0~N 个工具调用
}

interface ToolCall {
  id: string                      // 工具调用 ID（由 LLM 生成）
  name: string                    // 工具名称
  arguments: Record<string, any>  // 工具参数（已解析的 JSON）
}

// ============ 工具定义 ============

interface ToolDefinition {
  name: string                    // 工具名称（与 LLM 看到的一致）
  description: string             // 传递给 LLM 的工具描述
  parameters: Record<string, any> // JSON Schema 格式的参数定义
  execute(args: Record<string, any>): Promise<string>  // 执行函数，返回字符串结果
}

// ============ Agent 配置 ============

interface AgentConfig {
  model: string                   // 模型 ID
  systemPrompt: string            // 系统提示词
  tools: ToolDefinition[]         // 可用工具集
  maxSteps?: number               // 最大循环步数（防止死循环）
}
```

---

## 3. 核心组件详细设计

### 3.1 工具系统

从 OpenCode 提炼的工具系统核心：

**OpenCode 的做法**（`tool/tool.ts:124-137`）：

```typescript
// OpenCode: Tool.define() 工厂函数
// 1. 接受 Zod schema 作为参数定义
// 2. 自动包装参数验证
// 3. 自动包装输出截断
// 4. 返回 Tool.Def 对象

export function define(id, init) {
  return Effect.gen(function* (_) {
    const { description, parameters, execute } = yield* _(init)
    return { id, description, parameters, execute: wrap(id, execute, parameters) }
  })
}
```

**简化版**：

```typescript
class ToolRegistry {
  private tools: Map<string, ToolDefinition> = new Map()

  register(tool: ToolDefinition) {
    this.tools.set(tool.name, tool)
  }

  get(name: string): ToolDefinition | undefined {
    return this.tools.get(name)
  }

  // 转换为 LLM API 需要的 tools 格式
  toApiFormat(): Array<{ type: "function"; function: {...} }> {
    return Array.from(this.tools.values()).map(tool => ({
      type: "function",
      function: {
        name: tool.name,
        description: tool.description,
        parameters: tool.parameters,
      }
    }))
  }
}
```

**示例工具**：

```typescript
// 读文件工具
const readTool: ToolDefinition = {
  name: "read_file",
  description: "Read the contents of a file at the given path",
  parameters: {
    type: "object",
    properties: {
      path: {
        type: "string",
        description: "Absolute path to the file to read"
      }
    },
    required: ["path"]
  },
  execute: async (args) => {
    const content = await fs.readFile(args.path, "utf-8")
    return content
  }
}

// 执行命令工具
const bashTool: ToolDefinition = {
  name: "bash",
  description: "Execute a shell command and return its output",
  parameters: {
    type: "object",
    properties: {
      command: {
        type: "string",
        description: "The shell command to execute"
      }
    },
    required: ["command"]
  },
  execute: async (args) => {
    const { stdout, stderr } = await execPromise(args.command, { timeout: 30000 })
    return stdout + (stderr ? `\nstderr:\n${stderr}` : "")
  }
}
```

### 3.2 消息管理

从 OpenCode 的 Part-based 模型简化为标准的 role-based 消息数组：

**OpenCode 的做法** (`message-v2.ts:585-838`)：

- 消息由多个 Part 组成（text, tool, reasoning, file...）
- 转换为 LLM 格式时需要将 Part 拆解为独立的 tool-call/tool-result 消息对
- 支持压缩（compaction）：旧工具输出替换为 `[Old tool result content cleared]`

**简化版**：直接使用 LLM API 的原生消息格式，无需 Part 模型。

```typescript
class MessageStore {
  private messages: Message[] = []

  addSystem(content: string) {
    this.messages.push({ role: "system", content })
  }

  addUser(content: string) {
    this.messages.push({ role: "user", content })
  }

  addAssistant(content: string | null, toolCalls?: ToolCall[]) {
    // 实际发送给 LLM 时，assistant 消息同时携带 content 和 tool_calls
    // 需要根据 LLM API 的格式要求进行适配
  }

  addToolResult(toolCallId: string, toolName: string, output: string) {
    this.messages.push({
      role: "tool",
      toolCallId,
      toolName,
      content: output,
    })
  }

  getAll(): Message[] {
    return [...this.messages]
  }
}
```

### 3.3 LLM 调用层

从 OpenCode 的 Vercel AI SDK + Effect-ts + 多 Provider 支持简化为直接的 API 调用：

**OpenCode 的做法** (`llm.ts:322-400`)：
- 使用 Vercel AI SDK 的 `streamText()` 实现流式调用
- 通过 Provider 层抽象多模型支持
- Effect-ts 管理副作用和依赖注入
- 插件系统在调用前后拦截

**简化版**：使用 OpenAI-compatible API（大多数 LLM 提供商都支持）。

```typescript
interface LLMResponse {
  content: string | null
  toolCalls?: ToolCall[]
  finishReason: string  // "stop" | "tool_calls" | ...
}

class LLMClient {
  constructor(
    private endpoint: string,
    private apiKey: string,
    private model: string,
  ) {}

  async chat(messages: Message[], tools?: ToolDefinition[]): Promise<LLMResponse> {
    const response = await fetch(`${this.endpoint}/chat/completions`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Authorization": `Bearer ${this.apiKey}`,
      },
      body: JSON.stringify({
        model: this.model,
        messages,
        tools: tools?.length ? tools : undefined,
        // 当有工具时，允许模型自由选择是否调用
        // tool_choice: "auto"
      }),
    })

    const data = await response.json()
    const choice = data.choices[0]

    return {
      content: choice.message.content,
      toolCalls: choice.message.tool_calls?.map((tc: any) => ({
        id: tc.id,
        name: tc.function.name,
        arguments: JSON.parse(tc.function.arguments),
      })),
      finishReason: choice.finish_reason,
    }
  }
}
```

### 3.4 运行循环

从 OpenCode 的 `prompt.ts:1312-1541` 提炼核心循环逻辑：

**OpenCode 的循环决策逻辑**：

```typescript
// prompt.ts:1352-1360
// 退出条件：assistant 回复完成（非 tool-calls），且无待处理工具
if (
  lastAssistant.finish &&
  lastAssistant.finish !== "tool-calls" &&
  lastAssistant.finish !== "unknown" &&
  !hasPendingToolCalls(lastAssistant) &&
  lastUserMessage.id < lastAssistantMessage.id
) {
  break
}
```

**简化版循环**：

```typescript
class Agent {
  private config: AgentConfig
  private llm: LLMClient
  private registry: ToolRegistry
  private store: MessageStore

  constructor(config: AgentConfig, llm: LLMClient) {
    this.config = config
    this.llm = llm
    this.registry = new ToolRegistry()
    this.store = new MessageStore()
    config.tools.forEach(t => this.registry.register(t))
    this.store.addSystem(config.systemPrompt)
  }

  async run(userMessage: string): Promise<string> {
    this.store.addUser(userMessage)

    let step = 0
    const maxSteps = this.config.maxSteps ?? 50

    while (step < maxSteps) {
      step++

      // 1. 调用 LLM
      const response = await this.llm.chat(
        this.store.getAll(),
        this.registry.toApiFormat(),
      )

      // 2. 记录 assistant 回复
      this.store.addAssistant(response.content, response.toolCalls)

      // 3. 如果没有工具调用，循环结束
      if (!response.toolCalls?.length) {
        return response.content ?? ""
      }

      // 4. 执行所有工具调用
      for (const toolCall of response.toolCalls) {
        const tool = this.registry.get(toolCall.name)
        if (!tool) {
          this.store.addToolResult(
            toolCall.id,
            toolCall.name,
            `Error: Unknown tool "${toolCall.name}"`,
          )
          continue
        }

        try {
          const result = await tool.execute(toolCall.arguments)
          this.store.addToolResult(toolCall.id, toolCall.name, result)
        } catch (err) {
          this.store.addToolResult(
            toolCall.id,
            toolCall.name,
            `Error: ${err instanceof Error ? err.message : String(err)}`,
          )
        }
      }

      // 5. 工具结果已追加到消息历史，继续循环让 LLM 处理结果
    }

    return "Reached maximum number of steps."
  }
}
```

---

## 4. 数据流与核心循环

### 4.1 单轮用户输入的完整数据流

以用户输入 "帮我读一下 config.json 并告诉我里面有什么配置项" 为例：

```
时间线                    消息历史（发送给 LLM 的内容）
─────────────────────────────────────────────────────────────────
初始化                    [system: "你是一个编码助手..."]

用户输入                  [system, user: "帮我读一下 config.json..."]
  │
  ▼
Turn 1: LLM 响应         [system, user, assistant: null, tool_calls: [
                            { id: "tc_1", name: "read_file", args: { path: "config.json" } }
                          ]]
  │
  ▼
执行工具                  [system, user, assistant, tool: { toolCallId: "tc_1",
                                                          content: '{"port": 3000, ...}' }]
  │
  ▼
Turn 2: LLM 响应         [system, user, assistant, tool, assistant: {
                            content: "config.json 包含以下配置项：\n- port: 3000\n..."
                          }]
  │
  ▼
无更多工具调用 → 返回     → "config.json 包含以下配置项：..."
```

### 4.2 消息数组的状态演进

```
Step 0: [system]
Step 1: [system, user]
Step 2: [system, user, assistant(+tool_calls)]
Step 3: [system, user, assistant(+tool_calls), tool_result]
Step 4: [system, user, assistant(+tool_calls), tool_result, assistant(text)]
                                                         ↑ 最终回复
```

### 4.3 多工具并发调用

OpenCode 支持在一次 LLM 响应中返回多个 tool calls（`processor.ts:259-331`），AI SDK 会自动并发执行。简化版也可以支持：

```
Turn 1: LLM 响应 → tool_calls: [
  { id: "tc_1", name: "read_file", args: { path: "a.ts" } },
  { id: "tc_2", name: "read_file", args: { path: "b.ts" } },
]

并行执行 → tool_result: { tc_1: "..." }
           tool_result: { tc_2: "..." }

Turn 2: LLM 看到 a.ts 和 b.ts 的内容，给出综合分析
```

---

## 5. 关键设计决策与取舍

### 5.1 从 OpenCode 中保留什么

| 设计模式 | 来自 OpenCode | 为什么保留 |
|---------|-------------|-----------|
| **声明式工具定义** | `tool/tool.ts` | 工具的 description/parameters/execute 三要素分离，LLM 只看前两者，执行逻辑独立 |
| **while(true) 循环** | `prompt.ts:1312` | 这是最核心的模式：LLM 决定是否继续，而不是外部代码决定 |
| **LLM 驱动的终止** | `prompt.ts:1352-1360` | 当 LLM 不再请求工具时自然终止，而非硬编码轮数 |
| **工具结果直接注入消息** | `processor.ts:333` | 工具输出作为 tool role 消息追加，让 LLM 自己决定如何使用 |
| **系统提示词分层** | `llm.ts:99-124` | 固定部分（角色/规则）与动态部分（环境/指令）分离 |
| **错误作为工具结果** | `processor.ts:338` | 工具执行失败不中断循环，将错误作为结果让 LLM 自行处理 |

### 5.2 从 OpenCode 中去掉什么

| 被去掉的复杂度 | 原始位置 | 为什么可以去掉 |
|-------------|---------|-------------|
| Effect-ts 依赖注入 | 全局 | 简单场景用 class + 构造函数即可 |
| Part-based 消息模型 | `message-v2.ts` | 直接用 LLM API 原生的 role-based 消息格式 |
| 流式处理 | `processor.ts` | 非实时交互场景不需要流式输出 |
| 上下文压缩 (compaction) | `compaction.ts` | 短对话场景不会溢出上下文窗口 |
| 子 Agent (subagent) | `agent/agent.ts` | 简单场景不需要层级 Agent |
| 权限系统 (permission) | `permission/` | 单用户场景不需要权限控制 |
| 多 Provider 支持 | `provider/` | 只需支持一种 API 格式 |
| 插件系统 | `plugin/` | 不需要可扩展性 |
| Doom loop 检测 | `processor.ts:306-330` | 可以在后续迭代中添加 |
| MCP 工具集成 | `prompt.ts:448-524` | 不需要外部工具协议 |
| 文件 patch/编辑系统 | `tool/edit.ts`, `tool/patch.ts` | 作为工具之一按需添加 |

### 5.3 OpenCode 教给我们的核心洞察

1. **Agent = 配置 + 循环 + 工具**：Agent 没有魔法，核心就是一个 while 循环，不断调用 LLM，根据返回决定执行工具或终止。

2. **LLM 是控制流的决策者**：循环是否继续由 LLM 的 `finish_reason` 决定（`"tool_calls"` → 继续，`"stop"` → 结束），而不是由代码逻辑决定。

3. **工具是 Agent 的手脚**：工具定义的两个核心字段（`description` 和 `parameters`）是给 LLM 看的"说明书"，`execute` 是给机器执行的"实现"。好的 description 决定了 LLM 能否正确使用工具。

4. **消息历史是 Agent 的记忆**：所有上下文（用户输入、LLM 回复、工具调用、工具结果）都追加到消息数组中，每次调用 LLM 时发送完整历史。LLM 通过阅读历史来理解"我在做什么"。

5. **错误也是一种信息**：工具执行失败时不应该抛异常中断流程，而应该把错误信息作为工具结果返回给 LLM，让 LLM 自己决定如何应对（换策略、换参数、或者告知用户）。

---

## 6. 实现建议

### 6.1 技术栈选择

| 组件 | 推荐方案 | 理由 |
|------|---------|------|
| 语言 | TypeScript / Python | TypeScript 与 OpenCode 同生态；Python 有最丰富的 LLM SDK |
| LLM 调用 | OpenAI SDK 或直接 fetch | OpenAI-compatible API 是事实标准 |
| 参数验证 | Zod (TS) / Pydantic (Python) | OpenCode 使用 Zod 定义工具参数 schema |
| 工具描述 | JSON Schema | LLM API 的标准格式 |

### 6.2 实现路线图

**Phase 1：最小可用（~200 行代码）**

- `LLMClient`：封装 chat completions API
- `ToolDefinition` + `ToolRegistry`：工具注册与执行
- `MessageStore`：简单的消息数组管理
- `Agent.run()`：核心循环
- 2 个工具：`read_file` + `bash`

**Phase 2：增强（~500 行代码）**

- 添加更多工具：`write_file`、`glob`、`grep`
- 并发工具执行（Promise.all）
- 错误重试（rate limit / network error）
- 输出截断（防止工具输出过长撑爆上下文）

**Phase 3：生产级**

- 流式输出（SSE）
- 上下文窗口管理（token 计数 + 压缩）
- 工具调用审计日志
- Doom loop 检测
- 超时控制（per-tool timeout）

### 6.3 关键代码模式速查

**从 OpenCode 学到的"工具描述写法"**（`tool/bash.txt`）：

好的工具描述应该包含：
1. **一句话功能说明**
2. **参数解释**
3. **使用限制和注意事项**
4. **何时使用 / 何时不用**
5. **示例**（可选但推荐）

```
// 好的描述
description: `Execute a shell command and return its output.
- Commands run in the current working directory.
- Output is truncated at 10000 characters.
- Timeout: 30 seconds.
- Prefer dedicated tools (read_file, write_file) over shell commands for file operations.`
```

**从 OpenCode 学到的"系统提示词写法"**（`session/prompt/default.txt`）：

好的系统提示词应该定义：
1. **身份**：你是什么
2. **风格**：怎么说话
3. **工具使用策略**：什么时候用什么工具
4. **约束**：绝对不能做什么
5. **环境信息**：你在哪里运行

### 6.4 完整的最小实现示例

```typescript
import fs from "fs/promises"
import { execFile } from "child_process"
import { promisify } from "util"

const execFileAsync = promisify(execFile)

// ===== 工具定义 =====

const tools = [
  {
    name: "read_file",
    description: "Read the contents of a file",
    parameters: {
      type: "object" as const,
      properties: {
        path: { type: "string", description: "File path to read" }
      },
      required: ["path"]
    },
    execute: async (args: { path: string }) => {
      return fs.readFile(args.path, "utf-8")
    }
  },
  {
    name: "list_directory",
    description: "List files in a directory",
    parameters: {
      type: "object" as const,
      properties: {
        path: { type: "string", description: "Directory path to list" }
      },
      required: ["path"]
    },
    execute: async (args: { path: string }) => {
      const entries = await fs.readdir(args.path, { withFileTypes: true })
      return entries.map(e => `${e.isDirectory() ? "dir" : "file"}  ${e.name}`).join("\n")
    }
  },
  {
    name: "bash",
    description: "Execute a shell command",
    parameters: {
      type: "object" as const,
      properties: {
        command: { type: "string", description: "Command to execute" }
      },
      required: ["command"]
    },
    execute: async (args: { command: string }) => {
      const { stdout, stderr } = await execFileAsync("sh", ["-c", args.command], {
        timeout: 30000,
        maxBuffer: 1024 * 1024,
      })
      let output = stdout
      if (stderr) output += `\nstderr:\n${stderr}`
      return output.slice(0, 10000)  // 截断
    }
  },
]

// ===== 核心循环 =====

async function runAgent(
  userMessage: string,
  systemPrompt: string,
  apiKey: string,
  model: string = "gpt-4o",
  maxSteps: number = 20,
) {
  const toolMap = new Map(tools.map(t => [t.name, t]))

  // 消息历史
  const messages: any[] = [
    { role: "system", content: systemPrompt },
    { role: "user", content: userMessage },
  ]

  for (let step = 0; step < maxSteps; step++) {
    // 调用 LLM
    const response = await fetch("https://api.openai.com/v1/chat/completions", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Authorization": `Bearer ${apiKey}`,
      },
      body: JSON.stringify({
        model,
        messages,
        tools: tools.map(t => ({
          type: "function",
          function: { name: t.name, description: t.description, parameters: t.parameters }
        })),
      }),
    })

    const data = await response.json()
    const choice = data.choices[0]
    const assistantMsg = choice.message

    // 追加 assistant 消息
    messages.push(assistantMsg)

    // 无工具调用 → 结束
    if (!assistantMsg.tool_calls?.length) {
      return assistantMsg.content
    }

    // 执行工具调用
    for (const tc of assistantMsg.tool_calls) {
      const tool = toolMap.get(tc.function.name)
      if (!tool) {
        messages.push({
          role: "tool",
          tool_call_id: tc.id,
          content: `Error: unknown tool "${tc.function.name}"`,
        })
        continue
      }

      try {
        const args = JSON.parse(tc.function.arguments)
        const result = await tool.execute(args)
        messages.push({ role: "tool", tool_call_id: tc.id, content: result })
      } catch (err: any) {
        messages.push({
          role: "tool",
          tool_call_id: tc.id,
          content: `Error: ${err.message}`,
        })
      }
    }
    // 工具结果已追加，循环继续 → LLM 看到结果后决定下一步
  }

  return "Agent reached maximum steps without completing."
}

// ===== 使用 =====

const result = await runAgent(
  "列出当前目录的文件，然后读取 package.json 的内容",
  "You are a helpful coding assistant. Use tools to accomplish tasks. When done, respond with a clear summary.",
  process.env.OPENAI_API_KEY!,
)
console.log(result)
```

这段 ~120 行的代码实现了 OpenCode Agent 的核心机制：
1. **工具定义**：声明式定义工具的 name/description/parameters/execute
2. **消息累积**：每一步都将新消息追加到历史中
3. **LLM 驱动的循环**：LLM 决定调用工具 → 执行 → 结果追加 → LLM 再决策 → ... → LLM 不再调用工具 → 结束
4. **错误内化**：工具执行失败不中断流程，错误信息作为工具结果让 LLM 自行处理

---

## 附录：OpenCode 关键源码索引

| 功能模块 | 源码路径 | 核心函数/类型 |
|---------|---------|-------------|
| Agent 定义 | `packages/opencode/src/agent/agent.ts` | `Info` schema, 内置 Agent 定义 |
| 工具基础类型 | `packages/opencode/src/tool/tool.ts` | `Tool.Def`, `Tool.define()` |
| 工具注册表 | `packages/opencode/src/tool/registry.ts` | `ToolRegistry` service |
| Bash 工具 | `packages/opencode/src/tool/bash.ts` | 完整的工具实现范例 |
| LLM 调用 | `packages/opencode/src/session/llm.ts` | `streamText()`, `stream()` |
| 核心循环 | `packages/opencode/src/session/prompt.ts` | `runLoop()` (L1312-1541) |
| 流式事件处理 | `packages/opencode/src/session/processor.ts` | `handleEvent()` (L217-459) |
| 消息模型 | `packages/opencode/src/session/message-v2.ts` | `MessageV2`, Part types |
| 系统提示词 | `packages/opencode/src/session/system.ts` | `SystemPrompt.provider()`, `environment()` |
| 上下文压缩 | `packages/opencode/src/session/compaction.ts` | `process()`, `create()` |
| 上下文溢出检测 | `packages/opencode/src/session/overflow.ts` | `isOverflow()` |
