# 测试说明

## 概述

本项目的测试分为两类：

1. **集成测试** (`integration_test.rs`) - 测试 PostgreSQL 连接和查询服务
2. **命令测试** (`commands_test.rs`) - 测试所有服务层功能（缓存服务、元数据服务、查询解析器等）

## 运行方式

### 方式 1：使用环境变量（推荐）

设置环境变量并运行测试：

```bash
export TEST_DB_HOST=localhost
export TEST_DB_PORT=5432
export TEST_DB_NAME=testdb
export TEST_DB_USER=testuser
export TEST_DB_PASSWORD=testpass

# 运行所有集成测试
cargo test --test integration_test

# 运行所有命令测试（服务层测试）
cargo test --test commands_test -- --test-threads=1

# 运行所有测试
cargo test --test integration_test --test commands_test -- --test-threads=1
```

### 方式 2：使用 Docker 手动启动 PostgreSQL

1. **启动 PostgreSQL 容器**：
```bash
docker run -d \
  --name test-postgres \
  -e POSTGRES_USER=testuser \
  -e POSTGRES_PASSWORD=testpass \
  -e POSTGRES_DB=testdb \
  -p 5432:5432 \
  postgres:15-alpine
```

2. **等待数据库就绪**（约 5-10 秒）

3. **运行测试**：
```bash
export TEST_DB_HOST=localhost
export TEST_DB_PORT=5432
export TEST_DB_NAME=testdb
export TEST_DB_USER=testuser
export TEST_DB_PASSWORD=testpass

cargo test --test integration_test --test commands_test -- --test-threads=1
```

4. **清理容器**（测试完成后）：
```bash
docker stop test-postgres
docker rm test-postgres
```

## 测试内容

### integration_test.rs - PostgreSQL 服务测试

- ✅ `test_connect` - 测试数据库连接功能
- ✅ `test_test_connection` - 测试连接测试功能
- ✅ `test_execute_query` - 测试查询执行（创建表、插入数据、查询）
- ✅ `test_execute_query_empty_result` - 测试空结果查询
- ✅ `test_connection_failure` - 测试连接失败处理

### commands_test.rs - 服务层测试

#### 缓存服务测试
- ✅ `test_cache_service_init_and_get_connection` - 测试数据库初始化和连接获取
- ✅ `test_cache_service_save_and_load_connection_with_data` - 测试连接保存和加载（需要 TEST_DB_* 环境变量）
- ✅ `test_cache_service_save_and_load_metadata` - 测试元数据保存和加载
- ✅ `test_cache_service_save_query_history` - 测试查询历史保存（SQL 和自然语言查询）

#### 元数据服务测试
- ✅ `test_metadata_service_extract_metadata` - 测试从 PostgreSQL 提取元数据（需要 TEST_DB_* 环境变量）

#### 查询解析器测试
- ✅ `test_query_parser_inject_limit` - 测试 LIMIT 注入功能
  - 测试不带 LIMIT 的 SELECT 查询
  - 测试已有 LIMIT 的查询
  - 测试带分号的查询
  - 测试非 SELECT 语句
- ✅ `test_query_parser_is_ddl_statement` - 测试 DDL 语句检测
  - 测试 CREATE/DROP/ALTER 语句检测
  - 测试非 DDL 语句

#### AI 服务测试
- ✅ `test_ai_service_with_openai_key` - 测试 AI 服务（需要 OPENAI_API_KEY 环境变量）
  - 如果未设置 API Key，测试会被跳过

## 注意事项

1. **数据库权限**：确保测试用户有创建表和插入数据的权限
2. **数据清理**：测试会创建临时表，每次运行前会清理旧数据
3. **环境变量**：如果未设置环境变量，测试会被跳过（不会失败）
4. **端口冲突**：确保测试端口（默认 5432）未被占用
5. **测试隔离**：使用 `--test-threads=1` 运行测试以确保环境变量隔离

## 运行特定测试

```bash
# 运行特定测试函数
cargo test --test commands_test test_query_parser_inject_limit

# 运行包含特定关键词的测试
cargo test --test commands_test -- --test-threads=1 query_parser

# 运行测试并显示输出
cargo test --test commands_test -- --test-threads=1 --nocapture
```

## 测试统计

- **集成测试**: 5 个测试函数（需要 TEST_DB_* 环境变量）
- **命令测试**: 8 个测试函数（部分需要 TEST_DB_* 环境变量）
- **总计**: 13 个测试函数，覆盖所有核心服务层功能

## CI/CD 集成

在 CI/CD 环境中，可以这样运行：

```yaml
# GitHub Actions 示例
- name: Run integration tests
  env:
    TEST_DB_HOST: localhost
    TEST_DB_PORT: 5432
    TEST_DB_NAME: testdb
    TEST_DB_USER: testuser
    TEST_DB_PASSWORD: testpass
  run: |
    docker run -d \
      --name test-postgres \
      -e POSTGRES_USER=testuser \
      -e POSTGRES_PASSWORD=testpass \
      -e POSTGRES_DB=testdb \
      -p 5432:5432 \
      postgres:15-alpine
    sleep 5
    cargo test --test integration_test --test commands_test -- --test-threads=1
```

## 故障排除

### 连接失败
- 检查 Docker 容器是否正在运行：`docker ps`
- 检查端口是否被占用：`lsof -i :5432`
- 检查环境变量是否正确设置：`env | grep TEST_DB`

### 权限错误
- 确保 PostgreSQL 用户有足够的权限
- 检查数据库是否存在：`psql -U testuser -d testdb -c "\l"`

### 测试超时
- 增加等待时间（在 `setup_postgres` 中）
- 检查数据库日志：`docker logs test-postgres`

### 环境变量冲突
- 使用 `--test-threads=1` 串行运行测试，避免环境变量冲突
- 确保每个测试使用独立的临时数据库文件
