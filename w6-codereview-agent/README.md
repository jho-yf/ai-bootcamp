# w6-codereview-agent

An AI-powered code review agent built on `agent-core`. It reviews code changes by calling git/GitHub tools and reading source files, then produces a structured report.

## Prerequisites

- Rust toolchain
- `OPENAI_API_KEY` environment variable set
- `git` available in PATH
- `gh` (GitHub CLI) — only needed for PR review

## Quick Start

```bash
# From the repo root
cd w6-codereview-agent

export OPENAI_API_KEY=your-key
export OPENAI_API_BASE=https://api.openai.com   # optional, default shown
export OPENAI_MODEL=gpt-4o                       # optional, default shown

cargo run
```

## Usage

```
cargo run -- [OPTIONS] [TARGET]...
```

### Review targets

| Command | What gets reviewed |
|---|---|
| `cargo run` | All uncommitted changes (staged + unstaged + untracked) |
| `cargo run -- "current branch"` | Current branch vs `main` |
| `cargo run -- <commit-hash>` | A specific commit (e.g. `d7868c7`) |
| `cargo run -- "after <hash>"` | All changes since a commit |
| `cargo run -- "PR 123"` | GitHub pull request #123 |
| `cargo run -- <branch-name>` | A named branch relative to `main` |

### Options

| Flag | Description |
|---|---|
| `-o, --output <FILE>` | Save the review report to a file (e.g. `review.md`) |
| `-m, --model <MODEL>` | LLM model to use (overrides `OPENAI_MODEL`) |
| `--api-base <URL>` | API base URL (overrides `OPENAI_API_BASE`) |
| `--max-steps <N>` | Max agent reasoning steps (default: 30) |
| `-s, --system-prompt <FILE>` | Custom system prompt file |
| `-v, --verbose` | Show tracing logs |
| `-h, --help` | Print help |

### Examples

```bash
# Review uncommitted changes
cargo run

# Review current branch and save report
cargo run -- "current branch" --output review.md

# Review a specific commit
cargo run -- abc1234

# Review all changes since a commit
cargo run -- "after abc1234"

# Review a GitHub PR
cargo run -- "PR 42"

# Use a different model
cargo run -- "current branch" --model gpt-4o-mini

# Pipe report to a file (stderr shows tool call progress)
cargo run -- "current branch" 2>/dev/null > review.md
```

## Output

Progress (tool calls) is written to **stderr**. The review report is written to **stdout**. This makes it easy to separate them:

```bash
# Show only the report
cargo run 2>/dev/null

# Save report, show progress
cargo run -- "current branch" > review.md
```

## Tools available to the agent

| Tool | Purpose |
|---|---|
| `read_file` | Read any file in the repository |
| `list_directory` | Explore directory structure |
| `write_file` | Save the review report to a file |
| `run_git` | Read-only git commands (diff, log, show, status, rev-parse, merge-base, branch, stash list) |
| `run_gh` | GitHub CLI read commands (pr view, pr diff, pr list) |

Destructive git operations (`commit`, `push`, `reset`, etc.) and GitHub write operations are blocked at the tool level.

## Environment variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `OPENAI_API_KEY` | Yes | — | API key |
| `OPENAI_API_BASE` | No | `https://api.openai.com` | API base URL |
| `OPENAI_MODEL` | No | `gpt-4o` | Model to use |
| `CODE_REVIEW_SYSTEM_PROMPT` | No | — | Path to a custom system prompt file |
