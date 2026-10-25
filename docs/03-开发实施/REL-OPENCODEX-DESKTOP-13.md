---
id: REL-OPENCODEX-DESKTOP-13
object_kind: release
state: verified
title: 0.2.8 原型对齐与导出修复发布
summary: 0.2.8已公开并晋升stable/Latest；PR16合并main 50d3844fa、tag v0.2.8；30条待修全部落地，前端797项、门禁18项、双平台CI与制品签名核验通过。
source_refs:
  - 50d3844fad06ce8c340e112b26af193650ef6006
  - https://github.com/gzers/opencodex-desktop/pull/16
  - https://github.com/gzers/opencodex-desktop/releases/tag/v0.2.8
---

# 0.2.8 原型对齐与导出修复发布

2026-10-25，Asia/Shanghai。用户以目标模式授权贯通「文档治理 → 开发 → 测试 → 发布」，要求界面逐项对齐 0.2.0 原型、不得自由发挥。

## 源码

工作树 `ocx-022-model-selector`、分支 `fix/0.2.8-export-and-parity`，PR [#16](https://github.com/gzers/opencodex-desktop/pull/16) 合并到 main：`50d3844fad06ce8c340e112b26af193650ef6006`，tag `v0.2.8`。

修复提交：`1a1b5bbe1`（30 条对齐 + 客户端分区）、`3666f6dd1`（导出弹窗细节）、`8eb8be2fb`（Windows 冒烟跟随改名）。

## 范围与依据

- 差异依据：[0.2.8-审计清单](https://github.com/gzers/opencodex-desktop/blob/docs/governance-main/docs/04-项目资料/03-项目协作资料/01-协作包/2026-10-04-模型配置管理与同步/03-测试验收/0.2.8-%E5%AE%A1%E8%AE%A1%E6%B8%85%E5%8D%95.md)（31 条：30 待修 + 1 已关闭 + 2 登记）
- 交互契约：UI 规范 §38 / §40
- 决策 A：导出凭据口令保持可选、允许明文，改原型不改软件

## 验收证据

| 层 | 结果 |
| --- | --- |
| 原型回归 | `node --test *.test.cjs` → 274 通过 / 0 失败 |
| 前端类型 | `vue-tsc --noEmit` 通过 |
| 前端单测 | 156 文件 / 797 通过 / 10 跳过 |
| 前端门禁 | `gate:release` 18 通过 / 0 失败 / 0 跳过（含 WebKit 复跑） |
| 后端 | `cargo test --locked --workspace --features integration-test` 全绿（lib 968） |
| fmt / clippy | 绿 |
| CI（PR #16） | frontend / backend / backend-windows / windows-smoke-evidence / release-tools 全部通过 |
| 出包 | release run `40166639700`：macOS aarch64 与 Windows x64 两个 job 均 success |
| 公开 | publish `40167625914` + promote-stable `40167719156` |

## 制品

`v0.2.8` Release（非草稿、非预发布，Latest=`v0.2.8`）：

- `OpenCodeX.Desktop_0.2.8_aarch64.dmg`
- `OpenCodeX.Desktop_aarch64.app.tar.gz` + `.sig`
- `OpenCodeX.Desktop_0.2.8_x64-setup.exe` + `.sig`
- `OpenCodeX.Desktop_0.2.8_x64_en-US.msi` + `.sig`
- `latest.json`

发布流水线对三项更新制品做了密码学核验（原文）：

- `Cryptographically verified darwin-aarch64: ... sha256=dbdca6869c9ef01b3562750b0be1cd500b784c363309f68922990dcc20420ec4`
- `Cryptographically verified windows-x86_64: ... sha256=eacace0ff56052f3cae3f13e4c1b9a163cfaf861efa85c79b47980d37b2391a2`
- `Cryptographically verified windows-x86_64-nsis: ... sha256=ebfd3ec80d6016186ad235537244da8a4e363c3845f2e3cd6e606e9e262c4bc0`
- `Target: v0.2.8; mode: stable; previous Latest: v0.2.7` → `Verified public release v0.2.8; Latest=v0.2.8`

## 边界（不得据此宣称）

- 触屏长按 500ms 与「抬手不改启停」未在真机复核。
- Windows WebView2 下的拖动与长按手感未在本轮复核（preview 分支的 backend-windows 已通过，含 EXE 冒烟）。
- 真实更新源下的下载与重启、真实凭据加密导出端到端未实测。
- 安装包未做代码签名 / Apple 公证，延续测试分发策略。

