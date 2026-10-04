---
id: RUN-OPENCODEX-DESKTOP-20261004-01
title: 0.1.0 macOS 白屏与托盘交互事故观察
source_refs:
  - MNT-OPENCODEX-DESKTOP-20261004-01
  - REL-OPENCODEX-DESKTOP-01
  - CODEX-THREAD:01a1047b-e13d-7241-acc1-279626511d94
---

# RUN-OPENCODEX-DESKTOP-20261004-01 事故观察

## 状态与范围

登记日：2026-10-04；时间均为 Asia/Shanghai（UTC+08:00），特别标注 UTC 的日志除外。受影响管理器版本为 **0.1.0**。用户完全退出后重启，确认窗口恢复；**首次触发原因未锁定，修复与回归未完成，事故未闭合**。拟修复版本备注为 **0.1.1**，尚未实施或发布。

本记录拥有跨时间运行事实；问题解释、源码核对与候选处置见 [MNT-OPENCODEX-DESKTOP-20261004-01](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261004-白屏与托盘左键菜单/README.md)，不把分析假设当执行结果。记录的是这次本机事故，不代表全部 0.1.0 安装都能复现。旧协作包继续保持 open，本次用户单独授权承接维护附件。

## 环境与身份

| 项目 | 核对值 |
|---|---|
| 安装应用 | /Applications/OpenCodeX Desktop.app |
| Bundle ID / 管理器版本 | com.gzers.opencodex.desktop / 0.1.0 |
| 平台 | macOS 27.0.1 (26A434)，ARM64 |
| 主二进制 | 6,771,472 bytes；mtime 2026-10-03 21:29:19 +0800 |
| 二进制 SHA-256 | 8d5ba8c0b000de5847f2f6e9792e29691ba3008f69497d8eb04bc1db36357239 |
| 故障进程 | desktop 65237；GPU 65239；主 WebContent 65240 / page 8；Networking 65241；嵌入面板 WebContent 65274 / page 38 |
| 故障进程启动 | 2026-10-03 22:55:51.929 +0800 |
| 托管官方运行时 | 观察为 OpenCodex v2.69.0、运行中；与管理器版本分别记录 |
| 非敏感偏好摘录 | keep_proxy_on_close=true；visual_effects=high；glow_render=mesh；面板嵌入模式 |

源码核对基线为 main@9ca7a9458d951ba313e8ea46c51c99cc14f8cd44，v0.1.0 指向 ed35cba89fbdf2e440ff788f6015dc4c2bb82c40；本次相关核对文件在两者间无差异。**尚无安装二进制与该提交及依赖解析一一对应的构建证据**。首个测试版版本背景引用 [REL-OPENCODEX-DESKTOP-01](REL-OPENCODEX-DESKTOP-01.md)，不追加发布结论。

## 观察与时间线

| 时间 / 顺序 | 实际观察 | 证据与限制 |
|---|---|---|
| 2026-10-04 上午，首次报障时间未精确留存 | 用户报告打开白屏，重新打开仍白；顶部托盘菜单点不出来 | 用户反馈；未还原首次触发动作或前夜窗口状态 |
| 09:15:40–09:15:48 过滤日志窗口 | page 8 多次图层 volatile 操作失败；09:15:44.434 GPU 65239 出现 mach message error 10000003；page 38 于 09:15:47.673 报图层操作超时 | [WebKit 图层日志](../../.adg/evidence/MNT-OPENCODEX-DESKTOP-20261004-01/webkit-layers.log)；异常同时出现不证明因果 |
| 09:18:06.252 开始短时采样 | 主线程主要在正常 AppKit 事件循环，间有 WebKit IPC / evaluateJavaScript；RSS 50.0M，峰值 51.1M | [原生线程采样摘录](../../.adg/evidence/MNT-OPENCODEX-DESKTOP-20261004-01/native-thread-sample-excerpt.txt)；未观察主线程死锁，不排除采样之外的间歇异常 |
| 故障窗口交互，精确点击时间未留存 | 白屏时 AX 可读完整设置升级页；原生“视图 → 打开主界面”后 AX 变为完整概览，实窗仍白 | 调查会话的 UI 观察；截图未持久留存，用户确认作为交叉依据 |
| 09:19:00–09:19:08 过滤日志窗口 | 主 page 8 连续有脚本执行成功日志 | [WebKit 脚本请求日志](../../.adg/evidence/MNT-OPENCODEX-DESKTOP-20261004-01/webkit-javascript.log)；证明请求成功，不保证每个应用逻辑或每帧绘制正确 |
| 上述切换后 | 用户确认“仍白屏；右键能弹菜单” | [用户反馈与登记授权](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261004-白屏与托盘左键菜单/问题来源记录/用户反馈与登记授权.md)；左键和右键不能混写为全体托盘不可用 |
| 用户自行退出、重开后 | 用户确认“是的右键完全退出是再打开是可以的” | 恢复操作人为执行；本轮文档任务没有再次启停应用 |
| 恢复后只读核对 | desktop PID 51714，启动 2026-10-04 09:28:06；app.log 有 2026-10-04T01:28:48.120Z start confirmed（本地 09:28:48.120） | 进程已更换；视觉恢复由用户确认。没有完成复现或长时回归 |
| 2026-10-04 后续反馈，精确反馈时刻未留存 | 用户报告重开管理器后 OpenCodex 关闭，并明确“代理已停止，需要点启动” | 用户确认是后台状态停止，不只是面板关闭；未采集停止瞬间健康、退出码或信号，不构造另一次重启时间 |
| 补充只读核对 | 当前 desktop 51714 → Node 51882 → Bun 51884；后两者启动 09:28:48；keep_proxy_on_close=true；安装版本仍 0.1.0 | [生命周期观察](../../.adg/evidence/MNT-OPENCODEX-DESKTOP-20261004-01/lifecycle-observations.json)；代理晚于管理器约 42 秒启动，和用户手动恢复反馈相符，PID 父子关系不证明终止机制 |
| 09:27:30–09:28:49 补采历史日志窗口 | launchd 于 09:28:01.199 记录旧应用 service inactive，随后移除旧服务 / pid 65237；09:28:06.468 有重开相关记录 | [系统退出日志](../../.adg/evidence/MNT-OPENCODEX-DESKTOP-20261004-01/lifecycle-launchd.log)；未包含代理实际信号或退出码 |

## 影响与恢复边界

可确认影响为管理器界面不可见、无法通过可视页面正常操作；右键原生托盘菜单可用。调查中页面路由能改变，官方运行时观察为运行中。未采集到本次管理器崩溃报告；已看到的 9 月 27 日、29 日报告不能充当本次崩溃证据。没有据此证明所有数据、同步、后台运行均不受影响，也没有证明完整退出前后托管进程连续。

本次用户选择“完全退出 → 重开”后恢复，仅作为已发生的临时处置事实。窗口关闭路径在 keep_proxy_on_close=true 时可能只是隐藏，入口重开复用原窗口，因此“重开窗口”和“重启进程”的恢复范围不同。不能将此推断为隐藏动作必定造成第一次白屏，或将重启成功标成 0.1.1 修复通过。

## 证据保存与后续

2026-10-04 补充恢复边界：此前“未证明完整退出前后托管进程连续”的限制保留；现有用户反馈进一步确认此次重开后代理已停止、需手动启动。白屏恢复不能等同整体服务恢复，准确代理终止机制仍未锁定。新增分析见 [生命周期与版本归属](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261004-白屏与托盘左键菜单/分析/04-重启管理器后代理停止与版本归属.md)，建议纳入未实施的 0.1.1；0.1.2 未排期。没有再次操作应用启停或执行更新重启验证。

机器证据入口为 [机器证据清单](../../.adg/evidence/MNT-OPENCODEX-DESKTOP-20261004-01/manifest.json) 与 [非敏感观察摘录](../../.adg/evidence/MNT-OPENCODEX-DESKTOP-20261004-01/observations.json)。仅保存本次过滤日志、采样前 90 行和非敏感身份摘录；不保存配置全文、凭据、完整进程参数。原始完整采样仍在临时目录，原文件 SHA-256 单独记录，长期复核以保留摘录为限。

当前最匹配“进程存活、页面和脚本响应、实窗不绘制”的异常；WebKit 图层/合成状态是重点假设，具体触发点仍待控制变量复现。后续验证矩阵见 [处置与验证计划](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261004-白屏与托盘左键菜单/分析/02-处置与验证计划.md)；需补首次触发、渲染配置对照、激活调用结果、原生子视图可见性与边界、制品构建来源，并在真正执行后追加本 RUN 的观察，不覆盖本次证据。

## 追加（2026-10-04）：0.1.1 已确认缺口修复与发布执行事实

以上是 0.1.0 事故的原始观察，不改写。本节按维护范围冻结记录 [MNT-OPENCODEX-DESKTOP-20261004-01 冻结计划](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261004-白屏与托盘左键菜单/分析/05-0.1.1-已确认缺口冻结与实施计划.md)，追加真正执行过的 0.1.1 事实。

### 代码与版本

- 特性分支 `hotfix/0.1.1-ui-recovery` 从 `main`（当时 `9ca7a945`）新建；修复提交 `e474645e`，合并提交 `e56ec397`，版本同步提交 `22dbbba6`。
- 版本号 `0.1.0` → `0.1.1`：`tauri.conf.json`、`Cargo.toml`、`ui/package.json`、`ui/package-lock.json` 一致；`CHANGELOG.md` 新增 `[0.1.1] - 2026-10-04` 段。
- 提交 `22dbbba6` 已推送 `origin/main`；tag `v0.1.1` 指向 `22dbbba6` 并已推送。

### 实际实施项（对应冻结范围）

| 编号 | 实施事实 |
|---|---|
| F-01 | `tauri.conf.json` 的 `showMenuOnLeftClick` 由 `false` 改为 `true`；新增 `apps/desktop/tauri/tests/tray_left_click.rs` 断言配置，右键行为未改 |
| F-03 | 新增统一入口 `reveal_main_window`，Dock reopen / 托盘 / 应用菜单共用，记录 `show/focus` 成败；`apply_dock_visibility` 泛型化 |
| F-06 | 应用菜单「视图 → 重载主界面」(`menu-view-reload`) 直接调用主 WebView `reload()`，不进前端请求队列；`TrayAction::ReloadMain` 与托盘动作域分离 |
| F-04 | `glowRender.installGlowRenderContextGuard` 监听 `webglcontextlost/restored`，丢失即回退 CSS 并由同一 `mode` 派生 |
| F-05 | `installEarlyFaultCapture`（挂载前 window.error）与 `mountMinimalFaultNotice`（不依赖 Vue 的刷新入口），`main.ts` 在业务代码前装配 |
| F-07 | `useAppController` 托盘排空失败不再静默，首次可见提示、恢复后登记事件；新增 `tray-bridge-failure.test.ts` |
| F-09 | `build.rs` 注入 `OPENCODEX_BUILD_COMMIT`；启动事件写 `startup version=... commit=...` |
| F-13 | 校正「启动时打开主界面」「关闭窗口后保持代理运行」设置说明与托盘「完全退出桌面壳」文案 |

### 门禁与真实制品回归

- 前端：`vue-tsc --noEmit` 通过；`vitest --run` 71 文件 **355 passed / 0 failed**；`vite build` 通过。
- 后端：`cargo fmt --check` 通过；`cargo clippy --workspace --all-targets -- -D warnings` 通过；`cargo test --workspace --features integration-test` **407 passed / 2 ignored / 0 failed**。
- 本机 aarch64 制品：`tauri build --target aarch64-apple-darwin` 产出 `OpenCodeX Desktop.app`（`CFBundleShortVersionString=0.1.1`）与 `OpenCodeX Desktop_0.1.1_aarch64.dmg`（3,866,776 bytes）。
- 真实安装验证：把 0.1.0 安装实例移入 `~/Library/Application Support/OpenCodeX Desktop backups/0.1.0-20261004105248` 备份，再安装 0.1.1 到 `/Applications/OpenCodeX Desktop.app`；`Info.plist` 为 0.1.1，主二进制 SHA-256 `ef82a9880d999e76c5db9d9db54f24cfcbdef58098891374149404dbf3455576`。
- 制品冒烟（隔离 HOME，`test/smoke/mnt011-ui-recovery-smoke.sh`）：启动日志 `startup version=0.1.1 commit=22dbbba6...`；「视图」菜单项为「打开主界面, 重载主界面, 打开扩展面板, 快速设置」；点击后日志 `reload main: requested`，窗口仍可见。

### 公开发布（2026-10-04 用户授权）

- 新增 `.github/workflows/publish-release.yml`，由 tag `publish/v0.1.1` 触发，仅把 Release 草稿/预发布切换为公开，不重建制品。
- 首次运行因 GitHub 拒绝 `--draft=false --latest`（HTTP 422：latest 不能是草稿/预发布）失败；改为先清 prerelease 再标 latest 后成功（run `37173494511`）。
- 结果：`v0.1.1` Release `draft=false / prerelease=false`，制品 4 个已上传；0.1.0 的 Release 未动。诊断留存于公开 `ci-logs` 分支的 `publish-diag.txt`。

### 仍未闭合

首次触发的原因仍未锁定，本节记录的是恢复能力与可观测性，不是根因修复。F-08（原生面板并发）、F-10（更新通道）、F-11/F-12（代理退出信号链与自动恢复运行）按冻结计划排除于 0.1.1，未实施也未验收。原始 0.1.0 观察保持原样，0.1.1 不替换其证据。
