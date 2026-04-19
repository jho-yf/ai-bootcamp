# 003 - OpenCode LLM Input/Output Visualizer

## 背景

`w6-opencode-logging` 插件将 opencode 与 LLM 的交互记录为 JSONL 文件，存储在 `./logs` 目录下。目录结构为：

```
logs/
├── 20260417_081711_ses_2673383b4ffeBmeS11sF3X7qvV/   # session (日期_time_sessionId)
│   ├── 001_conversation.jsonl                         # conversation file
│   └── 002_conversation.jsonl
└── 20260417_081858_ses_26731dfd8ffeRsf4r45rDnH59Z/
    └── 001_readmd.jsonl
```

需要构建一个 React 前端应用，支持两种打开方式：
1. **打开单个 JSONL 文件** — 直接展示该 conversation 的所有 turn
2. **打开 session 文件夹** — 展示该 session 下的所有 conversation，可通过 tab/侧栏切换

在一个页面中分类展示不同 turn 的输入输出。

## JSONL Schema 分析

每行是一个独立 JSON 对象，通过 `type` 字段区分：

### `turn_start`
```json
{ "type": "turn_start", "turn": 1, "ts": "2026-04-17T00:17:11.317Z" }
```
标记新一轮对话开始。

### `request`
```json
{
  "type": "request",
  "messageCount": 2,
  "messages": [
    {
      "info": {
        "role": "user" | "assistant",
        "time": { "created": 1776385031270 },
        "agent": "build",
        "model": { "providerID": "opencode", "modelID": "big-pickle" },
        "id": "msg_xxx",
        "sessionID": "ses_xxx"
      },
      "parts": [
        { "type": "text", "text": "user input", "id": "prt_xxx" }
      ]
    }
  ]
}
```
发送给 LLM 的完整消息列表。第二轮及以后 `messageCount > 1`，包含历史 assistant 消息（含 tool_call 结果）。

### `system`
```json
{ "type": "system", "system": ["<system prompt string>"], "model": { "id": "...", "providerID": "..." } }
```
系统提示词。一个 turn 中可能出现多次（不同 agent 的 system prompt）。

### `params`
```json
{ "type": "params", "agent": "build" | "title", "model": { "providerID": "opencode", "modelID": "big-pickle" } }
```
LLM 调用参数（agent 名称、模型标识）。

### `text_output`
```json
{
  "type": "text_output",
  "messageID": "msg_xxx",
  "partID": "prt_xxx",
  "text": "assistant response text"
}
```
LLM 返回的文本内容。

### `tool_call`
```json
{
  "type": "tool_call",
  "tool": "read" | "bash" | "write" | "glob" | "edit",
  "callID": "call_xxx",
  "args": { "filePath": "/path/to/file" },
  "result": {
    "title": "filename",
    "output": "<xml-formatted result>",
    "metadata": { "preview": "...", "truncated": false }
  }
}
```
LLM 发起的工具调用及其结果。

### `response`
```json
{
  "type": "response",
  "finish": "stop" | "tool-calls",
  "model": "undefined",
  "tokens": {
    "total": 12268, "input": 65, "output": 89, "reasoning": 0,
    "cache": { "write": 11604, "read": 510 }
  },
  "cost": 0,
  "error": "undefined"
}
```
LLM 响应的元数据：完成原因、token 用量、费用。

## 数据模型

```
Session                                       # 打开文件夹时的顶层结构
├── sessionName: string                       # 文件夹名
├── conversations: ConversationFile[]

ConversationFile                              # 单个 JSONL 文件
├── fileName: string
├── turns: Turn[]
│   ├── Turn
│   │   ├── turnNumber: number
│   │   ├── startTime: string
│   │   ├── agent: string
│   │   ├── model: { providerID, modelID }
│   │   ├── systemPrompts: string[]
│   │   ├── request: RequestData
│   │   ├── outputs: TurnOutput[]
│   │   │   ├── TextOutput { text }
│   │   │   └── ToolCallOutput { tool, args, result }
│   │   └── response: ResponseData
```

两种入口最终都产出 `ConversationFile[]`：
- 打开单个文件 → `[parseJsonl(fileContent)]`
- 打开文件夹 → 遍历所有 `.jsonl` 文件 → `files.map(parseJsonl)`

## UI 设计

### 页面布局 — Session 模式（打开文件夹）

```
┌──────────────────────────────────────────────────────────────┐
│  OpenCode LLM Visualizer                  [Open File] [Open  │
│  Session: 20260417_081858_ses_26731dfd8...  ▼]  Dir]         │
│  Conversation: [ 001_readmd.jsonl          ▼]                │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─ Turn 1 ─────────────────────────────────────────────┐    │
│  │  ┌─ User Input ──────────────────────────────────┐   │    │
│  │  │  [badge: user] model: big-pickle               │   │    │
│  │  │  "在当前项目中编写一个 READMD.md"               │   │    │
│  │  └────────────────────────────────────────────────┘   │    │
│  │                                                       │    │
│  │  ┌─ Assistant Response ────────────────────────────┐   │    │
│  │  │  [badge: assistant] finish: tool-calls           │   │    │
│  │  │                                                  │   │    │
│  │  │  "现在我对项目有了全面了解..."                     │   │    │
│  │  │                                                  │   │    │
│  │  │  ┌─ Tool Call: read ─────────────────────────┐  │   │    │
│  │  │  │  args: { filePath: "/home/..." }           │  │   │    │
│  │  │  │  [Collapsible Result]                      │  │   │    │
│  │  │  └────────────────────────────────────────────┘  │   │    │
│  │  │                                                  │   │    │
│  │  │  ┌─ Tokens ──────────────────────────────────┐  │   │    │
│  │  │  │  input: 65  output: 89  cache: 510/11604  │  │   │    │
│  │  │  └────────────────────────────────────────────┘  │   │    │
│  │  └──────────────────────────────────────────────────┘   │    │
│  └───────────────────────────────────────────────────────┘    │
│                                                              │
│  ┌─ Turn 2 ─────────────────────────────────────────────┐    │
│  │  ...                                                  │   │
│  └───────────────────────────────────────────────────────┘    │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

Header 包含两个下拉框：
- **Session 下拉框**：切换不同 session 文件夹（仅打开文件夹时可见）
- **Conversation 下拉框**：切换当前 session 下的不同 `.jsonl` 文件

### 页面布局 — 单文件模式（打开 JSONL）

与 Session 模式主区域相同，仅 Conversation 下拉框显示单个文件名（不可切换）。

### 组件拆分

| 组件 | 职责 |
|------|------|
| `App` | 顶层容器，管理文件/文件夹打开状态，路由 Session/单文件模式 |
| `Header` | 顶部导航栏，含 Session 下拉框、Conversation 下拉框、打开按钮 |
| `FileOpener` | 文件/文件夹选择（支持 drag & drop 和点击选择） |
| `ConversationViewer` | 解析 JSONL，渲染当前选中 conversation 的所有 Turn |
| `TurnCard` | 单个 Turn 卡片，包含请求和响应 |
| `MessageBubble` | 用户/助手消息气泡，使用 markdown 渲染 |
| `ToolCallCard` | 工具调用卡片（可折叠），展示 tool name、args、result |
| `TokenStats` | Token 用量统计条 |
| `SystemPromptSection` | 可折叠的 system prompt 展示区域 |
| `MarkdownRenderer` | 封装 react-markdown，用于渲染文本内容 |

### 交互设计

1. **打开方式**：Header 中有 [Open File] 和 [Open Dir] 两个按钮
   - Open File：选择单个 `.jsonl` 文件
   - Open Dir：选择 session 文件夹，自动扫描其下所有 `.jsonl` 文件
2. **Session 切换**：打开文件夹后，Session 下拉框列出所有子文件夹，切换时重新加载其下的 conversations
3. **Conversation 切换**：下拉框列出当前 session 下所有 `.jsonl` 文件（按文件名排序），切换时刷新主区域
4. **Turn 展开/折叠**：默认展开所有 Turn，点击 Turn header 可折叠
5. **Tool Call 展开/折叠**：tool call 结果默认折叠，点击展开查看完整结果
6. **System Prompt 展开/折叠**：system prompt 默认折叠（内容很长），点击展开
7. **滚动控制**：每个 Turn 内容区域使用独立 scrollbar，避免页面无限增长
8. **Markdown 渲染**：所有 `text` 和 `text_output` 内容使用 markdown 渲染，支持代码高亮

### 状态管理

```
AppState
├── mode: 'idle' | 'single-file' | 'session'
├── sessions: Map<sessionName, string[]>          # session → jsonl 文件名列表
├── currentSession: string | null
├── currentConversation: string | null            # 当前查看的 jsonl 文件名
├── conversations: Map<fileName, ConversationFile> # 已解析的 conversation 缓存
```

打开单个文件时 `mode='single-file'`，sessions 为空，直接展示。
打开文件夹时 `mode='session'`，填充 sessions map，通过下拉框切换。

## 技术方案

### 技术栈

- **Vite + React + TypeScript** - 构建与框架
- **react-markdown** + **rehype-highlight** - Markdown 渲染与代码高亮
- **Design tokens** - 引用 `@w6-opencode-logging-ui/styles` 中的 CSS 变量

### 项目结构

```
w6-opencode-logging-ui/
├── styles/
│   ├── design-tokens.css      # 已有
│   └── global.css             # 已有
├── src/
│   ├── main.tsx               # 入口
│   ├── App.tsx                # 顶层组件
│   ├── types.ts               # JSONL 数据类型定义
│   ├── parser.ts              # JSONL 解析逻辑
│   ├── components/
│   │   ├── Header.tsx
│   │   ├── FileOpener.tsx
│   │   ├── ConversationViewer.tsx
│   │   ├── TurnCard.tsx
│   │   ├── MessageBubble.tsx
│   │   ├── ToolCallCard.tsx
│   │   ├── TokenStats.tsx
│   │   ├── SystemPromptSection.tsx
│   │   └── MarkdownRenderer.tsx
│   └── hooks/
│       └── useFileReader.ts   # 文件/文件夹读取 hook
├── index.html
├── package.json
├── tsconfig.json
└── vite.config.ts
```

### 关键实现细节

#### JSONL 解析 (parser.ts)

```typescript
// 解析流程：
// 1. 逐行读取 JSONL，按 type 分类
// 2. 以 turn_start 为分隔符，将行分组为 Turn[]
// 3. 每个 Turn 内：
//    - system → Turn.systemPrompts
//    - params → Turn.agent, Turn.model
//    - request → Turn.request (提取 user message 的 text parts)
//    - text_output → Turn.outputs
//    - tool_call → Turn.outputs
//    - response → Turn.response
```

#### Markdown 渲染策略

- `text_output` 内容：作为 markdown 渲染
- `tool_call.result.output`：先清理 XML 标签（`<path>`, `<type>`, `<content>` 等），然后作为 code block 渲染
- User input：作为纯文本或 markdown 渲染

#### 样式方案

直接使用 CSS 变量，不引入 CSS-in-JS 或 Tailwind：
- 卡片：`var(--md-cloud)` 背景 + `var(--border-strong)` 边框
- 角色标识：使用 `.badge-user` / `.badge-assistant` 样式
- 滚动区域：使用 `.scrollable` + `max-height` 限制
- 代码块：使用已有的 `pre` / `code` 样式

## 实施步骤

1. 初始化 Vite + React + TypeScript 项目
2. 定义类型 `types.ts`
3. 实现 JSONL 解析器 `parser.ts`
4. 实现 `MarkdownRenderer` 组件
5. 实现 `FileOpener` 组件
6. 实现 `TurnCard` + `MessageBubble` + `ToolCallCard` + `TokenStats`
7. 组装 `ConversationViewer` + `App`
8. 接入 design tokens 样式
9. 验证：用真实 JSONL 文件测试
