# 快速开始：数据库查询工具

**Feature**: 001-db-query  
**Date**: 2026-01-17  
**目标受众**: 开发者（首次设置项目）

## 概述

本指南帮助开发者快速启动数据库查询工具项目，从零到运行应用只需 15 分钟。

---

## 前置要求

### 必需工具
- **Rust**: 1.75+ ([安装指南](https://www.rust-lang.org/tools/install))
- **Node.js**: 18+ ([下载](https://nodejs.org/))
- **PostgreSQL**: 12+ (用于测试连接)

### 可选工具
- **cargo-tauri**: Tauri CLI（自动安装）
- **PostgreSQL 客户端**: psql 或 pgAdmin（验证数据库）

### 系统要求
- **Windows**: Windows 10+ (64-bit)
- **macOS**: 11.0+ (Big Sur or later)
- **Linux**: GTK 3.0+ 和 webkit2gtk

---

## 快速启动（5 分钟）

### 1. 克隆并初始化项目

```bash
# 创建项目目录
mkdir w2-db-query
cd w2-db-query

# 初始化 Tauri 项目
npm create tauri-app@latest

# 按提示选择：
# - Framework: React
# - Language: TypeScript
# - Package manager: npm

# 进入项目目录
cd w2-db-query
```

### 2. 安装依赖

```bash
# 安装前端依赖
npm install

# 安装额外的前端库
npm install @refinedev/core @refinedev/antd antd
npm install @monaco-editor/react
npm install @tauri-apps/api

# 安装 Tauri 依赖（自动在首次构建时安装）
```

### 3. 配置后端依赖（Cargo.toml）

编辑 `src-tauri/Cargo.toml`，添加依赖：

```toml
[dependencies]
tauri = { version = "1.5", features = ["shell-open"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
tokio-postgres = "0.7"
rusqlite = { version = "0.30", features = ["bundled"] }
sqlparser = "0.38"
async-openai = "0.16"
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
```

### 4. 设置环境变量

创建 `.env` 文件（项目根目录）：

```bash
# OpenAI API Key（用于自然语言查询）
OPENAI_API_KEY=sk-your-api-key-here

# OpenAI API Base URL（可选，默认为官方 endpoint）
# 如果使用自定义代理或其他兼容的 OpenAI API 服务，可以设置此变量
# OPENAI_API_BASE=https://api.openai.com/v1
# OPENAI_API_BASE=https://your-proxy.com/v1

# OpenAI 模型名称（可选，默认为 gpt-4o-mini）
# 根据你使用的 API 服务，可能需要设置不同的模型名称
# 官方 OpenAI API: gpt-4o-mini, gpt-4, gpt-3.5-turbo
# 其他服务可能需要不同的模型名称，请参考对应的 API 文档
# OPENAI_MODEL=gpt-4o-mini
# OPENAI_MODEL=glm-4  # 对于某些国内服务
```

### 5. 运行开发服务器

```bash
# 启动 Tauri 开发模式（首次启动会编译 Rust 代码，耗时 2-5 分钟）
npm run tauri dev
```

应用窗口将自动打开！🎉

---

## 项目结构说明

```
w2-db-query/
├── src/                      # React 前端源码
│   ├── main.tsx             # 前端入口
│   ├── App.tsx              # 根组件
│   ├── pages/               # 页面组件
│   ├── components/          # 可复用组件
│   └── services/            # Tauri API 调用
│
├── src-tauri/               # Rust 后端源码
│   ├── src/
│   │   ├── main.rs          # Tauri 入口
│   │   ├── commands/        # Tauri Command 函数
│   │   ├── models/          # 数据模型
│   │   ├── services/        # 业务逻辑
│   │   └── utils/           # 工具函数
│   ├── Cargo.toml           # Rust 依赖
│   └── tauri.conf.json      # Tauri 配置
│
├── package.json             # 前端依赖
├── tsconfig.json            # TypeScript 配置
├── .env                     # 环境变量（不提交到 Git）
└── db_query.db              # SQLite 缓存（运行时生成）
```

---

## 验证安装

### 1. 检查 Rust 环境

```bash
rustc --version
# 应输出: rustc 1.75.0 或更高版本

cargo --version
# 应输出: cargo 1.75.0 或更高版本
```

### 2. 检查 Node.js 环境

```bash
node --version
# 应输出: v18.x.x 或更高版本

npm --version
# 应输出: 9.x.x 或更高版本
```

### 3. 测试 PostgreSQL 连接

准备一个测试数据库：

```bash
# 使用 psql 连接（根据你的环境调整）
psql -U postgres

# 创建测试数据库
CREATE DATABASE test_db;

# 创建测试表
\c test_db
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    age INTEGER,
    created_at TIMESTAMP DEFAULT NOW()
);

# 插入测试数据
INSERT INTO users (name, email, age) VALUES
    ('张三', 'zhangsan@example.com', 25),
    ('李四', 'lisi@example.com', 30),
    ('王五', 'wangwu@example.com', 35);
```

---

## 第一次使用

### 步骤 1: 添加数据库连接

1. 启动应用（`npm run tauri dev`）
2. 点击 "添加数据库" 按钮
3. 填写连接信息：
   - 名称: `测试数据库`
   - 主机: `localhost`
   - 端口: `5432`
   - 数据库名: `test_db`
   - 用户名: `postgres`
   - 密码: `your_password`
4. 点击 "连接"

### 步骤 2: 浏览数据库结构

连接成功后，左侧边栏将显示：
- 📁 public schema
  - 📊 users (表)
    - 🔑 id (integer, 主键)
    - 📝 name (varchar)
    - 📧 email (varchar)
    - 🎂 age (integer)
    - 📅 created_at (timestamp)

### 步骤 3: 执行第一个查询

1. 在 SQL 编辑器中输入：
   ```sql
   SELECT * FROM users WHERE age > 25
   ```
2. 点击 "执行" 按钮（或按 Ctrl+Enter）
3. 查看结果表格

### 步骤 4: 尝试自然语言查询（需要 OpenAI API Key）

1. 点击 "自然语言查询" 标签
2. 输入: `查询所有年龄大于30岁的用户姓名和邮箱`
3. 点击 "生成 SQL"
4. 审查生成的 SQL（会自动显示在编辑器中）
5. 点击 "执行" 运行查询

---

## 开发工作流

### 前端开发（热重载）

```bash
# 启动开发服务器（前端修改自动重载）
npm run tauri dev
```

修改 `src/` 下的文件，保存后浏览器自动刷新。

### 后端开发（需要重启）

修改 `src-tauri/src/` 下的 Rust 代码后：

```bash
# 方式 1: 重启开发服务器
# Ctrl+C 停止，然后再次运行 npm run tauri dev

# 方式 2: 仅重新编译后端（更快）
cd src-tauri
cargo build
cd ..
npm run tauri dev
```

### 代码格式化

```bash
# Rust 代码格式化
cd src-tauri
cargo fmt

# TypeScript 代码格式化
npm run format  # 或 npx prettier --write "src/**/*.{ts,tsx}"
```

### 代码检查

```bash
# Rust 代码检查
cd src-tauri
cargo clippy -- -D warnings

# TypeScript 类型检查
npm run typecheck  # 或 npx tsc --noEmit
```

---

## 构建生产版本

### 开发构建（快速测试）

```bash
npm run tauri build --debug
```

输出位置: `src-tauri/target/debug/bundle/`

### 生产构建（优化性能）

```bash
npm run tauri build
```

输出位置: `src-tauri/target/release/bundle/`

### 平台特定构建

- **Windows**: 生成 `.msi` 安装包
- **macOS**: 生成 `.dmg` 和 `.app`
- **Linux**: 生成 `.deb` 和 `.AppImage`

---

## 常见问题

### Q1: Tauri 编译失败
**A**: 确保安装了系统依赖：
- **Windows**: Visual Studio C++ Build Tools
- **macOS**: Xcode Command Line Tools (`xcode-select --install`)
- **Linux**: 
  ```bash
  sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
  ```

### Q2: 无法连接到 PostgreSQL
**A**: 检查：
1. PostgreSQL 服务是否运行
2. 防火墙是否允许端口 5432
3. `pg_hba.conf` 是否允许本地连接

### Q3: OpenAI API 调用失败
**A**: 检查：
1. `.env` 文件中的 `OPENAI_API_KEY` 是否正确
2. 如果使用自定义 endpoint，检查 `OPENAI_API_BASE` 是否正确设置
3. 如果使用自定义 API 服务，检查 `OPENAI_MODEL` 模型名称是否正确（某些服务需要特定的模型名称）
4. 网络是否能访问 OpenAI API（或自定义 endpoint）
5. API 配额是否充足
6. 查看控制台错误信息，确认是认证问题、模型问题还是网络问题

### Q4: SQLite 数据库文件在哪里？
**A**: `./db_query.db`（项目根目录）。首次运行应用时自动创建。

---

## 下一步

✅ 环境已就绪！现在可以：

1. **阅读设计文档**:
   - [data-model.md](./data-model.md) - 数据模型设计
   - [contracts/tauri-commands.md](./contracts/tauri-commands.md) - API 契约

2. **开始实现**:
   - 参考 `/speckit.tasks` 命令生成任务列表
   - 按照用户故事优先级（P1 → P2 → P3）实现

3. **测试功能**:
   - P1: 数据库连接和元数据浏览
   - P2: SQL 查询执行
   - P3: 自然语言查询生成

---

## 技术支持

- **项目文档**: `specs/001-db-query/`
- **API 文档**: `specs/001-db-query/contracts/tauri-commands.md`
- **章程**: `.specify/memory/constitution.md`

**祝开发顺利！** 🚀
