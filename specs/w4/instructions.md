# Instructions

## Notebook LLM DeepResearch

研究在AI时代下，作为软件开发人员/架构师都是如何阅读大型代码库的，有什么最佳实践？特别是如何梳理它的架构，数据处理流程，接口设计，子系统及其接口设计，设计思路等。

## codex 代码阅读

仔细 @vendors/codex 的代码，撰写一个详细的架构分析文档，使用 mermaid chart 辅助说明，放在 `@specs/w4/001-codex-arch.md` 文件中，输出为中文。

## codex 架构绘制 (Gemini)

根据 codex 的架构文档，通过卡通形象绘制一张工作流程的图，可以让人清晰的看出来整个软件系统架构，事件循环机制，工作调用处理过程等

## codex history 阅读

查看 @vendors/codex repo 的所有 commit history，梳理其代码变更的脉络，必要时辅以 mermaid chart。写入 @specs/w4/004-codex-changes.md

## codex 事件循环

帮我梳理 @venders/codex 的代码，梳理其事件循环机制，详细解读当用户发起一个任务后，codex 是如何分解处理这个任务，并不断自我迭代，最终完成整个任务。这个过程发生了什么？codex 如何决定这个任务是否完成？必要时使用 mermaid chart 辅助说明，写入 @specs/w4/005-codex-event-loop.md

## codex 工具调用

帮我梳理 @venders/codex 的代码，梳理其处理工具调用的机制，详细解读 codex 是如何知道有哪些工具可以调用，如何选择工具，如何调用工具，如何处理工具的返回结果，如何决定工具是否调用成功等。必要时使用 mermaid chart 辅助说明。写入 @specs/w4/006-codex-tool-call.md

## codex 上下文压缩

帮我梳理 @venders/codex 的代码，梳理其上下文压缩的机制，详细解读 codex 是如何压缩上下文，如何决定是否需要压缩上下文，如何压缩上下文，压缩上下文时采取的算法或策略。必要时使用 mermaid chart 辅助说明。写入 @specs/w4/007-codex-context-compression.md

## codex apply_patch 工具

帮我梳理 @venders/codex 的代码，详细解读 apply_patch 工具的代码是如何跟 codex 其他组件集成的，另外我注意到 apply_patch_tool_instructions.md 文件，这个文件是用来做什么的？如何跟 apply_patch crate 打交道。必要时使用 mermaid chart 辅助说明。写入 @specs/w4/008-codex-apply-patch.md

## openclaw 架构设计

仔细 @vendors/openclaw 的代码，撰写一个详细的架构分析文档，包括但不限于：系统架构总览、分层架构设计、技术栈、数据模型设计、核心业务流程、系统安全设计等。必要时使用 mermaid chart 辅助说明，放在 `@specs/w4/009-openclaw-arch.md` 文件中，输出为中文。
