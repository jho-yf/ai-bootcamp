# Pre-commit 和 GitHub Actions 配置说明

## Pre-commit 配置

本项目使用 [pre-commit](https://pre-commit.com/) 来确保代码质量。在提交代码之前，会自动运行以下检查：

### Rust 后端检查

- **cargo fmt**: 检查代码格式是否符合 Rust 标准
- **cargo clippy**: 运行 Clippy 静态分析，检查代码质量和潜在问题

### TypeScript 前端检查

- **eslint**: 运行 ESLint 检查并自动修复问题
- **typescript-check**: 运行 TypeScript 类型检查
- **prettier**: 自动格式化代码

### 通用检查

- **trailing-whitespace**: 移除行尾空白
- **end-of-file-fixer**: 确保文件以换行符结尾
- **check-yaml/json/toml**: 验证配置文件格式
- **check-added-large-files**: 检查大文件
- **check-merge-conflict**: 检查合并冲突标记
- **mixed-line-ending**: 统一换行符为 LF

## 安装和设置

### 1. 安装 pre-commit

```bash
# 使用 pip 安装
pip install pre-commit

# 或使用 Homebrew (macOS)
brew install pre-commit

# 或使用 conda
conda install -c conda-forge pre-commit
```

### 2. 安装 Git hooks

在项目根目录运行：

```bash
pre-commit install
```

这会在 `.git/hooks/pre-commit` 中安装 pre-commit hook。

### 3. 手动运行所有 hooks

```bash
pre-commit run --all-files
```

### 4. 更新 hooks

```bash
pre-commit autoupdate
```

## GitHub Actions

项目配置了两个 GitHub Actions 工作流：

### 1. CI (`ci.yml`)

在每次 push 或 pull request 时运行：

- **Rust 后端**:
  - 设置 PostgreSQL 数据库服务
  - 运行数据库迁移
  - 代码格式检查 (`cargo fmt`)
  - Clippy 检查 (`cargo clippy`)
  - 构建项目 (`cargo build`)
  - 运行测试 (`cargo test`)

- **TypeScript 前端**:
  - 安装依赖
  - ESLint 检查
  - TypeScript 类型检查
  - 构建项目

- **Pre-commit 检查**:
  - 运行所有 pre-commit hooks

### 2. Pre-commit (`pre-commit.yml`)

专门用于运行 pre-commit hooks 的工作流，确保代码质量。

## 使用说明

### 提交代码前

Pre-commit hooks 会在你运行 `git commit` 时自动执行。如果检查失败，提交会被阻止。你需要：

1. 修复报告的问题
2. 重新提交

### 跳过 hooks（不推荐）

如果确实需要跳过 hooks（例如紧急修复），可以使用：

```bash
git commit --no-verify
```

**注意**: 这会跳过所有 pre-commit 检查，可能导致代码质量问题。

### 只运行特定 hook

```bash
# 只运行 Rust 格式化检查
pre-commit run cargo-fmt --all-files

# 只运行 ESLint
pre-commit run eslint --all-files

# 只运行 Prettier
pre-commit run prettier --all-files
```

## 故障排除

### Pre-commit hooks 运行失败

1. 检查错误信息，修复报告的问题
2. 确保所有依赖已安装：
   - Rust toolchain
   - Node.js 和 npm 依赖
   - Python 和 pre-commit

### GitHub Actions 失败

1. 查看 Actions 日志了解失败原因
2. 确保所有测试通过
3. 检查代码格式和类型检查

### 更新 hooks

如果 hooks 版本过旧，运行：

```bash
pre-commit autoupdate
```

## 配置自定义

### 修改检查规则

编辑 `.pre-commit-config.yaml` 文件，可以：
- 添加新的 hooks
- 修改现有 hooks 的行为
- 排除特定文件或目录

### 修改 GitHub Actions

编辑 `.github/workflows/` 目录下的 YAML 文件，可以：
- 修改触发条件
- 添加新的检查步骤
- 修改运行环境

## 最佳实践

1. **提交前运行检查**: 在提交前运行 `pre-commit run --all-files` 确保所有检查通过
2. **修复所有警告**: 不要忽略 Clippy 或 ESLint 的警告
3. **保持 hooks 更新**: 定期运行 `pre-commit autoupdate`
4. **不要跳过 hooks**: 除非特殊情况，不要使用 `--no-verify`

## 相关资源

- [Pre-commit 文档](https://pre-commit.com/)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Rust Clippy](https://github.com/rust-lang/rust-clippy)
- [ESLint](https://eslint.org/)
- [Prettier](https://prettier.io/)
