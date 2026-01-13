# 自动化集成测试文档

## 概述

本项目包含完整的自动化集成测试套件，覆盖所有 API 端点的功能测试。

## 测试结构

### 测试文件
- `integration_test.rs` - 完整的集成测试套件

### 测试覆盖范围

#### Tag API 测试 (`test_tag_crud`)
- ✅ 获取所有标签（空列表）
- ✅ 创建标签（带颜色）
- ✅ 创建标签（不带颜色）
- ✅ 获取单个标签
- ✅ 更新标签（完整更新）
- ✅ 更新标签（部分更新 - 只更新名称）
- ✅ 更新标签（部分更新 - 只更新颜色）
- ✅ 创建重复标签（验证失败）
- ✅ 获取不存在的标签（404）
- ✅ 更新不存在的标签（404）
- ✅ 删除标签
- ✅ 验证标签已删除

#### Ticket API 测试 (`test_ticket_crud`)
- ✅ 获取所有 tickets（空列表）
- ✅ 创建 ticket（不带标签）
- ✅ 创建 ticket（带标签）
- ✅ 创建 ticket（不带描述）
- ✅ 获取所有 tickets
- ✅ 获取单个 ticket
- ✅ 获取带标签的 ticket
- ✅ 更新 ticket（完整更新）
- ✅ 更新 ticket（部分更新）
- ✅ 切换完成状态
- ✅ 再次切换完成状态
- ✅ 添加标签到 ticket
- ✅ 添加另一个标签
- ✅ 移除标签
- ✅ 获取不存在的 ticket（404）
- ✅ 更新不存在的 ticket（404）
- ✅ 删除 ticket
- ✅ 验证 ticket 已删除

#### Ticket 筛选测试 (`test_ticket_filtering`)
- ✅ 按标签筛选
- ✅ 按完成状态筛选（已完成）
- ✅ 按完成状态筛选（未完成）
- ✅ 搜索 tickets（区分大小写）
- ✅ 搜索 tickets（不区分大小写）
- ✅ 组合筛选（标签 + 完成状态 + 搜索）

#### 错误处理测试 (`test_error_handling`)
- ✅ 添加标签到不存在的 ticket（404）
- ✅ 从不存在的 ticket 移除标签（404）
- ✅ 切换不存在的 ticket 的完成状态（404）
- ✅ 删除不存在的 ticket（404）
- ✅ 从不存在的 ticket 移除不存在的标签（404）

## 运行测试

### 前置要求

1. **数据库设置**
   - 确保 PostgreSQL 正在运行
   - 创建测试数据库：`project_alpha_test`
   - 设置环境变量：`DATABASE_URL=postgresql://postgres:postgres@localhost/project_alpha_test`

2. **环境变量**
   ```bash
   export DATABASE_URL=postgresql://postgres:postgres@localhost/project_alpha_test
   export HOST=127.0.0.1
   export PORT=0
   ```

### 运行所有测试

```bash
cd backend
cargo test --test integration_test
```

### 运行特定测试

```bash
# 运行 Tag CRUD 测试
cargo test --test integration_test test_tag_crud

# 运行 Ticket CRUD 测试
cargo test --test integration_test test_ticket_crud

# 运行筛选测试
cargo test --test integration_test test_ticket_filtering

# 运行错误处理测试
cargo test --test integration_test test_error_handling
```

### 运行测试并显示输出

```bash
cargo test --test integration_test -- --nocapture
```

## 测试工作原理

### 测试服务器

每个测试都会：
1. 启动一个独立的测试服务器（绑定到随机端口）
2. 运行数据库迁移
3. 清理测试数据（TRUNCATE）
4. 执行测试用例
5. 自动清理（服务器在测试结束时关闭）

### 测试隔离

- 每个测试使用独立的数据库连接
- 每个测试前都会清理数据
- 测试之间互不干扰

### 测试客户端

测试使用 `reqwest` HTTP 客户端发送请求：
- GET 请求
- POST 请求（带 JSON body）
- PUT 请求（带 JSON body）
- PATCH 请求
- DELETE 请求

## 测试统计

- **测试用例总数**: 4 个测试函数
- **API 端点覆盖**: 13 个端点全部覆盖
- **测试场景**: 50+ 个测试场景
- **错误处理**: 完整的错误场景测试

## 持续集成

### GitHub Actions 示例

```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: project_alpha_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
    
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost/project_alpha_test
        run: |
          cd backend
          cargo test --test integration_test
```

## 故障排除

### 常见问题

1. **数据库连接失败**
   - 检查 PostgreSQL 是否运行
   - 检查 `DATABASE_URL` 环境变量
   - 检查数据库是否存在

2. **端口冲突**
   - 测试使用随机端口（PORT=0），通常不会有冲突
   - 如果仍有问题，检查是否有其他服务占用端口

3. **迁移失败**
   - 确保数据库用户有创建表的权限
   - 检查迁移文件是否正确

4. **测试超时**
   - 增加测试超时时间：`cargo test --test integration_test -- --test-threads=1`

## 添加新测试

### 测试模板

```rust
#[tokio::test]
async fn test_new_feature() {
    let server = TestServer::start().await;
    let client = TestClient::new(server.base_url.clone());
    
    // 你的测试代码
    let resp = client.get("/api/endpoint").await;
    assert_eq!(resp.status(), 200);
}
```

### 最佳实践

1. **测试命名**: 使用描述性的测试函数名
2. **测试隔离**: 每个测试应该独立运行
3. **清理数据**: 测试前清理，测试后验证
4. **断言清晰**: 使用明确的断言消息
5. **错误测试**: 包含错误场景测试

## 性能考虑

- 测试使用真实的数据库连接
- 每个测试都会清理数据，确保一致性
- 测试服务器在后台运行，不影响测试速度
- 使用异步测试，提高并发性能

## 相关文档

- [API 测试文档](./README.md) - REST Client 手动测试
- [Phase 3 完成文档](../../PHASE3_COMPLETE.md) - 测试与优化阶段
- [实现计划文档](../../specs/0002-implementation-plan.md) - 项目实现计划
