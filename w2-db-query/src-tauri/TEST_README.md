# 测试说明

## 单元测试

运行所有单元测试（不需要 Docker）：

```bash
cargo test --lib
```

这些测试包括：
- `query_parser.rs` - SQL 解析和验证测试
- `cache_service.rs` - SQLite 缓存服务测试

## 集成测试（需要 PostgreSQL 数据库）

集成测试需要连接到 PostgreSQL 数据库。有两种运行方式：

### 方式 1：使用环境变量（推荐）

通过环境变量配置外部 PostgreSQL 数据库：

```bash
export TEST_DB_HOST=localhost
export TEST_DB_PORT=5432
export TEST_DB_NAME=testdb
export TEST_DB_USER=testuser
export TEST_DB_PASSWORD=testpass

cargo test --test integration_test
```

### 方式 2：使用 Docker（手动启动）

1. 启动 PostgreSQL 容器：
```bash
docker run -d \
  --name test-postgres \
  -e POSTGRES_USER=testuser \
  -e POSTGRES_PASSWORD=testpass \
  -e POSTGRES_DB=testdb \
  -p 5432:5432 \
  postgres:15-alpine
```

2. 设置环境变量并运行测试：
```bash
export TEST_DB_HOST=localhost
export TEST_DB_PORT=5432
export TEST_DB_NAME=testdb
export TEST_DB_USER=testuser
export TEST_DB_PASSWORD=testpass

cargo test --test integration_test
```

3. 测试完成后清理容器：
```bash
docker stop test-postgres
docker rm test-postgres
```

### 集成测试内容

- ✅ PostgreSQL 连接测试
- ✅ 连接测试功能
- ✅ 查询执行测试（包括数据插入和查询）
- ✅ 空结果查询测试
- ✅ 连接失败测试

### 注意事项

- 测试会创建和操作真实的数据库表（`test_table`）
- 确保测试数据库有足够的权限
- 如果未设置环境变量，测试会被跳过（不会失败）

## 测试覆盖率

当前测试覆盖：
- ✅ SQL 解析和 LIMIT 注入
- ✅ DDL 语句检测
- ✅ SQLite 缓存服务（连接配置、元数据）
- ⏳ PostgreSQL 连接和查询（需要 Docker）

## 注意事项

- 集成测试会启动真实的 PostgreSQL 容器，运行时间较长
- 确保 Docker 有足够的资源（内存、CPU）
- 如果 Docker 不可用，集成测试会被跳过
