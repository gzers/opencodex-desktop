---
id: IMP-OPENCODEX-DESKTOP-16
object_kind: implementation.change
state: planned
title: "0.1.3 更新通道自更新、官方更新代跑与网络代理"
summary: "承接 MNT-OPENCODEX-DESKTOP-20261004-03（COL-LOCAL-20261004-03）。0.1.3 冻结范围：U-01 桌面自更新真实端点（本仓库 GitHub Releases）、U-02 签名公私钥与 CI 签名制品、U-03 官方版本卡片只读远端查询、U-04 复用受控 install_runtime 代跑官方更新、U-05 网络代理配置（通用网络卡片 + 版本升级 tab 快捷跳转，不涉及钥匙串、含开发规定的网络行为配置文件）、C 阶段存储迁移。发布安全操作（生成签名私钥、写入 CI secrets、打包发布）另行单独确认。"
demand_ids: ["DMD-OPENCODEX-DESKTOP-MANAGER"]
task_ids: []
source_refs: ["MNT-OPENCODEX-DESKTOP-20261004-03", "MNT-OPENCODEX-DESKTOP-20261004-02", "IMP-OPENCODEX-DESKTOP-15", "REL-OPENCODEX-DESKTOP-03"]
---

# IMP-OPENCODEX-DESKTOP-16 0.1.3 更新通道自更新、官方更新代跑与网络代理

2026-10-04。承接运行维护协作包 [COL-LOCAL-20261004-03](https://github.com/gzers/opencodex-desktop/tree/docs/governance-main/docs/04-项目资料/03-项目协作资料/01-协作包/2026-10-04-更新通道自更新与官方更新代跑) 与其维护记录 MNT-OPENCODEX-DESKTOP-20261004-03。

## 1. 范围（已冻结）

| 编号 | 实施内容 |
|---|---|
| U-01 | 桌面自更新使用真实端点：本仓库 GitHub Releases 的 `latest.json` / `beta.json`，替换 `runtime.defaults.json` 中 `updates.desktop.channels` 的占位地址 |
| U-02 | 生成 minisign 公私钥；公钥写入 `tauri.conf.json`（编译期常量）；`createUpdaterArtifacts` + CI `TAURI_SIGNING_PRIVATE_KEY` 产出签名 `latest.json` / `.sig` |
| U-03 | 官方版本卡片新增只读远端查询（`npm view @bitkyc08/opencodex@latest version` + `dist.integrity`），与本地/运行版本比较出「有可用更新」 |
| U-04 | 代跑官方更新：复用受控 `install_runtime`（`source=registry, version=<tag>, prefix=当前登记前缀`），沿用离线包 / 代理 / `.runtime-manifest.json` / 重启提示；不代理官方更新器改托管前缀 |
| U-04b | 查询与安装共用同一来源策略，绑定确定版本，不浮动 `latest` |
| U-05 | 网络代理：通用设置新增「网络」卡片（无代理 / 自动(系统) / 手动 HTTP|SOCKS5 + 例外列表），「版本升级」tab 加「配置网络」快捷跳转；**不涉及钥匙串，本版无代理鉴权** |
| U-05b | 独立网络行为配置文件 `config/network.defaults.json`（超时/重试/UA/域名白名单/TLS 约束/并发；构建只读，开发规定，暂不开放） |
| C | 配置存储 schema 迁移（磁盘嵌套 + 容器 / 同步兼容投影），复用 0.1.2 迁移引擎与沙箱测试 |

## 2. 分支与版本

- 从最新 `main`（`f14e061a`）新建 `feature/0.1.3-update-channel-network`。
- 版本号 0.1.2 → 0.1.3：`tauri.conf.json`、`Cargo.toml`、`ui/package.json`、`ui/package-lock.json` 同步；`CHANGELOG.md` 新增 `[0.1.3]`。

## 3. 已确认裁决

| 编号 | 裁决 |
|---|---|
| D-1 | 端点指向本仓库 Releases；公钥 `tauri signer generate` 生成，私钥进 CI secrets，公钥编译期固定 |
| D-2 | 授权 CI 用 `TAURI_SIGNING_PRIVATE_KEY` 产出签名制品 |
| D-3 | 官方卡片新增只读远端查询 |
| D-4 | 复用受控 `install_runtime` 代跑官方更新 |
| P-1 | C 阶段存储迁移纳入 0.1.3 |
| P-2 | stable/beta 两个通道都发布签名清单 |
| P-3 | 网络代理纳入 0.1.3；不使用钥匙串；本版不做代理鉴权；「自动」= 读系统代理设置 |

## 4. 验收（草案）

前后端门禁全绿；自更新真实端点可解析、签名校验通过才安装、失败保留上一版本、stable/beta 生效；官方卡片远端查询带超时且失败不改写本地状态、查询与安装对象一致；代跑官方更新落在当前登记前缀并写 `.runtime-manifest.json`，不写全局 npm 前缀；网络卡片三模式生效、快捷跳转可用、代理作用于应用自更新/官方查询/托管安装，「检查连接」带超时且不改状态；C 阶段通过沙箱隔离与日常根未变验收；结果回写 `REL-*` / `RUN-*`。

## 5. 执行结果

（待实施后回写；未实施前不写结论。）
