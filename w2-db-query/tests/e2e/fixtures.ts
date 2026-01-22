import { test as base } from "@playwright/test";

/**
 * Playwright 测试 Fixtures
 * 提供自定义的测试上下文和工具
 */

// 扩展测试上下文
export const test = base.extend({
  // 带 Tauri mock 的页面
  pageWithMock: async ({ page }, use) => {
    // 注入 Tauri API mock
    await page.addInitScript(() => {
      if (!window.__TAURI__) {
        (window as any).__TAURI__ = {
          core: {
            invoke: async (cmd: string, args?: any) => {
              console.log("[Mock Tauri] invoke:", cmd, args);

              // Mock 响应
              const mocks: Record<string, (args?: any) => any> = {
                list_databases: () => [
                  {
                    id: "test-db-1",
                    name: "测试数据库",
                    host: "localhost",
                    port: 5432,
                    databaseName: "testdb",
                    user: "testuser",
                    status: "connected",
                    createdAt: new Date().toISOString(),
                    updatedAt: new Date().toISOString(),
                  },
                ],
                test_connection: () => true,
                add_database: (args) => ({
                  id: "test-db-new",
                  name: args?.request?.name || args?.name || "新数据库",
                  host: args?.request?.host || args?.host || "localhost",
                  port: args?.request?.port || args?.port || 5432,
                  databaseName:
                    args?.request?.databaseName ||
                    args?.databaseName ||
                    "testdb",
                  user: args?.request?.user || args?.user || "testuser",
                  status: "connected",
                  createdAt: new Date().toISOString(),
                  updatedAt: new Date().toISOString(),
                }),
                get_database_metadata: (args) => ({
                  connectionId: args?.databaseId || "test-db-1",
                  tables: [
                    {
                      schema: "public",
                      name: "users",
                      tableType: "BASE TABLE",
                      columns: [
                        {
                          name: "id",
                          dataType: "integer",
                          nullable: false,
                          isPrimaryKey: true,
                          ordinalPosition: 1,
                        },
                        {
                          name: "name",
                          dataType: "character varying",
                          nullable: false,
                          isPrimaryKey: false,
                          ordinalPosition: 2,
                        },
                      ],
                      primaryKeys: ["id"],
                      foreignKeys: [],
                    },
                  ],
                  views: [],
                  extractedAt: new Date().toISOString(),
                }),
                refresh_metadata: (args) => ({
                  connectionId: args?.databaseId || "test-db-1",
                  tables: [
                    {
                      schema: "public",
                      name: "users",
                      tableType: "BASE TABLE",
                      columns: [
                        {
                          name: "id",
                          dataType: "integer",
                          nullable: false,
                          isPrimaryKey: true,
                          ordinalPosition: 1,
                        },
                      ],
                      primaryKeys: ["id"],
                      foreignKeys: [],
                    },
                  ],
                  views: [],
                  extractedAt: new Date().toISOString(),
                }),
                run_sql_query: (args) => ({
                  columns: ["id", "name"],
                  rows: [
                    { id: 1, name: "Alice" },
                    { id: 2, name: "Bob" },
                  ],
                  total: 2,
                  execTimeMs: 45,
                  sql: args?.request?.sql || args?.sql || "SELECT * FROM users",
                  truncated: false,
                }),
                generate_sql_from_nl: () => "SELECT * FROM users LIMIT 100",
                run_nl_query: (args) => ({
                  generatedSql: "SELECT * FROM users LIMIT 100",
                  result: {
                    columns: ["id", "name"],
                    rows: [
                      { id: 1, name: "Alice" },
                      { id: 2, name: "Bob" },
                    ],
                    total: 2,
                    execTimeMs: 120,
                    sql: "SELECT * FROM users LIMIT 100",
                    truncated: false,
                  },
                }),
              };

              const handler = mocks[cmd];
              if (handler) {
                return handler(args);
              }

              console.warn("[Mock Tauri] Unknown command:", cmd);
              return null;
            },
          },
        };
      }
    });

    await use(page);
  },
});

export { expect } from "@playwright/test";
