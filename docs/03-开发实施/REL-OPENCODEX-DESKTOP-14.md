---
id: REL-OPENCODEX-DESKTOP-14
object_kind: release
state: verified
title: 0.2.9 保存预览刷屏修复发布
summary: 0.2.9已公开并晋升stable/Latest；PR17合并main 813faf405、tag v0.2.9；修WP-65保存预览被提示挤成0高度，门禁18项与双平台CI/制品签名核验通过。
source_refs:
  - 813faf405ea5928e2fb6f389bc1dde6bd9b79043
  - https://github.com/gzers/opencodex-desktop/pull/17
  - https://github.com/gzers/opencodex-desktop/releases/tag/v0.2.9
---

# 0.2.9 保存预览刷屏修复发布

2026-10-25，Asia/Shanghai。0.2.8 公开为 stable 后装机发现 P0 缺陷，修完整走发布流程。

## 缺陷

保存预览被待核对提示挤成 0 高度：账号里有几十个「已选但当前清单缺失」的型号时，后端逐条发提示、前端逐条平铺，flex 高度竞争把渠道/型号区压到 0，保存前唯一核对入口失效。详见 [WP-65](https://github.com/gzers/opencodex-desktop/blob/docs/governance-main/docs/04-项目资料/01-项目支撑资料/06-问题记录/0.2.8-bugs.md)。

不是原型差异：原型预览没有 notices 概念，不存在该布局路径。

## 源码

工作树 `ocx-022-model-selector`、分支 `fix/0.2.8-export-and-parity`，PR [#17](https://github.com/gzers/opencodex-desktop/pull/17) 合并 main：`813faf405ea5928e2fb6f389bc1dde6bd9b79043`，tag `v0.2.9`。

修复提交 `5ba8c1356`（提示收起 + 兜底），版本与 CHANGELOG 提交 `b348188cc`。

## 验收

| 层 | 结果 |
| --- | --- |
| 原型回归 | 274 / 0 |
| 前端类型 | vue-tsc 通过 |
| 前端单测 | 156 文件 / 798 通过 / 10 跳过 |
| 前端门禁 | gate:release 18 / 0 / 0 |
| 后端 | workspace + integration-test 全绿 |
| CI（PR #17） | frontend / backend / backend-windows / windows-smoke / release-tools 全通过 |
| 出包 | release 40178734369：macOS aarch64 + Windows x64 均 success |
| 公开 | publish 40179772374 + promote-stable 40179833872 |

## 制品

`v0.2.9`（非草稿、非预发布，Latest=`v0.2.9`）：dmg / app.tar.gz(+sig) / x64-setup.exe(+sig) / x64_en-US.msi(+sig) / latest.json。

签名核验原文：

- `Cryptographically verified darwin-aarch64: ... sha256=c90c0a84a63d6615e7e2de6fe1eeed2c8e895cef70475a29a6a2c68470b54b05`
- `Cryptographically verified windows-x86_64: ... sha256=3fce9dbb4089abd70438752f1a40fe77e30ad2269f86d33f4578dc3c910d75e9`
- `Cryptographically verified windows-x86_64-nsis: ... sha256=7b2cde6a35d9a6acf9ee68147c0084d29a5c0027b7474e0625b20cde18f9c3a2`
- `Target: v0.2.9; mode: stable; previous Latest: v0.2.8` → `Verified public release v0.2.9; Latest=v0.2.9`

## 边界

- 「系统占用」数字与原型演示值不是同一量纲，口径待定。
- 同渠道连续未发现型号仍逐条提示，按渠道合并需改后端契约。
- 触屏长按、窄窗展开提示后的滚动未真机复核。
- 安装包延续未做代码签名 / Apple 公证。

