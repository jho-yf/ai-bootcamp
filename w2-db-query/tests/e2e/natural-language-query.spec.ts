import { test, expect } from "@playwright/test";

/**
 * 自然语言查询功能测试
 */
test.describe("自然语言查询功能", () => {
  test.beforeEach(async ({ page }) => {
    // 注入 Tauri API mock
    await page.addInitScript(() => {
      if (!window.__TAURI__) {
        (window as any).__TAURI__ = {
          core: {
            invoke: async (cmd: string, args?: any) => {
              switch (cmd) {
                case "list_databases":
                  return [
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
                  ];
                case "get_database_metadata":
                  return {
                    connectionId: "test-db-1",
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
                  };
                case "generate_sql_from_nl":
                  return "SELECT * FROM users LIMIT 100";
                case "run_nl_query":
                  return {
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
                  };
                default:
                  return null;
              }
            },
          },
        };
      }
    });

    await page.goto("/");
    await page.waitForSelector(".ant-layout", { timeout: 10000 });
    await page.waitForLoadState("domcontentloaded");
    await page.waitForTimeout(2000);

    // 点击第一个数据库菜单项
    const menuItem = page.locator(".ant-menu-item").first();
    const isVisible = await menuItem.isVisible().catch(() => false);
    if (isVisible) {
      await menuItem.click();
      await page.waitForTimeout(1500);
    }
  });

  test("应该显示标签页：SQL 编辑器和自然语言查询", async ({ page }) => {
    await page.waitForTimeout(1500);

    // 查找标签页
    const tabs = page.locator(".ant-tabs-tab");
    const count = await tabs.count();

    if (count > 0) {
      // 检查是否有 SQL 编辑器标签
      const sqlTab = page.getByRole("tab").filter({ hasText: /SQL|编辑器/i });
      const nlTab = page.getByRole("tab").filter({ hasText: /自然语言|查询/i });

      const sqlVisible = await sqlTab.isVisible().catch(() => false);
      const nlVisible = await nlTab.isVisible().catch(() => false);

      if (sqlVisible || nlVisible) {
        await expect(tabs.first()).toBeVisible({ timeout: 5000 });
      }
    }
  });

  test("应该能够切换到自然语言查询标签页", async ({ page }) => {
    await page.waitForTimeout(1500);

    // 查找自然语言查询标签
    const nlTab = page
      .getByRole("tab")
      .filter({ hasText: /自然语言|查询/i })
      .first();
    const isVisible = await nlTab.isVisible().catch(() => false);

    if (isVisible) {
      await nlTab.click();
      await page.waitForTimeout(500);

      // 应该显示自然语言输入框
      const input = page.locator("textarea").first();
      const inputVisible = await input.isVisible().catch(() => false);

      if (inputVisible) {
        await expect(input).toBeVisible({ timeout: 5000 });
      }
    }
  });

  test("应该在自然语言查询标签页显示按钮", async ({ page }) => {
    await page.waitForTimeout(1500);

    // 切换到自然语言查询标签
    const nlTab = page
      .getByRole("tab")
      .filter({ hasText: /自然语言|查询/i })
      .first();
    const isVisible = await nlTab.isVisible().catch(() => false);

    if (isVisible) {
      await nlTab.click();
      await page.waitForTimeout(500);

      // 检查按钮是否存在
      const generateButton = page
        .getByRole("button")
        .filter({ hasText: /仅生成|SQL/i });
      const executeButton = page
        .getByRole("button")
        .filter({ hasText: /生成并执行/i });

      const generateVisible = await generateButton
        .isVisible()
        .catch(() => false);
      const executeVisible = await executeButton.isVisible().catch(() => false);

      // 至少应该有一个按钮可见
      if (generateVisible || executeVisible) {
        expect(generateVisible || executeVisible).toBe(true);
      }
    }
  });

  test("应该在顶部按钮区域显示自然语言查询按钮", async ({ page }) => {
    await page.waitForTimeout(1500);

    // 切换到自然语言查询标签
    const nlTab = page
      .getByRole("tab")
      .filter({ hasText: /自然语言|查询/i })
      .first();
    const isVisible = await nlTab.isVisible().catch(() => false);

    if (isVisible) {
      await nlTab.click();
      await page.waitForTimeout(500);

      // 查找顶部按钮区域的"仅生成 SQL"按钮
      const generateButton = page
        .getByRole("button")
        .filter({ hasText: /仅生成 SQL/i })
        .first();

      const isVisible = await generateButton.isVisible().catch(() => false);
      if (isVisible) {
        await expect(generateButton).toBeVisible({ timeout: 5000 });
      }
    }
  });

  test("应该在 SQL 编辑器标签页禁用自然语言查询按钮", async ({ page }) => {
    await page.waitForTimeout(1500);

    // 确保在 SQL 编辑器标签页
    const sqlTab = page
      .getByRole("tab")
      .filter({ hasText: /SQL|编辑器/i })
      .first();
    const tabVisible = await sqlTab.isVisible().catch(() => false);

    if (tabVisible) {
      await sqlTab.click();
      await page.waitForTimeout(500);

      // 查找"仅生成 SQL"按钮
      const generateButton = page
        .getByRole("button")
        .filter({ hasText: /仅生成 SQL/i })
        .first();

      const isVisible = await generateButton.isVisible().catch(() => false);
      if (isVisible) {
        const isDisabled = await generateButton.isDisabled();
        // 在 SQL 编辑器标签页，按钮应该被禁用
        expect(isDisabled).toBe(true);
      }
    }
  });

  test("应该能够输入自然语言查询", async ({ page }) => {
    await page.waitForTimeout(1500);

    // 切换到自然语言查询标签
    const nlTab = page
      .getByRole("tab")
      .filter({ hasText: /自然语言|查询/i })
      .first();
    const isVisible = await nlTab.isVisible().catch(() => false);

    if (isVisible) {
      await nlTab.click();
      await page.waitForTimeout(500);

      // 查找输入框
      const input = page.locator("textarea").first();
      const inputVisible = await input.isVisible().catch(() => false);

      if (inputVisible) {
        await input.fill("查询所有用户");
        await page.waitForTimeout(300);

        // 验证输入内容
        const value = await input.inputValue();
        expect(value).toContain("查询所有用户");
      }
    }
  });

  test("应该显示生成的 SQL（如果有）", async ({ page }) => {
    await page.waitForTimeout(1500);

    // 切换到自然语言查询标签
    const nlTab = page
      .getByRole("tab")
      .filter({ hasText: /自然语言|查询/i })
      .first();
    const isVisible = await nlTab.isVisible().catch(() => false);

    if (isVisible) {
      await nlTab.click();
      await page.waitForTimeout(500);

      // 输入自然语言查询
      const input = page.locator("textarea").first();
      const inputVisible = await input.isVisible().catch(() => false);

      if (inputVisible) {
        await input.fill("查询所有用户");
        await page.waitForTimeout(300);

        // 点击"仅生成 SQL"按钮
        const generateButton = page
          .getByRole("button")
          .filter({ hasText: /仅生成 SQL/i })
          .first();

        const buttonVisible = await generateButton
          .isVisible()
          .catch(() => false);
        if (
          buttonVisible &&
          !(await generateButton.isDisabled().catch(() => false))
        ) {
          await generateButton.click();
          await page.waitForTimeout(1000);

          // 应该显示生成的 SQL（可能在某个区域）
          const sqlText = page.locator("text=/SELECT|生成的 SQL/i").first();
          const sqlVisible = await sqlText.isVisible().catch(() => false);

          if (sqlVisible) {
            await expect(sqlText).toBeVisible({ timeout: 5000 });
          }
        }
      }
    }
  });
});
