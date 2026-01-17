## cursor rust best practices rules

撰写一个 cursor rust best practices rules. 输入到 `.cursor/rules/rust-besr-practices.mdc` 中。其中：

- 出错处理使用：anyhow(app), thiserror(lib)
- concurrent sync: 优先使用 channel, 比如：mpsc
- async：tokio
- web/grpc：arum/tonic
- alway use system traits, e.g. implement From, TryFrom, FromStr whenever there's conversion from one type to other
- no unsafe

## commit command

构建一个新的 command, 用于提交代码变更。输入到 `.cursor/rules/commit.md` 中。


## 设计风格复刻

仔细浏览 https://ugmonk.com ，分析其页面设计风格，抽取设计风格的核心要素，如：boder padding margin fron color typographics component, 撰写一个可以复刻该网站的设计风格

## 生成前端设计风格 rules

生成一个 ugmonk 风格的前端设计 rules 放在 `.cursor/rules/style.md` 中, 参考 `docs/ugmonk-web-design.md` 的 【Ugmonk 设计风格系统指南】。
