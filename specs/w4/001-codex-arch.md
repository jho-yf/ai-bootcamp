# Codex 架构分析文档

> 本文档基于 OpenAI Codex CLI 源码进行深度分析，详细介绍其架构设计、核心模块、数据流和技术栈。

## 1. 项目概述

Codex 是 OpenAI 开发的本地 AI 编码助手，采用 Rust 语言编写核心逻辑，提供终端用户界面（TUI）和命令行界面（CLI）两种交互方式。该项目支持与多种 AI 模型（OpenAI、Ollama、LM Studio 等）集成，并通过 MCP（Model Context Protocol）协议实现工具扩展。

### 1.1 项目规模

| 指标 | 数值 |
|------|------|
| 核心 Rust 代码行数 | ~107,266 行 |
| Rust Crates 数量 | 50+ 个 |
| 主要编程语言 | Rust (Edition 2024), TypeScript |
| 构建系统 | Bazel, Cargo |

---

## 2. 整体架构

### 2.1 系统架构图

```mermaid
graph TB
    subgraph "用户界面层"
        CLI[CLI 入口<br/>codex-rs/cli]
        TUI[终端界面<br/>codex-rs/tui]
        APP[应用服务器<br/>codex-rs/app-server]
    end

    subgraph "核心业务层"
        CORE[核心引擎<br/>codex-rs/core]
        PROTO[协议层<br/>codex-rs/protocol]
        THREAD[会话管理<br/>thread_manager]
        CTX[上下文管理<br/>context_manager]
    end

    subgraph "工具与集成层"
        SHELL[Shell 执行<br/>shell-command]
        MCP[MCP 集成<br/>mcp-server]
        FILE[文件搜索<br/>file-search]
        SKILLS[Skills 框架<br/>skills]
    end

    subgraph "基础设施层"
        STATE[状态存储<br/>codex-rs/state]
        CONFIG[配置管理<br/>codex-rs/config]
        AUTH[认证模块<br/>codex-rs/login]
        NET[网络代理<br/>network-proxy]
    end

    subgraph "外部服务"
        OPENAI[OpenAI API]
        OLLAMA[Ollama]
        LMSTUDIO[LM Studio]
        MCPS[MCP Servers]
    end

    CLI --> CORE
    TUI --> CORE
    APP --> CORE

    CORE --> PROTO
    CORE --> THREAD
    CORE --> CTX
    CORE --> SHELL
    CORE --> MCP
    CORE --> FILE
    CORE --> SKILLS

    CORE --> STATE
    CORE --> CONFIG
    CORE --> AUTH
    CORE --> NET

    CORE --> OPENAI
    CORE --> OLLAMA
    CORE --> LMSTUDIO
    MCP --> MCPS
```

### 2.2 分层架构说明

| 层级 | 职责 | 主要模块 |
|------|------|----------|
| **用户界面层** | 处理用户交互，渲染界面 | cli, tui, app-server |
| **核心业务层** | 实现核心 AI 对话逻辑 | core, protocol, thread_manager |
| **工具与集成层** | 提供工具调用和外部集成 | shell-command, mcp-server, file-search |
| **基础设施层** | 提供持久化、配置、认证等基础能力 | state, config, login |

---

## 3. 目录结构

### 3.1 顶层目录

```
venders/codex/
├── codex-cli/          # Node.js CLI 包装器
├── codex-rs/           # Rust 核心代码 (主要业务逻辑)
├── sdk/                # SDK 组件
├── shell-tool-mcp/     # MCP Shell 工具
└── docs/               # 文档
```

### 3.2 核心模块结构 (codex-rs)

```mermaid
graph LR
    subgraph "入口模块"
        A1[cli]
        A2[tui]
        A3[app-server]
        A4[mcp-server]
    end

    subgraph "核心模块"
        B1[core<br/>107K LOC]
        B2[protocol]
        B3[state]
        B4[config]
    end

    subgraph "功能模块"
        C1[shell-command]
        C2[file-search]
        C3[backend-client]
        C4[login]
        C5[skills]
        C6[hooks]
    end

    subgraph "集成模块"
        D1[ollama]
        D2[lmstudio]
        D3[rmcp-client]
        D4[network-proxy]
    end

    subgraph "工具库"
        E1[utils/*<br/>14+ 子模块]
    end

    A1 --> B1
    A2 --> B1
    A3 --> B1
    A4 --> B1

    B1 --> B2
    B1 --> B3
    B1 --> B4

    B1 --> C1
    B1 --> C2
    B1 --> C3
    B1 --> C4
    B1 --> C5
    B1 --> C6

    B1 --> D1
    B1 --> D2
    B1 --> D3
    B1 --> D4

    B1 --> E1
```

---

## 4. 核心模块详解

### 4.1 CLI 模块 (codex-rs/cli)

CLI 是系统的入口点，提供多种子命令：

```mermaid
graph TD
    MAIN[codex 命令] --> SUB1[codex<br/>交互式 TUI]
    MAIN --> SUB2[codex exec<br/>非交互式执行]
    MAIN --> SUB3[codex review<br/>代码审查]
    MAIN --> SUB4[codex login<br/>身份认证]
    MAIN --> SUB5[codex mcp<br/>MCP 管理]
    MAIN --> SUB6[codex app-server<br/>启动应用服务器]
    MAIN --> SUB7[codex resume<br/>恢复会话]
    MAIN --> SUB8[codex fork<br/>会话分支]
    MAIN --> SUB9[codex cloud<br/>云任务]
```

**主要子命令说明：**

| 命令 | 功能 | 使用场景 |
|------|------|----------|
| `codex` | 启动交互式 TUI | 日常编码辅助 |
| `codex exec` | 非交互式执行 | 脚本集成、CI/CD |
| `codex review` | 代码审查 | PR 审查、代码质量检查 |
| `codex login` | 身份认证 | 首次使用、更换账户 |
| `codex mcp-server` | 作为 MCP 服务器运行 | 与其他 AI 工具集成 |
| `codex app-server` | 启动应用服务器 | IDE 扩展集成 |
| `codex resume` | 恢复会话 | 继续之前的工作 |
| `codex fork` | 创建会话分支 | 探索不同方案 |

### 4.2 核心引擎 (codex-rs/core)

核心引擎是整个系统的核心，包含约 107,266 行代码，负责：

```mermaid
graph TB
    subgraph "core 模块内部结构"
        CODEX[Codex 主结构<br/>codex.rs]
        CLIENT[模型客户端<br/>client/]
        THREAD_MGR[会话管理器<br/>thread_manager/]
        CTX_MGR[上下文管理器<br/>context_manager/]
        COMPACT[上下文压缩<br/>compact/]
        TOOLS[工具系统<br/>tools/]
        MCP[MCP 集成<br/>mcp/]
        AGENT[Agent 控制<br/>agent/]
        SHELL[Shell 执行<br/>shell/]
        EXEC[执行策略<br/>exec/]
        AUTH[认证管理<br/>auth/]
        CONFIG[配置系统<br/>config/]
        MODELS[模型管理<br/>models_manager/]
        FEATURES[功能标志<br/>features/]
        HOOKS[Hook 系统<br/>hooks/]
        MEMORIES[记忆系统<br/>memories/]
    end

    CODEX --> CLIENT
    CODEX --> THREAD_MGR
    CODEX --> CTX_MGR
    CODEX --> TOOLS
    CODEX --> MCP
    CODEX --> AGENT

    THREAD_MGR --> CTX_MGR
    CTX_MGR --> COMPACT
    TOOLS --> SHELL
    TOOLS --> MCP
    SHELL --> EXEC
```

#### 4.2.1 Codex 主结构

`Codex` 是核心结构体，负责：
- 管理与 AI 模型的实时对话
- 协调各个子系统（工具、MCP、Shell 等）
- 处理事件流和消息分发

#### 4.2.2 会话管理器 (ThreadManager)

负责：
- 创建和管理会话（CodexThread）
- 会话持久化和恢复
- 会话分支（fork）功能

#### 4.2.3 上下文管理器 (ContextManager)

负责：
- 管理发送给模型的上下文窗口大小
- 触发上下文压缩
- 保持对话连贯性

### 4.3 协议层 (codex-rs/protocol)

协议层定义了系统中的核心数据类型：

```mermaid
classDiagram
    class TurnItem {
        <<enum>>
        UserMessage
        AgentMessage
        Plan
        Reasoning
        WebSearch
        ContextCompaction
    }

    class UserInput {
        <<enum>>
        Text
        Image
        LocalImage
    }

    class DynamicToolSpec {
        +name: String
        +description: String
        +input_schema: JsonValue
    }

    class ApprovalRequest {
        +id: String
        +tool_name: String
        +args: JsonValue
    }

    TurnItem --> UserInput : contains
    TurnItem --> DynamicToolSpec : references
```

**核心类型说明：**

| 类型 | 用途 |
|------|------|
| `TurnItem` | 对话中的每一项（用户消息、AI 响应、计划等） |
| `UserInput` | 用户输入类型（文本、远程图片、本地图片） |
| `DynamicToolSpec` | 动态工具规范定义 |
| `ApprovalRequest` | 工具执行审批请求 |

### 4.4 TUI 模块 (codex-rs/tui)

终端用户界面模块，基于 Ratatui 框架构建：

```mermaid
graph TB
    subgraph "TUI 模块结构"
        APP[应用状态<br/>app.rs]
        TUI[TUI 封装<br/>tui.rs]
        CHAT[聊天组件<br/>chatwidget/]
        BOTTOM[底部输入<br/>bottom_pane/]
        HISTORY[历史单元格<br/>history_cell/]
        DIFF[Diff 渲染<br/>diff_render/]
        FILE_SEARCH[文件搜索<br/>file_search/]
        RESUME[会话恢复<br/>resume_picker/]
        ONBOARD[新用户引导<br/>onboarding/]
        COLLAB[协作模式<br/>collaboration_modes/]
        EDITOR[外部编辑器<br/>external_editor/]
        MARKDOWN[Markdown 渲染<br/>markdown_render/]
    end

    APP --> TUI
    APP --> CHAT
    CHAT --> BOTTOM
    CHAT --> HISTORY
    CHAT --> DIFF
    APP --> FILE_SEARCH
    APP --> RESUME
    APP --> ONBOARD
    APP --> COLLAB
    APP --> EDITOR
    CHAT --> MARKDOWN
```

**主要组件说明：**

| 组件 | 功能 |
|------|------|
| `app.rs` | 主应用状态机和事件循环 |
| `chatwidget/` | 聊天消息显示和交互 |
| `bottom_pane/` | 用户输入区域 |
| `history_cell/` | 历史消息渲染 |
| `diff_render/` | 代码差异可视化 |
| `markdown_render/` | Markdown 内容渲染 |

---

## 5. 核心业务流程

### 5.1 会话生命周期

```mermaid
sequenceDiagram
    participant User as 用户
    participant TUI as 终端界面
    participant TM as ThreadManager
    participant CT as CodexThread
    participant CM as ContextManager
    participant MC as ModelClient
    participant API as AI API

    User->>TUI: 输入消息
    TUI->>TM: 创建/获取会话
    TM->>CT: 初始化会话
    CT->>CM: 注册上下文
    CM->>MC: 发送请求
    MC->>API: WebSocket 连接
    API-->>MC: 流式响应
    MC-->>CM: 处理事件流
    CM-->>CT: 更新会话状态
    CT-->>TUI: 渲染响应
    TUI-->>User: 显示结果
```

### 5.2 工具调用流程

```mermaid
sequenceDiagram
    participant Agent as AI Agent
    participant Codex as Codex 核心
    participant Policy as 执行策略
    participant TUI as 用户界面
    participant Tool as 工具执行器
    participant MCP as MCP 服务器

    Agent->>Codex: 请求工具调用
    Codex->>Policy: 检查执行策略

    alt 需要审批
        Policy->>TUI: 显示审批请求
        TUI->>User: 等待用户决定
        User->>TUI: 批准/拒绝/修改
        TUI->>Policy: 返回决定
    end

    Policy-->>Codex: 允许执行

    alt Shell 工具
        Codex->>Tool: 执行 Shell 命令
        Tool-->>Codex: 返回结果
    else MCP 工具
        Codex->>MCP: 调用 MCP 工具
        MCP-->>Codex: 返回结果
    end

    Codex-->>Agent: 工具执行结果
```

### 5.3 上下文管理流程

```mermaid
flowchart TD
    A[用户消息] --> B[添加到上下文]
    B --> C{检查上下文大小}
    C -->|未超限| D[继续对话]
    C -->|超过阈值| E[触发压缩]
    E --> F[Compact 模块处理]
    F --> G[保留关键信息]
    G --> H[删除过时内容]
    H --> I[生成 ContextCompactionItem]
    I --> J[更新上下文窗口]
    J --> D
```

### 5.4 MCP 集成流程

```mermaid
sequenceDiagram
    participant CM as 连接管理器
    participant Config as 配置
    participant Proc as 子进程/HTTP
    participant Server as MCP 服务器
    participant Tool as 工具调用

    CM->>Config: 加载 MCP 服务器配置

    loop 每个 MCP 服务器
        alt stdio 传输
            CM->>Proc: 启动子进程
            Proc->>Server: stdin/stdout 通信
        else HTTP 传输
            CM->>Server: 建立 HTTP 连接
        end

        Server-->>CM: 返回可用工具列表
    end

    Note over CM: 工具调用时

    CM->>Server: 调用工具
    Server-->>CM: 返回结果
    CM-->>Tool: 返回给调用者
```

---

## 6. 配置系统

### 6.1 配置类型

```mermaid
classDiagram
    class McpServerConfig {
        +name: String
        +transport: Transport
        +enabled: bool
    }

    class Transport {
        <<enum>>
        stdio
        streamable_http
    }

    class AppsConfigToml {
        +apps: Vec~AppConfig~
    }

    class AppConfig {
        +name: String
        +command: String
        +args: Vec~String~
    }

    class SandboxMode {
        <<enum>>
        ReadOnly
        WorkspaceWrite
        DangerFullAccess
    }

    class AskForApproval {
        <<enum>>
        UnlessTrusted
        OnFailure
        OnRequest
        Reject
        Never
    }

    McpServerConfig --> Transport
    AppsConfigToml --> AppConfig
```

### 6.2 沙盒模式

| 模式 | 权限 | 使用场景 |
|------|------|----------|
| `ReadOnly` | 只读文件系统 | 安全审查模式 |
| `WorkspaceWrite` | 工作区可写 | 日常开发（默认） |
| `DangerFullAccess` | 完全访问 | 信任的环境 |

### 6.3 审批策略

| 策略 | 行为 |
|------|------|
| `UnlessTrusted` | 除非是信任的项目，否则请求审批 |
| `OnFailure` | 执行失败时请求审批 |
| `OnRequest` | 按需请求审批 |
| `Reject` | 自动拒绝（可配置具体行为） |
| `Never` | 从不请求审批 |

---

## 7. 工具系统

### 7.1 工具类型

```mermaid
graph TB
    subgraph "内置工具"
        T1[Shell 工具]
        T2[文件操作]
        T3[代码编辑]
    end

    subgraph "MCP 工具"
        T4[外部 MCP 服务器]
        T5[自定义工具]
    end

    subgraph "动态工具"
        T6[运行时定义工具]
        T7[App Connectors]
    end

    TOOLS[工具系统] --> T1
    TOOLS --> T2
    TOOLS --> T3
    TOOLS --> T4
    TOOLS --> T5
    TOOLS --> T6
    TOOLS --> T7
```

### 7.2 Shell 命令执行

```mermaid
flowchart LR
    A[AI 请求命令] --> B[命令解析]
    B --> C{策略检查}
    C -->|允许| D[沙盒执行]
    C -->|需审批| E[用户审批]
    E -->|批准| D
    E -->|拒绝| F[返回错误]
    D --> G[捕获输出]
    G --> H[返回结果]
```

---

## 8. MCP 协议集成

### 8.1 MCP 架构

```mermaid
graph LR
    subgraph "Codex"
        MGR[连接管理器]
        CLIENT[MCP 客户端]
    end

    subgraph "传输层"
        STDIO[stdio]
        HTTP[streamable_http]
    end

    subgraph "MCP 服务器"
        S1[文件系统]
        S2[数据库]
        S3[API 集成]
        S4[自定义工具]
    end

    MGR --> CLIENT
    CLIENT --> STDIO
    CLIENT --> HTTP
    STDIO --> S1
    STDIO --> S2
    HTTP --> S3
    HTTP --> S4
```

### 8.2 MCP 功能

- **工具 (Tools)**: 可调用的函数
- **资源 (Resources)**: 可读取的数据
- **提示词 (Prompts)**: 预定义的提示模板

---

## 9. 技术栈

### 9.1 核心依赖

| 类别 | 依赖 | 用途 |
|------|------|------|
| **异步运行时** | tokio | 异步任务调度 |
| **UI 框架** | ratatui | 终端界面渲染 |
| **序列化** | serde, serde_json | JSON/TOML 序列化 |
| **网络** | reqwest, tokio-tungstenite | HTTP/WebSocket 通信 |
| **数据库** | sqlx | SQLite 持久化 |
| **CLI** | clap | 命令行解析 |
| **沙盒** | landlock, seccompiler | Linux 安全沙盒 |
| **认证** | keyring | 密钥存储 |
| **搜索** | nucleo, ignore | 文件搜索和忽略 |
| **解析** | tree-sitter, tree-sitter-bash | 语法解析 |
| **渲染** | pulldown-cmark, syntect | Markdown/语法高亮 |
| **遥测** | opentelemetry | 可观测性 |

### 9.2 构建系统

```mermaid
graph LR
    A[源代码] --> B{构建系统}
    B -->|Bazel| C[生产构建]
    B -->|Cargo| D[开发/测试]
    C --> E[二进制文件]
    D --> E
```

---

## 10. 安全架构

### 10.1 多层安全模型

```mermaid
graph TB
    subgraph "安全层"
        L1[审批策略层<br/>AskForApproval]
        L2[执行策略层<br/>ExecPolicy]
        L3[沙盒层<br/>Sandbox]
        L4[系统安全层<br/>OS Security]
    end

    A[工具调用请求] --> L1
    L1 --> L2
    L2 --> L3
    L3 --> L4
    L4 --> B[实际执行]
```

### 10.2 平台特定沙盒

| 平台 | 沙盒技术 |
|------|----------|
| Linux | Landlock + Seccomp |
| macOS | Seatbelt (sandbox-exec) |
| Windows | Restricted Token |

---

## 11. 状态持久化

### 11.1 状态存储架构

```mermaid
erDiagram
    SESSION {
        string id PK
        string title
        datetime created_at
        datetime updated_at
    }

    MESSAGE {
        string id PK
        string session_id FK
        string role
        text content
        datetime timestamp
    }

    STATE {
        string key PK
        text value
        datetime updated_at
    }

    SESSION ||--o{ MESSAGE : contains
```

### 11.2 会话记录 (Rollout)

- 完整的会话历史记录
- 支持会话恢复和分支
- 存储在本地文件系统

---

## 12. 扩展机制

### 12.1 Skills 框架

基于 `SKILL.md` 文件的自定义行为定义：

```mermaid
flowchart LR
    A[SKILL.md] --> B[Skill 解析器]
    B --> C[Skill 注册]
    C --> D[执行上下文]
    D --> E[自定义行为]
```

### 12.2 Hooks 系统

```mermaid
sequenceDiagram
    participant Event as 事件触发
    participant Hook as Hook 系统
    participant Handler as 处理器

    Event->>Hook: 触发事件
    Hook->>Handler: 执行预钩子
    Handler-->>Hook: 返回结果
    Hook->>Event: 继续执行
    Event->>Hook: 完成事件
    Hook->>Handler: 执行后钩子
```

---

## 13. App Server 协议

### 13.1 协议版本

| 版本 | 状态 | 特点 |
|------|------|------|
| V1 | 已弃用 | 旧版协议 |
| V2 | 当前 | camelCase, JSON-RPC, TypeScript 绑定自动生成 |

### 13.2 V2 协议特性

- 实验性 API 支持 (ExperimentalApi 宏)
- 游标分页支持
- 自动生成 TypeScript 类型定义

---

## 14. 总结

### 14.1 架构优势

1. **模块化设计**: 50+ 个独立 crates，职责清晰
2. **分层架构**: UI、业务、协议、基础设施分离
3. **事件驱动**: 异步处理，高响应性
4. **可扩展性**: Skills、MCP、Hooks 多种扩展机制
5. **安全性**: 多层安全模型，平台特定沙盒

### 14.2 核心设计模式

| 模式 | 应用场景 |
|------|----------|
| 分层架构 | 整体系统设计 |
| 事件驱动 | UI 更新、消息处理 |
| 策略模式 | 审批、沙盒、执行策略 |
| 插件系统 | Skills、MCP、Hooks |
| 仓库模式 | 状态持久化 |

### 14.3 关键文件路径

| 功能 | 路径 |
|------|------|
| CLI 入口 | `codex-rs/cli/src/main.rs` |
| 核心引擎 | `codex-rs/core/src/codex.rs` |
| 协议定义 | `codex-rs/protocol/src/items.rs` |
| 配置类型 | `codex-rs/core/src/config/types.rs` |
| 状态管理 | `codex-rs/state/src/lib.rs` |
| TUI 应用 | `codex-rs/tui/src/app.rs` |

---

## 附录：Mermaid 图表源码

本文档中所有 Mermaid 图表均可直接在支持 Mermaid 的 Markdown 渲染器中显示，包括：
- GitHub
- GitLab
- VS Code (with Mermaid extension)
- Typora
- Obsidian

---

*文档生成时间: 2026-02-27*
*分析版本: Codex CLI (latest)*
