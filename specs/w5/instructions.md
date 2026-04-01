# Instructions

## postgres mcp server 技术栈（Gemini DeepResearch）

帮我探索 postgres mcp server 的 rust 实现方案，帮我研究一下这个需要使用 rust 来实现的需要用到哪些库，以及为什么要用到这些库

要求：

- 该 mcp server 启动的时候应该读取可以访问的数据库并且缓存这些数据库的元数据，包括：table,view,index,types 等
- mcp server 根据用户输入的自然语言，结合缓存的元数据，调用 OpenAI 规范的大模型接口，返回一个 sql 或者对应的查询结果
- mcp server 应该能够校验 sql 只允许查询语句并且测试这个 sql 保证返回有意义的结果
- 大模型接口的配置，可以由用户在配置文件中配置，也可以在运行时通过命令行参数配置

## postgres mcp server 需求分析

帮我分析一下 postgres mcp server 的 rust 实现的需求，必要时使用 mermaid chart 辅以说明，必要时使用 web search 确保信息的准确性。将结果输出到 `@specs/w5/001-postgres-mcp-prd.md` 文件中，输出为中文。

要求：

- 该 mcp server 启动的时候应该读取可以访问的数据库并且缓存这些数据库的元数据，包括：table,view,index,types 等
- mcp server 根据用户输入的自然语言，结合缓存的元数据，调用 OpenAI 规范的大模型接口，返回一个 sql 或者对应的查询结果
- mcp server 应该能够校验 sql 只允许查询语句并且测试这个 sql 保证返回有意义的结果
- 大模型接口的配置，可以由用户在配置文件中配置，也可以在运行时通过命令行参数配置

## postgres mcp server 设计文档

think ultra hard, 根据 @specs/w5/001-postgres-mcp-prd.md 文档，使用 Rust + sqlx + sqlparser-rs + rmcp + rmcp-macros + tokio + serde/serde_json + schemars 构建 pg-mcp 的设计文档，必要时使用 mermaid chart 辅助说明，输出到 @specs/w5/001-postgres-mcp-design.md, 输出为中文

## review

使用 sub agent 调用 codex review skill 让 codex review @specs/w5/001-postgres-mcp-prd.md 文件。之后仔细阅读 review 结果，思考是否合理，然后相应地更新 @specs/w5/001-postgres-mcp-design.md

## 研究技术栈（Gemini DeepResearch）

帮我深度研究 tokio，了解其设计理念，应用场景，以及相比同类框架的优劣

## CLAUDE.md 生成

为 @w5-pg-mcp 生成 CLAUDE.md 文件。要求：

1. 符合 rust bust practice 
2. 符合 SOLID/DRY 等设计原则
3. 代码质量和测试质量要高，性能要好

## postgres mcp server impl plan

根据 @specs/w5/002-postgres-mcp-design.md 构建 pg-mcp 的实现计划，think ultra hard, 文档输出到 @specs/w5/003-postgres-mcp-impl-plan.md 中。之后调用 /codex:review review @specs/w5/003-postgres-mcp-impl-plan.md 并输出 review 结果到 @specs/w5/004-postgres-mcp-impl-plan-review-by-codex.md

## postgres mcp server impl

commit and 根据 @specs/w5/003-postgres-mcp-impl-plan.md 实现 phase 1-6，think ultra hard，代码输出到 @w5-pg-mcp 目录下

commit and 根据 @specs/w5/003-postgres-mcp-impl-plan.md 实现剩下所有 task，think ultra hard，代码输出到 @w5-pg-mcp 目录下

## postgres mcp server test plan

根据 @specs/w5/003-postgres-mcp-impl-plan.md 和 @specs/w5/002-postgres-mcp-design.md 构建 pg-mcp 的测试计划，think ultra hard,文档放在 @specs/w5/005-postgres-mcp-test-plan.md 并使用 /codex:review review 输出到 @specs/w5/006-postgres-mcp-test-plan-review.md。最后根据 review 内容，修订并输出到 @specs/w5/005-postgres-mcp-test-plan-v2.md

## 测试数据构建

根据 @specs/w5/001-postgres-mcp-prd.md 在 @w5-pg-mcp/fixtures 下构建三个有意义的数据库，分别有少量，中等量级以及大量的 table/view/types/index 等schema,且有足够多的有意义的数据。生成这三个数据库的 sql 文件，并构建 Makefile 来重建这些测试数据库。

## 测试 prompt 构建

根据 @w5-pg-mcp/fixtures/ ,假设用户要用自然语言提问，然后 pg-mcp 来生成相应的 sql。帮我生成一个 test-prompts.md 的文档，里面包含各种对数据库内部数据的简单到复杂的提问 

## 自动化测试

对于 @w5-pg-mcp, 将这个 mcp 添加到 @.claude 中，打开一个 claude code headless cli 选择 @@w5-pg-mcp/fixtures/test-prompts.md 下面的某些 query 来运行，查看是否调用了这个 mcp 以及结果是否符合预期
