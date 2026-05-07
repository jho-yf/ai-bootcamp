# Code Review Agent 设计文档

## 概述

基于 `w6-simple-agent` 的 `agent-core` SDK，构建一个 Code Review Agent。

**核心设计原则：LLM 驱动，Agent 只提供工具和 system prompt。**

- **LLM 负责**：理解用户意图、决定调用哪些工具、以什么顺序调用、读取哪些文件、如何组织 review 报告
- **Agent 负责**：加载 system prompt、注册工具、启动对话循环
- **工具负责**：执行具体的 I/O 操作（读文件、跑 git 命令），不包含任何业务逻辑

System prompt（`specs/w6/codereview-prompt/system.md`）是 Agent 行为的唯一来源。工具只是 LLM 的"手"，不做意图判断，不限制使用场景。

---

## 架构

```
用户输入 (CLI 参数)
      │
      ▼
  main.rs
  ├── 读取 system.md 作为 system prompt
  ├── 构建 Agent (agent-core)
  ├── 注册工具: read_file / list_directory / write_file / run_git / run_gh
  └── agent.run(user_message)
            │
            ▼
       LLM 工具调用循环
       ├── run_git → 获取 diff / log / status
       ├── read_file → 读取完整文件上下文
       ├── list_directory → 探索目录结构
       └── write_file → 输出 review 报告到文件（可选）
```

Agent 使用 `agent-core` 的标准 `Tool` trait，无需修改 SDK 本身。

---

## 工具设计

### 1. `read_file`

读取单个文件的完整内容。

**参数：**
```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "文件路径（绝对路径或相对于仓库根目录的相对路径）"
    }
  },
  "required": ["path"]
}
```

**实现要点：**
- 直接使用 `std::fs::read_to_string`
- 文件不存在时返回描述性错误，不 panic
- 不限制路径（Agent 需要读取任意项目文件）
- 相对路径相对于进程启动时的 CWD 解析（即调用 `cargo run` 时所在的目录）；从子目录调用时行为可能不同，建议使用绝对路径

**示例调用：**
```json
{ "path": "src/auth.rs" }
{ "path": "/home/user/project/Cargo.toml" }
```

---

### 2. `list_directory`

列出目录下的文件和子目录，用于探索项目结构。

**参数：**
```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "目录路径"
    }
  },
  "required": ["path"]
}
```

**实现要点：**
- 目录名后缀 `/` 以区分文件和目录
- 结果按字母排序
- 与 `basic_tools` 示例中的实现一致

---

### 3. `write_file`

将内容写入文件。工具本身不限制用途，由 LLM 根据 system prompt 的指引决定何时调用（例如用户要求保存报告时）。

**参数：**
```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "目标文件路径"
    },
    "content": {
      "type": "string",
      "description": "要写入的内容"
    }
  },
  "required": ["path", "content"]
}
```

**实现要点：**
- 使用 `std::fs::write`，覆盖已有文件
- 自动创建父目录（`std::fs::create_dir_all`）
- 写入成功后返回确认信息，如 `"Written 1234 bytes to review.md"`

---

### 4. `run_git`

执行只读 git 命令，获取 diff、log、status 等信息。

**参数：**
```json
{
  "type": "object",
  "properties": {
    "args": {
      "type": "array",
      "items": { "type": "string" },
      "description": "git 子命令及参数，JSON 字符串数组，例如 [\"diff\", \"main...HEAD\"]"
    }
  },
  "required": ["args"]
}
```

**安全限制 — 命令白名单：**

只允许以下只读子命令，拒绝其他所有命令：

| 允许的子命令 | 用途 |
|---|---|
| `diff` | 获取变更内容 |
| `log` | 查看提交历史 |
| `show` | 查看单个提交 |
| `status` | 查看工作区状态 |
| `rev-parse` | 解析引用（获取分支名、merge-base 等） |
| `merge-base` | 找到分支分叉点 |
| `branch` | 列出分支（只读标志） |
| `stash list` | 列出 stash |

拒绝 `commit`、`push`、`reset`、`checkout`、`merge`、`rebase` 等写操作，返回错误信息。

**实现要点：**
- 使用 `std::process::Command::new("git")` 执行
- 将 stdout 和 stderr 合并返回
- 超时设置：30 秒
- 命令执行失败时返回 stderr 内容（LLM 可据此调整参数）

**常用调用示例：**

```json
// 未暂存的变更
{ "args": ["diff"] }

// 已暂存的变更
{ "args": ["diff", "--cached"] }

// 所有未提交变更
{ "args": ["diff", "HEAD"] }

// 查看单个提交
{ "args": ["show", "abc1234"] }

// 某提交之后的所有变更
{ "args": ["diff", "abc1234..HEAD"] }

// 当前分支 vs main
{ "args": ["diff", "main...HEAD"] }

// 获取当前分支名
{ "args": ["rev-parse", "--abbrev-ref", "HEAD"] }

// 找到与 main 的分叉点
{ "args": ["merge-base", "main", "HEAD"] }

// 查看最近 10 条提交
{ "args": ["log", "--oneline", "-10"] }

// 查看工作区状态（含未跟踪文件）
{ "args": ["status", "--short"] }
```

---

### 5. `run_gh`

执行 GitHub CLI 命令，用于 PR review 场景。

**参数：**
```json
{
  "type": "object",
  "properties": {
    "args": {
      "type": "array",
      "items": { "type": "string" },
      "description": "gh 子命令及参数，JSON 字符串数组，例如 [\"pr\", \"diff\", \"123\"]"
    }
  },
  "required": ["args"]
}
```

**安全限制 — 命令白名单：**

正向规则：`args[0]` 必须为 `"pr"`，且 `args[1]` 必须在 `{view, diff, list}` 中。不满足此规则的所有命令一律拒绝。

| 允许的命令 | 用途 |
|---|---|
| `pr view <number>` | 获取 PR 标题、描述、作者 |
| `pr diff <number>` | 获取 PR 完整 diff |
| `pr view <number> --json files` | 获取变更文件列表 |
| `pr view <number> --json reviews,comments` | 获取 review 和评论 |
| `pr view` | 获取当前分支的 PR |
| `pr list` | 列出开放的 PR |

**实现要点：**
- 使用 `std::process::Command::new("gh")` 执行
- `gh` 不存在时返回友好错误提示
- 超时设置：30 秒

**常用调用示例：**

```json
// 查看 PR 基本信息
{ "args": ["pr", "view", "123"] }

// 获取 PR 完整 diff
{ "args": ["pr", "diff", "123"] }

// 获取变更文件列表
{ "args": ["pr", "view", "123", "--json", "files"] }

// 获取 PR 的 review 和评论
{ "args": ["pr", "view", "123", "--json", "reviews,comments"] }

// 获取当前分支对应的 PR
{ "args": ["pr", "view"] }
```

---

## Agent 组装

### 文件结构

```
w6-codereview-agent/
├── Cargo.toml
└── src/
    └── main.rs
```

`agent-core` 作为路径依赖引入：

```toml
[dependencies]
agent-core = { path = "../w6-simple-agent/crates/agent-core" }
tokio = { version = "1", features = ["full"] }
anyhow = "1"
async-trait = "0.1"
serde_json = "1"
```

### main.rs 核心逻辑

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 1. 读取 system prompt
    let system_prompt = fs::read_to_string("specs/w6/codereview-prompt/system.md")?;

    // 2. 解析 CLI 参数，构造用户消息
    let args: Vec<String> = std::env::args().skip(1).collect();
    let user_message = build_user_message(&args);

    // 3. 构建 Agent
    let mut agent = Agent::new(AgentConfig {
        model: std::env::var("OPENAI_MODEL").unwrap_or("gpt-4o".into()),
        system_prompt,
        max_steps: 30,
        api_base: std::env::var("OPENAI_API_BASE")
            .unwrap_or("https://api.openai.com".into()),
        api_key: std::env::var("OPENAI_API_KEY")?,
    });

    // 4. 注册工具
    agent.add_tool(ReadFileTool);
    agent.add_tool(ListDirectoryTool);
    agent.add_tool(WriteFileTool);
    agent.add_tool(RunGitTool);
    agent.add_tool(RunGhTool);

    // 5. 运行
    let reply = agent.run(&user_message).await?;
    println!("{}", reply);
    Ok(())
}
```

### 用户消息构造

```rust
fn build_user_message(args: &[String]) -> String {
    // CLI 不做意图解析，原样传给 LLM
    // system prompt 中的规则负责判断 review 类型（含无参数时的默认行为）
    args.join(" ")
}
```

---

## 用户使用方式

```bash
# 1. review 所有未提交变更（默认）
cargo run

# 2. review 当前 branch 相对于 main 的新代码
cargo run -- "current branch"
cargo run -- "review current branch changes"

# 3. review 某个 commit 之后的所有代码
cargo run -- "after commit 14hjd5"
cargo run -- "14hjd5..HEAD"

# 4. review 某个具体 commit
cargo run -- "14hjd5"

# 5. review 某个 PR
cargo run -- "PR 123"
cargo run -- "123"

# 6. review 某个分支相对于 HEAD 的变更
cargo run -- "feature/my-branch"
```

---

## 工具调用流程示例

### 场景：`cargo run -- "current branch"`

```
用户: "review current branch changes"

LLM → run_git(["rev-parse", "--abbrev-ref", "HEAD"])
    ← "feature/auth-refactor"

LLM → run_git(["merge-base", "main", "HEAD"])
    ← "a1b2c3d4"

LLM → run_git(["diff", "a1b2c3d4...HEAD"])
    ← <完整 diff>

LLM → run_git(["status", "--short"])
    ← <未跟踪文件列表>

LLM → read_file("src/auth.rs")          ← 读取被修改文件的完整内容
    ← <文件内容>

LLM → read_file("src/auth/middleware.rs")
    ← <文件内容>

LLM → 输出 review 报告
```

### 场景：`cargo run -- "after commit 14hjd5"`

```
用户: "after commit 14hjd5"

LLM → run_git(["log", "--oneline", "14hjd5..HEAD"])
    ← "a1b2c3 feat: add login\nb2c3d4 fix: token expiry"

LLM → run_git(["diff", "14hjd5..HEAD"])
    ← <完整 diff>

LLM → read_file("src/login.rs")
    ← <文件内容>

LLM → 输出 review 报告
```

---

## 安全边界

| 工具 | 风险 | 缓解措施 |
|---|---|---|
| `read_file` | 读取敏感文件（.env、密钥） | 不限制路径（review 需要读取任意文件）；system prompt 指示 LLM 不在报告中输出密钥值 |
| `write_file` | 覆盖源代码 | 软约束（system prompt 指示 LLM 只在用户明确要求时写文件）。这是已接受的权衡：review agent 需要能写任意路径的报告文件，在工具层限制路径会妨碍合法用途。如需更强隔离，可在工具层限制只允许写 `.md` 文件。 |
| `run_git` | 执行破坏性 git 操作 | 工具层子命令白名单，拒绝所有写操作，这是硬限制，不依赖 LLM 判断 |
| `run_gh` | 创建/关闭 PR、发评论 | 工具层子命令白名单，只允许 `pr view`/`pr diff`/`pr list`，硬限制 |

---

## 扩展方向

- **输出格式**：支持 `--output review.md` 参数，自动调用 `write_file` 保存报告
- **多仓库**：支持 `--repo /path/to/repo` 参数，在指定目录执行 git 命令
- **严重级别过滤**：支持 `--level bug` 只输出 bug 级别问题
- **增量 review**：记录上次 review 的 commit hash，下次自动 review 新增变更
