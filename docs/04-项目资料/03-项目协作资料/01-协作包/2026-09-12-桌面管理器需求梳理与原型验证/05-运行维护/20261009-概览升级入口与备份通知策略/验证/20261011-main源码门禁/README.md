# 2026-10-11 main 源码门禁

## 结论

0.1.10 的实现已经集成到 `main@3bcd9318eeba4dc0e69d41c9dfe77b4bdb0a27ea`，`origin/main` 已同步。本轮按当前 `main` 重跑并复核源码级门禁：Rust、UI、生产构建、Windows bundle identity smoke 和差异检查均通过。

这只闭合当前集成提交的源码门禁。它不能替代 macOS 原生 UI / 安装回归、Windows 普通与管理员双权限、实际安装目录和自定义数据根、DPI / 云母 / 原生标题栏 / 动效 / 透明下拉框、自动检查性能与真实周期、真实更新 / 重启 / 恢复、签名制品、全域事件跨平台执行证据或真实用户迁移。`release_authorized: false`。

## 当前证据

- Rust metadata、fmt、Clippy `-D warnings`：通过。
- Rust 全量：library `694 passed / 0 failed / 4 ignored`，binary `2 passed / 0 failed`，集成测试目标通过。
- UI：94 个测试文件、459 项通过；类型检查通过；生产构建通过，转换 170 个模块。
- Windows bundle identity smoke：5/5 通过。
- `git diff --check`：通过。

证据原件：`.adg/evidence/OCX-0110-20261011-MAIN-SOURCE-GATES/`。

## 制品与发布边界

此前 Windows 候选包的真实身份仍为 0.1.9，不能复用为 0.1.10 发布资产。下一步必须基于 `main@3bcd9318` 重新生成明确身份为 0.1.10 的 macOS / Windows 制品，再执行普通权限、安装路径、原生外观、更新重启、签名和发布门禁。源码通过不创建标签、不公开 Release、不替换日常 0.1.7，也不改变稳定通道。

## 非阻断提示

既有 Vite `INEFFECTIVE_DYNAMIC_IMPORT` 警告和 jsdom canvas notice 未阻断 UI 结果；测试夹具中的 0.1.9 输入保留，用于验证升级场景。
# 20261011 main 源码门禁

## 范围

本记录绑定 0.1.10 当前代码基线 main@3bcd9318eeba4dc0e69d41c9dfe77b4bdb0a27ea。代码工作树与 origin/main 指向同一提交；文档与证据归属 docs/governance-main。

## 结果

- Cargo metadata、fmt、Clippy -D warnings：通过。
- Rust workspace integration tests：通过；library 694 passed / 0 failed / 4 ignored，binary 2 passed / 0 failed。
- UI 全量：94 个测试文件、459 项通过；类型检查通过。
- 生产构建：通过，170 modules transformed。
- Windows bundle identity smoke：5 passed / 0 failed。
- git diff --check：通过。

原始命令、提交绑定和机器可读结果见 MAIN-SOURCE-GATES 证据目录下的 manifest.json。

## 解释与边界

本记录只确认 main 的源码、构建和版本身份门禁通过。feature 分支的历史记录不改写；此前 Windows 候选包真实身份仍为 0.1.9，不能直接作为 0.1.10 发布资产，必须从当前 main 重新生成明确身份为 0.1.10 的 macOS / Windows 制品。

以下门禁仍开放：

- macOS 原生 UI、隔离安装、升级保留、真实更新 / 重启 / 恢复。
- Windows 普通权限与管理员权限、实际安装目录、受保护路径、自定义数据根、升级保留、卸载边界，以及 DPI、云母、原生标题栏、动效和透明下拉框。
- 配置、备份、缓存、日志和托管 OPENCODEX_HOME 对活动数据根的统一遵循。
- 自动检查在启动、概览进入、定时、在线恢复、休眠恢复和跨启动场景的请求次数、缓存命中、CPU、RSS、长帧、子进程生命周期，以及 stable 24 小时 / beta 6 小时真实周期。
- 真实更新端点、下载、签名、公证 / Authenticode、更新后重启、失败恢复和真实用户迁移。
- 全域事件注册表的跨平台实际调用证据；注册表登记或局部源码测试不能替代执行验收。

因此 release_authorized: false、状态为 passed_with_scope_limits；不创建 v0.1.10 标签、不公开发布、不晋升 stable、不替换日常 0.1.7。
