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

## 使用 Gemini 探索技术方案

帮我探索 elevenlabs 实时 transcribe API （scribe v2 realtime）的 typescript 例子，并帮我构思如何实现一个类似 Wispr Flow 的工具。要求：app 使用 tauri 2 实现，app 打开后，常驻 systray, 用户使用 "ctrl + shit + \" hotkey 可以开启或者停止 transcribing。 从 scribe v2 api 获取的文本插入到当前 active app 的光标的位置。如果当前光标位置不可输入，那么就停止 transcribing 时，把内容拷贝到剪切板，用户粘贴到想要的地方。
