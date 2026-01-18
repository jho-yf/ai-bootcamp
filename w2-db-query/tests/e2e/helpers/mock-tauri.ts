/**
 * Tauri API Mock
 * 在浏览器测试环境中模拟 Tauri API
 */

// Mock window.__TAURI__ 对象
if (typeof window !== 'undefined' && !window.__TAURI__) {
  (window as any).__TAURI__ = {
    core: {
      invoke: async (cmd: string, args?: any) => {
        console.log('[Mock Tauri] invoke:', cmd, args);

        // Mock 不同的命令响应
        switch (cmd) {
          case 'list_databases':
            return [
              {
                id: 'test-db-1',
                name: '测试数据库',
                host: 'localhost',
                port: 5432,
                databaseName: 'testdb',
                user: 'testuser',
                status: 'connected',
                createdAt: new Date().toISOString(),
                updatedAt: new Date().toISOString(),
              },
            ];

          case 'test_connection':
            // 模拟连接测试
            return true;

          case 'add_database':
            return {
              id: 'test-db-new',
              name: args?.name || '新数据库',
              host: args?.host || 'localhost',
              port: args?.port || 5432,
              databaseName: args?.databaseName || 'testdb',
              user: args?.user || 'testuser',
              status: 'connected',
              createdAt: new Date().toISOString(),
              updatedAt: new Date().toISOString(),
            };

          case 'get_database_metadata':
            return {
              connectionId: args?.databaseId || 'test-db-1',
              tables: [
                {
                  schema: 'public',
                  name: 'users',
                  tableType: 'BASE TABLE',
                  columns: [
                    {
                      name: 'id',
                      dataType: 'integer',
                      nullable: false,
                      isPrimaryKey: true,
                      ordinalPosition: 1,
                    },
                    {
                      name: 'name',
                      dataType: 'character varying',
                      nullable: false,
                      isPrimaryKey: false,
                      ordinalPosition: 2,
                    },
                  ],
                  primaryKeys: ['id'],
                  foreignKeys: [],
                },
              ],
              views: [],
              extractedAt: new Date().toISOString(),
            };

          case 'run_sql_query':
            return {
              columns: ['id', 'name'],
              rows: [
                { id: 1, name: 'Alice' },
                { id: 2, name: 'Bob' },
              ],
              total: 2,
              execTimeMs: 45,
              sql: args?.sql || 'SELECT * FROM users',
              truncated: false,
            };

          default:
            console.warn('[Mock Tauri] Unknown command:', cmd);
            return null;
        }
      },
    },
  };
}

// 导出 mock 函数供测试使用
export const mockTauriInvoke = (cmd: string, args?: any) => {
  if (typeof window !== 'undefined' && (window as any).__TAURI__) {
    return (window as any).__TAURI__.core.invoke(cmd, args);
  }
  return Promise.resolve(null);
};
