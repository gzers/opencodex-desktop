# MCP 配置终态

代码分支：`feature/0.1.10-maintenance`
代码提交：`a392afd9b6a81ea6d82eb5ba591600466765e0f4`
证据目录：`.adg/evidence/OCX-0110-20261011-MCP-CONFIG-TERMINAL/`

## 本轮范围

- MCP 配置读取统一使用 16 MiB 有界、拒绝符号链接和非普通文件的安全读取；不存在仍表示未创建配置，其他读取错误显式失败。
- `WriteMcp`、`RemoveMcp`、`AddMcp`、`EditMcp` 对目标文件按稳定路径顺序加锁，锁覆盖读取、合并、备份、原子写和写后核验。
- `WriteMcp` 复用准备阶段读取的原件，避免同一事务重复读取不同文件版本；原子写后精确回读，字节不一致不报告成功。
- home、Skills / MCP / lock 路径、越界路径和重复物理目标在执行前拒绝。

## 本地验证

`cargo test --workspace --features integration-test`：690 passed / 0 failed / 4 ignored。

`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`git diff --check`：通过。

## CI 与限制

新 CI [38077500332](https://github.com/gzers/opencodex-desktop/actions/runs/38077500332)（attempt 2）已结束：frontend、backend、release-tools 通过；backend-windows 在测试进程启动前因 `STATUS_ENTRYPOINT_NOT_FOUND`（`0xc0000139`）失败，Windows release build 与 final EXE smoke 被跳过。因此 Windows 回归、打包和 EXE smoke 门禁均未通过。旧 CI [38076431305](https://github.com/gzers/opencodex-desktop/actions/runs/38076431305) 同样保留为失败证据。

本证据只证明 MCP 配置写入局部终态。它不替代 Skills / MCP 客户端资产投射、原生 Windows / 安装器、普通与管理员权限、实际更新 / 重启、性能、真实 stable 24h / beta 6h 周期或全域事件验收；没有迁移真实用户数据，也没有发布 0.1.10。
