# Instructions

## codex prompt 架构和工具调用

仔细阅读 @vendors/codex 的代码。仔细研究其 system prompt 和工具相关的 prompt 架构，构建详细的介绍这些 prompt 的文档。输出到 @specs/w6/001-codex-prompts-and-tools.md

## opencode prompt 架构和工具调用

仔细阅读 @vendors/opencode/ 的代码。仔细研究其 system prompt 和工具相关的 prompt 架构，构建详细的介绍这些 prompt 的文档。输出到 @specs/w6/002-opencode-prompts-system.md

## opencode 输入输出记录

查看 @vendors/opencode/ 的代码，帮我了解如何最方便地获得 opencode 每次向 llm 发送的包含完整内容的输入输出，最好是有 hook / plugin 之类的，避免我直接修改源码。先不要撰写，告诉我方案

注意一次完整的对话（用户输入，agent 多轮工具调用，最后得到完整结果）的内容放在同一个 jsonl 里，新的内容 append 进去，不同的对话使用不同的 jsonl。请捕获每一个 turn 到 llm 的完整输入和输出，输出的内容放在 ./logs 下。


这样的话就变成一个 session 的多个 conversation 都放一个文件里了，因为每个 conversation 与 llm 交互的次数很多，而我希望每次交互的 input output 都能记录下来，能否将 sesssionid 作为文件夹，而每个 conversation 都是一个独立的文件，多个 turn 放在同一个文件中，文件名使用当前时间作为前缀，后面使用5个以内的单词概括本次 conversation。请先不要编写代码，先与我确认方案 

生成的代码输出到 @w6-opencode-logging 中

## opencode 输入输出记录可视化

读取 ./logs 下的文件，该文件夹下：

- 每一个子文件夹都是一个 session，其中前缀为日期+时间，后面是一个真正的 sessionid。
- 每个 conversation 都是一个独立的文件，文件名使用当前时间作为前缀，后面使用5个以内的单词概括本次 conversation。
- 每个 conversation 文件中，包含多轮 turn，每次 opencode 与 llm 的 input 和 output 都是独立的一行 json

分析每行 json 的 schema,帮我构建一个前端可视化 react app,用户打开一个 jsonl 文件，你可以将其很好地分门别类的在一个页面中展示不用 turn 的输入输出。

- 使用 scrollbar 来控制区域长度，文件内容使用 markdown renderer 来渲染。
- design token 使用 @w6-opencode-logging-ui/styles 中的 token
- 根据这些需求，先撰写一个 design doc 放在 @specs/w6/003-visualize-opencode-input-output.md
