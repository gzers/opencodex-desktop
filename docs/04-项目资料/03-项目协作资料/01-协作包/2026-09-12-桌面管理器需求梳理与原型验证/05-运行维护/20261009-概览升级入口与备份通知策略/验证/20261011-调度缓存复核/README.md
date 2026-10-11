# 2026-10-11 调度与缓存定向复核

本轮绑定代码主分支 main@861fdeea39d1c45fa2beddb2cba6659b67735d9f，文档归属 docs/governance-main。测试使用现有唯一隔离测试实例，不新增桌面实例，不接触日常安装。

## 执行

    cwd: /Users/ezio/.codex/worktrees/ocx-0110-main
    cargo test --locked --manifest-path apps/desktop/tauri/Cargo.toml --features integration-test schedule -- --nocapture
    结果：5 passed; 0 failed; 0 ignored；exit 0

    cwd: /Users/ezio/.codex/worktrees/ocx-0110-main/apps/desktop/ui
    npm test -- --run tests/update-scheduler.test.ts tests/update-query-ownership.test.ts tests/update-install-ownership.test.ts tests/update-version.test.ts
    结果：4 test files passed; 18 tests passed；exit 0

Rust 定向项覆盖固定通道周期、持久 deadline 与损坏状态处理、路由 / 重启无请求、退避睡眠与通道隔离 / 缓存清理、受限 release notes 缓存和超限拒绝。UI 定向项覆盖调度、查询所有权、安装所有权和版本身份。

本轮可以确认源码合同通过，不能确认真实 CPU、RSS、网络请求数、查询子进程、启动 p95、帧耗时或 stable 24 小时 / beta 6 小时周期；也不能替代 Windows 双权限 / DPI / 原生外观、macOS 原生安装、真实更新 / 重启、签名和发布门禁。IMP-OPENCODEX-DESKTOP-20 继续 in_progress，release_authorized: false。

机器可读证据见 .adg/evidence/OCX-0110-20261011-SCHEDULE-CACHE-REVALIDATION/。
