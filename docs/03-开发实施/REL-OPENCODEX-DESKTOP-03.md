---
id: REL-OPENCODEX-DESKTOP-03
object_kind: release.candidate
state: verified
title: 0.1.2 更新通道与配置分层最小闭环发布记录
summary: 承接 MNT-OPENCODEX-DESKTOP-20261004-02 的 A/B 最小闭环：更新通道解耦并接通消费、固化默认配置与类型收敛、appearance.theme 入偏好、通用配置迁移引擎骨架、测试沙箱隔离、中英文 README 配置说明。C 阶段存储迁移与真实自更新端点/签名延期。通过前后端门禁、本机制品构建与沙箱隔离冒烟，合并入 main。
source_refs:
  - MNT-OPENCODEX-DESKTOP-20261004-02
  - IMP-OPENCODEX-DESKTOP-15
  - cb02a45b
---

# REL-OPENCODEX-DESKTOP-03 0.1.2 发布记录

## 结论

0.1.2 是 0.1.1 之后的维护版本，只收 [范围冻结](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261004-更新通道与配置分层/分析/06-0.1.2-范围冻结与待裁决.md) 的 A/B 最小闭环。**真实自更新端点与签名（U-01/U-02）未接通、C 阶段磁盘存储迁移未执行、官方版本卡片远端查询与代跑（U-03/U-04）未实现**，均登记为延期项。本节只记录已执行事实。

## 范围与实现

| 编号 | 实现 |
|---|---|
| U-05/U-07 | `Preferences` 将旧 `app_update_channel` 复合枚举拆为 channel + `app_update_auto_check` + `app_update_check_interval_seconds`；`UpdateChannel::endpoint()` 占位假地址删除；检查/安装/启动通道解析共用 `update_channel()`；设置页删除第二个内存通道事务 |
| U-06a | 新增 `config/preferences.defaults.json` 与 `config/runtime.defaults.json`；`build.rs` 构建期校验存在且为合法 JSON；`Preferences::default()` 从嵌入文件读取 |
| U-06b | `Preferences.theme` 落入偏好（后端事实源）；`theme.ts` 保留 localStorage 首屏缓存并取消旧缓存反向覆盖；顶栏切换落盘 |
| U-08 | `load_preferences` 读取即 `validate`；非法/损坏显式失败不覆盖原件 |
| U-09a | 新增 `modules/config_migration`：文档种类登记、schema 识别、legacy v0 迁移链、未来版本拒绝、锁内备份与原子提交；`Preferences.schema_version` 落地 |
| — | 测试沙箱：`modules/test_sandbox` 沙箱身份与沙箱根解析，根缺失即停止不回退日常目录；keychain 服务名随身份切换；集成测试与冒烟脚本 |
| U-10a | 根 `README.md` / `README.en.md` 新增「配置文件与设置 / Configuration files and settings」说明 |

## 门禁与证据

- 前端：`vue-tsc --noEmit` 通过；`vitest --run` **355 passed / 0 failed**（71 文件）；`vite build` 通过。
- 后端：`cargo fmt --check` 通过；`cargo clippy --workspace --all-targets -- -D warnings` 通过；`cargo test --workspace --features integration-test` **414 lib passed / 0 failed / 2 ignored** 及各集成测试全绿（含 `sandbox_isolation`）。
- 本机制品：`tauri build --target aarch64-apple-darwin` 产出 `OpenCodeX Desktop.app`（`CFBundleShortVersionString=0.1.2`）与 `OpenCodeX Desktop_0.1.2_aarch64.dmg`。
- 沙箱隔离冒烟（macOS）：沙箱身份启动，启动日志 `startup version=0.1.2 commit=cb02a45b...`；全部写入落在沙箱数据根；受控模拟日常根的哨兵文件 SHA-256 与 mtime 在整轮后不变。
- CI（main，`cb02a45b`）：见下方 CI 结果段。

## 提交与版本

- 特性分支 `feature/0.1.2-update-channel-config-layers` 从 `main`（`aec115d0`）新建；
- 关键提交：`08fd076d`（通道解耦与主题）、`0f2f16ae`（迁移引擎骨架）、`18040e86`（沙箱身份与版本号）、`a5999419`（README）、`cb02a45b`（Default 读固化文件）；
- 版本号 0.1.1 → 0.1.2：`tauri.conf.json`、`Cargo.toml`、`ui/package.json`、`ui/package-lock.json` 同步；`CHANGELOG.md` 新增 `[0.1.2] - 2026-10-04`。
- 已合并入 `main`（fast-forward，`aec115d0..cb02a45b`）。

## CI 结果（main）

CI run [37183755672](https://github.com/gzers/opencodex-desktop/actions/runs/37183755672) 四 job 全绿：frontend **success**、backend **success**、backend-windows **success**、build **success**（构建 job 产出 macOS aarch64 包）。

## 延期与未完成（独立门禁）

- U-01/U-02：真实自更新端点、签名公钥、CI `createUpdaterArtifacts` 与 `TAURI_SIGNING_PRIVATE_KEY` 未接通，端点与公钥仍为占位；依赖用户裁决与发布安全授权。
- U-03/U-04：官方版本卡片远端查询、外部代跑官方更新未实现，待产品裁决。
- C 阶段：磁盘嵌套化、容器/同步 schema 迁移、中断恢复正式启用未执行。
- 签名与公证、Intel macOS、Windows 实机核验仍属单独门禁。


## 追加（2026-10-04）：A/B 范围剩余项实施事实

首轮合并（cb02a45b）后继续完成 A/B 范围的剩余固化项，仍未接通真实自更新端点：

| 编号 | 追加实现 |
|---|---|
| U-06a（runtime.defaults 消费） | 新增 `modules/runtime_defaults`：进程动作/runner 窗口、状态轮询、WebDAV 超时、备份保留天数、日志读取行数、版本/Doctor/shim 探针窗口、安装 tag/闲置/探测窗口、扩展初始策略、桌面更新端点均从固化 `runtime.defaults.json` 读取；数值与此前冻结常量一一对应 |
| U-05（端点收敛落地） | 删除 `UpdateChannel::endpoint()` 内联假地址，改读固化 `updates.desktop.channels`；命令层检查/安装用当前通道端点构建 updater，检查与安装共用同一来源 |
| H-24/H-25/H-26（前端窗口） | 新增 `src/config/runtimeDefaults.ts`（构建期内联 ui.* 只读子集）；首屏快照/偏好等待、动作兜底轮询、启停提示窗口与迟到观察、面板加载提示、托盘轮询改读固化值 |
| H-29（扩展初始策略） | 扩展默认分发方式与客户端启用缺省读取固化策略 |

追加提交：`6b9d4441`（runtime.defaults 消费者）、`1aac784a`（前端窗口）、`9f2a9d2d`（扩展初始策略与更新端点）。以上均 fast-forward 并入 `main`。

追加门禁：`cargo test --workspace --features integration-test` **415 lib passed / 0 failed / 2 ignored**，`clippy -D warnings` 通过；`vue-tsc` 通过、`vitest` 355 passed、`vite build` 通过；macOS aarch64 重建制品并在沙箱身份冒烟，日志 `startup version=0.1.2 commit=9f2a9d2d...`，日常根哨兵未变。

**仍未完成：** 真实自更新端点与签名公钥（U-01/U-02）仍为占位，C 阶段存储迁移与 U-03/U-04 未实施。

## 追加 CI 结果（main 9f2a9d2d）

追加提交合并后的 main 提交 `9f2a9d2d` CI run [37185334531](https://github.com/gzers/opencodex-desktop/actions/runs/37185334531) 四 job 全绿：frontend success、backend success、backend-windows success、build success。

## 追加（2026-10-04）：配置迁移引擎完善与 CI 结果

在 A/B 范围之外的通用迁移引擎（U-09a）继续补完，使「配置格式自动转换」具备可运行的最小闭环：

- 中断恢复判定 `recover`：manifest 记录原件/候选 SHA-256；重启时目标为候选则补 committed 记录、为原件则可重做、皆不等则判定外部变更并保留证据。
- 启动自动迁移 `migrate_preferences_on_startup`：缺文件或当前 schema 不写盘；旧 schema 自动转换并保留原件备份；接入启动 setup，失败只记日志不阻断启动。

追加提交：`be5f2c63`（恢复判定）、`54de4bf2`（启动自动迁移）。均 fast-forward 并入 main。

追加门禁：`cargo test --workspace --features integration-test` **420 lib passed / 0 failed / 2 ignored**，`clippy -D warnings` 通过（含迁移引擎 10 项单测）。

真机沙箱冒烟（macOS 0.1.2 制品）：沙箱内放置 legacy 偏好（旧枚举 manual），启动后自动迁移为 schema_version=1 / channel=stable / auto_check=false，并生成 config-migrations/preferences/txn-* 原件备份；受控模拟日常根哨兵未变。

CI（main `54de4bf2`）run [37186315036](https://github.com/gzers/opencodex-desktop/actions/runs/37186315036) 四 job 全绿。

**仍未完成：** 真实自更新端点与签名公钥（U-01/U-02）仍为占位；C 阶段存储迁移与 U-03/U-04 未实施。