# Data Model: MySQL Database Support

## Overview

This document describes the data model changes needed to support MySQL databases alongside the existing PostgreSQL implementation.

## Existing Models (PostgreSQL)

### DatabaseConnection
Located: `src-tauri/src/models/database.rs`

```rust
pub struct DatabaseConnection {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub user: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub status: ConnectionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## New Models

### DatabaseType (NEW)
**Purpose**: Enum to distinguish between database types

**Location**: `src-tauri/src/models/database.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
}
```

### Updated DatabaseConnection
**Change**: Add `database_type` field

```rust
pub struct DatabaseConnection {
    pub id: String,
    pub name: String,
    pub database_type: DatabaseType,  // NEW FIELD
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub user: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub status: ConnectionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Updated AddDatabaseRequest
**Change**: Add `database_type` field

```rust
pub struct AddDatabaseRequest {
    pub name: String,
    pub database_type: DatabaseType,  // NEW FIELD
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub user: String,
    pub password: String,
}
```

### Updated UpdateDatabaseRequest
**Change**: Add optional `database_type` field

```rust
pub struct UpdateDatabaseRequest {
    pub id: String,
    pub name: Option<String>,
    pub database_type: Option<DatabaseType>,  // NEW FIELD
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database_name: Option<String>,
    pub user: Option<String>,
    pub password: Option<String>,
}
```

## Frontend Type Updates

### DatabaseType (NEW)
**Location**: `src/services/types.ts`

```typescript
export type DatabaseType = "postgresql" | "mysql";
```

### Updated DatabaseConnection
**Location**: `src/services/types.ts`

```typescript
export interface DatabaseConnection {
  id: string;
  name: string;
  databaseType: DatabaseType;  // NEW FIELD
  host: string;
  port: number;
  databaseName: string;
  user: string;
  status: ConnectionStatus;
  createdAt: string;
  updatedAt: string;
}
```

### Updated AddDatabaseRequest
**Location**: `src/services/types.ts`

```typescript
export interface AddDatabaseRequest {
  name: string;
  databaseType: DatabaseType;  // NEW FIELD
  host: string;
  port: number;
  databaseName: string;
  user: string;
  password: string;
}
```

## Service Layer Models

### MySQL Client (Internal)
**Location**: `src-tauri/src/services/mysql_service.rs`

Uses `mysql_async::Pool` and `mysql_async::Conn` for MySQL connections.

```rust
use mysql_async::Pool;
use mysql_async::prelude::*;

// Connection pool for MySQL
pub struct MySqlConnection {
    pool: Pool,
}
```

## Metadata Model (No Changes Required)

The existing `DatabaseMetadata`, `TableInfo`, `ColumnInfo`, `ViewInfo`, and `ForeignKeyInfo` models are database-agnostic and work for both PostgreSQL and MySQL.

**Location**: `src-tauri/src/models/metadata.rs`

```rust
// These models remain unchanged - they work for both databases
pub struct DatabaseMetadata { ... }
pub struct TableInfo { ... }
pub struct ColumnInfo { ... }
pub struct ViewInfo { ... }
pub struct ForeignKeyInfo { ... }
```

## Data Type Mappings

### PostgreSQL → MySQL Type Mapping

| PostgreSQL | MySQL | Notes |
|------------|-------|-------|
| integer | INT | Direct mapping |
| bigint | BIGINT | Direct mapping |
| smallint | SMALLINT | Direct mapping |
| varchar(n) | VARCHAR(n) | Direct mapping |
| text | TEXT | Direct mapping |
| boolean | TINYINT(1) | MySQL boolean workaround |
| timestamp | TIMESTAMP | Use TIMESTAMP for timezones |
| timestamptz | TIMESTAMP | MySQL TIMESTAMP has timezone |
| date | DATE | Direct mapping |
| time | TIME | Direct mapping |
| numeric(p,s) | DECIMAL(p,s) | Direct mapping |
| real | FLOAT | Direct mapping |
| double precision | DOUBLE | Direct mapping |
| uuid | CHAR(36) | String representation |
| jsonb | JSON | MySQL 5.7+ supports JSON |
| bytea | LONGBLOB | Binary data |
| array[] | JSON | No native array type |

## ER Diagram

```
┌─────────────────────────────────────────┐
│         DatabaseConnection              │
├─────────────────────────────────────────┤
│ id: String (PK)                         │
│ name: String                            │
│ database_type: DatabaseType (NEW)       │
│ host: String                            │
│ port: u16                               │
│ database_name: String                   │
│ user: String                            │
│ password: String (encrypted)            │
│ status: ConnectionStatus                │
│ created_at: DateTime<Utc>               │
│ updated_at: DateTime<Utc>               │
├─────────────────────────────────────────┤
│                                         │
│ 1 ──────────────────────────────┐       │
│                                  │       │
│                                  ▼       │
│                    ┌─────────────────────────────┐
│                    │      DatabaseMetadata      │
│                    ├─────────────────────────────┤
│                    │ connection_id: String (FK)  │
│                    │ tables: Vec<TableInfo>      │
│                    │ views: Vec<ViewInfo>        │
│                    │ extracted_at: DateTime<Utc> │
│                    └─────────────────────────────┘
│                                  │
│                    ┌─────────────┴─────────────┐
│                    ▼                           ▼
│          ┌─────────────────┐        ┌─────────────────┐
│          │    TableInfo    │        │    ViewInfo     │
│          ├─────────────────┤        ├─────────────────┤
│          │ schema: String  │        │ schema: String  │
│          │ name: String    │        │ name: String    │
│          │ table_type: ... │        │ columns: [...]  │
│          │ columns: [...]  │        │ definition: ... │
│          │ primary_keys: []│        └─────────────────┘
│          │ foreign_keys: []│
│          └─────────────────┘
│                  │
│                  │ 1
│      ┌───────────┴───────────┐
│      ▼                       ▼
│ ┌─────────────┐      ┌─────────────────┐
│ │ ColumnInfo  │      │ ForeignKeyInfo  │
│ ├─────────────┤      ├─────────────────┤
│ │ name: ...   │      │ constraint_name │
│ │ data_type:  │      │ column_name     │
│ │ nullable:   │      │ referenced_...  │
│ │ default:    │      │ referenced_...  │
│ │ is_pk: ...  │      └─────────────────┘
│ │ position:   │
│ └─────────────┘
└─────────────────────────────────────────┘
```

## Migration Strategy

Since we're adding a new field to existing models, we need a migration strategy:

1. **New installations**: `database_type` is required
2. **Existing installations**: Default to `PostgreSQL` for existing connections without `database_type`

### SQLite Schema Migration

```sql
-- Add database_type column to existing connections table
ALTER TABLE database_connections ADD COLUMN database_type TEXT DEFAULT 'postgresql';
```
