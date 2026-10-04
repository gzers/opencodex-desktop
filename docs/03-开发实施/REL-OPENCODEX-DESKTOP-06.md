---
id: REL-OPENCODEX-DESKTOP-06
object_kind: release.candidate
state: verified
title: 0.1.6 OTA 重试与稳定通道发布验收
summary: 0.1.6 已公开并晋升稳定 Latest；双平台构建、匿名端点和旧公钥真实验签通过。本机 macOS 0.1.3 → 0.1.6 OTA 安装与自动重启实测通过，托管代理持续运行、偏好未变，重启后检查显示已是最新版本；Windows 与 0.1.4 实机安装未覆盖。
source_refs:
  - REL-OPENCODEX-DESKTOP-05
  - 0547f68aedfae9488a839a437e9792cba5ad661c
  - 806c0479d4d7b1fc55549bd1926bf5756365ebfd
  - 4c5afb5189c02cba6a84b7e0faebbdbf3153d66d
---

# 0.1.6 OTA 重试与稳定通道发布验收

2026-10-05（Asia/Shanghai）。用户在本地候选验收后明确回复“同意。授权”，授权推送、打标签、签名出包与稳定通道晋升；随后在本机 GUI 安装的执行时确认中明确回复“允许安装并重启”。发布动作通过 Git / GitHub Actions 执行；project-governance 仅承载发布事实与证据。

## 范围与身份

- 应用源码与版本标签：`main` / `v0.1.6` → `806c0479d4d7b1fc55549bd1926bf5756365ebfd`。
- `0547f68` 修复安装失败后的重试、重复安装拦截、发布错误传播及 stable 显式晋升；`806c0479` 追加真实下载包密码学验签、明确的 `promote-stable/v*` 入口，以及手动 Release 构建检出目标标签。
- 所有公开制品由标签源码重新构建；此前本机免签候选包不作为 OTA 发布资产。历史 0.1.3 / 0.1.4 / 0.1.5 标签及包不改写。
- 发布执行脚本最终为 `main@4c5afb5189c02cba6a84b7e0faebbdbf3153d66d`，追加草稿 Release 查询与 `untagged-*` 临时地址兼容。它不改变 `v0.1.6` 的源码、版本标签或制品。
- GitHub Release `403126668` / [v0.1.6](https://github.com/gzers/opencodex-desktop/releases/tag/v0.1.6) 已公开、非 prerelease，并成为 Latest。发布时间 `2026-10-04T17:41:33Z`（北京时间 2026-10-05 01:41:33）。

## 验证事实

本地应用门禁沿用 [候选验收](2026-10-05-OTA修复实施与候选验收.md)：前端 364、后端 559 通过，5 项忽略；类型检查、格式、clippy、生产构建通过。此次发布脚本门禁扩展至 **29 项通过**，包含真实临时测试密钥验签、同长度篡改、错误公钥、截断包、摘要不匹配、版本不匹配、发布失败传播、草稿查询及临时地址兼容。兼容临时地址仍强制匹配目标仓库和文件名，仅适用于草稿；错误仓库、旧版本地址、错误文件名和重复资产均拒绝。

0.1.3～0.1.6 的更新公钥完全相同（解码文本 SHA-256 `30190687026fdc7b7e223884315baa52f6d42c1c37d447b8631e9597c1ecdfee`）；真实 0.1.3 macOS 更新包通过 minisign 验签。签名公钥兼容不等同于旧客户端实际更新与重启通过。

- 标签源码 [CI 37220011919](https://github.com/gzers/opencodex-desktop/actions/runs/37220011919)：backend、frontend、release-tools、backend-windows、build 全部成功。
- 双平台 [Release 37220014530](https://github.com/gzers/opencodex-desktop/actions/runs/37220014530)：macOS arm64、Windows x64 均成功。
- 最终发布脚本 [CI 37221433173](https://github.com/gzers/opencodex-desktop/actions/runs/37221433173)：前端、后端、Windows 后端、release-tools、build 全部成功。匿名 API 曾限流，最终状态改由公开 Actions 页面核实，见 `publisher-ci-final.json`；较早的 jobs JSON 保留其采集时的进行中状态。
- 稳定通道 [Publish Release 37221452456](https://github.com/gzers/opencodex-desktop/actions/runs/37221452456)：成功，Latest 从 v0.1.3 晋升为 v0.1.6。
- 独立匿名核对：固定 v0.1.6 清单与 `/releases/latest/download/latest.json` 内容相同、版本为 0.1.6；公开 API 确认 `draft=false`、`prerelease=false`。所有三个不同更新包均以 **v0.1.3 标签内公钥** 完成真实 minisign 验签，大小与 GitHub SHA-256 元数据一致。macOS 包内版本为 0.1.6；修改同长度包的一个字节后验签失败。
- 本机 0.1.3 检查后两个安装入口从禁用变为可用，旧版仍残留原失败提示；用户确认后执行「安装并重启」，通过稳定通道直接升级为 0.1.6。无需先手动安装 0.1.5。

| 更新资产 | 字节数 | SHA-256 |
|---|---:|---|
| `OpenCodeX.Desktop_aarch64.app.tar.gz` | 3889908 | `e8bfb4ca69f474c687b469e6727b191ba88ef7b37d6a00037b9a835e9ea56025` |
| `OpenCodeX.Desktop_0.1.6_x64_en-US.msi` | 3919872 | `9ee06a3cb2d356b33f0b1e1c4beaae2fc113e1cfeaa8429e60b3aafdf8352ce5` |
| `OpenCodeX.Desktop_0.1.6_x64-setup.exe` | 2831437 | `65e5c93eab15288a5b1c1828ee6c218d399e4272de862a54d098974761767ca9` |

macOS 发布包与安装后应用主二进制 SHA-256 均为 `0455a1771c6b28d7f95143551941414647445a3ff9990bcce8238b27a162fffb`。安装前 0.1.3 主二进制为 `6abeb53cd6ac35c98b9f02282b8831d0cd40d4e6dd4eca039cb3e5fd1073029c`。

## 本机 OTA 实测

实际安装对象为 `/Applications/OpenCodeX Desktop.app`。2026-10-05 01:49:59（北京时间）的独立文件与进程核验确认版本及 build 均为 0.1.6，安装后二进制与上述已验签公开发布包完全一致。桌面进程从 PID 17819 自动重启为 PID 44788（PPID 1）；核验新进程之前没有手动重新启动应用。

托管 OpenCodex 运行时仍为 2.69.0、PID 18504、PPID 18502，启动时间仍是 2026-10-04 21:51:26，端口 10100。安装后原生界面显示“运行中 / 健康正常”，进程身份与启动时间未变。偏好文件安装前后 SHA-256 均为 `46e87c0c3bb8501b77642a33db05b7bb6c177a0334c2486de44ff510ea119a12`，字节内容未改变。

重启后进入「设置 → 版本升级」，自动检查完成，桌面管理器显示 **v0.1.6 · 稳定通道 / 已是最新版本；未发现可安装更新**，无残留失败提示；「安装更新」「重新安装」在无更新时禁用。进入该页按既有 `autoBackupUpgrade` 偏好自动生成备份 `bk_20261004175133_7f468a1b`。这不改变偏好内容，也不代表升级了官方运行时。

证据：`installed-before.json` / `installed-after.json`、`runtime-before.json` / `runtime-after.json`、`runtime-after-ui.txt`、`ota-after-ui.txt` 与 [安装后截图](../../.adg/evidence/OTA-RELEASE-20261005/ota-success.png)。发布记录中的本机用户目录已在界面文本证据中替换为 `$HOME`；不保存偏好原文、密钥或官方面板用量数据。

证据入口：[发布回执](../../.adg/evidence/OTA-RELEASE-20261005/receipt.json)。原候选证据保留于 `.adg/evidence/OTA-FIX-20261005/`，不把本轮后续结论倒写为旧时点事实。

## 发布阻断与修正

前三次晋升均在公开发布之前失败，没有触及 Latest：

1. run `37220705481`：按标签查询草稿返回 HTTP 404。`4f54a5dd` 改用 `gh release view` 解析草稿身份，再按数值 ID 读取完整元数据。
2. run `37221134313`：严格地址比较发现清单的最终地址与草稿资产地址不等。`ed5fe2c1` 增加资产身份诊断。
3. run `37221278273`：诊断确认 GitHub 草稿地址使用 `untagged-25955b20eb61985d71ae`；清单正确指向未来 `v0.1.6` 地址。`4c5afb51` 在同一草稿 Release 已上传资产范围内映射最终地址，并继续执行真实下载、大小、摘要、公钥验签门禁。

重试只重定向操作触发标签 `promote-stable/v0.1.6`；应用版本标签 `v0.1.6` 一直保持 `806c0479`，没有重建或替换发布资产。第四次成功日志及全部失败诊断均保留在本轮证据目录。

## 兼容与回退

0.1.6 沿用现有更新公钥和偏好 schema，不新增数据迁移。此次变更不涉及官方 OpenCodex 运行时包升级。若稳定通道出现故障，保留旧标签/资产作为恢复来源；稳定降级必须另行明确授权并核验，不通过本次拒绝降级的正常晋升入口强行执行。已安装的 0.1.6 不会因 Latest 回退而自动降级。

## 剩余边界

公开发布及本机 macOS 0.1.3 → 0.1.6 的实际安装、桌面壳自动重启、托管代理持续运行和偏好一致性已通过。本轮足以否定“0.1.3 确实无法 OTA”的判断；它证明当前发布链与本机配置下可完成 OTA，不代表任意网络条件下均可成功。安装失败后重试与取消状态恢复有前端自动化回归覆盖，本轮未在已安装的 0.1.6 上人为注入失败再执行安装。

Windows 实机安装/重启未验收；macOS Apple 开发者签名/公证与 Windows Authenticode 签名不在本轮范围，OTA minisign 签名不等同于操作系统代码签名。0.1.4 与 0.1.3 的更新公钥一致，但这不是两个旧版本实机安装均已通过的结论。
