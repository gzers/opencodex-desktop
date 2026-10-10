# 0.1.10 目标执行检查点

日期：2026-10-11  版本：0.1.10  状态：in_progress

本记录确认 0.1.10 已进入目标模式执行。目标是按 IMP-OPENCODEX-DESKTOP-20 完成文档事实、代码实现、源码回归、双平台原生与安装验收、性能观察、签名和发布门禁；本检查点只记录当前执行状态，不构成完成证书、合并授权或发布授权。

## 当前基线

| 项目 | 结果 |
|---|---|
| 代码分支 | feature/0.1.10-maintenance |
| 代码提交 | 24ed093aea0758b5fdf542fa38fc98a0c1d6b1e9 |
| 文档分支 | docs/governance-main |
| 检查开始时文档 HEAD | 1c2656f9168b2ffca20630cacae511b1c763e6a1 |
| 统一树表组件 | apps/desktop/ui/src/components/ui/UiTreeTable.vue + treeTable.ts |
| 生产树表盘点 | 备份文件树使用共享组件；未发现其他生产层级树表绕过入口；设置页两个 data-root-table 是扁平路径表 |

## 本轮已复核

- git diff --check 通过，代码工作树无未提交改动。
- UI 定向回归：npm test -- --run tests/ui-tree-table.test.ts tests/backup-files.test.ts，2 个测试文件、12 项测试全部通过。
- 共享组件统一复选框、箭头占位、图标与文字的垂直基线，使用 22px 行高和统一控制尺寸；不改变选择、展开、备份范围或树表数据语义。
- `24ed093a` 将概览路由进入固定为 `foreground` 唤醒，`deadline` 仅用于持久化调度到期；旧 `app_update_check_interval_seconds` 仅保留迁移 / 序列化兼容，运行时周期由 native `Target::interval()` 唯一决定。
- 更新调度定向 Rust 测试 5/5；UI 定向回归 6 个测试文件、34 项通过。
- 既有本地全量证据仍有效：Rust 690 passed / 0 failed / 4 ignored；UI 94 个测试文件、459 项通过；类型检查和生产构建通过。完整记录见 [20261011 本地全量验证](../20261011-本地全量验证/README.md)。
- 既有代码实现和范围冻结见 [IMP-20](../../../../../../../../03-开发实施/IMP-OPENCODEX-DESKTOP-20.md)。0.2.0 的同步原型可以继承统一树表组件合同，但不因此提前启用同步业务范围。

## 仍需闭合的门禁

以下项目必须取得真实证据后，才能形成 0.1.10 候选并进入发布确认：

- macOS 原生 UI、隔离安装器、升级保留、真实更新 / 重启与恢复。
- Windows 普通权限人工复验；实际安装目录、受保护路径、自定义路径、升级保留与卸载选项；DPI、云母、原生标题栏、动效和透明下拉框。
- Windows 普通权限与管理员权限共用同一数据根，并验证配置、备份、缓存、日志和卸载边界。
- 自动检查启动 / 概览进入 / 定时 / 休眠恢复 / 跨启动的请求次数、缓存命中、CPU、RSS、长帧、子进程和 24 小时 stable / 6 小时 beta 周期。
- 真实更新端点、下载、签名、公证 / Authenticode、更新后重启与失败恢复；重新生成明确为 0.1.10 的身份制品。
- 全域事件注册表覆盖审阅、跨平台执行证据和真实用户迁移确认。

现有 Windows 候选包实际版本为 0.1.9，不能作为 0.1.10 发布资产。源码测试通过不替代上述门禁；在所有 required checks 关闭前保持 in_progress，不合并 main、 不替换日常安装、 不晋升 stable、 不公开发布。

## 证据归属

- 长期实施事实：docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md
- 本次运行维护协作记录：20261009-概览升级入口与备份通知策略/
- 本地源码证据：.adg/evidence/OCX-0110-20261011-LOCAL-VALIDATION/
- 代码提交：feature/0.1.10-maintenance
- 本记录和后续运行证据：docs/governance-main

后续新增设备、性能、安装、更新或发布证据，应继续追加到对应验证目录，并在 IMP-20 与本协作包索引中引用；不得用原型截图或源码定向测试替代原生验收。

## 2026-10-11 版本身份固化与源码门禁收口

代码分支 `feature/0.1.10-maintenance` 的最新提交为 `9497b4e56f01b92abf977c6b244ec8e087ffa6b5`，已推送。Tauri、Cargo、UI 包及锁文件的版本已统一为 `0.1.10`；旧 Windows 候选包仍按其真实身份记录为 0.1.9，不能直接作为本版本候选。

本轮命令结果：

- Cargo metadata、fmt、Clippy `-D warnings`、Rust workspace integration tests、`git diff --check` 均通过；Rust 测试命令退出码为 0，库目标 691 项通过 / 4 忽略，二进制目标 2 项通过，集成测试目标均通过。
- UI 全量 94 个测试文件、459 项通过；类型检查和生产构建通过，构建转换 170 个模块。
- Windows bundle identity smoke 5/5 通过，确认候选身份检查会接受 0.1.10 的包元数据。

证据目录：`.adg/evidence/OCX-0110-20261011-TARGET-EXECUTION/`。该结果只闭合源码身份与源码回归；仍需在本提交上重新生成双平台候选，完成 macOS / Windows 原生安装、普通权限、路径、外观、性能、真实更新 / 重启、签名和发布门禁。`CHANGELOG.md` 属于 `main`，在集成阶段补写；当前仍为 `in_progress`，不构成发布授权。
