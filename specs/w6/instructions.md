# Instructions

## codex prompt 架构和工具调用

仔细阅读 @vendors/codex 的代码。仔细研究其 system prompt 和工具相关的 prompt 架构，构建详细的介绍这些 prompt 的文档。输出到 @specs/w6/001-codex-prompts-and-tools.md

## opencode prompt 架构和工具调用

仔细阅读 @vendors/opencode/ 的代码。仔细研究其 system prompt 和工具相关的 prompt 架构，构建详细的介绍这些 prompt 的文档。输出到 @specs/w6/002-opencode-prompts-system.md

## opencode 输入输出

查看 @vendors/opencode/ 的代码，帮我了解如何最方便地获得 opencode 每次向 llm 发送的包含完整内容的输入输出，最好是有 hook / plugin 之类的，避免我直接修改源码。先不要撰写，告诉我方案
