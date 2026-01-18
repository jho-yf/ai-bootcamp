# 集成测试说明

## 概述

集成测试验证 PostgreSQL 数据库连接和查询功能。测试使用环境变量配置数据库连接，支持连接到外部 PostgreSQL 实例。

## 运行方式

### 方式 1：使用环境变量（推荐）

设置环境变量并运行测试：

```bash
export TEST_DB_HOST=localhost
export TEST_DB_PORT=5432
export TEST_DB_NAME=testdb
export TEST_DB_USER=testuser
export TEST_DB_PASSWORD=testpass

cargo test --test integration_test
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

cargo test --test integration_test
```

4. **清理容器**（测试完成后）：
```bash
docker stop test-postgres
docker rm test-postgres
```

## 测试内容

- ✅ `test_connect` - 测试数据库连接功能
- ✅ `test_test_connection` - 测试连接测试功能
- ✅ `test_execute_query` - 测试查询执行（创建表、插入数据、查询）
- ✅ `test_execute_query_empty_result` - 测试空结果查询
- ✅ `test_connection_failure` - 测试连接失败处理

## 注意事项

1. **数据库权限**：确保测试用户有创建表和插入数据的权限
2. **数据清理**：测试会创建 `test_table` 表，每次运行前会清理旧数据
3. **环境变量**：如果未设置环境变量，测试会被跳过（不会失败）
4. **端口冲突**：确保测试端口（默认 5432）未被占用

## 跳过测试

如果未设置环境变量，测试会被自动跳过：

```bash
cargo test --test integration_test
# 输出: 跳过测试：未设置 TEST_DB_* 环境变量
```

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
    cargo test --test integration_test
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
