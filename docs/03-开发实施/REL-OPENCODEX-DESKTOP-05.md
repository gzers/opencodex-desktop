---
id: REL-OPENCODEX-DESKTOP-05
object_kind: release.candidate
state: candidate-passed
title: 0.1.4/0.1.5 更新检查修复记录（官方远端查询 PATH、检查结果与代理）
summary: 承接 IMP-OPENCODEX-DESKTOP-16 的检查更新反馈。0.1.4 修复官方 npm 查询缺 PATH；0.1.5 修复成功检查残留旧错误，两项均经 2026-10-05 定向审计验证。手动代理注入在 0.1.3 已存在；旧草稿诊断只核对 manifest 与签名 key ID，不等于制品验签。两版已并入 main、打 tag，原记录为草稿 Release；安装重试、发布失败传播与稳定通道投递仍未闭合，运行验收未通过。
source_refs:
  - IMP-OPENCODEX-DESKTOP-16
  - REL-OPENCODEX-DESKTOP-04
  - 7f73a4e0
  - 5b90fe85
  - cb0edcae
---

# REL-OPENCODEX-DESKTOP-05 0.1.4/0.1.5 更新检查修复记录

2026-10-04/05。承接 0.1.3 发布后用户实测的两个检查更新失败反馈，以及 `REL-*` 候选记录。本节只记录已执行事实，不把构建/测试成功写成运行验收通过。

2026-10-05 审计补正：详见 [OTA 修复有效性审计](2026-10-05-OTA修复有效性审计.md)。`candidate-passed` 仅保留原构建级门禁状态；不能据此认定真实 OTA 升级通过。现已复现安装失败后不能在当前页面重试、发布命令失败仍可能返回成功；稳定通道仍为 0.1.3。

后续实施：上述遗留问题已在本地 0.1.6 候选中修复，见 [OTA 修复实施与候选验收](2026-10-05-OTA修复实施与候选验收.md)。本页保留 0.1.4 / 0.1.5 的历史结论，不把后续代码修复写成旧版制品已改变。

## 结论

- **0.1.4**：修复官方版本卡片「远端最新版本」在隔离环境下失败——`official_remote_latest` 启动 `npm view` 子进程时 `env_clear()` 后未提供 `PATH`，而 npm 是 `#!/usr/bin/env node` 脚本，清空环境后找不到 node，spawn 直接失败。现注入 npm 自身目录 + 系统兜底组成的最小 `PATH`（与受控安装同策略）。已并入 `main`（`5b90fe85`）、打 tag `v0.1.4`、出草稿 Release。
- **0.1.5**：修复「检查应用更新」成功后仍显示失败——`check_for_update` 成功分支（有更新/已是最新）未清空 `UpdateStatus.error`，若此前失败过一次，界面固定优先显示 `error`，于是成功的检查仍显示旧的「更新下载或连接失败」并标记失败。现抽出 `record_check_success` 统一清空 `error` 并更新时间戳。代理三分支在 0.1.3 已存在，本版抽取 helper 并改变非法代理 URL 的回退，不是首次支持手动代理。已并入 `main`（`cb0edcae`）、打 tag `v0.1.5`、出草稿 Release。

## 诊断与实现

| 版本 | 症状 | 根因 | 修复 | 提交 |
|---|---|---|---|---|
| 0.1.4 | 官方版本卡片显示「远端查询不可用」 | `env_clear()` 后无 `PATH`，npm 找不到 node | 注入 npm 目录 + 系统兜底的最小 `PATH` | `7f73a4e0` |
| 0.1.5 | 配好代理后「检查应用更新」仍显示失败 | 成功分支未清 `error`，旧的网络失败提示盖住成功结果 | `record_check_success` 清空 `error` 并更新 `last_checked_at` | `cb0edcae` |
| 0.1.5 | 原记录误称此前未显式注入手动代理 | 0.1.3 原码已包含三分支和 `proxy(...)` | 本版仅抽取 helper，并使非法代理 URL 回退 `no_proxy()`；不构成原先所称的根因修复 | `cb0edcae` |

## 门禁与证据（构建/测试级）

- 后端：`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings` 通过；`cargo test --workspace --features integration-test` 全绿（lib **434 passed / 0 failed / 4 ignored**，含新增 `commands::update::tests::successful_check_clears_stale_failure_and_records_result`）。
- 前端：`vue-tsc --noEmit` 通过；`vitest --run` **360 passed / 0 failed**；`vite build` 通过。
- 本机制品：`tauri build --target aarch64-apple-darwin` 产出 `OpenCodeX Desktop.app` 与 `OpenCodeX.Desktop_0.1.5_aarch64.dmg`。
- CI：`main` 上 `cb0edcae` 的 **CI**（run 37214564016）与 **Release**（run 37214791225，macOS + Windows 两 job）均 success。

## 发布事实

- 分支：`feature/0.1.4-update-check-fixes`（`7f73a4e0`/`5b90fe85`）、`feature/0.1.5-update-check-fixes`（`cb0edcae`）已快进并入 `main`。
- tag：`v0.1.4` → `5b90fe85`；`v0.1.5` → `cb0edcae`。
- 草稿 Release：v0.1.4（id 403061310）、v0.1.5（草稿 + 预发布），资产 8 个（`latest.json`、macOS `dmg`/`app.tar.gz` + `.sig`、Windows `exe`/`msi` + `.sig`）。
- 签名元数据核对（经一次性诊断工作流读取草稿 `latest.json`，未据此执行制品密码学验签）：v0.1.5 `version=0.1.5`，平台键 `darwin-aarch64, darwin-aarch64-app, windows-x86_64, windows-x86_64-msi, windows-x86_64-nsis`；各平台签名 payload `keyid=6753bc16fc759b10`，与 `tauri.conf.json` 编译期公钥 `keyid` 一致（该公钥文本显示 `109B75FC16BC5367`）。
- 一次性诊断分支/workflow（`ci/verify015`/`015b`/`015c`/`015d`）已本地 + 远端删除；`ci-logs` 仅保留最近一次诊断文本。

## 未完成 / 另行确认

- **公开发布未执行**：原记录中 v0.1.4、v0.1.5 为草稿；2026-10-05 匿名核查仍不可见，稳定通道仍为 v0.1.3。`publish/vX.Y.Z` 只切可见性、不重建制品，且保留此前 Latest，不能单靠取消草稿完成新版 OTA 投递；发布及稳定通道晋升另行确认。
- **运行验收未做**：原记录为构建/测试门禁、制品产出及签名元数据核对。本轮补充了隔离网络、真实 0.1.3 包验签与临时 macOS 包替换测试，仍未完成已安装旧版到 0.1.5 的实际升级与重启验收。
