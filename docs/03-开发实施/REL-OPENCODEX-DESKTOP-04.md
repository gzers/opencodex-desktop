---
id: REL-OPENCODEX-DESKTOP-04
object_kind: release.candidate
state: candidate-passed
title: 0.1.3 更新通道真实端点、官方远端查询与代跑、网络代理与分域存储候选记录
summary: 承接 IMP-OPENCODEX-DESKTOP-16。U-01/U-03/U-04/U-05/U-05b/C 已在 feature/0.1.3-update-channel-network 实施并通过构建级门禁（cargo test 431 lib passed、vitest 360 passed），本机产出 0.1.3 aarch64 制品。U-02（签名公钥与 CI 签名制品）属发布安全操作，未实施、待单独确认；未合并 main、未打 tag、未发布。
source_refs:
  - IMP-OPENCODEX-DESKTOP-16
  - MNT-OPENCODEX-DESKTOP-20261004-03
  - 5dfbd161
  - c9eeff3b
  - feeb1c9a
  - a509ddbb
  - d1b6e83b
---

# REL-OPENCODEX-DESKTOP-04 0.1.3 候选记录

## 结论

0.1.3 承接 [IMP-OPENCODEX-DESKTOP-16](IMP-OPENCODEX-DESKTOP-16.md)，在 `feature/0.1.3-update-channel-network`（基于 `main@f14e061a`，已并入 `origin/main@2323d5fe`）实施 **U-01 桌面自更新真实端点、U-03 官方版本卡片只读远端查询、U-04 代跑官方更新、U-05/U-05b 网络代理、C 阶段分域存储迁移**，并通过构建级门禁。**U-02（签名公钥与 CI 签名制品）属发布安全操作，未实施、待用户单独确认**；**本版尚未合并 `main`、未打 tag、未出 Draft Release、未公开发布**。本节只记录已执行事实，不把构建/测试成功写成运行验收通过。

## 范围与实现

| 编号 | 实现 | 提交 |
|---|---|---|
| U-01 | 端点真实化：`config/runtime.defaults.json` 与 `tauri.conf.json` 改指本仓库 Releases（stable=`releases/latest/download/latest.json`，beta=`releases/download/beta/latest.json`） | `5dfbd161` |
| U-03 | `official_remote_latest`（受控 `npm view` + tag 白名单 + 固化超时）与前端语义化版本比较、官方卡片「远端最新版本」行 | `5dfbd161` |
| U-04 / U-04b | `install_official_update`：先解析远端确定版本，再复用受控 `install_runtime`（registry + 确定版本 + 当前登记前缀，不浮动 `latest`） | `5dfbd161` |
| U-05 / U-05b | 代理偏好（无凭据）、`config/network.defaults.json` + `modules/network_defaults`、网络卡片、升级 tab 快捷跳转、自更新/托管安装/官方查询共用代理、「检查连接」命令 | `c9eeff3b`、`a509ddbb` |
| C | 偏好磁盘结构升级为分域（schema v2）；统一识别分域/扁平；迁移引擎落盘分域；容器与 WebDAV 同步走分域投影、接收端按 schema 迁移、过新拒绝 | `feeb1c9a` |
| 版本 | 0.1.2 → 0.1.3：`tauri.conf.json`、`Cargo.toml`、`ui/package.json`、`ui/package-lock.json`；`CHANGELOG.md` 新增 `[0.1.3] - 2026-10-04` | `feeb1c9a` |

## 门禁与证据（构建/测试级）

- 合并 `origin/main` 后后端 `cargo test --workspace --features integration-test` 无失败（合并前为 **431 lib passed / 0 failed / 2 ignored**）；`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings` 通过。
- 前端 `vue-tsc --noEmit` 通过；`vitest --run` **360 passed / 0 failed**（72 文件）；`vite build` 通过。
- 本机制品：`tauri build --target aarch64-apple-darwin` 产出 `OpenCodeX Desktop.app`（`CFBundleShortVersionString=0.1.3`）与 `OpenCodeX Desktop_0.1.3_aarch64.dmg`（3,922,274 bytes；sha256 `1eb19bba7745d334da32a7c4…`）。
- 真实远端只读查询冒烟（本机）：`npm view @bitkyc08/opencodex@latest` 返回 `2.77.0` 与 `dist.integrity`，确认 U-03 依赖的外部命令形态。

## 运行验收（本机 0.1.3 制品，2026-10-04 追加）

在隔离沙箱身份下用上文本机制品做运行级验证（非仅构建门禁）：

- **启动构建来源**：启动日志 `startup version=0.1.3`（`a509ddbb`）。
- **C 阶段分域迁移（真实制品）**：沙箱内预置 legacy 扁平偏好（含旧枚举 `beta-6h`），启动后自动迁移为 schema v2 分域结构（`appearance`/`shell`/`backup`/`extensions`/`maintenance`/`sync`/`updates`/`network`/`cli`），通道迁移为 `beta` + 自动检查 + 21600 秒；迁移事务目录保留 `original.json` 与 `manifest.json`（`from_schema:0`、`to_schema:2`、步骤含 `preferences.v0.legacy_update_channel` 与 `preferences.v2.sectioned_disk_layout`）；受控「模拟日常根」哨兵 SHA-256 整轮未变（`mnt013-migration-smoke.sh`：PASS）。
- **受控出网运行检查**：`cargo test --lib modules::about::remote -- --include-ignored` 经生产函数真实拉取 `@bitkyc08/opencodex@latest` 版本成功；`cargo test --lib commands::network -- --include-ignored` 经生产 HTTP 客户端真实探测受控地址成功。
- **U-04 复用的受控安装真实链路**：`OCX_TEST_RUNTIME_REAL=1 cargo test --test runtime_managed_real` 走完整真实链路——真实 npm 把 `@bitkyc08/opencodex@2.77.0` 装到托管前缀（`--ignore-scripts` 安全默认即可运行）、入口可执行并输出 `opencodex 2.77.0`、运行来源 `managed`、一级卸载往返、离线 tarball 导入、系统保护目录与坏包拒绝均通过。该路径即 U-04 `install_official_update` 复用的 `install_managed_runtime` 核心；「先解析远端确定版本 → 再复用该安装」的组合由单测覆盖，但**经打包 UI 的整条点击链路未执行**。
- **UI 运行冒烟**：沙箱启动后设置页可交互，概览/设置/画质与官方共享配置卡片渲染正常（截图 `test/out/0.1.3-overview.png`、`013-02-settings.png`）。
- **已知限制（如实记录）**：其一，应用内「桌面管理器版本」显示的**当前版本**在沙箱首屏读状态快照时可能早于 `get_update_status` 返回，显示为构建期兜底文本而非 0.1.3，属既有首屏时序表现，未单独修复；其二，U-01 端点虽指向真实地址，但远端当前稳定的 `latest.json` 解析到旧 `v0.1.2` Release、尚无该资产（beta 同样 404），故自更新端到端生效以 U-02 产出签名清单为准；其三，U-04 经打包 UI 的整条点击链路未执行。

## 未完成 / 另行确认

- **U-02 未实施**：`tauri.conf.json` 的 `pubkey` 仍为占位串，`bundle.createUpdaterArtifacts` 未打开，`release.yml` 未注入 `TAURI_SIGNING_PRIVATE_KEY`。生成 minisign 私钥、写入 GitHub secret 属发布安全操作，**需用户单独确认**。在此之前 U-01 的真实端点尚无签名 `latest.json` 可拉取（远端目前仅有稳定通道可解析，beta 清单未发布）。
- **运行验收未做**：真机自更新、官方代跑、代理链路与分域存储的真机运行验收均未进行；本记录只证明构建与测试门禁。
- **合并与发布未做**：未合并 `main`、未打 tag、未出 Draft Release；公开发布另行确认。

## 待用户裁决

1. 是否现在单独授权执行 U-02（生成签名公私钥 → 写入 GitHub secret → 打开签名制品与工作流注入）。
2. 合并 `main`、打 tag 与出 Draft Release 的时机：先合 U-01/U-03/U-04/U-05/C（CHANGELOG 已注明签名待补），还是待 U-02 就绪后一并合并发布。
