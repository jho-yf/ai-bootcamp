# Instructions

## codex prompt 架构和工具调用

仔细阅读 @vendors/codex 的代码。仔细研究其 system prompt 和工具相关的 prompt 架构，构建详细的介绍这些 prompt 的文档。输出到 @specs/w6/001-codex-prompts-and-tools.md

## opencode prompt 架构和工具调用

仔细阅读 @vendors/opencode/ 的代码。仔细研究其 system prompt 和工具相关的 prompt 架构，构建详细的介绍这些 prompt 的文档。输出到 @specs/w6/002-opencode-prompts-system.md

## opencode 输入输出记录

查看 @vendors/opencode/ 的代码，帮我了解如何最方便地获得 opencode 每次向 llm 发送的包含完整内容的输入输出，最好是有 hook / plugin 之类的，避免我直接修改源码。先不要撰写，告诉我方案

注意一次完整的对话（用户输入，agent 多轮工具调用，最后得到完整结果）的内容放在同一个 jsonl 里，新的内容 append 进去，不同的对话使用不同的 jsonl。请捕获每一个 turn 到 llm 的完整输入和输出，输出的内容放在 ./logs 下。


这样的话就变成一个 session 的多个 turn 都放一个文件里了，因为每个 turn 与llm交互的次数很多，而我希望每次交互的 input output 都能记录下来，能否将 sesssionid 作为文件夹，而每个 turn 都是一个独立的文件，文件名使用当前时间作为前缀，后面使用5个以内的单词概括本次 turn。请先不要编写代码，先与我确认方案 

生成的代码输出到 @w6-opencode-logging 中
