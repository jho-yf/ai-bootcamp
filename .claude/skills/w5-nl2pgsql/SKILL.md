---
name: w5-nl2pgsql
description: Natural Language to SQL converter for PostgreSQL databases (pg_mcp_small/medium/large). Convert natural language queries to safe, read-only SQL with automatic schema lookup, security validation, and confidence scoring.
---

# PostgreSQL Natural Language to SQL (Week 5)

## Quick Start

Execute queries directly using shell script:

```bash
./scripts/pg_mcp_query.sh "查询 pg_mcp_small 数据库中所有产品的库存状态"
```

For SQL-only output, add "只返回SQL" to request:

```bash
./scripts/pg_mcp_query.sh "查询 pg_mcp_medium 数据库中学生的平均成绩，只返回SQL"
```

## Databases

| Database | Description | Schema Reference |
|----------|-------------|------------------|
| pg_mcp_small | E-commerce platform | `references/pg_mcp_small_schema.md` |
| pg_mcp_medium | School management | `references/pg_mcp_medium_schema.md` |
| pg_mcp_large | Hospital information system | `references/pg_mcp_large_schema.md` |

## Security

- **READ ONLY**: Only SELECT queries allowed
- **NO SQL INJECTION**: Uses psql -c with parameterized approach
- **AUTO LIMIT**: All queries include LIMIT 100 by default
- **DANGEROUS OPS BLOCKED**: Rejects INSERT/UPDATE/DELETE/DROP/CREATE/etc.

## Workflow

1. Detect database from user input
2. Load corresponding schema reference
3. Generate SQL based on natural language patterns
4. Validate SQL safety (read-only check)
5. Execute SQL with automatic LIMIT
6. Score confidence (0-10), retry if <7
7. Return SQL or results based on user preference

## Example Queries

```bash
# E-commerce
./scripts/pg_mcp_query.sh "查询 pg_mcp_small 数据库中待处理的订单"

# School
./scripts/pg_mcp_query.sh "查询 pg_mcp_medium 数据库中出勤率低于95%的学生"

# Hospital
./scripts/pg_mcp_query.sh "查询 pg_mcp_large 数据库中今日挂号的患者数量"
```
