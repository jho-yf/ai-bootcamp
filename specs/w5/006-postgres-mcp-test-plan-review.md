# PostgreSQL MCP Server Test Plan Review

## Executive Summary
The test plan is broad and aligned with most core modules (`config`, `validator`, `llm`, `metadata`, `executor`) and includes dedicated sections for MCP contract, performance, and security. However, it has several high-impact gaps: critical behavior from the implementation plan is not fully asserted (especially retry boundaries and lifecycle cleanup), many test cases are still skeleton comments rather than executable specs, and some examples are not directly compilable. In its current state, this is a strong draft but not yet an execution-ready, CI-reliable test plan.

Overall quality score: **7/10 (good coverage intent, medium execution readiness)**.

---

## Detailed Findings by Category

## 1. Coverage Completeness

### Finding COV-01: `main.rs` lifecycle and refresh-task cleanup are not covered
- Severity: **High**
- Evidence:
  - Test plan focuses on module tests and integration flows but has no explicit `main.rs` lifecycle tests ([specs/w5/005-postgres-mcp-test-plan.md:56](./005-postgres-mcp-test-plan.md), [specs/w5/005-postgres-mcp-test-plan.md:857](./005-postgres-mcp-test-plan.md)).
  - Implementation plan requires `JoinHandle::abort` cleanup on shutdown ([specs/w5/003-postgres-mcp-impl-plan.md:577](./003-postgres-mcp-impl-plan.md), [specs/w5/003-postgres-mcp-impl-plan.md:610](./003-postgres-mcp-impl-plan.md)).
- Risk: refresh loop leaks/background-task orphaning may regress without detection.

### Finding COV-02: retry boundary behavior not fully covered
- Severity: **High**
- Evidence:
  - Plan has retry success/failure tests ([specs/w5/005-postgres-mcp-test-plan.md:1020](./005-postgres-mcp-test-plan.md), [specs/w5/005-postgres-mcp-test-plan.md:1026](./005-postgres-mcp-test-plan.md)).
  - But missing explicit test for "validation errors must not retry" required by implementation detail ([specs/w5/003-postgres-mcp-impl-plan.md:550](./003-postgres-mcp-impl-plan.md)).
- Risk: server may retry unsafe SQL paths, increasing cost and ambiguity.

### Finding COV-03: metadata security behavior has partial but incomplete coverage
- Severity: **Medium**
- Evidence:
  - Comment stripping is tested ([specs/w5/005-postgres-mcp-test-plan.md:663](./005-postgres-mcp-test-plan.md)).
  - But no explicit test that **view definitions are excluded** from prompt context (design/impl requirement: only view names) ([specs/w5/003-postgres-mcp-impl-plan.md:359](./003-postgres-mcp-impl-plan.md)).
- Risk: prompt-injection surface via view definitions may regress silently.

## 2. Test Quality

### Finding QLT-01: several tests are placeholders, not actionable test specs
- Severity: **High**
- Evidence: many cases contain only comments without arrange/act/assert details (e.g., [specs/w5/005-postgres-mcp-test-plan.md:407](./005-postgres-mcp-test-plan.md), [specs/w5/005-postgres-mcp-test-plan.md:766](./005-postgres-mcp-test-plan.md), [specs/w5/005-postgres-mcp-test-plan.md:970](./005-postgres-mcp-test-plan.md), [specs/w5/005-postgres-mcp-test-plan.md:1121](./005-postgres-mcp-test-plan.md)).
- Risk: inconsistent implementation by different engineers; hard to estimate effort.

### Finding QLT-02: sample code contains compile/runtime-quality issues
- Severity: **Medium**
- Evidence:
  - Same `Result` consumed twice with `unwrap_err()` ([specs/w5/005-postgres-mcp-test-plan.md:372](./005-postgres-mcp-test-plan.md), [specs/w5/005-postgres-mcp-test-plan.md:373](./005-postgres-mcp-test-plan.md)).
  - Async `.await` used under `#[test]` in metadata unit snippets ([specs/w5/005-postgres-mcp-test-plan.md:612](./005-postgres-mcp-test-plan.md), [specs/w5/005-postgres-mcp-test-plan.md:617](./005-postgres-mcp-test-plan.md)).
- Risk: teams copy snippets and lose time on avoidable fixes.

## 3. Security Testing

### Finding SEC-01: lock-clause coverage is narrow
- Severity: **High**
- Evidence: only `FOR UPDATE` and `FOR SHARE` are listed ([specs/w5/005-postgres-mcp-test-plan.md:315](./005-postgres-mcp-test-plan.md), [specs/w5/005-postgres-mcp-test-plan.md:323](./005-postgres-mcp-test-plan.md)).
- Missing: `FOR NO KEY UPDATE`, `FOR KEY SHARE`, `NOWAIT`, `SKIP LOCKED` variants.
- Risk: partial lock-path regressions despite AST lock checks.

### Finding SEC-02: nested CTE bypass scenarios are not explicitly tested
- Severity: **High**
- Evidence:
  - Design strategy explicitly calls out nested subquery/CTE security depth ([specs/w5/002-postgres-mcp-design.md:1663](./002-postgres-mcp-design.md)).
  - Plan lacks explicit nested malicious CTE cases.
- Risk: AST traversal bugs in nested forms can bypass top-level checks.

### Finding SEC-03: allowlist/excludelist tests miss schema-qualified authorization edge cases
- Severity: **Medium**
- Evidence: schema-qualified names are tested only in unrestricted mode ([specs/w5/005-postgres-mcp-test-plan.md:416](./005-postgres-mcp-test-plan.md)).
- Missing: `allowed_tables=["users"]` with `public.users`/`other_schema.users` behavior.
- Risk: inconsistent canonicalization can cause false allow/deny.

## 4. Integration and MCP Contract Tests

### Finding MCP-01: MCP negative-contract coverage is incomplete
- Severity: **Medium**
- Evidence: current list covers initialize/list/call success + two error types ([specs/w5/005-postgres-mcp-test-plan.md:989](./005-postgres-mcp-test-plan.md)).
- Missing important cases:
  - unknown tool name
  - invalid params schema (missing `question`, wrong type)
  - malformed JSON-RPC / unknown method behavior
  - correlation/id handling across concurrent requests
- Risk: protocol regressions not caught until client interoperability testing.

## 5. Missing Scenarios

### Finding MIS-01: LLM client failure-path tests required by impl plan are missing
- Severity: **Medium**
- Evidence:
  - Impl acceptance includes API error formatting and timeout behavior ([specs/w5/003-postgres-mcp-impl-plan.md:510](./003-postgres-mcp-impl-plan.md), [specs/w5/003-postgres-mcp-impl-plan.md:512](./003-postgres-mcp-impl-plan.md)).
  - Test plan’s `llm.rs` section focuses on extract/prompt only ([specs/w5/005-postgres-mcp-test-plan.md:444](./005-postgres-mcp-test-plan.md)).
- Risk: HTTP edge paths break silently.

### Finding MIS-02: non-functional observability/audit assertions are absent
- Severity: **Low**
- Evidence: design defines audit/log strategy, including masking and attempt metrics ([specs/w5/002-postgres-mcp-design.md:1580](./002-postgres-mcp-design.md)).
- Missing tests for password masking and retry-attempt log fields in integrated flows.

## 6. Practical Feasibility

### Finding FEA-01: infrastructure strategy is internally inconsistent
- Severity: **High**
- Evidence:
  - Plan defines custom `testcontainers` fixture ([specs/w5/005-postgres-mcp-test-plan.md:865](./005-postgres-mcp-test-plan.md)).
  - Same section uses `#[sqlx::test]` pool-managed tests ([specs/w5/005-postgres-mcp-test-plan.md:884](./005-postgres-mcp-test-plan.md)).
  - Design also lists both approaches without clear precedence ([specs/w5/002-postgres-mcp-design.md:1679](./002-postgres-mcp-design.md)).
- Risk: duplicated setup, flaky local/CI behavior, unclear ownership.

### Finding FEA-02: environment variable guidance conflicts with config contract
- Severity: **Medium**
- Evidence:
  - Impl defines `PG_MCP_DATABASE_URL` mapping ([specs/w5/003-postgres-mcp-impl-plan.md:160](./003-postgres-mcp-impl-plan.md)).
  - CI/local examples use `DATABASE_URL` ([specs/w5/005-postgres-mcp-test-plan.md:1230](./005-postgres-mcp-test-plan.md), [specs/w5/005-postgres-mcp-test-plan.md:1254](./005-postgres-mcp-test-plan.md)).
- Risk: tests fail unexpectedly depending on which loader path is used.

### Finding FEA-03: performance thresholds in regular test runs are likely flaky
- Severity: **Medium**
- Evidence: strict wall-clock assertions (`<5s`, `<15s`) are listed as hard acceptance ([specs/w5/005-postgres-mcp-test-plan.md:1088](./005-postgres-mcp-test-plan.md), [specs/w5/005-postgres-mcp-test-plan.md:1283](./005-postgres-mcp-test-plan.md)).
- Risk: CI noise from host variability rather than regressions.

## 7. Organization and Maintainability

### Finding ORG-01: duplicated security content across sections causes drift risk
- Severity: **Low**
- Evidence: validator unit section and standalone security section repeat similar cases ([specs/w5/005-postgres-mcp-test-plan.md:182](./005-postgres-mcp-test-plan.md), [specs/w5/005-postgres-mcp-test-plan.md:1116](./005-postgres-mcp-test-plan.md)).
- Risk: one section updated, another forgotten.

### Finding ORG-02: test naming consistency issue
- Severity: **Low**
- Evidence: mixed-language typo in test name (`test_end_to端_aggregation_query`) ([specs/w5/005-postgres-mcp-test-plan.md:939](./005-postgres-mcp-test-plan.md)).
- Risk: searchability and readability degradation.

---

## Specific Recommendations (with examples)

1. Convert placeholders into executable test cards.
Example template per case:
```markdown
- Test ID: VAL-LOCK-003
- Precondition: validator with empty allow/exclude sets
- Input SQL: SELECT * FROM users FOR NO KEY UPDATE
- Steps: call validate(sql)
- Assert: returns Err and message contains "锁子句"
```

2. Add missing high-risk security cases.
Example additions:
- `WITH a AS (SELECT * FROM users WHERE id IN (WITH d AS (DELETE FROM users RETURNING id) SELECT id FROM d)) SELECT * FROM a` -> must reject.
- `SELECT * FROM users FOR KEY SHARE SKIP LOCKED` -> must reject.
- `allowed_tables=["users"]` + `SELECT * FROM public.users` -> expected behavior explicitly documented/tested.

3. Strengthen MCP contract negative tests.
Add tests for:
- `tools/call` missing `question`
- unknown tool name
- invalid method name
- parallel calls with distinct `id` preserving response correlation

4. Align infra strategy to one primary path.
Recommendation:
- Unit/integration default: `#[sqlx::test]` + service postgres in CI.
- Optional stress/nightly: `testcontainers` as separate profile/job.
- Document one canonical local command path.

5. Add explicit `main.rs` lifecycle tests.
At least:
- `metadata_refresh_secs > 0` starts refresh loop once.
- server shutdown triggers refresh task abort.
- startup error path exits on DB connect failure.

6. Fix env-var contract examples.
Use `PG_MCP_DATABASE_URL` in CI/local snippets, or clearly state dual support and include both mapping tests.

7. Split performance tests into non-blocking suites.
- Keep functional assertions in CI.
- Move time-budget assertions to nightly or benchmark job with stable runner.

---

## Verdict
**Verdict: Conditionally approved (requires revision before execution).**

The plan is directionally strong and mostly aligned with design/implementation intent, but should be revised for executability and risk coverage before implementation starts. Priority should be: (1) convert placeholder tests to concrete specs, (2) close high-severity security and retry-boundary gaps, (3) resolve infrastructure/env inconsistencies.
