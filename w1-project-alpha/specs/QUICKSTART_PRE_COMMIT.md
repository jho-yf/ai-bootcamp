# Pre-commit 快速开始指南

## 5 分钟快速设置

### 1. 安装 pre-commit

```bash
# 使用 pip
pip install pre-commit

# 或使用 Homebrew (macOS)
brew install pre-commit
```

### 2. 安装 Git hooks

在项目根目录运行：

```bash
pre-commit install
```

### 3. 安装前端依赖（如果还没有）

```bash
cd frontend
npm install
cd ..
```

### 4. 测试配置

运行所有 hooks 验证配置：

```bash
pre-commit run --all-files
```

首次运行会下载并安装所有 hooks，可能需要几分钟。

### 5. 开始使用

现在每次运行 `git commit` 时，pre-commit hooks 会自动运行！

```bash
git add .
git commit -m "Your commit message"
# Pre-commit hooks 会自动运行
```

## 常用命令

```bash
# 运行所有 hooks
pre-commit run --all-files

# 运行特定 hook
pre-commit run cargo-fmt --all-files
pre-commit run eslint --all-files
pre-commit run prettier --all-files

# 更新 hooks
pre-commit autoupdate

# 卸载 hooks（如果需要）
pre-commit uninstall
```

## 如果 hooks 失败

1. 查看错误信息
2. 修复报告的问题
3. 重新提交

大多数情况下，hooks 会自动修复问题（如格式化、移除空白字符等）。

## 需要帮助？

查看详细文档：[README_PRE_COMMIT.md](./README_PRE_COMMIT.md)
