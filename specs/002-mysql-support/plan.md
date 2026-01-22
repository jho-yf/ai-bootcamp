# Plan: MySQL Database Support

## Overview

Extend the existing PostgreSQL database query tool (w2-db-query) to support MySQL databases. This includes MySQL metadata extraction, query execution, and natural language to SQL generation with MySQL syntax support.

## Tech Stack

### Backend (Rust/Tauri)
- **Existing**: tokio-postgres for PostgreSQL
- **New**: mysql_async (rust-mysql-simple) for MySQL connectivity
- **Existing**: async-openai for AI SQL generation
- **Existing**: rusqlite for local caching
- **Existing**: sqlparser for SQL parsing
- **Existing**: tauri 2.0 for desktop framework

### Frontend (TypeScript/React)
- **Existing**: React + TypeScript with Vite
- **Existing**: Ant Design UI components
- **Existing**: Tauri APIs for backend communication

## Project Structure

```
w2-db-query/
├── src-tauri/src/
│   ├── models/
│   │   ├── database.rs       # Add database_type field
│   │   └── metadata.rs       # Already database-agnostic
│   ├── services/
│   │   ├── mod.rs            # Export mysql_service
│   │   ├── postgres_service.rs   # Existing PostgreSQL
│   │   ├── mysql_service.rs      # NEW: MySQL implementation
│   │   ├── metadata_service.rs   # Update for MySQL support
│   │   └── ai_service.rs         # Update for MySQL syntax
│   └── commands/
│       ├── database.rs       # Add database type routing
│       ├── metadata.rs       # Already database-agnostic
│       └── ai.rs             # Pass database type to AI
├── src/
│   ├── services/
│   │   ├── types.ts          # Add DatabaseType enum
│   │   └── api.ts            # Already database-agnostic
│   └── components/
│       └── database/         # Add database type selection
```

## Key Dependencies

### Cargo.toml Additions
```toml
mysql_async = "0.34"  # MySQL async client
```

## Implementation Strategy

### Phase 1: Database Type Support
1. Add `database_type` field to `DatabaseConnection` model
2. Update frontend to include database type selection
3. Update connection storage to persist database type

### Phase 2: MySQL Service Implementation
1. Create `mysql_service.rs` mirroring `postgres_service.rs`
   - `connect()` - MySQL connection
   - `test_connection()` - Connection validation
   - `execute_query()` - Query execution
2. Implement MySQL-specific metadata extraction
   - Use `information_schema` with MySQL syntax
   - Handle MySQL-specific data types
   - Extract indexes and constraints

### Phase 3: AI Service Enhancement
1. Update AI prompts to support MySQL syntax
2. Detect database type and generate appropriate SQL dialect
3. Handle MySQL-specific features (AUTO_INCREMENT, backtick quotes, etc.)

## MySQL vs PostgreSQL Differences

### Connection
| PostgreSQL | MySQL |
|------------|-------|
| `host=localhost port=5432` | `mysql://host:3306/db` |
| `tokio-postgres` | `mysql_async` |
| Default port: 5432 | Default port: 3306 |

### Metadata Queries
| PostgreSQL | MySQL |
|------------|-------|
| `current_schema()` | `database()` |
| `pg_catalog` | `information_schema` |
| `pg_class`, `pg_attribute` | `tables`, `columns` |
| `$1`, `$2` parameters | `?` parameters |

### SQL Syntax
| PostgreSQL | MySQL |
|------------|-------|
| `LIMIT 100` (implicit offset) | `LIMIT 100` |
| `ILIKE` (case-insensitive) | `LIKE` with `LOWER()` |
| `TRUE`/`FALSE` | `1`/`0` or `TRUE`/`FALSE` (8.0+) |
| Double quotes for identifiers | Backticks for identifiers |
| `RETURNING` clause | `LAST_INSERT_ID()` function |

### Data Type Mapping
| PostgreSQL | MySQL |
|------------|-------|
| `integer` | `INT` |
| `varchar` | `VARCHAR` |
| `timestamp` | `TIMESTAMP` or `DATETIME` |
| `text` | `TEXT` or `LONGTEXT` |
| `boolean` | `TINYINT(1)` or `BOOL` |
| `uuid` | `CHAR(36)` or `BINARY(16)` |
| `json`/`jsonb` | `JSON` |

## Open Questions

1. **SSL/TLS Support**: Should we support MySQL SSL connections?
2. **Connection Pooling**: Implement pooling for better performance?
3. **SSH Tunneling**: Support SSH tunneling for remote MySQL access?
4. **MySQL Versions**: Target minimum MySQL version (5.7 or 8.0+)?

## Success Criteria

1. Users can connect to MySQL databases (host, port, credentials)
2. Metadata extraction works (tables, columns, keys, indexes)
3. SQL queries execute successfully on MySQL
4. Natural language generates MySQL-compatible SQL
5. Existing PostgreSQL functionality remains intact
