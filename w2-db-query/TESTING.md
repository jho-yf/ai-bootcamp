# 测试指南

## 概述

本文档说明如何运行项目的所有测试，包括后端集成测试和前端 E2E 测试。

## 后端测试

### 集成测试

测试 PostgreSQL 连接和查询服务：

```bash
cd src-tauri

# 设置测试数据库环境变量
export TEST_DB_HOST=localhost
export TEST_DB_PORT=5432
export TEST_DB_NAME=testdb
export TEST_DB_USER=testuser
export TEST_DB_PASSWORD=testpass

# 运行集成测试
cargo test --test integration_test
```

**测试覆盖**:
- PostgreSQL 连接功能
- 连接测试功能
- 查询执行（包含数据操作）
- 空结果查询
- 连接失败处理

### 服务层测试

测试所有服务层功能（缓存服务、元数据服务、查询解析器等）：

```bash
cd src-tauri

# 设置测试数据库环境变量（部分测试需要）
export TEST_DB_HOST=localhost
export TEST_DB_PORT=5432
export TEST_DB_NAME=testdb
export TEST_DB_USER=testuser
export TEST_DB_PASSWORD=testpass

# 运行服务层测试（使用串行模式避免环境变量冲突）
cargo test --test commands_test -- --test-threads=1
```

**测试覆盖**:
- 缓存服务（数据库连接、元数据、查询历史）
- 元数据服务（元数据提取）
- 查询解析器（LIMIT 注入、DDL 检测）
- AI 服务（需要 OPENAI_API_KEY 环境变量）

### 运行所有后端测试

```bash
cd src-tauri

# 设置环境变量
export TEST_DB_HOST=localhost
export TEST_DB_PORT=5432
export TEST_DB_NAME=testdb
export TEST_DB_USER=testuser
export TEST_DB_PASSWORD=testpass

# 运行所有测试（串行模式）
cargo test --test integration_test --test commands_test -- --test-threads=1
```

## 前端 E2E 测试

### 前置条件

1. 启动开发服务器（在另一个终端）：
```bash
npm run dev
```

确保服务器运行在 `http://localhost:1420`

2. 安装 Playwright 浏览器（首次运行）：
```bash
npx playwright install
```

### 运行 E2E 测试

```bash
# 运行所有 E2E 测试
npm run test:e2e

# UI 模式（推荐用于调试）
npm run test:e2e:ui

# 调试模式
npm run test:e2e:debug

# 有头模式（显示浏览器窗口）
npm run test:e2e:headed

# 运行特定测试文件
npm run test:e2e tests/e2e/natural-language-query.spec.ts
```

**测试覆盖**:
- 数据库列表页面
- 导航功能
- SQL 查询编辑器
- 数据库浏览器
- 自然语言查询功能（新增）

## 快速开始

### 使用 Docker 运行后端测试

```bash
# 1. 启动 PostgreSQL 容器
docker run -d \
  --name test-postgres \
  -e POSTGRES_USER=testuser \
  -e POSTGRES_PASSWORD=testpass \
  -e POSTGRES_DB=testdb \
  -p 5432:5432 \
  postgres:15-alpine

# 2. 等待数据库就绪
sleep 5

# 3. 设置环境变量并运行测试
cd src-tauri
export TEST_DB_HOST=localhost
export TEST_DB_PORT=5432
export TEST_DB_NAME=testdb
export TEST_DB_USER=testuser
export TEST_DB_PASSWORD=testpass

cargo test --test integration_test --test commands_test -- --test-threads=1

# 4. 清理（测试完成后）
docker stop test-postgres
docker rm test-postgres
```

### 运行前端 E2E 测试

```bash
# 终端 1: 启动开发服务器
npm run dev

# 终端 2: 运行 E2E 测试
npm run test:e2e
```

## 测试统计

### 后端测试
- **集成测试**: 5 个测试函数
- **服务层测试**: 8 个测试函数
- **总计**: 13 个测试函数

### 前端 E2E 测试
- **测试文件**: 5 个测试文件
- **测试用例**: 25+ 个测试场景
- **覆盖功能**: 所有主要用户界面和交互流程

## CI/CD 集成

### GitHub Actions 示例

```yaml
name: Test

on: [push, pull_request]

jobs:
  test-backend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Start PostgreSQL
        run: |
          docker run -d \
            --name test-postgres \
            -e POSTGRES_USER=testuser \
            -e POSTGRES_PASSWORD=testpass \
            -e POSTGRES_DB=testdb \
            -p 5432:5432 \
            postgres:15-alpine
          sleep 5
      
      - name: Run backend tests
        working-directory: src-tauri
        env:
          TEST_DB_HOST: localhost
          TEST_DB_PORT: 5432
          TEST_DB_NAME: testdb
          TEST_DB_USER: testuser
          TEST_DB_PASSWORD: testpass
        run: |
          cargo test --test integration_test --test commands_test -- --test-threads=1

  test-frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: 18
      
      - name: Install dependencies
        run: npm ci
      
      - name: Install Playwright
        run: npx playwright install --with-deps
      
      - name: Start dev server
        run: npm run dev &
      
      - name: Run E2E tests
        run: npm run test:e2e
        env:
          CI: true
```

## 故障排除

### 后端测试问题

**问题**: 测试失败，提示连接错误
- **解决**: 检查 PostgreSQL 是否运行，环境变量是否正确设置

**问题**: 环境变量冲突
- **解决**: 使用 `--test-threads=1` 串行运行测试

**问题**: 外键约束失败
- **解决**: 确保先创建数据库连接，再创建关联的元数据或查询历史

### 前端 E2E 测试问题

**问题**: 测试超时
- **解决**: 增加等待时间，确保开发服务器已启动

**问题**: Monaco Editor 未找到
- **解决**: 增加等待时间，Monaco Editor 加载可能需要几秒钟

**问题**: Mock 不工作
- **解决**: 检查 `fixtures.ts` 中的 mock 配置是否正确

## 相关文档

- [后端测试 README](./src-tauri/tests/README.md)
- [前端 E2E 测试 README](./tests/e2e/README.md)
- [Playwright E2E 测试指南](./PLAYWRIGHT_E2E_TESTING.md)
