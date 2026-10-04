---
id: REL-OPENCODEX-DESKTOP-05
object_kind: release.candidate
state: candidate-passed
title: 0.1.4/0.1.5 更新检查修复记录（官方远端查询 PATH、检查结果与代理）
summary: 承接 IMP-OPENCODEX-DESKTOP-16 的检查更新反馈。0.1.4 修复官方版本卡片远端查询在隔离环境下 npm 找不到 node（env_clear 后未补 PATH）；0.1.5 修复「检查应用更新」成功后仍显示上一次失败（成功分支未清空 status.error）并把自更新代理注入改为按偏好显式三分支。两者已并入 main、打 tag 并出草稿 Release（v0.1.4 id 403061310、v0.1.5 草稿），签名与 latest.json 校验通过；公开发布（取消草稿）另行确认。
source_refs:
  - IMP-OPENCODEX-DESKTOP-16
  - REL-OPENCODEX-DESKTOP-04
  - 7f73a4e0
  - 5b90fe85
  - cb0edcae
---

# REL-OPENCODEX-DESKTOP-05 0.1.4/0.1.5 更新检查修复记录

2026-10-04/05。承接 0.1.3 发布后用户实测的两个检查更新失败反馈，以及 `REL-*` 候选记录。本节只记录已执行事实，不把构建/测试成功写成运行验收通过。

## 结论

- **0.1.4**：修复官方版本卡片「远端最新版本」在隔离环境下失败——`official_remote_latest` 启动 `npm view` 子进程时 `env_clear()` 后未提供 `PATH`，而 npm 是 `#!/usr/bin/env node` 脚本，清空环境后找不到 node，spawn 直接失败。现注入 npm 自身目录 + 系统兜底组成的最小 `PATH`（与受控安装同策略）。已并入 `main`（`5b90fe85`）、打 tag `v0.1.4`、出草稿 Release。
- **0.1.5**：修复「检查应用更新」成功后仍显示失败——`check_for_update` 成功分支（有更新/已是最新）未清空 `UpdateStatus.error`，若此前失败过一次，界面固定优先显示 `error`，于是成功的检查仍显示旧的「更新下载或连接失败」并标记失败。现抽出 `record_check_success` 统一清空 `error` 并更新时间戳；同时把自更新代理注入改为按偏好显式三分支（无代理 `no_proxy` / 手动 `proxy` / 自动交给系统探测）。已并入 `main`（`cb0edcae`）、打 tag `v0.1.5`、出草稿 Release。

## 诊断与实现

| 版本 | 症状 | 根因 | 修复 | 提交 |
|---|---|---|---|---|
| 0.1.4 | 官方版本卡片显示「远端查询不可用」 | `env_clear()` 后无 `PATH`，npm 找不到 node | 注入 npm 目录 + 系统兜底的最小 `PATH` | `7f73a4e0` |
| 0.1.5 | 配好代理后「检查应用更新」仍显示失败 | 成功分支未清 `error`，旧的网络失败提示盖住成功结果 | `record_check_success` 清空 `error` 并更新 `last_checked_at` | `cb0edcae` |
| 0.1.5 | 自更新代理注入隐式、行为与界面选项不对应 | 仅「无代理」显式 `no_proxy`，手动未显式注入 | 显式三分支：无代理/手动/自动 | `cb0edcae` |

## 门禁与证据（构建/测试级）

- 后端：`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings` 通过；`cargo test --workspace --features integration-test` 全绿（lib **434 passed / 0 failed / 4 ignored**，含新增 `commands::update::tests::successful_check_clears_stale_failure_and_records_result`）。
- 前端：`vue-tsc --noEmit` 通过；`vitest --run` **360 passed / 0 failed**；`vite build` 通过。
- 本机制品：`tauri build --target aarch64-apple-darwin` 产出 `OpenCodeX Desktop.app` 与 `OpenCodeX.Desktop_0.1.5_aarch64.dmg`。
- CI：`main` 上 `cb0edcae` 的 **CI**（run 37214564016）与 **Release**（run 37214791225，macOS + Windows 两 job）均 success。

## 发布事实

- 分支：`feature/0.1.4-update-check-fixes`（`7f73a4e0`/`5b90fe85`）、`feature/0.1.5-update-check-fixes`（`cb0edcae`）已快进并入 `main`。
- tag：`v0.1.4` → `5b90fe85`；`v0.1.5` → `cb0edcae`。
- 草稿 Release：v0.1.4（id 403061310）、v0.1.5（草稿 + 预发布），资产 8 个（`latest.json`、macOS `dmg`/`app.tar.gz` + `.sig`、Windows `exe`/`msi` + `.sig`）。
- 签名校验（经一次性诊断工作流读取草稿 `latest.json`）：v0.1.5 `version=0.1.5`，平台键 `darwin-aarch64, darwin-aarch64-app, windows-x86_64, windows-x86_64-msi, windows-x86_64-nsis`；各平台签名 payload `keyid=6753bc16fc759b10`，与 `tauri.conf.json` 编译期公钥 `keyid` 一致（该公钥文本显示 `109B75FC16BC5367`）。
- 一次性诊断分支/workflow（`ci/verify015`/`015b`/`015c`/`015d`）已本地 + 远端删除；`ci-logs` 仅保留最近一次诊断文本。

## 未完成 / 另行确认

- **公开发布未执行**：v0.1.3 已公开并是 `Latest`；v0.1.4、v0.1.5 仍为草稿，对匿名访问不可见，稳定通道 `latest.json` 仍解析到 v0.1.3。取消草稿需用 `publish/vX.Y.Z` 流程（只切可见性、不重建制品），另行确认。
- **运行验收未做**：以上为构建/测试门禁 + 制品与签名校验，不等于真机自更新/代理链路的运行验收通过。

