# Spec: MySQL Database Support

## Feature Overview

Add MySQL database support to the existing PostgreSQL-only database query tool, enabling users to connect to, explore, and query MySQL databases using both SQL and natural language.

## User Stories

### US1: Database Type Selection (Priority: P1) 🎯 MVP

**As a** user
**I want to** select MySQL as a database type when adding a new connection
**So that** I can connect to MySQL databases

**Acceptance Criteria**:
- Database connection form includes database type dropdown (PostgreSQL/MySQL)
- MySQL is selected by default or users can choose between PostgreSQL and MySQL
- Form validation adapts to selected database type (default ports: MySQL=3306, PostgreSQL=5432)
- Connection stores the database type for future use

**Notes**: This is the foundation - without database type tracking, no other MySQL features work.

---

### US2: MySQL Connection Support (Priority: P1) 🎯 MVP

**As a** user
**I want to** connect to a MySQL database using host, port, database name, username, and password
**So that** I can query my MySQL data

**Acceptance Criteria**:
- Users can enter MySQL connection details
- Application connects successfully to MySQL 5.7+ and MySQL 8.0+
- Connection test validates MySQL connectivity before saving
- Connection status shows as "Connected" for successful MySQL connections
- Connection failures display helpful error messages

**Dependencies**: Requires US1 (database type selection)

---

### US3: MySQL Metadata Extraction (Priority: P1) 🎯 MVP

**As a** user
**I want to** view MySQL database schema information (tables, columns, keys)
**So that** I can understand my database structure

**Acceptance Criteria**:
- Extract all tables from MySQL database (including schema/database name)
- Extract column information: name, data type, nullable, default value, primary key status
- Extract primary keys for each table
- Extract foreign key relationships
- Extract view definitions
- Display metadata in database explorer UI (existing UI should work)
- Metadata is cached locally for performance

**Dependencies**: Requires US2 (MySQL connection)

**MySQL-Specific Behavior**:
- Handle MySQL's `database()` instead of PostgreSQL's `current_schema()`
- Parse MySQL data types (INT, VARCHAR, TEXT, DATETIME, JSON, etc.)
- Handle MySQL identifier quoting (backticks vs double quotes)
- Use MySQL-compatible `information_schema` queries

---

### US4: MySQL Query Execution (Priority: P1) 🎯 MVP

**As a** user
**I want to** execute raw SQL queries on my MySQL database
**So that** I can retrieve and analyze my data

**Acceptance Criteria**:
- SQL query editor accepts and executes MySQL SQL syntax
- Query results display correctly with proper data type formatting
- Query execution time is measured and displayed
- Long-running queries can be cancelled
- Query errors display MySQL-specific error messages

**Dependencies**: Requires US2 (MySQL connection)

**MySQL-Specific Behavior**:
- Use mysql_async for query execution
- Handle MySQL result set format
- Support LIMIT clause (MySQL-compatible)
- Handle MySQL-specific data type conversions

---

### US5: Natural Language to MySQL SQL (Priority: P2)

**As a** user
**I want to** generate MySQL SQL from natural language descriptions
**So that** I can query my database without writing SQL

**Acceptance Criteria**:
- AI generates MySQL-compatible SQL from natural language
- Generated SQL uses MySQL syntax (backtick quotes, LIMIT, etc.)
- AI prompt includes MySQL database context
- Generated SQL executes successfully on MySQL

**Dependencies**: Requires US3 (metadata extraction for context)

**MySQL-Specific Behavior**:
- System prompt specifies MySQL (not PostgreSQL)
- Use backticks (`) for quoted identifiers
- Generate LIMIT without OFFSET by default
- Handle MySQL function names (NOW(), CURDATE(), etc.)
- Avoid PostgreSQL-specific features (ILIKE, RETURNING, etc.)

---

### US6: MySQL Data Type Handling (Priority: P2)

**As a** user
**I want to** see MySQL data types correctly formatted in query results
**So that** I can understand my data

**Acceptance Criteria**:
- MySQL INT displays as integer/number
- MySQL VARCHAR/TEXT displays as string
- MySQL DATETIME/TIMESTAMP displays as formatted date
- MySQL JSON displays as JSON object (if supported)
- MySQL TINYINT(1) displays as boolean where appropriate

**Dependencies**: Requires US4 (query execution)

---

### US7: Connection Management for MySQL (Priority: P2)

**As a** user
**I want to** manage (update, delete, test) my MySQL connections
**So that** I can keep my connection settings current

**Acceptance Criteria**:
- Update MySQL connection details
- Delete MySQL connections
- Test MySQL connection before saving
- Connection type is preserved when updating

**Dependencies**: Requires US2 (MySQL connection)

---

## Non-Functional Requirements

### Performance
- Metadata extraction should complete within 5 seconds for databases with < 100 tables
- Query results should display within 2 seconds for queries returning < 1000 rows

### Compatibility
- Support MySQL 5.7+
- Support MySQL 8.0+ (preferred)
- Support MariaDB 10.3+ (if feasible, defer if not)

### Reliability
- Connection timeout after 30 seconds
- Query timeout after 60 seconds (configurable)
- Graceful error handling for network failures

### Security
- Passwords encrypted in local storage (existing behavior)
- Support for SSL/TLS connections (defer to future story)

## Out of Scope

- SSH tunneling for remote MySQL access
- Connection pooling
- MySQL-specific features (stored procedures, triggers, events)
- Migration tools
- Data export/import
- Multiple result sets from stored procedures
- MySQL 5.6 or older versions

## Dependencies

- Existing PostgreSQL implementation (w2-db-query)
- OpenAI API for natural language queries
- SQLite for local caching (existing)
