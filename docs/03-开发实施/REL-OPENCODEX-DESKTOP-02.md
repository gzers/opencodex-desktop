---
id: REL-OPENCODEX-DESKTOP-02
object_kind: release.candidate
state: verified
title: 0.1.1 界面恢复与可观测修复发布记录
summary: 承接 MNT-OPENCODEX-DESKTOP-20261004-01 冻结范围，实施 F-01/03/04/05/06/07/09/13，通过本地门禁、真实制品冒烟与安装验证，并入 main、打 v0.1.1，CI 与 Release 流水线全绿。
source_refs:
  - MNT-OPENCODEX-DESKTOP-20261004-01
  - RUN-OPENCODEX-DESKTOP-20261004-01
  - e474645e
  - e56ec397
  - 22dbbba6
---

# REL-OPENCODEX-DESKTOP-02 0.1.1 发布记录

## 结论

0.1.1 是首个测试版（0.1.0）之后的维护版本，只收 MNT-OPENCODEX-DESKTOP-20261004-01 冻结的**已确认缺口**，提供界面恢复能力与可观测性。首次触发的原因仍未锁定，本版不宣称修复根因。

## 范围与实现

| 编号 | 实现 |
|---|---|
| F-01 | 托盘左键展开状态菜单（`showMenuOnLeftClick=true`），右键不变 |
| F-03 | 统一 `reveal_main_window`，Dock/托盘/应用菜单共用并记录 show/focus 结果 |
| F-06 | 应用菜单「视图 → 重载主界面」，直接重载主 WebView |
| F-04 | WEBGL 运行中上下文丢失回退 CSS、恢复重探测 |
| F-05 | 挂载前 window.error 记录 + 不依赖 Vue 的最小刷新入口 |
| F-07 | 托盘命令通道失败可见、恢复登记 |
| F-09 | `build.rs` 注入源码提交，启动事件写 version/commit |
| F-13 | 校正设置与退出文案 |

排除（未实施、未验收）：F-08 原生面板并发、F-10 更新通道、F-11/F-12 代理退出信号链与自动恢复运行。

## 门禁与证据

- 前端：`vue-tsc --noEmit`、`vite build` 通过；`vitest --run` 71 文件 **355 passed / 0 failed**。
- 后端：`cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings` 通过；`cargo test --workspace --features integration-test` **407 passed / 2 ignored / 0 failed**。
- 本机制品：`tauri build --target aarch64-apple-darwin` 产出 `OpenCodeX Desktop.app`（`CFBundleShortVersionString=0.1.1`）与 `OpenCodeX Desktop_0.1.1_aarch64.dmg`（3,866,776 bytes）。
- 真实制品冒烟：启动日志 `startup version=0.1.1 commit=22dbbba6...`；「视图」菜单含「重载主界面」；点击后日志 `reload main: requested`。
- 安装验证：0.1.0 安装实例备份到 `~/Library/Application Support/OpenCodeX Desktop backups/0.1.0-20261004105248`，0.1.1 安装到 `/Applications/OpenCodeX Desktop.app`（`Info.plist` 0.1.1，主二进制 SHA-256 `ef82a9880d999e76c5db9d9db54f24cfcbdef58098891374149404dbf3455576`）。
- CI/Release（GitHub Actions）：CI（main，`22dbbba6`）四 job（frontend/backend/backend-windows/build）全绿；Release（tag `v0.1.1`）两平台 job 成功，`tauri-action` 建草稿 + 预发布 Release 并上传制品。

## 发布产物与状态

- tag `v0.1.1` → `22dbbba6`（已推送 `origin`）。
- GitHub Release 由流水线创建为草稿 + 预发布（`releaseDraft: true` / `prerelease: true`），与 0.1.0 同口径。
- **已按用户 2026-10-04 明确授权公开发布**：新增 `.github/workflows/publish-release.yml`，由专用 tag `publish/v0.1.1` 触发；只切换可见性，不重建制品、不改说明正文。
- 第一次发布运行因 `gh release edit --draft=false --latest` 被 GitHub 拒绝（`Latest release cannot be draft or prerelease`，HTTP 422）而失败；改为先 `--draft=false --prerelease=false` 再 `--latest` 后，publish 工作流成功（run `37173494511`）。
- 公开发布结果：Release `v0.1.1` 现为 `draft=false`、`prerelease=false`，地址 <https://github.com/gzers/opencodex-desktop/releases/tag/v0.1.1>；四个制品 uploaded：macOS `.dmg`、Windows `_x64-setup.exe`（NSIS）、Windows `.msi`、`.app.tar.gz`。
- 发布诊断以公开可读的 `ci-logs` 分支留存（`publish-diag.txt`），本次记录 `tag=v0.1.1 / undraft: ok / latest: ok`。0.1.0 的 Release 未触碰，仍为草稿 + 预发布。
- 配套代码提交：`9df1248`（发布工作流）、`be362d8a`（安装核验脚本）、`aea0338`/`8b36414`/`dad00725`（发布诊断与本机核对）。

## 未完成（独立门禁）

- 生产代码签名与公证、Intel macOS、Windows 实机核验仍属单独门禁。
- 首次触发根因、代理退出信号链、重开后自动恢复运行未纳入本版。
