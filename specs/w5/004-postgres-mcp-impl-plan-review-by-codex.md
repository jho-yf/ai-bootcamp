# PostgreSQL MCP Server 实现计划 - Code Review

## Review 信息

| 项目 | 内容 |
|------|------|
| Review 日期 | 2026-03-31 |
| Review 工具 | Codex (GPT-5.3-codex) |
| Review 对象 | [003-postgres-mcp-impl-plan.md](./003-postgres-mcp-impl-plan.md) |
| Review 维度 | 完整性、依赖关系、风险评估、可行性、测试覆盖、实现顺序 |

---

## 总体评价

计划结构清晰、模块边界明确，但当前版本有几个会直接影响可实现性的缺口（配置优先级、依赖准确性、数据访问约束、执行器序列化策略）。不修正的话，实施中会出现编译/行为偏差。

---

## Findings（按严重级别）

### 1. Critical | Completeness/Feasibility

**位置**：Task 2 (config.rs)

**问题**：目标写的是 `CLI > 文件 > 环境变量 > 默认值`，但示例实现完全没有环境变量层。

**建议**：在 Task 2 明确 `env` 映射表（如 `PG_MCP_DATABASE_URL` 等）、优先级合并顺序和冲突规则，并加入对应测试用例。

---

### 2. Critical | Dependencies/Feasibility

**位置**：Task 1 (项目初始化)

**问题**：`sqlparser-rs` 作为 crate 名称很可能不正确（Rust 依赖名通常是 `sqlparser`）。这会导致 Task 1 卡死在编译阶段。

**建议**：在 Task 1 增加"锁定并验证依赖名/版本"的子步骤，实际以 `cargo check` + 最小解析示例验证。

---

### 3. High | Completeness/Risk

**位置**：Task 2 (allowed_tables), Task 4 (excluded_tables)

**问题**：配置里有 `allowed_tables`，但后续任务只提 `excluded_tables`，未定义"执行时强制表白名单"策略。

**建议**：在 `validator` 或 `executor` 增加 AST 级表名提取与白名单校验；并新增拒绝用例（访问未授权表时失败）。

---

### 4. High | Feasibility

**位置**：Task 5 (executor.rs)

**问题**：`row.get::<String>(i)` 对很多 PostgreSQL 类型不可行，运行时易报类型解码错误。

**建议**：改为类型分派到 `serde_json::Value`（常见标量/时间/uuid/jsonb/array），并对未知类型回退字符串化。

---

### 5. High | Risk Assessment/Feasibility

**位置**：Task 5 (executor.rs - apply_limit)

**问题**：`apply_limit` 仅靠"是否包含 LIMIT"判断过于脆弱（子查询 LIMIT、注释/字符串、UNION/CTE）。

**建议**：把 LIMIT 注入改为 AST 重写或"外层包裹 `SELECT * FROM (...) t LIMIT n`"；并补充复杂 SQL 测试。

---

### 6. High | Completeness/Ordering

**位置**：Task 4 (metadata.rs - start_refresh_loop), Task 8 (main.rs)

**问题**：刷新循环只有 `start_refresh_loop()`，没有停止机制和任务句柄管理，主进程退出/重载时有生命周期风险。

**建议**：返回 `JoinHandle` 或 `CancellationToken`，在 `main` 中统一 shutdown。

---

### 7. Medium | Dependencies/Ordering

**位置**：依赖图 (Section 9)

**问题**：Task 2/3/4/5/6 已要求单测，但依赖图又写 `Task 8 -> Task 9`，逻辑冲突。

**建议**：把测试拆成"随模块并行的单测" + "最终回归测试"，更新依赖图。

---

### 8. Medium | Test Coverage

**位置**：Task 10 (集成测试)

**问题**：集成测试缺少 MCP 协议层关键场景（`list_tools` schema、handshake、tool 错误码映射、retry 分支）。

**建议**：新增协议契约测试，至少覆盖 handshake、list_tools、一次执行失败后二次重试成功。

---

### 9. Medium | Risk Assessment

**位置**：Section 8 (风险与缓解)

**问题**：风险表漏项较多：测试容器环境不可用、连接池/超时参数不当、prompt budget 按字符/token 不一致、敏感日志泄露。

**建议**：补风险项并给出"检测信号 + 预案"（如 CI 无 Docker 时降级策略、日志脱敏基线、压测阈值）。

---

### 10. Low | Feasibility/Actionability

**位置**：Task 3, Task 6 (验收标准)

**问题**："错误信息清晰"这类验收标准不可量化。

**建议**：定义可验证标准（错误码、固定前缀、是否包含建议动作）。

---

## 按维度总结

| 维度 | 评价 |
|------|------|
| **Completeness** | 中等偏好，但有关键漏项（env 层、allowed_tables 执法、生命周期管理） |
| **Dependencies** | 主干依赖基本对，但测试依赖和模块内依赖表达不一致 |
| **Risk Assessment** | 已有风险方向对，但覆盖不足，缺运维/安全/测试环境风险 |
| **Feasibility** | 总体可行；若不修复 `sqlparser` 与序列化/LIMIT 策略，会显著受阻 |
| **Test Coverage** | 不足以保障协议层和复杂 SQL 边界 |
| **Ordering** | 基本合理；建议把"测试左移"并提前做协议契约与依赖探针验证 |
