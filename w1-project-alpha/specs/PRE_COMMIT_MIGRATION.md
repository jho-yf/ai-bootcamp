# Pre-commit 和 GitHub Actions 迁移完成

## 概述

已将 pre-commit 配置和 GitHub Actions 工作流从 `w1-project-alpha/` 目录移动到项目根目录 `/home/jho/workspace/mine/ai-bootcamp/`。

## 迁移的文件

### 1. Pre-commit 配置

- **源位置**: `w1-project-alpha/.pre-commit-config.yaml`
- **目标位置**: `.pre-commit-config.yaml` (根目录)
- **状态**: ✅ 已迁移

### 2. GitHub Actions 工作流

- **源位置**: `w1-project-alpha/.github/workflows/`
- **目标位置**: `.github/workflows/` (根目录)
- **文件**:
  - `ci.yml` - 主 CI 工作流
  - `pre-commit.yml` - Pre-commit 专用工作流
- **状态**: ✅ 已迁移

## 配置更新

### Pre-commit 配置

配置已更新以支持项目根目录结构：
- 所有路径检查都优先查找 `w1-project-alpha/backend` 和 `w1-project-alpha/frontend`
- 如果不存在，回退到根目录下的 `backend` 和 `frontend`（兼容性）
- 文件匹配模式已更新：`^(w1-project-alpha/)?frontend/.*\.(ts|tsx|js|jsx)$`

### GitHub Actions 工作流

所有工作流已更新工作目录路径：

#### CI 工作流 (`ci.yml`)

- **Rust 后端**:
  - `working-directory: ./w1-project-alpha/backend`
  - 缓存路径：`w1-project-alpha/backend/target`
  - Cargo.lock 路径：`w1-project-alpha/backend/Cargo.lock`

- **TypeScript 前端**:
  - `working-directory: ./w1-project-alpha/frontend`
  - 缓存依赖路径：`w1-project-alpha/frontend/package-lock.json`

#### Pre-commit 工作流 (`pre-commit.yml`)

- 前端依赖安装：`working-directory: ./w1-project-alpha/frontend`
- Cargo 缓存路径：`w1-project-alpha/backend/target`
- Cargo.lock 路径：`w1-project-alpha/backend/Cargo.lock`

## 验证

所有配置文件已验证：

- ✅ `.pre-commit-config.yaml` - YAML 格式正确
- ✅ `.github/workflows/ci.yml` - YAML 格式正确
- ✅ `.github/workflows/pre-commit.yml` - YAML 格式正确

## 使用说明

### 安装 Pre-commit Hooks

在项目根目录运行：

```bash
cd /home/jho/workspace/mine/ai-bootcamp
pre-commit install
```

### 运行 Pre-commit Hooks

```bash
# 运行所有 hooks
pre-commit run --all-files

# 运行特定 hook
pre-commit run cargo-fmt --all-files
pre-commit run eslint --all-files
```

### GitHub Actions

工作流会在以下情况自动触发：
- Push 到 `main`、`master` 或 `develop` 分支
- 创建 Pull Request 到这些分支

## 文件结构

```
/home/jho/workspace/mine/ai-bootcamp/
├── .pre-commit-config.yaml          # Pre-commit 配置（根目录）
├── .github/
│   └── workflows/
│       ├── ci.yml                   # CI 工作流
│       └── pre-commit.yml           # Pre-commit 工作流
├── .gitignore                       # Git 忽略文件（根目录）
└── w1-project-alpha/
    ├── backend/                     # Rust 后端
    └── frontend/                    # TypeScript 前端
```

## 注意事项

1. **路径兼容性**: 配置支持两种路径结构：
   - `w1-project-alpha/backend` 和 `w1-project-alpha/frontend`（当前）
   - `backend` 和 `frontend`（如果项目结构改变）

2. **Git 仓库根目录**: Pre-commit hooks 使用 `git rev-parse --show-toplevel` 来获取仓库根目录，确保路径正确。

3. **Node.js 检查**: 所有需要 Node.js 的 hooks 都会检查 `npm`/`npx` 是否可用，如果不可用会优雅跳过。

4. **删除旧文件**: `w1-project-alpha/` 目录下的旧配置文件已删除：
   - `w1-project-alpha/.pre-commit-config.yaml` ✅ 已删除
   - `w1-project-alpha/.github/` ✅ 已删除

## 下一步

1. 在根目录运行 `pre-commit install` 安装 hooks
2. 测试 pre-commit hooks：`pre-commit run --all-files`
3. 提交更改到 Git 仓库
4. 推送到 GitHub 测试 GitHub Actions

## 相关文档

- Pre-commit 配置：`.pre-commit-config.yaml`
- CI 工作流：`.github/workflows/ci.yml`
- Pre-commit 工作流：`.github/workflows/pre-commit.yml`
