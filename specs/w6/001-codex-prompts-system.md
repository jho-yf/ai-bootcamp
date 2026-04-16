# OpenAI Codex CLI — System Prompt 与工具 Prompt 架构详解

> 基于对 `vendors/codex` 代码库（主要是 `codex-rs/` Rust 核心部分）的深入分析。
> 本文档旨在全面介绍 Codex CLI 如何组织 system prompt、工具定义 prompt、以及相关的 prompt 组装流水线。

---

## 目录

1. [整体架构概览](#1-整体架构概览)
2. [System Prompt 文件清单](#2-system-prompt-文件清单)
3. [核心 System Prompt 内容分析](#3-核心-system-prompt-内容分析)
4. [Prompt 组装流水线](#4-prompt-组装流水线)
5. [人格系统 (Personality)](#5-人格系统-personality)
6. [协作模式 (Collaboration Mode)](#6-协作模式-collaboration-mode)
7. [审批与沙箱 Prompt](#7-审批与沙箱-prompt)
8. [记忆系统 Prompt](#8-记忆系统-prompt)
9. [上下文压缩 (Compaction) Prompt](#9-上下文压缩-compaction-prompt)
10. [代码审查 (Review) Prompt](#10-代码审查-review-prompt)
11. [工具定义体系](#11-工具定义体系)
12. [核心工具详解](#12-核心工具详解)
13. [工具注册与路由](#13-工具注册与路由)
14. [Prompt 模板变量系统](#14-prompt-模板变量系统)
15. [关键源文件索引](#15-关键源文件索引)

---

## 1. 整体架构概览

Codex CLI 是一个由 Rust 编写的终端编程助手。其 prompt 系统的核心设计理念是**分层组装**：通过多层 prompt 片段的叠加，构建出最终的 `instructions` 字段（即 system prompt），连同工具定义（`tools`）和对话历史（`input`）一起发送给 OpenAI Responses API。

```
┌─────────────────────────────────────────────────┐
│                  API Request                     │
│  ┌───────────────────────────────────────────┐  │
│  │  instructions (system prompt)              │  │
│  │  ┌─ Base Instructions ─────────────────┐  │  │
│  │  │  模型特定的核心 prompt              │  │  │
│  │  ├─ Personality Spec ──────────────────┤  │  │
│  │  │  人格定义 (pragmatic/friendly)      │  │  │
│  │  ├─ Developer Instructions ────────────┤  │  │
│  │  │  审批策略 + 沙箱模式 + 执行策略     │  │  │
│  │  ├─ Collaboration Mode ────────────────┤  │  │
│  │  │  协作模式 (default/plan/execute)    │  │  │
│  │  ├─ Memory Instructions ───────────────┤  │  │
│  │  │  记忆系统使用指南                   │  │  │
│  │  └─────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────┐  │
│  │  input (对话历史)                          │  │
│  │  ├─ Environment Context (user message)    │  │
│  │  ├─ AGENTS.md (user message fragments)    │  │
│  │  ├─ Skill Instructions (user message)     │  │
│  │  └─ Conversation turns                    │  │
│  └───────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────┐  │
│  │  tools (工具定义数组)                      │  │
│  │  ├─ shell / exec_command                  │  │
│  │  ├─ apply_patch                           │  │
│  │  ├─ update_plan                           │  │
│  │  ├─ view_image                            │  │
│  │  ├─ request_user_input                    │  │
│  │  ├─ request_permissions                   │  │
│  │  ├─ MCP tools                             │  │
│  │  └─ ...                                   │  │
│  └───────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

### 关键数据结构

**`Prompt` 结构体** (`codex-rs/core/src/client_common.rs`)：

```rust
pub struct Prompt {
    pub input: Vec<ResponseItem>,       // 对话历史
    pub(crate) tools: Vec<ToolSpec>,    // 可用工具列表
    pub(crate) parallel_tool_calls: bool,
    pub base_instructions: BaseInstructions,  // 系统 prompt
    pub personality: Option<Personality>,
    pub output_schema: Option<Value>,   // 结构化输出约束
}
```

**`BaseInstructions` 结构体** (`codex-rs/protocol/src/models.rs`)：

包含最终拼接的 `text` 字符串，通过层层叠加产生。

---

## 2. System Prompt 文件清单

Codex 为不同模型维护了多套 system prompt 文件：

| 文件路径 | 行数 | 用途 |
|---------|------|------|
| `codex-rs/core/prompt.md` | 8 | AGENTS.md 规范（最小基座） |
| `codex-rs/core/gpt-5.2-codex_prompt.md` | 299 | **GPT-5.2 Codex 最完整 prompt**（主 prompt） |
| `codex-rs/core/gpt_5_codex_prompt.md` | 81 | GPT-5 基础版 prompt |
| `codex-rs/core/gpt_5_1_prompt.md` | 332 | GPT-5.1 prompt（含 User Updates Spec） |
| `codex-rs/core/gpt-5.1-codex-max_prompt.md` | 81 | GPT-5.1 Codex Max 变体 |
| `codex-rs/core/gpt_5_2_prompt.md` | 276 | GPT-5.2 变体（不含 User Updates Spec） |
| `codex-rs/core/prompt_with_apply_patch_instructions.md` | 352 | 含完整 apply_patch 语法的 prompt |
| `codex-rs/protocol/src/prompts/base_instructions/default.md` | 276 | 协议层默认 instructions 常量 |

### 模板文件

| 文件路径 | 用途 |
|---------|------|
| `codex-rs/core/templates/model_instructions/gpt-5.2-codex_instructions_template.md` | 带 `{{ personality }}` 占位符的模板 |
| `codex-rs/core/templates/personalities/gpt-5.2-codex_pragmatic.md` | 务实型人格 |
| `codex-rs/core/templates/personalities/gpt-5.2-codex_friendly.md` | 友好型人格 |

### 功能性 Prompt

| 文件路径 | 用途 |
|---------|------|
| `codex-rs/core/review_prompt.md` | 代码审查模式 prompt |
| `codex-rs/core/templates/compact/prompt.md` | 上下文压缩 prompt |
| `codex-rs/core/templates/compact/summary_prefix.md` | 压缩摘要前缀 |
| `codex-rs/core/templates/collaboration_mode/default.md` | 默认协作模式 |
| `codex-rs/core/templates/collaboration_mode/execute.md` | 执行模式 |
| `codex-rs/core/templates/collaboration_mode/plan.md` | 计划模式 |
| `codex-rs/core/templates/memories/read_path.md` | 记忆读取指南 |
| `codex-rs/core/templates/memories/stage_one_system.md` | 记忆写入 agent prompt |
| `codex-rs/core/templates/memories/consolidation.md` | 记忆整合 prompt |
| `codex-rs/tui/prompt_for_init_command.md` | AGENTS.md 生成 prompt |

### 审批策略 Prompt

| 文件路径 | 用途 |
|---------|------|
| `codex-rs/protocol/src/prompts/permissions/approval_policy/never.md` | 从不审批策略 |
| `codex-rs/protocol/src/prompts/permissions/approval_policy/on_failure.md` | 失败时审批策略 |
| `codex-rs/protocol/src/prompts/permissions/approval_policy/unless_trusted.md` | 非可信命令审批策略 |
| `codex-rs/protocol/src/prompts/permissions/approval_policy/on_request.md` | 按需审批策略 |
| `codex-rs/protocol/src/prompts/permissions/sandbox_mode/workspace_write.md` | 工作区写入沙箱模式 |
| `codex-rs/protocol/src/prompts/permissions/sandbox_mode/danger_full_access.md` | 完全访问沙箱模式 |
| `codex-rs/protocol/src/prompts/permissions/sandbox_mode/read_only.md` | 只读沙箱模式 |

---

## 3. 核心 System Prompt 内容分析

以 `gpt-5.2-codex_prompt.md` 为例（最完整的版本），核心 prompt 涵盖以下模块：

### 3.1 身份与能力声明

```
You are a coding agent running in the Codex CLI, a terminal-based coding
assistant. Codex CLI is an open source project led by OpenAI. You are
expected to be precise, safe, and helpful.
```

声明三个能力：
- 接收用户 prompt 和工作区上下文
- 通过流式思考/响应与用户沟通，创建和更新计划
- 发出函数调用来运行终端命令和应用补丁

### 3.2 人格设定 (Personality)

简洁、直接、友好。高效沟通，优先提供可操作的指导。除非被明确要求，否则避免冗长解释。

### 3.3 AGENTS.md 规范

定义了项目中 `AGENTS.md` 文件的语义：
- 作用域：包含 AGENTS.md 的目录的整个子树
- 优先级：更深层嵌套的 AGENTS.md 优先
- 系统/开发者/用户指令优先于 AGENTS.md 指令

### 3.4 前导消息 (Preamble Messages)

在每次工具调用前，发送简短的 8-12 词更新消息。示例：
- "I've explored the repo; now checking the API route definitions."
- "Config's looking tidy. Next up is patching helpers to keep things in sync."

### 3.5 计划工具使用规范

- 简单任务（约最简单的 25%）不使用计划工具
- 不创建单步骤计划
- 提供高质量计划的正面和反面示例

### 3.6 任务执行准则

核心原则：
- 持续工作直到查询完全解决
- 使用 `apply_patch` 工具编辑文件（禁止 `applypatch` 或 `apply-patch`）
- 根因修复而非表面补丁
- 不修复无关 bug
- 不自动添加版权头、内联注释或单字母变量
- NEVER 输出 `【F:README.md†L5-L14】` 格式的引用

### 3.7 验证工作

测试策略：先具体后广泛。在非交互审批模式下主动运行测试；在交互模式下等待用户确认。

### 3.8 雄心 vs 精准

- 无历史上下文的新任务：大胆创新
- 现有代码库：精确执行，尊重现有代码

### 3.9 最终答案格式

详细的 Markdown 格式规范：
- **节标题**: `**Title Case**`，1-3 个词
- **列表**: `- ` 开头，4-6 条，按重要性排序
- **等宽字体**: 反引号包裹命令/路径/代码标识
- **文件引用**: `src/app.ts:42` 格式，禁止 `file://` URI
- **语气**: 协作、简洁、事实性、现在时、主动语态

### 3.10 工具指南

- Shell 命令：优先使用 `rg` 搜索
- `update_plan` 工具使用规范
- `apply_patch` 的完整语法定义（Freeform 模式下）

---

## 4. Prompt 组装流水线

Prompt 的组装是一个多层叠加的过程，发生在 `Codex::spawn_internal()` 和 `build_prompt()` 中：

### 4.1 组装顺序（从内到外）

```
Layer 1: Base Instructions
  └── 从模型特定的 .md 文件加载（如 gpt-5.2-codex_prompt.md）
  └── 或从 instructions_template.md + personality 组装

Layer 2: Personality Spec
  └── 从 templates/personalities/ 加载
  └── 用 <personality_spec> 标签包裹

Layer 3: Developer Instructions
  └── 审批策略模板 (approval_policy/*.md)
  └── 沙箱模式模板 (sandbox_mode/*.md)
  └── 可写入根目录列表
  └── 执行策略规则

Layer 4: Collaboration Mode Instructions
  └── 从 templates/collaboration_mode/*.md 加载
  └── 用 <collaboration_mode> 标签包裹

Layer 5: Memory Instructions
  └── 从 templates/memories/read_path.md 加载
  └── 渲染 {{ base_path }} 和 {{ memory_summary }} 变量

Layer 6: User Instructions (注入为 user role 消息)
  └── AGENTS.md 文件内容
  └── 用户自定义指令

Layer 7: Environment Context (注入为 user role 消息)
  └── CWD、Shell 类型、日期、时区
  └── 用 <environment_context> 标签包裹
```

### 4.2 优先级规则

1. **系统/开发者指令** > **AGENTS.md 指令** > **隐式约定**
2. 更深层嵌套的 AGENTS.md 文件优先于浅层文件
3. 配置中的 `base_instructions` 覆盖 > 对话历史中的 `base_instructions` > 模型默认值

### 4.3 上下文片段 (Context Fragments)

通过 `ContextualUserFragmentDefinition` 以 user role 消息注入对话历史：

| 片段类型 | 标签 | 用途 |
|---------|------|------|
| AGENTS_MD_FRAGMENT | `# AGENTS.md instructions for <dir>` / `</INSTRUCTIONS>` | 项目级指令 |
| ENVIRONMENT_CONTEXT_FRAGMENT | `<environment_context>` / `</environment_context>` | 环境信息 |
| SKILL_FRAGMENT | `<skill>` / `</skill>` | 技能指令 |
| USER_SHELL_COMMAND_FRAGMENT | `<user_shell_command>` / `</user_shell_command>` | Shell 命令输出 |
| TURN_ABORTED_FRAGMENT | `<turn_aborted>` / `</turn_aborted>` | 回合中断标记 |
| SUBAGENT_NOTIFICATION_FRAGMENT | `<subagent_notification>` / `</subagent_notification>` | 子 agent 通知 |

---

## 5. 人格系统 (Personality)

Codex 支持两种内置人格，通过模板变量注入到 instructions 中：

### 5.1 Pragmatic (务实型)

文件: `templates/personalities/gpt-5.2-codex_pragmatic.md`

核心价值观：
- **Clarity**: 明确、具体地沟通推理过程
- **Pragmatism**: 聚焦目标和前进动力
- **Rigor**: 技术论点必须连贯可辩护

交互风格：简洁、尊重、避免虚夸和激励性语言。不做拉拉队，但会真诚地指出有趣的方法。

### 5.2 Friendly (友好型)

文件: `templates/personalities/gpt-5.2-codex_friendly.md`

核心价值观：
- **Empathy**: 根据对方的水平调整解释和节奏
- **Collaboration**: 主动邀请输入、综合观点
- **Ownership**: 不仅对代码负责，也对团队是否通畅负责

交互风格：温暖、鼓励、使用"我们"等合作性语言。永远不会简慢或轻视。

### 5.3 人格选择机制

人格通过 `templates/model_instructions/gpt-5.2-codex_instructions_template.md` 中的 `{{ personality }}` 占位符注入：

```
You are Codex, a coding agent based on GPT-5. ...

{{ personality }}

# Working with the user
...
```

---

## 6. 协作模式 (Collaboration Mode)

Codex 支持三种协作模式，每种模式有独立的 prompt 模板：

### 6.1 Default Mode

文件: `templates/collaboration_mode/default.md`

标准交互模式。声明当前模式，说明如何通过 `<collaboration_mode>` 标签切换模式。包含 `request_user_input` 的可用性说明。

模板变量：
- `{{KNOWN_MODE_NAMES}}` — 已知的模式名称列表
- `{{REQUEST_USER_INPUT_AVAILABILITY}}` — 用户输入工具的可用性
- `{{ASKING_QUESTIONS_GUIDANCE}}` — 提问指南

### 6.2 Execute Mode (执行模式)

文件: `templates/collaboration_mode/execute.md` (46 行)

核心理念：**假设优先执行**。当信息缺失时，不做提问，而是：
1. 做出合理假设
2. 在最终消息中简短说明假设
3. 继续执行

关键原则：
- **Think out loud**: 分享推理但保持简短
- **Use reasonable assumptions**: 缺失信息时建议合理选择
- **Think ahead**: 预测用户可能还需要什么
- **Be mindful of time**: 用户在等待，尽快完成任务
- **Long-horizon execution**: 将工作分解为里程碑，逐步验证

### 6.3 Plan Mode (计划模式)

文件: `templates/collaboration_mode/plan.md` (129 行)

最详细的协作模式。三阶段流程：

**Phase 1 — Ground in environment (先探索再提问)**
- 先通过非变异操作消除未知
- 只有在无法通过探索获取信息时才提问
- 禁止问可以通过代码搜索回答的问题

**Phase 2 — Intent chat (理解真正意图)**
- 持续提问直到明确：目标 + 成功标准、受众、范围、约束、偏好/权衡

**Phase 3 — Implementation chat (如何构建)**
- 持续提问直到规格决策完整：方法、接口、数据流、边界情况、测试标准

最终计划用 `<proposed_plan>` 标签包裹，计划内容需包含：
- 清晰标题
- 简短摘要
- 关键变更
- 测试计划
- 显式假设

---

## 7. 审批与沙箱 Prompt

### 7.1 审批策略 (Approval Policy)

四种审批策略，每种对应一段 prompt 注入到 Developer Instructions 中：

**`never`** (从不审批):
> Approval policy is currently never. Do not provide the `sandbox_permissions` for any reason, commands will be rejected.

**`on_failure`** (失败时审批):
> ...allow all commands to run in the sandbox (if enabled), and failures will be escalated to the user for approval to run again without the sandbox.

**`unless_trusted`** (非可信命令审批):
> ...escalate most commands for user approval, apart from a limited allowlist of safe "read" commands.

**`on_request`** (按需审批) — 最详细的版本 (58 行):

包含：
- 如何请求提权（`sandbox_permissions: "require_escalated"` + `justification` + `prefix_rule`）
- 何时请求提权（需要写外部目录、运行 GUI、网络错误、破坏性操作）
- `prefix_rule` 指引和禁止列表
- 示例：`["npm", "run", "dev"]`、`["gh", "pr", "check"]`

### 7.2 沙箱模式 (Sandbox Mode)

三种沙箱模式模板：

- **workspace_write**: 可读取文件，可在 cwd 和 writable_roots 中编辑
- **danger_full_access**: 完全访问
- **read_only**: 只读

模板使用 `{{network_access}}` 变量控制网络权限描述。

### 7.3 审批参数在工具定义中的体现

Shell 类工具（`exec_command`、`shell`、`shell_command`）都包含审批相关参数：

```json
{
  "sandbox_permissions": "use_default | with_additional_permissions | require_escalated",
  "justification": "向用户请求审批的问题",
  "prefix_rule": ["命令", "前缀"],
  "additional_permissions": {
    "network": { "enabled": true },
    "file_system": { "read": ["/path"], "write": ["/path"] }
  }
}
```

---

## 8. 记忆系统 Prompt

文件: `templates/memories/read_path.md` (130 行)

记忆系统为 agent 提供跨会话的知识积累能力。

### 8.1 记忆布局

```
{{ base_path }}/
├── memory_summary.md     # 已提供的摘要（不要重复打开）
├── MEMORY.md             # 可搜索的索引文件
├── skills/<name>/
│   ├── SKILL.md          # 入口指令
│   ├── scripts/          # 辅助脚本
│   ├── examples/         # 示例输出
│   └── templates/        # 模板
└── rollout_summaries/    # 每次执行的记录
```

### 8.2 使用决策

**跳过记忆**：自包含请求（当前时间、简单翻译、单行命令）

**使用记忆**（默认）：当以下任一条件为真时：
- 查询涉及 MEMORY_SUMMARY 中的工作区/模块/路径
- 用户要求先前上下文/一致性/先前决策
- 任务模糊，可能依赖先前的项目选择
- 是非平凡的并与 MEMORY_SUMMARY 相关

### 8.3 快速记忆遍历

1. 浏览 MEMORY_SUMMARY，提取任务相关关键词
2. 搜索 MEMORY.md
3. 如有直接指向，打开 1-2 个最相关的 rollout_summaries 或 skills 文件
4. 搜索 `rollout_path` 获取精确证据
5. 无相关结果则停止

预算：4-6 步搜索。

### 8.4 引用要求

使用记忆时，在最终回复末尾附加 `<oai-mem-citation>` 块：

```xml
<oai-mem-citation>
<citation_entries>
MEMORY.md:234-236|note=[描述]
rollout_summaries/xxx.md:10-12|note=[描述]
</citation_entries>
<rollout_ids>
uuid-1
uuid-2
</rollout_ids>
</oai-mem-citation>
```

---

## 9. 上下文压缩 (Compaction) Prompt

当上下文窗口接近限制时，触发压缩回合。

### 9.1 压缩 Prompt

文件: `templates/compact/prompt.md` (10 行)

```
You are performing a CONTEXT CHECKPOINT COMPACTION. Create a handoff
summary for another LLM that will resume the task.

Include:
- Current progress and key decisions made
- Important context, constraints, or user preferences
- What remains to be done (clear next steps)
- Any critical data, examples, or references needed to continue

Be concise, structured, and focused on helping the next LLM seamlessly continue the work.
```

### 9.2 摘要前缀

文件: `templates/compact/summary_prefix.md` (1 行)

```
Another language model started to solve this problem and produced a summary
of its thinking process. You also have access to the state of the tools that
were used by that language model. Use this to build on the work that has
already been done and avoid duplicating work. Here is the summary produced
by the other language model:
```

压缩后的摘要替换完整历史，下一轮从压缩状态继续。

---

## 10. 代码审查 (Review) Prompt

文件: `review_prompt.md` (88 行)

### 10.1 审查员身份

> You are acting as a reviewer for a proposed code change made by another engineer.

### 10.2 Bug 判定标准（8 条）

1. 对准确性、性能、安全性或可维护性有重大影响
2. Bug 是离散且可操作的
3. 修复不需要超出代码库现有严谨度
4. Bug 在本次提交中引入
5. 原始作者如果知道会修复它
6. 不依赖于未声明的假设
7. 必须能识别具体受影响的代码
8. 不是原始作者的故意变更

### 10.3 评论规范

- 正文最多 1 段
- 代码块不超过 3 行
- 明确说明触发 Bug 的场景
- 避免过度赞美或无用的客套

### 10.4 优先级标签

- **P0**: 立即修复（阻塞发布）
- **P1**: 紧急（下个迭代修复）
- **P2**: 正常（最终修复）
- **P3**: 低（有了更好）

### 10.5 输出格式

严格的 JSON Schema：

```json
{
  "findings": [
    {
      "title": "<≤80 chars, 祈使句>",
      "body": "<Markdown 解释>",
      "confidence_score": 0.0-1.0,
      "priority": 0-3,
      "code_location": {
        "absolute_file_path": "<path>",
        "line_range": { "start": 1, "end": 5 }
      }
    }
  ],
  "overall_correctness": "patch is correct | patch is incorrect",
  "overall_explanation": "<1-3 句>",
  "overall_confidence_score": 0.0-1.0
}
```

---

## 11. 工具定义体系

### 11.1 ToolSpec 枚举

所有工具定义通过 `ToolSpec` 枚举表示（`codex-rs/tools/src/tool_spec.rs`）：

```rust
pub enum ToolSpec {
    Function(ResponsesApiTool),      // 标准 Function Calling
    ToolSearch { execution, description, parameters },  // 可发现工具搜索
    LocalShell {},                    // 内置本地 Shell
    ImageGeneration { output_format }, // 图像生成
    WebSearch { external_web_access, filters, ... }, // Web 搜索
    Freeform(FreeformTool),          // 自定义语法工具（如 apply_patch）
}
```

### 11.2 两种工具调用协议

**Function Calling (标准)**：通过 `ResponsesApiTool` 定义，包含 `name`、`description`、`parameters` (JSON Schema)、`strict`、`output_schema`。

**Freeform (自定义语法)**：通过 `FreeformTool` 定义，包含 `name`、`description`、`format`（指定 Lark 语法定义）。Codex 的 `apply_patch` 工具使用这种模式。

### 11.3 工具条件注册

工具的注册由 `ToolsConfig` 中的各种标志位控制：

| 标志 | 控制的工具 |
|------|-----------|
| `shell_type` | `shell` / `exec_command` + `write_stdin` / `shell_command` / `local_shell` |
| `apply_patch_tool_type` | `apply_patch` (Freeform 或 Function) |
| `request_user_input` | `request_user_input` |
| `request_permissions_tool_enabled` | `request_permissions` |
| `web_search_mode` | `web_search` |
| `image_gen_tool` | `image_generation` |
| `js_repl_enabled` | `js_repl` + `js_repl_reset` |
| `collab_tools` | 多 agent 工具组 |
| `code_mode_enabled` | Code Mode 嵌套工具 |
| `search_tool` | `tool_search` |
| `tool_suggest` | `tool_suggest` |
| `agent_jobs_tools` | `spawn_agents_on_csv` / `report_agent_job_result` |

---

## 12. 核心工具详解

### 12.1 Shell 执行工具组

根据 `shell_type` 配置选择不同工具：

**`shell` (默认模式)**
- 参数: `command` (数组), `workdir`, `timeout_ms`, `sandbox_permissions`, `justification`, `prefix_rule`
- 通过 `execvp()` 执行，需要 `["bash", "-lc"]` 前缀
- 支持并行工具调用

**`exec_command` (UnifiedExec 模式)**
- 参数: `cmd` (字符串), `workdir`, `shell`, `tty`, `yield_time_ms`, `max_output_tokens`, `sandbox_permissions` 等
- 在 PTY 中运行，返回输出或 session_id
- 配套 `write_stdin` 工具用于交互
- 输出结构化 JSON：`{ chunk_id, wall_time_seconds, exit_code, session_id, output }`

**`shell_command` (ShellCommand 模式)**
- 参数: `command` (字符串), `workdir`, `timeout_ms`, `sandbox_permissions` 等
- 在用户默认 shell 中执行

**`local_shell` (内置)**
- 使用 OpenAI 内置的 local_shell 能力

### 12.2 `apply_patch` — 文件编辑工具

两种实现模式：

**Freeform 模式** (GPT-5 推荐)：
- 类型: `FreeformTool`，使用 Lark 语法定义
- 不需要 JSON 包裹，直接输出 patch 文本
- 语法：`*** Begin Patch` / `*** End Patch` 包裹
- 操作: `*** Add File:` / `*** Delete File:` / `*** Update File:`
- 移动: `*** Move to:`
- Hunk: `@@` 引入，行前缀 ` ` (上下文) / `-` (删除) / `+` (新增)

**Function 模式** (gpt-oss 模型)：
- 类型: `ResponsesApiTool`，参数为 `input` 字符串
- 相同的 patch 语法，但包裹在 JSON 中

### 12.3 `update_plan` — 计划管理工具

用于创建和管理任务计划，状态包括 `pending`、`in_progress`、`completed`。

Prompt 中的使用指南要求：
- 简单任务不使用
- 不创建单步骤计划
- 每步 5-7 个词
- 完成步骤后标记为 completed

### 12.4 `view_image` — 图像查看工具

参数: `path`, `detail` (auto/low/high)

用于查看用户附带的图片文件。

### 12.5 `request_user_input` — 用户输入请求

参数: `question`, `options` (2-4 个选项), `multi_select`

用于向用户提问结构化问题。仅在 `request_user_input` 配置启用时可用。

### 12.6 `request_permissions` — 权限请求

参数: `reason`, `permissions` (network + file_system)

请求额外的文件系统或网络权限。

### 12.7 MCP 工具

通过 MCP (Model Context Protocol) 动态加载的外部工具：
- `list_mcp_resources` — 列出 MCP 资源
- `list_mcp_resource_templates` — 列出资源模板
- `read_mcp_resource` — 读取资源
- 动态注册的 MCP 工具（转为 `ResponsesApiTool`）

### 12.8 多 Agent 工具

当 `collab_tools` 启用时，注册一整套多 agent 协作工具：

**V1**: `spawn_agent`, `send_input`, `resume_agent`, `wait_agent`, `close_agent`

**V2** (推荐): `spawn_agent`, `send_message`, `assign_task`, `wait_agent`, `close_agent`, `list_agents`

### 12.9 其他工具

- `tool_search` — 搜索可用工具
- `tool_suggest` — 推荐相关工具
- `list_dir` — 列出目录内容
- `js_repl` / `js_repl_reset` — JavaScript REPL
- `spawn_agents_on_csv` — 批量 agent 任务
- `report_agent_job_result` — 报告任务结果
- `test_sync_tool` — 测试同步工具
- `web_search` — 网页搜索
- `image_generation` — 图像生成

---

## 13. 工具注册与路由

### 13.1 注册流程

在 `build_specs_with_discoverable_tools()` 中完成（`codex-rs/core/src/tools/spec.rs`）：

1. 创建 `ToolRegistryBuilder`
2. 实例化所有 handler（`Arc<Handler>`）
3. 根据 `ToolsConfig` 标志位条件注册工具 spec 和 handler
4. 构建 `ToolRouter`

### 13.2 ToolHandler trait

```rust
pub trait ToolHandler: Send + Sync {
    type Output: ToolOutput + 'static;
    fn kind(&self) -> ToolKind;
    async fn is_mutating(&self, invocation: &ToolInvocation) -> bool;
    async fn handle(&self, invocation: ToolInvocation) -> Result<Self::Output, FunctionCallError>;
}
```

所有 handler 都实现此 trait，包括：`ShellHandler`, `ApplyPatchHandler`, `PlanHandler`, `McpHandler`, `ViewImageHandler`, `RequestPermissionsHandler`, `RequestUserInputHandler`, `ToolSearchHandler`, `SpawnAgentHandler` 等。

### 13.3 工具调度流程

```
Model 输出 tool_call (ResponseItem)
    │
    ▼
ToolRouter::build_tool_call()
    │ 解析 ResponseItem → ToolCall + ToolPayload
    ▼
ToolOrchestrator
    │ 审批检查 → 沙箱选择 → 执行
    ▼
ToolHandler::handle()
    │ 执行具体工具逻辑
    ▼
FunctionCallOutput (返回给模型)
```

---

## 14. Prompt 模板变量系统

模板使用 `codex_utils_template::Template` 进行渲染，变量语法为 `{{ variable }}`。

### 14.1 常用变量

| 变量 | 使用位置 | 说明 |
|------|---------|------|
| `{{ personality }}` | instructions_template.md | 人格定义文本 |
| `{{ network_access }}` | sandbox_mode/*.md | 网络访问权限描述 |
| `{{KNOWN_MODE_NAMES}}` | collaboration_mode/default.md | 已知模式名称列表 |
| `{{REQUEST_USER_INPUT_AVAILABILITY}}` | collaboration_mode/default.md | 用户输入工具可用性 |
| `{{ASKING_QUESTIONS_GUIDANCE}}` | collaboration_mode/default.md | 提问指南 |
| `{{ base_path }}` | memories/read_path.md | 记忆基础路径 |
| `{{ memory_summary }}` | memories/read_path.md | 记忆摘要内容 |
| `{{ rollout_path }}` | memories/stage_one_input.md | 执行记录路径 |
| `{{ rollout_cwd }}` | memories/stage_one_input.md | 执行时工作目录 |
| `{{ rollout_contents }}` | memories/stage_one_input.md | 执行记录内容 |
| `{{results}}` | review/exit_success.xml | 审查结果内容 |

---

## 15. 关键源文件索引

### Prompt 文件

| 文件 | 说明 |
|------|------|
| `codex-rs/core/gpt-5.2-codex_prompt.md` | 最完整的 system prompt |
| `codex-rs/core/gpt_5_codex_prompt.md` | GPT-5 基础版 |
| `codex-rs/core/gpt_5_1_prompt.md` | GPT-5.1 版（含 User Updates） |
| `codex-rs/core/gpt_5_2_prompt.md` | GPT-5.2 版 |
| `codex-rs/core/prompt_with_apply_patch_instructions.md` | 含 apply_patch 语法的完整 prompt |
| `codex-rs/core/prompt.md` | 最小 AGENTS.md 规范 |
| `codex-rs/core/review_prompt.md` | 代码审查 prompt |

### Prompt 组装代码

| 文件 | 说明 |
|------|------|
| `codex-rs/core/src/client_common.rs` | `Prompt` 结构体定义 |
| `codex-rs/core/src/models_manager/model_info.rs` | 模型 instructions 加载、人格选择 |
| `codex-rs/protocol/src/models.rs` | `BaseInstructions`、`DeveloperInstructions` 组装 |
| `codex-rs/core/src/models_manager/collaboration_mode_presets.rs` | 协作模式加载与渲染 |
| `codex-rs/core/src/environment_context.rs` | 环境上下文生成 |
| `codex-rs/core/src/project_doc.rs` | AGENTS.md 发现与加载 |
| `codex-rs/instructions/src/fragment.rs` | 上下文片段标签定义 |
| `codex-rs/instructions/src/user_instructions.rs` | 用户指令序列化 |

### 工具定义代码

| 文件 | 说明 |
|------|------|
| `codex-rs/tools/src/tool_spec.rs` | `ToolSpec` 枚举定义 |
| `codex-rs/tools/src/tool_definition.rs` | `ToolDefinition` 结构体 |
| `codex-rs/tools/src/local_tool.rs` | Shell 类工具定义 |
| `codex-rs/tools/src/apply_patch_tool.rs` | `apply_patch` 工具定义 |
| `codex-rs/tools/src/plan_tool.rs` | `update_plan` 工具定义 |
| `codex-rs/tools/src/request_user_input_tool.rs` | 用户输入请求工具 |
| `codex-rs/tools/src/view_image.rs` | 图像查看工具 |
| `codex-rs/tools/src/agent_tool.rs` | 多 agent 工具 |
| `codex-rs/tools/src/tool_discovery.rs` | 工具发现/搜索 |
| `codex-rs/tools/src/mcp_tool.rs` | MCP 工具转换 |
| `codex-rs/tools/src/dynamic_tool.rs` | 动态工具解析 |

### 工具注册与路由代码

| 文件 | 说明 |
|------|------|
| `codex-rs/core/src/tools/spec.rs` | **中心工具注册器** — `build_specs_with_discoverable_tools()` |
| `codex-rs/core/src/tools/router.rs` | `ToolRouter` — 工具调度 |
| `codex-rs/core/src/tools/registry.rs` | `ToolRegistry` + `ToolHandler` trait |
| `codex-rs/core/src/tools/orchestrator.rs` | `ToolOrchestrator` — 审批 + 沙箱 + 重试 |
| `codex-rs/core/src/tools/handlers/` | 各工具 handler 实现 |

### 会话与 API 通信

| 文件 | 说明 |
|------|------|
| `codex-rs/core/src/codex.rs` | `Codex` + `Session` + `TurnContext` |
| `codex-rs/core/src/client.rs` | `ModelClient` + `ModelClientSession`，API 调用 |
| `codex-rs/core/src/context_manager/history.rs` | `ContextManager` 对话历史管理 |
| `codex-rs/codex-api/` | OpenAI Responses API 客户端 |
