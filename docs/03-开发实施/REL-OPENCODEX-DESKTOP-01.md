---
id: REL-OPENCODEX-DESKTOP-01
object_kind: release.candidate
state: verified
title: macOS aarch64 发布候选验证
summary: 复核 aarch64 Release app bundle、自动化与实机候选证据；生产签名/公证、R-17 与公开发布保持单独门禁。
source_refs:
  - TASK-OPENCODEX-DESKTOP-56
  - TASK-OPENCODEX-DESKTOP-57
  - TASK-OPENCODEX-DESKTOP-58
  - PERF-OPENCODEX-DESKTOP-01
  - cbaedb8dd8c66c081ef6412aa70cc8cb46cc7a55
---

# REL-OPENCODEX-DESKTOP-01 macOS aarch64 发布候选验证

## 候选结论

- 候选代码：`cbaedb8 perf(基准): TASK-57 采集发布候选性能证据`
- 制品：`src-tauri/target/aarch64-apple-darwin/release/bundle/macos/OpenCodeX Desktop.app`
- 尺寸：7.0 MiB / 6,303,744 bytes
- 性能：启动到窗口 687ms；运行态 RSS 113.3MiB
- 自动化：Rust 230 单测与集成合计 291 项；前端 26 文件 91 项；`vue-tsc`、`vite build` 通过。

## 候选限制与发布门禁

1. 该制品为本地未签名、未公证候选，不进入生产发布通道。
2. 生产面板承载方式仍受 R-17 发布前用户与安全评审冻结门禁。
3. Intel macOS、Windows x64 与生产签名公证未完成发布验证。
4. 候选记录只支持内部验收，不构成公开发布决定。

## 后续

公开发布需单独用户确认，并补齐发布签名、公证、安装/升级回退与平台矩阵验证。

## 追加（2026-10-03）：分支收敛、首个测试版与发布流水线

### 1. 分支约定（用户确认）

- `main` = **代码分支**：只放 `apps/`、`.github/`、`test/` 与仓库根配置。
- `docs/governance-main` = **文档与治理产物**：`docs/`、`.adg/`。
- 已在本机把应用代码快照合并到 `main`（单条提交 `8a5fe546`，历史不做整理），
  `main` 树为 `.github/ .gitignore AGENTS.md LICENSE README.en.md README.md apps test`，
  不含 `docs/`、`.adg/`。

> 推送：本机没有可用的 GitHub 凭据（HTTPS 无凭据、SSH 对该仓库无权限），
> 需由本机执行 `git push origin main && git push origin v0.1.0`。

### 2. 首个测试版

- 版本号维持 `0.1.0`（`tauri.conf.json` 与 `Cargo.toml` 一致），tag 打 `v0.1.0`（本机已建，指向 `main`）。
- GitHub Release 由流水线创建为**草稿 + 预发布**（`releaseDraft: true` / `prerelease: true`），
  确认无误后再手动取消草稿。

### 3. 发布流水线（`.github/workflows/release.yml`）

- 触发：推送 `v*` tag，或手动 `Run workflow` 填 tag。
- 矩阵：`macos-14`（`aarch64-apple-darwin`）→ `tauri-apps/tauri-action@v0` 出 `.app` + `.dmg`
  并创建/更新 Release；Intel macOS 条目已写好、按需取消注释。
- 权限：`contents: write`（用仓库自带 `secrets.GITHUB_TOKEN`，无需额外密钥）。

### 4. Windows：流水线已就位，代码侧还有前置改造

`bundle.targets` 已由 `["app","dmg"]` 改为 `"all"`（macOS 仍只出 app/dmg，Windows 可出 nsis/msi）。
但 **Windows 目前编译不过**，阻塞点都在 `apps/desktop/tauri`：

| # | 阻塞点 | 现状 | 需要的改造 |
| --- | --- | --- | --- |
| 1 | `infrastructure/keychain.rs` | 直接 `use security_framework::…`（macOS 钥匙串，仅 Apple 平台可编译） | 抽「密钥存储」trait，macOS 走 security-framework，Windows 走凭据管理器（wincred / Credential Manager），Linux 走 Secret Service 或降级 |
| 2 | `Cargo.toml` | `security-framework = "3"` 与 `reqwest` 的 `macos-system-configuration` 无条件启用 | 按 `cfg(target_os)` 收口到 `[target.'cfg(target_os = "macos")'.dependencies]` |
| 3 | 打包目标 | 已改 `"all"` | Windows 侧补 `nsis` 安装器图标/语言（可选） |
| 4 | 窗口与外壳 | `titleBarStyle: "Overlay"`、自绘 28px 标题栏 + `data-tauri-drag-region` | Tauri v2 支持 Windows overlay，但需实机核对红黄绿按钮位置与拖拽区；托盘 `iconAsTemplate` 仅 macOS 生效（无害） |
| 5 | 视觉 | 字体栈含 `-apple-system` / `PingFang SC`；`-webkit-backdrop-filter` | Windows `WebView2`（Chromium）支持 `backdrop-filter`，只需补字体回退（Segoe UI / 微软雅黑）并复核树影/玻璃观感 |

**建议顺序**：先做 #1/#2（让 Windows 能编译），再开矩阵里的 windows 条目出未签名 nsis 测试包，
最后单独处理签名（Windows 代码签名证书）与实机核对（#4/#5）。

### 5. 门禁

- 产出前复跑：`vue-tsc`、`vitest`、`vite build`、`cargo test`、`qa/audit.browser.mjs` 全绿；
- 实机（WKWebView）复核通过；
- 证书/签名/公证、Windows 实机、安装升级回退仍属**单独门禁**，未完成前不构成公开发布决定。

## 追加（2026-10-03）：版本说明与 Release 文案约定

- **事实源**：`main` 仓库根的 `CHANGELOG.md`（Keep a Changelog 风格）。一个版本一个
  `## [X.Y.Z] - 日期` 段落，标题里的版本号**不带 `v`**。
- **流水线取用**：`.github/workflows/release.yml` 按 tag（`vX.Y.Z` → `X.Y.Z`）从 `CHANGELOG.md`
  抽取同名段落作为 GitHub Release body；抽不到对应段落即让 job 失败，避免发出没有说明的包。
  `tauri-action` 没有 `releaseBodyPath` 入参，所以用一步读入 `$GITHUB_OUTPUT` 再传给 `releaseBody`。
- **分工**：`CHANGELOG.md` = 给用户看的版本说明事实源（在 `main`）；本文件与其它 `REL-*` / `IMP-*`
  = 给治理看的门禁与验证长文（在 `docs`）。同一段版本说明不两处维护，changelog 里链接到本目录。
- **暂不自动化**：当前提交信息前缀不统一，`git-cliff` / `release-please` 生成质量差；提交规范稳定后再引入
  `git-cliff`，届时 `CHANGELOG.md` 仍是事实源。

### Windows 阻塞点补充（复核代码后）

上文第 4 节只列了 keychain 与 Cargo cfg；复核**非测试**代码后，Windows 编译还受一批 Unix 专属调用阻塞：

| 文件 | 阻塞点 |
| --- | --- |
| `src/infrastructure/atomic_write.rs` | `std::os::unix::fs::OpenOptionsExt` / `PermissionsExt`（`.mode()`） |
| `src/infrastructure/locking.rs` | `std::os::unix::io::AsRawFd`（flock） |
| `src/infrastructure/runtime_log.rs` | unix `.mode(0o600)` |
| `src/modules/extensions/projection.rs` | unix 权限位 + `std::os::unix::fs::symlink` |
| `src/modules/data_root/mod.rs` | `set_private_directory` 使用 unix 权限 |
| `src/bin/ocxd.rs` | `std::os::unix::net::UnixStream`（`ocxd` CLI） |

`commands/panel.rs`、`commands/workspace.rs`、`lib.rs` 已做平台 cfg，不阻塞。
因此 Windows 打通不是「补两个 cfg」，而是：凭据存储抽象 + 文件锁/权限/日志/符号链接的平台分支
（Windows 权限位退化为 no-op、文件锁换 `LockFileEx`、symlink 退化复制/junction、`ocxd` 先 Gate 到 unix）。

## 追加（2026-10-03）：Windows 跨平台改造完成

上文第 4 节与「Windows 阻塞点补充」列出的阻塞点已全部处理；`main` 侧改动如下（逐阶段提交）：

- `346f3c16` 凭据存储改用 `keyring`（macOS `apple-native` / Windows `windows-native`），
  `security-framework` 与 `reqwest` 的 `macos-system-configuration` 收口到 `cfg(target_os = "macos")`。
- `03b064a8` 新增 `infrastructure::platform`（权限位、OpenOptions 创建模式、符号链接、权限复制）；
  文件锁改用 `std::fs::File` 的 `lock/try_lock/unlock`（Rust **1.89** 起稳定，MSRV 由 1.85 提到 1.89，
  不再依赖 `libc::flock`）；`ocxd`、`ipc::endpoint` 等 Unix socket 代码在本机 IPC 实现 Windows 化之前
  先 Gate 到 unix，Windows 侧提供明确报错的 `main`。
- `5ae44855` CI 增加 `backend-windows`（`windows-latest` + `cargo check --workspace --all-targets`）；
  release 矩阵放开 `windows-latest` 条目（未签名 nsis/msi）。

### 验证证据

- **本地 Windows 交叉编译门禁**：`cargo check --target x86_64-pc-windows-msvc --all-targets` 通过。
  本机无 MSVC，故用一次性 cc/ar shim（放在 `/tmp`，不入库、不改主机 PATH）绕开原生 C 依赖
  （`ring` 等）的构建脚本，从而拿到真实的 Rust 层错误；shim 不参与链接，因此只证明**可编译**，
  不证明可链接/可打包。
- **macOS 门禁**：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings` 通过；
  `cargo test --workspace --features integration-test` 406 passed / 1 failed / 2 ignored。
- 唯一失败 `commands::workspace::tests::local_documents_are_frozen` 是**分支拆分带来的环境性失败**：
  该用例断言仓库根下存在 `docs/04-项目资料/.../LICENSE`，而 `main` 分支不含 `docs/`
  （该文件在 `docs` 分支）。用例代码本身未被本次改动触碰，属既有问题。
- Windows 目标上，依赖 Unix 语义（chmod 权限位、`#!/bin/sh` fixture）的测试模块已用
  `#[cfg(all(test, unix))]` / `#![cfg(unix)]` Gate 掉，因此 `--all-targets` 可编译。

### 仍未完成（属单独门禁，未通过前不构成 Windows 可发布）

1. **未产出 Windows 安装包**：矩阵已就绪，但需要一次真实 CI 运行（推 tag 或 `workflow_dispatch`）
   才能验证 nsis/msi 实际产出；本机无法产出 Windows 安装器（缺 MSVC/WiX）。
2. **Windows 运行期测试覆盖**：目前只保证编译，Unix 专属用例未在 Windows 上等价重写。
3. **Windows 实机核验**：安装/启动、WebView2 渲染、overlay 标题栏与拖拽区、托盘、
   字体回退（`Segoe UI` / 微软雅黑）与玻璃观感均未核对。
4. **Windows 代码签名**与**自动更新**（`tauri.conf.json` 的 `updater.endpoints` 仍是占位）未接通。

### 本地复现 Windows 编译门禁（macOS，无 MSVC）

```bash
rustup target add x86_64-pc-windows-msvc
cd apps/desktop/tauri
CC_x86_64_pc_windows_msvc=<shim>/cc AR_x86_64_pc_windows_msvc=<shim>/ar \
  cargo check --workspace --all-targets --target x86_64-pc-windows-msvc
```

`<shim>` 是一个临时目录里的 `cc`/`ar` 脚本，只把 `-o` / `/OUT:` 指定的产物建成空文件并退出 0。
原因：`ring` 等原生 C 依赖在 macOS 上无法为 MSVC 目标编译构建脚本，shim 让它们「通过」以便暴露
Rust 层错误。**shim 不参与链接，因此只证明可编译，不证明可链接/可打包**；权威门禁是 CI 的
`windows-latest` 任务。该 shim 属一次性本地手段，不入库。

### 回归核对

- 上文唯一的 macOS 测试失败已确认为环境性：在 worktree 内补齐 `docs/04-项目资料/04-品牌与图标素材/01-原始素材/LICENSE`
  后，`cargo test --lib local_documents_are_frozen` 通过；失败原因是 `main` 分支不含 `docs/`，与本次改造无关。

### 备注

- `apps/desktop/tauri/.gitignore` 忽略 `Cargo.lock`，依赖版本未固定，`gen/schemas/*` 会随解析漂移
  （每次本地构建都会让这几个已跟踪文件出现改动）。是否固定 `Cargo.lock` 属独立决策。
