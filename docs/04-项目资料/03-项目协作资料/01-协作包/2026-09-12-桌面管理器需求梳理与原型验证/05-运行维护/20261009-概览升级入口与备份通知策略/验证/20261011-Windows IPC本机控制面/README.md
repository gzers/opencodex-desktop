# 2026-10-11 Windows IPC 本机控制面

状态：源码局部通过，真实 Windows 门禁待验。

## 基线

- 代码分支：main
- 代码提交：861fdeea39d1c45fa2beddb2cba6659b67735d9f
- 文档分支：docs/governance-main
- 目标版本：0.1.10
- 代码提交已推送：origin/main 已同步。

## 已实现

Windows 本机控制面使用 Tokio named pipe，管道名称由 OpenCodeXDesktop 和当前用户名的稳定摘要组成；服务端拒绝远程客户端，并沿用创建者默认 DACL。Unix 继续使用私有 Unix socket 和既有 UID / socket 权限保护。

Windows 与 Unix 共用同一套 8 字节大端长度前缀和 JSON frame 协议。ocxd CLI、IPC 服务注册和 Tokio net feature 已跨平台接通；业务命令语义没有另行复制一套 Windows 实现。

## 源码验证

- cargo fmt --check：通过
- cargo test --lib：694 passed / 0 failed / 4 ignored
- cargo test --bin ocxd：2 passed / 0 failed
- cargo clippy --all-targets --all-features -- -D warnings：通过
- git diff --check：通过

## 未闭合边界

当前 macOS 宿主没有 Windows MSVC SDK/C 头文件，cargo check --target x86_64-pc-windows-msvc --all-targets 在 ring 找不到 assert.h 处失败。因此这次不能记为 Windows 编译通过，也不能替代 Windows 真机验证。

仍需在真实 Windows 设备验证普通权限和管理员权限下的 IPC smoke、单实例、连接限制、实际安装目录数据根、DPI、云母、原生标题栏、动效、透明下拉框、安装器、签名与真实更新。0.1.10 双平台制品需要基于当前 main 重新生成，旧的 0.1.9 候选不能复用。

证据原件：.adg/evidence/OCX-0110-20261011-WINDOWS-IPC-TARGET/。

## 发布状态

release_authorized: false。本回执只闭合 IPC 源码局部门禁，不关闭 IMP / TASK，不创建发布标签，不公开发布，不晋升 stable，也不替换日常 0.1.7。
