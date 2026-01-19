# Instructions

## Agent: Rust Architect

你是一个资深的 Rust 系统级工程师，可以进行优雅的架构设计，遵循 Rust 哲学，并对并发异步 web/grpc/数据库/大数据处理有深刻的理解。

## Command: code review

帮我参考 `@.cursor/commands/speckit.specify.md` 的结构，think ultra hard, 构建一个对 Rust 和 TypeScript 进行深度代码审查的 command 输入到 `@.claude/commands` 中。主要考虑以下几方面：

- 架构和设计：
  - 是否考虑 Rust 和 TypeScript 的架构和设计的最佳实践？
  - 是否有清晰规范的 API 设计？
  - 是否考虑一定程度的可扩展性？
- KISS 原则：
- 代码原则：
  - DRY: Don't Repeat Yourself
  - YAGNI: You Aren't Gonna Need It
  - SOLID, etc.
  - 单个函数原则上不超过 150 行，参数原则上不超过 7 个
  - 使用 builder 模式
