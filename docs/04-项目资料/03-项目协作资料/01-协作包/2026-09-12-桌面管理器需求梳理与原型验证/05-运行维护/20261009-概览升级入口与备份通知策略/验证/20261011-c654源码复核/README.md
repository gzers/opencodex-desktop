# 2026-10-11 c654287c 源码全量复核

本轮绑定代码分支 `feature/0.1.10-maintenance` 的完整提交 `c654287c06da3e621091c1051ebc19ce1c0b1077`，文档与证据归属 `docs/governance-main`。复核对象是运行态观测异常与启停执行异常隔离后的最新源码，不能引用旧提交的测试结果替代本轮绑定。

源码门禁均通过：

- Rust `cargo metadata --locked --format-version 1`、`cargo fmt --all -- --check`、`cargo clippy --locked --all-targets --all-features -- -D warnings` 均退出 0。
- `cargo test --locked` 退出 0：34 个测试组共 `861 passed / 0 failed / 5 ignored`；其中 Rust library 为 `694 passed / 0 failed / 4 ignored`，binary 为 `2 passed / 0 failed`。
- UI `npm test -- --run` 退出 0：`94` 个测试文件、`459` 项通过；`npm run typecheck` 通过；`npm run build` 通过，转换 `170` 个模块。
- Windows bundle identity smoke `node --test test/smoke/windows-bundle-identity.test.mjs` 为 `5 passed / 0 failed`。

运行态观测异常的解除条件继续要求可执行文件、工作目录、`OPENCODEX_HOME`、对象、动作、阶段、通道和候选身份全部匹配；可解除项限定为 `runtime-starting-failed`、`run-unreachable`、`run-at-risk`、`external-takeover`。这条恢复路径不能解除 start / stop / restart 的 execution 生命周期失败。当前源码注册表快照为 8 triggers、39 jobs、81 events、27 notification policies、87 emission sites、8 scheduling sites、8 UI feedback sites、12 paths、4 cleanup policies。

Vite 的既有 `INEFFECTIVE_DYNAMIC_IMPORT` 警告和测试中的 jsdom canvas notice 未阻断结果。详细命令与结构化结果见 `.adg/evidence/OCX-0110-20261011-C654-SOURCE-REVALIDATION/`。

本轮只闭合最新提交的源码级回归证据。macOS 原生 UI / 安装 / 更新重启、Windows 普通权限与安装位置 / DPI / 云母 / 标题栏 / 动效 / 透明下拉框、真实自动检查性能与 stable / beta 周期、真实安装器 / 签名 / 更新端点、真实用户迁移及发布资产仍未闭合。`release_authorization: false`，IMP 与 TASK 继续 `in_progress`；不能把旧 0.1.9 Windows 候选包复用为 0.1.10 制品。
