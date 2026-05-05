# Examples

三个示例展示了如何使用 `agent-core` SDK 构建不同场景的 Agent。

## 前置条件

所有示例都需要设置 OpenAI API Key：

```bash
export OPENAI_API_KEY=sk-...
```

可选环境变量（适用于所有示例）：

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `OPENAI_API_KEY` | 必填 | OpenAI 或兼容 API 的密钥 |
| `OPENAI_API_BASE` | `https://api.openai.com` | API 端点，支持任何 OpenAI 兼容服务 |
| `OPENAI_MODEL` | `gpt-4o` | 使用的模型 ID |

---

## 1. basic_tools — 文件系统工具

演示如何实现 `read_file` 和 `list_directory` 两个自定义工具，让 Agent 能够浏览文件系统。

**工具：**
- `read_file(path)` — 读取文件内容
- `list_directory(path)` — 列出目录下的文件和子目录

**运行：**

```bash
cargo run -p basic_tools
```

**预期输出：**

```
User: List the files in /tmp and tell me what you see
Assistant: The /tmp directory contains the following files: ...
```

**验证要点：**
- Agent 调用 `list_directory` 工具获取 `/tmp` 目录内容
- 如果目录中有文件，Agent 可能进一步调用 `read_file` 读取内容
- 最终给出自然语言描述，而非原始工具输出

---

## 2. calculator — 自定义计算工具

演示如何实现一个无外部依赖的数学表达式求值工具。支持 `+`、`-`、`*`、`/`、`%`、`^`（幂）和括号。

**工具：**
- `calculate(expression)` — 计算数学表达式，返回数值结果

**运行：**

```bash
cargo run -p calculator
```

**预期输出：**

```
User: What is 15% of 847, and then multiply that by 3?
Assistant: 15% of 847 is 127.05, and multiplying that by 3 gives 381.15.
```

**验证要点：**
- Agent 将问题拆解为多步计算（`847 * 0.15`，再 `* 3`）
- 每步调用 `calculate` 工具，而非自行推算
- 最终回答包含正确的数值结果

---

## 3. mcp_example — MCP 服务器集成

演示如何通过 MCP（Model Context Protocol）协议连接外部工具服务器，自动发现并使用其提供的工具。

本示例使用官方 `@modelcontextprotocol/server-filesystem` MCP 服务器，通过 stdio 通信。

**额外前置条件：**

```bash
# 安装 MCP filesystem 服务器
npm install -g @modelcontextprotocol/server-filesystem

# 验证安装
npx @modelcontextprotocol/server-filesystem --version
```

**运行：**

```bash
cargo run -p mcp_example
```

**预期输出：**

```
Connecting to MCP filesystem server (serving /tmp)...
Discovered 3 MCP tool(s)
  - read_file: Read the complete contents of a file...
  - read_multiple_files: Read the contents of multiple files...
  - list_directory: Get a detailed listing of all files...

User: List the files in /tmp and read the content of any text files you find there.
Assistant: Here are the files in /tmp: ...
```

**验证要点：**
- 启动时打印发现的 MCP 工具列表（工具数量取决于 MCP 服务器版本）
- Agent 通过 MCP 协议调用工具，与直接实现的工具行为一致
- 如果 `npx` 不在 PATH 中，会报错 `Failed to spawn MCP server`

---

## 实现自定义工具

实现 `Tool` trait 即可将任意逻辑注册为工具：

```rust
use agent_core::{Agent, AgentConfig, Tool};
use async_trait::async_trait;
use anyhow::Result;
use serde_json::Value;

struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }

    fn description(&self) -> &str {
        "描述这个工具做什么，这段文字会直接发给 LLM"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "input": { "type": "string", "description": "输入参数" }
            },
            "required": ["input"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let input = args["input"].as_str().unwrap_or("");
        Ok(format!("处理结果: {}", input))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut agent = Agent::new(AgentConfig {
        model: "gpt-4o".to_string(),
        system_prompt: "You are a helpful assistant.".to_string(),
        max_steps: 20,
        api_base: "https://api.openai.com".to_string(),
        api_key: std::env::var("OPENAI_API_KEY")?,
    });

    agent.add_tool(MyTool);
    let reply = agent.run("请使用工具处理 'hello'").await?;
    println!("{}", reply);
    Ok(())
}
```
