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
