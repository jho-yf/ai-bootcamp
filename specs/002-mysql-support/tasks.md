# Tasks: MySQL Database Support

**Input**: Design documents from `/specs/002-mysql-support/`
**Prerequisites**: plan.md, spec.md, data-model.md

**Tests**: Tests are not explicitly requested in the specification, so test tasks are optional. The tasks below focus on implementation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- Backend Rust: `w2-db-query/src-tauri/src/`
- Frontend TypeScript: `w2-db-query/src/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add MySQL dependency and update project structure

- [ ] T001 Add mysql_async dependency to Cargo.toml in w2-db-query/src-tauri/Cargo.toml
- [ ] T002 Create MySQL service module file in w2-db-query/src-tauri/src/services/mysql_service.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core model changes that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T003 Add DatabaseType enum to w2-db-query/src-tauri/src/models/database.rs
- [ ] T004 Add database_type field to DatabaseConnection struct in w2-db-query/src-tauri/src/models/database.rs
- [ ] T005 Add database_type field to AddDatabaseRequest struct in w2-db-query/src-tauri/src/models/database.rs
- [ ] T006 Add database_type field to UpdateDatabaseRequest struct in w2-db-query/src-tauri/src/models/database.rs
- [ ] T007 [P] Add DatabaseType type to w2-db-query/src/services/types.ts
- [ ] T008 [P] Add databaseType field to DatabaseConnection interface in w2-db-query/src/services/types.ts
- [ ] T009 [P] Add databaseType field to AddDatabaseRequest interface in w2-db-query/src/services/types.ts
- [ ] T010 Export mysql_service module in w2-db-query/src-tauri/src/services/mod.rs
- [ ] T011 Implement SQLite migration to add database_type column in w2-db-query/src-tauri/src/services/cache_service.rs

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Database Type Selection (Priority: P1) 🎯 MVP

**Goal**: Enable users to select MySQL as a database type when adding new connections

**Independent Test**: When adding a new database connection, user can choose between PostgreSQL and MySQL from a dropdown. Default port updates based on selection.

### Implementation for User Story 1

- [ ] T012 [US1] Add database type dropdown to connection form in w2-db-query/src/components/database/AddDatabaseModal.tsx
- [ ] T013 [US1] Add port default logic based on database type (MySQL=3306, PostgreSQL=5432) in w2-db-query/src/components/database/AddDatabaseModal.tsx
- [ ] T014 [US1] Update addDatabase API call to include databaseType in w2-db-query/src/services/api.ts

**Checkpoint**: At this point, users can select database type when adding connections

---

## Phase 4: User Story 2 - MySQL Connection Support (Priority: P1) 🎯 MVP

**Goal**: Enable users to connect to MySQL databases

**Independent Test**: User can add a MySQL connection with valid credentials and see "Connected" status

### Implementation for User Story 2

- [ ] T015 [P] [US2] Implement MySQL connect function in w2-db-query/src-tauri/src/services/mysql_service.rs
- [ ] T016 [US2] Implement MySQL test_connection function in w2-db-query/src-tauri/src/services/mysql_service.rs
- [ ] T017 [US2] Add database type routing in add_database command in w2-db-query/src-tauri/src/commands/database.rs
- [ ] T018 [US2] Add database type routing in update_database command in w2-db-query/src-tauri/src/commands/database.rs
- [ ] T019 [US2] Add database type routing in test_connection command in w2-db-query/src-tauri/src/commands/database.rs

**Checkpoint**: At this point, MySQL connections can be added and tested successfully

---

## Phase 5: User Story 3 - MySQL Metadata Extraction (Priority: P1) 🎯 MVP

**Goal**: Extract and display MySQL database schema information

**Independent Test**: After connecting to MySQL, clicking refresh metadata shows all tables, columns, primary keys, and foreign keys

### Implementation for User Story 3

- [ ] T020 [P] [US3] Implement MySQL extract_tables function in w2-db-query/src-tauri/src/services/mysql_service.rs
- [ ] T021 [P] [US3] Implement MySQL extract_columns function in w2-db-query/src-tauri/src/services/mysql_service.rs
- [ ] T022 [P] [US3] Implement MySQL extract_primary_keys function in w2-db-query/src-tauri/src/services/mysql_service.rs
- [ ] T023 [P] [US3] Implement MySQL extract_foreign_keys function in w2-db-query/src-tauri/src/services/mysql_service.rs
- [ ] T024 [P] [US3] Implement MySQL extract_views function in w2-db-query/src-tauri/src/services/mysql_service.rs
- [ ] T025 [US3] Create MySQL-specific extract_metadata function in w2-db-query/src-tauri/src/services/mysql_service.rs
- [ ] T026 [US3] Add database type routing to metadata_service for extraction in w2-db-query/src-tauri/src/services/metadata_service.rs
- [ ] T027 [US3] Add database type routing in refresh_metadata command in w2-db-query/src-tauri/src/commands/metadata.rs

**Checkpoint**: At this point, MySQL metadata extraction works and displays in UI

---

## Phase 6: User Story 4 - MySQL Query Execution (Priority: P1) 🎯 MVP

**Goal**: Execute raw SQL queries on MySQL databases

**Independent Test**: User can enter `SELECT * FROM table_name LIMIT 10` in query editor and see results from MySQL database

### Implementation for User Story 4

- [ ] T028 [P] [US4] Implement MySQL execute_query function in w2-db-query/src-tauri/src/services/mysql_service.rs
- [ ] T029 [P] [US4] Implement MySQL row to JSON conversion in w2-db-query/src-tauri/src/services/mysql_service.rs
- [ ] T030 [US4] Add database type routing in run_sql_query command in w2-db-query/src-tauri/src/commands/query.rs

**Checkpoint**: At this point, MySQL queries execute successfully and display results

---

## Phase 7: User Story 5 - Natural Language to MySQL SQL (Priority: P2)

**Goal**: Generate MySQL-compatible SQL from natural language

**Independent Test**: User types "show me all users" and gets MySQL-compatible SQL (with backticks, proper LIMIT)

### Implementation for User Story 5

- [ ] T031 [US5] Add database_type parameter to build_system_prompt in w2-db-query/src-tauri/src/services/ai_service.rs
- [ ] T032 [US5] Create MySQL-specific system prompt with backtick identifiers and MySQL syntax in w2-db-query/src-tauri/src/services/ai_service.rs
- [ ] T033 [US5] Update generate_sql_from_natural_language to accept database_type parameter in w2-db-query/src-tauri/src/services/ai_service.rs
- [ ] T034 [US5] Pass database type to AI service from generate_sql_from_nl command in w2-db-query/src-tauri/src/commands/ai.rs
- [ ] T035 [US5] Pass database type to AI service from run_nl_query command in w2-db-query/src-tauri/src/commands/ai.rs

**Checkpoint**: At this point, natural language generates MySQL-compatible SQL

---

## Phase 8: User Story 6 - MySQL Data Type Handling (Priority: P2)

**Goal**: Display MySQL data types correctly in query results

**Independent Test**: Query results show proper data types (INT as number, DATETIME as formatted date, TINYINT(1) as boolean)

### Implementation for User Story 6

- [ ] T036 [P] [US6] Implement MySQL data type conversion (INT, VARCHAR, DATETIME, JSON) in w2-db-query/src-tauri/src/services/mysql_service.rs
- [ ] T037 [US6] Handle MySQL TINYINT(1) to boolean conversion in w2-db-query/src-tauri/src/services/mysql_service.rs

**Checkpoint**: At this point, MySQL data types display correctly in UI

---

## Phase 9: User Story 7 - Connection Management for MySQL (Priority: P2)

**Goal**: Manage (update, delete, test) MySQL connections

**Independent Test**: User can update MySQL connection details, delete MySQL connections, and test connection before saving

### Implementation for User Story 7

- [ ] T038 [US7] Ensure database type is preserved when updating connections in w2-db-query/src-tauri/src/commands/database.rs
- [ ] T039 [US7] Add database type display to connection list UI in w2-db-query/src/components/database/DatabaseList.tsx

**Checkpoint**: At this point, MySQL connections can be fully managed

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T040 [P] Add MySQL connection example to README in w2-db-query/README.md
- [ ] T041 [P] Document MySQL-specific features and limitations in w2-db-query/docs/mysql-support.md
- [ ] T042 Add error handling for MySQL-specific error codes in w2-db-query/src-tauri/src/utils/error.rs
- [ ] T043 Update quickstart.md to include MySQL setup instructions in w2-db-query/quickstart.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-9)**: All depend on Foundational phase completion
  - US1 (Phase 3) - Database Type Selection: Can start after Foundational
  - US2 (Phase 4) - MySQL Connection: Depends on US1 completion
  - US3 (Phase 5) - Metadata Extraction: Depends on US2 completion
  - US4 (Phase 6) - Query Execution: Depends on US2 completion (parallel with US3)
  - US5 (Phase 7) - NL to SQL: Depends on US3 completion
  - US6 (Phase 8) - Data Type Handling: Depends on US4 completion
  - US7 (Phase 9) - Connection Management: Depends on US2 completion
- **Polish (Phase 10)**: Depends on all user stories being complete

### User Story Dependencies

```
Foundational (Phase 2)
       │
       ▼
US1: Database Type Selection (Phase 3)
       │
       ▼
US2: MySQL Connection (Phase 4)
       │
       ├───▶ US3: Metadata Extraction (Phase 5)
       │           │
       │           ▼
       │       US5: NL to MySQL SQL (Phase 7)
       │
       ├───▶ US4: Query Execution (Phase 6)
       │           │
       │           ▼
       │       US6: Data Type Handling (Phase 8)
       │
       └───▶ US7: Connection Management (Phase 9)
```

### Within Each User Story

- Models and services before commands
- Backend implementation before frontend
- Core implementation before integration

### Parallel Opportunities

- **Phase 2**: T007, T008, T009 (frontend types) can run in parallel
- **Phase 3**: None (sequential UI changes)
- **Phase 4**: T015 (service) can start before T017-T019 (commands)
- **Phase 5**: T020-T024 (extraction functions) can all run in parallel
- **Phase 6**: T028-T029 can run in parallel
- **Phase 8**: T036-T037 can run in parallel
- **Phase 10**: T040-T041 can run in parallel

---

## Parallel Example: User Story 3 (Metadata Extraction)

```bash
# Launch all metadata extraction functions together:
Task: "Implement MySQL extract_tables function"
Task: "Implement MySQL extract_columns function"
Task: "Implement MySQL extract_primary_keys function"
Task: "Implement MySQL extract_foreign_keys function"
Task: "Implement MySQL extract_views function"
```

---

## Parallel Example: User Story 6 (Data Type Handling)

```bash
# Launch both data type conversion tasks together:
Task: "Implement MySQL data type conversion"
Task: "Handle MySQL TINYINT(1) to boolean conversion"
```

---

## Implementation Strategy

### MVP First (User Stories 1-4)

1. Complete Phase 1: Setup (T001-T002)
2. Complete Phase 2: Foundational (T003-T011) - CRITICAL
3. Complete Phase 3: US1 Database Type Selection (T012-T014)
4. Complete Phase 4: US2 MySQL Connection (T015-T019)
5. Complete Phase 5: US3 Metadata Extraction (T020-T027)
6. Complete Phase 6: US4 Query Execution (T028-T030)
7. **STOP and VALIDATE**: Test full MySQL workflow (connect → metadata → query)
8. Deploy/demo if ready

### Incremental Delivery

1. **MVP** (US1-US4): Basic MySQL connection, metadata, and query
2. **AI Integration** (US5): Natural language to MySQL SQL
3. **Data Type Polish** (US6): Better type handling
4. **Management Polish** (US7): Connection management improvements
5. **Final Polish**: Documentation and cross-cutting improvements

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together (T001-T011)
2. Once Foundational is done:
   - Developer A: US3 Metadata Extraction (T020-T027) - complex, can parallelize internally
   - Developer B: US4 Query Execution (T028-T030) - simpler, depends on US2
3. US5 depends on US3, can start after metadata extraction is done

---

## Summary

**Total Tasks**: 43 tasks

**Tasks per User Story**:
- Setup: 2 tasks
- Foundational: 9 tasks
- US1 (Database Type Selection): 3 tasks
- US2 (MySQL Connection): 5 tasks
- US3 (Metadata Extraction): 8 tasks
- US4 (Query Execution): 3 tasks
- US5 (NL to SQL): 5 tasks
- US6 (Data Type Handling): 2 tasks
- US7 (Connection Management): 2 tasks
- Polish: 4 tasks

**Parallel Opportunities**: 15+ tasks can run in parallel with teammates

**Independent Test Criteria**:
- US1: Database type dropdown visible and functional
- US2: MySQL connection shows "Connected" status
- US3: Metadata shows tables/columns/keys
- US4: SQL query returns results from MySQL
- US5: Natural language generates MySQL-compatible SQL
- US6: Data types display correctly
- US7: Connection updates preserve database type

**Suggested MVP Scope**: US1-US4 (Phases 1-6, tasks T001-T030) - This delivers full MySQL connectivity, metadata extraction, and query execution capability.
