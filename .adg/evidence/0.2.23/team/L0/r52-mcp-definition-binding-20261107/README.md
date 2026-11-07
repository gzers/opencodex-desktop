# 0.2.23 MCP 双侧定义摘要补验

日期：2026-11-07
代码提交：`8ced5d442`
分支：`codex/0.2.23-integration`

## 结论

下载预览现在分别绑定本机所选客户端和云端受管定义的真实摘要。本机已有同名但定义不同的 MCP 时，确认可完成；云端定义写入 Desktop 受管 MCP 源，其他客户端文件保持原字节。本机没有该对象时，仍保留后续显式上传路径；伪造或未观察到的摘要被拒绝。

## 验证

- `cargo test --locked --features integration-test --lib mcp_ -- --nocapture`：27 passed，0 failed。
- `cargo test --locked --features integration-test --lib modules::model_config::sync_identity_v2::tests -- --nocapture`：27 passed，0 failed。
- `cargo clippy --locked --features integration-test --lib -- -D warnings`：通过。
- 生产 Service 补验：同名不同定义下载确认 `Applied`，云端 PUT 增量为 0；受管 `manager-state/mcp-definitions.json` 更新，所选 Codex 文件和未选客户端文件均保持原字节。

本证据使用隔离内存远端和临时 D/H，未执行真实业务 WebDAV、推理或日常配置修改。
