use agent_core::{Agent, AgentConfig, Tool};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use clap::Parser;
use colored::Colorize;
use serde_json::Value;
use std::fs;
use std::path::Path;
use tokio::process::Command as AsyncCommand;
use tokio::time::{timeout, Duration};

const CMD_TIMEOUT_SECS: u64 = 30;

// Embedded system prompt as fallback when file is not found at runtime.
const EMBEDDED_SYSTEM_PROMPT: &str =
    include_str!("../../specs/w6/codereview-prompt/system.md");

// ============ CLI ============

#[derive(Parser)]
#[command(
    name = "codereview-agent",
    version,
    about = "AI-powered code review agent"
)]
#[command(long_about = "Reviews code changes using an LLM agent with tool access.\n\n\
    TARGET examples:\n  \
      (empty)            Review uncommitted changes (default)\n  \
      \"current branch\"   Review changes on current branch vs main\n  \
      <commit-hash>      Review a specific commit\n  \
      \"after <hash>\"     Review all changes since a commit\n  \
      \"PR <number>\"      Review a GitHub pull request\n  \
      <branch-name>      Review a branch relative to main\n\n\
    Environment variables:\n  \
      OPENAI_API_KEY       (required) API key\n  \
      OPENAI_API_BASE      API base URL (default: https://api.openai.com)\n  \
      OPENAI_MODEL         Default model (overridden by --model)")]
struct Cli {
    /// What to review (leave empty for uncommitted changes)
    target: Vec<String>,

    /// Path to system prompt file (falls back to embedded default)
    #[arg(short, long, env = "CODE_REVIEW_SYSTEM_PROMPT")]
    system_prompt: Option<String>,

    /// Save review report to a file
    #[arg(short, long)]
    output: Option<String>,

    /// LLM model to use
    #[arg(short, long, env = "OPENAI_MODEL", default_value = "gpt-4o")]
    model: String,

    /// API base URL
    #[arg(long, env = "OPENAI_API_BASE", default_value = "https://api.openai.com")]
    api_base: String,

    /// Maximum agent reasoning steps
    #[arg(long, default_value_t = 30)]
    max_steps: usize,

    /// Show verbose output (agent internals via tracing)
    #[arg(short, long)]
    verbose: bool,
}

// ============ Helpers ============

fn log_tool(name: &str, detail: &str) {
    eprintln!(
        "  {} {}",
        "->".dimmed(),
        format!("{}({})", name, detail).cyan()
    );
}

async fn run_command(program: &str, args: &[String]) -> Result<String> {
    let future = AsyncCommand::new(program).args(args).output();
    let output = timeout(Duration::from_secs(CMD_TIMEOUT_SECS), future)
        .await
        .with_context(|| format!("{} command timed out after {}s", program, CMD_TIMEOUT_SECS))?
        .with_context(|| format!("Failed to execute '{}'. Is it installed?", program))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        let out = stdout.trim_end().to_string();
        if stderr.trim().is_empty() {
            Ok(out)
        } else {
            Ok(format!("{out}\n[stderr]: {}", stderr.trim_end()))
        }
    } else {
        let combined = format!("{}{}", stdout, stderr);
        Ok(format!(
            "Exit code {}: {}",
            output.status.code().unwrap_or(-1),
            combined.trim_end()
        ))
    }
}

fn is_git_allowed(args: &[String]) -> bool {
    if args.is_empty() {
        return false;
    }
    match args[0].as_str() {
        "diff" | "log" | "show" | "status" | "rev-parse" | "merge-base" => true,
        "branch" => !args.iter().skip(1).any(|a| {
            matches!(
                a.as_str(),
                "-d"
                    | "-D"
                    | "-m"
                    | "-M"
                    | "-c"
                    | "-C"
                    | "--delete"
                    | "--move"
                    | "--rename"
                    | "--copy"
            )
        }),
        "stash" => args.get(1).map_or(false, |a| a == "list"),
        _ => false,
    }
}

fn is_gh_allowed(args: &[String]) -> bool {
    args.get(0).map_or(false, |a| a == "pr")
        && args.get(1).map_or(false, |a| matches!(a.as_str(), "view" | "diff" | "list"))
}

fn parse_args_array(args: &Value) -> Result<Vec<String>> {
    let cmd_args: Vec<String> = match &args["args"] {
        Value::Array(arr) => arr
            .iter()
            .map(|v| match v {
                Value::String(s) => Ok(s.clone()),
                Value::Number(n) => Ok(n.to_string()),
                other => bail!("All 'args' elements must be strings, got: {}", other),
            })
            .collect::<Result<Vec<_>>>()?,
        Value::String(s) => {
            bail!(
                "'args' must be a JSON array of strings, not a single string. Use {} instead.",
                serde_json::json!(s.split_whitespace().collect::<Vec<_>>())
            );
        }
        _ => bail!("'args' must be a JSON array of strings"),
    };
    if cmd_args.is_empty() {
        bail!("'args' must not be empty");
    }
    Ok(cmd_args)
}

// ============ Tools ============

struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file at the given path"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path (absolute or relative to the repository root)"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("'path' argument is required"))?;
        log_tool("read_file", path);
        fs::read_to_string(path).with_context(|| format!("Failed to read '{}'", path))
    }
}

struct ListDirectoryTool;

#[async_trait]
impl Tool for ListDirectoryTool {
    fn name(&self) -> &str {
        "list_directory"
    }

    fn description(&self) -> &str {
        "List files and subdirectories in a directory (directories end with /)"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("'path' argument is required"))?;
        log_tool("list_directory", path);

        let entries = fs::read_dir(path).with_context(|| format!("Failed to list '{}'", path))?;

        let mut names: Vec<String> = entries
            .filter_map(|e| e.ok())
            .map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if e.path().is_dir() {
                    format!("{name}/")
                } else {
                    name
                }
            })
            .collect();
        names.sort();
        if names.is_empty() {
            Ok(format!("(empty directory: {})", path))
        } else {
            Ok(names.join("\n"))
        }
    }
}

struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file (creates parent directories if needed)"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Target file path"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write"
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("'path' argument is required"))?;
        let content = args["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("'content' argument is required"))?;

        log_tool("write_file", path);

        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent dirs for '{}'", path))?;
        }
        let bytes = content.len();
        fs::write(path, content).with_context(|| format!("Failed to write to '{}'", path))?;
        Ok(format!("Written {bytes} bytes to {path}"))
    }
}

struct RunGitTool;

#[async_trait]
impl Tool for RunGitTool {
    fn name(&self) -> &str {
        "run_git"
    }

    fn description(&self) -> &str {
        "Run a read-only git command. Allowed: diff, log, show, status, rev-parse, merge-base, branch (read-only), stash list"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "args": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Git subcommand and arguments, e.g. [\"diff\", \"main...HEAD\"]"
                }
            },
            "required": ["args"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let cmd_args = parse_args_array(&args)?;

        if !is_git_allowed(&cmd_args) {
            bail!(
                "Git command '{}' is not allowed. Only read-only commands: \
                 diff, log, show, status, rev-parse, merge-base, branch (read-only flags), stash list",
                cmd_args.join(" ")
            );
        }

        log_tool("run_git", &cmd_args.join(" "));
        run_command("git", &cmd_args).await
    }
}

struct RunGhTool;

#[async_trait]
impl Tool for RunGhTool {
    fn name(&self) -> &str {
        "run_gh"
    }

    fn description(&self) -> &str {
        "Run a GitHub CLI command. Allowed: pr view, pr diff, pr list"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "args": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "GitHub CLI arguments, e.g. [\"pr\", \"diff\", \"123\"]"
                }
            },
            "required": ["args"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let cmd_args = parse_args_array(&args)?;

        if !is_gh_allowed(&cmd_args) {
            bail!(
                "gh command '{}' is not allowed. Only 'pr view', 'pr diff', and 'pr list' are permitted",
                cmd_args.join(" ")
            );
        }

        log_tool("run_gh", &cmd_args.join(" "));
        run_command("gh", &cmd_args).await
    }
}

// ============ Main ============

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Configure tracing for verbose mode
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .init();
    }

    // Load system prompt: CLI flag > file at default path > embedded
    let system_prompt = match &cli.system_prompt {
        Some(path) => fs::read_to_string(path)
            .with_context(|| format!("System prompt file '{}' not found", path))?,
        None => fs::read_to_string("specs/w6/codereview-prompt/system.md")
            .unwrap_or_else(|_| EMBEDDED_SYSTEM_PROMPT.to_string()),
    };

    // API key is required
    let api_key = std::env::var("OPENAI_API_KEY")
        .context("OPENAI_API_KEY environment variable must be set")?;

    // Build user message: target on first line, output instruction on its own line
    let target = cli.target.join(" ");
    let user_message = match (target.is_empty(), &cli.output) {
        (true, None) => String::new(),
        (false, None) => target.clone(),
        (true, Some(path)) => format!("Save the report to {path}."),
        (false, Some(path)) => format!("{target}\nSave the report to {path}."),
    };

    // Header
    let target_display = if target.is_empty() {
        "uncommitted changes".to_string()
    } else {
        target.clone()
    };
    eprintln!(
        "\n  {} Reviewing: {}\n",
        "*".green().bold(),
        target_display.bold()
    );

    // Build agent
    let mut agent = Agent::new(AgentConfig {
        model: cli.model,
        system_prompt,
        max_steps: cli.max_steps,
        api_base: cli.api_base,
        api_key,
    });

    agent.add_tool(ReadFileTool);
    agent.add_tool(ListDirectoryTool);
    agent.add_tool(WriteFileTool);
    agent.add_tool(RunGitTool);
    agent.add_tool(RunGhTool);

    // Run
    let default_msg = "Review all uncommitted changes (unstaged, staged, and untracked files)";
    let message = if user_message.is_empty() {
        default_msg
    } else {
        &user_message
    };

    let reply = agent.run(message).await?;

    eprintln!("  {}\n", "---".repeat(20).dimmed());
    println!("{reply}");

    Ok(())
}
