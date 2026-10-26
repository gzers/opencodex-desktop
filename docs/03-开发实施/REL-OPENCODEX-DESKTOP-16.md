---
id: REL-OPENCODEX-DESKTOP-16
object_kind: release
state: verified
title: 0.2.11 保存流程统一发布
summary: 0.2.11已公开并晋升stable/Latest；PR19合并main 09c4eedbc、tag v0.2.11；保存流程收敛为四态一条线并新增放弃出口，前端824项与门禁18项及双平台CI/签名核验通过。
source_refs:
  - 09c4eedbc0637b922e1cef644b99801b0bd2d791
  - https://github.com/gzers/opencodex-desktop/pull/19
  - https://github.com/gzers/opencodex-desktop/releases/tag/v0.2.11
---

# 0.2.11 保存流程统一发布

2026-10-26，Asia/Shanghai。用户在 0.2.10 装机后被「已有保存计划待核对」挡住，核对与撤回两条路都被漂移拒绝。诊断后确认根因是**没有终态出口**，并按用户确认的「四态一条线」流程实施。

## 源码

工作树 `ocx-022-model-selector`、分支 `fix/0.2.11-save-flow`，PR [#19](https://github.com/gzers/opencodex-desktop/pull/19) 合并 main：`09c4eedbc0637b922e1cef644b99801b0bd2d791`，tag `v0.2.11`。

## 流程

```
① 未保存修改 → [保存] → [确认保存]
② 正在保存（唯一不可操作态）
③ 已写入 · 待重启 Codex → [我已重启，检查]
④ 未完整写入 → [重新检查] / [撤回本次未完成写入] / [放弃这次]
```

## 关键改动

| 层 | 内容 |
| --- | --- |
| 后端 | `check()` 去掉全库 `reject_incomplete`（无关旧计划不再挡住本次核对） |
| 后端 | 新增 `abandon()` → 终态 `abandoned`：prepared 拒绝、已终态幂等、进白名单、过滤 `latest()`、`history()` 可见但不可恢复、继续参与版本保护判定 |
| 命令 | 新增 `model_policy_abandon` |
| 前端 | `partialPlan` 作为第 ④ 态唯一判定；`driftBlocked` 识别漂移拒绝；三出口按钮 |
| 前端 | 弹窗去掉底部重复「关闭」 |
| 夹具 | `?partialSave=1` 可复现第 ④ 态，供实屏核对 |

## 验收

| 层 | 结果 |
| --- | --- |
| 前端类型 | `vue-tsc --noEmit` 通过 |
| 前端单测 | 159 文件 / 824 通过 / 10 跳过 |
| 前端门禁 | `gate:release` 18 / 0 / 0 |
| 后端 | 39 套件 / 1144 项 |
| fmt / clippy | 绿 |
| CI（PR #19） | frontend / backend / backend-windows / windows-smoke / release-tools 全绿 |
| 出包 | release 40277954455（双平台均 success） |
| 公开 | publish 40279606504 + promote-stable 40279699607 |

## 制品与签名核验

`v0.2.11`（非草稿、非预发布，Latest=`v0.2.11`）：

- `darwin-aarch64` sha256=`d82123dd…c9969b26`
- `windows-x86_64`（msi）sha256=`7469145e…42b75d8f`
- `windows-x86_64-nsis`（exe）sha256=`8d03f9b1…c1d077275`
- `Target: v0.2.11; mode: stable; previous Latest: v0.2.10` → `Verified public release v0.2.11; Latest=v0.2.11`

## 过程中修掉的 CI 缺陷

首轮 `backend-windows` clippy 失败：`cannot find function cli_fixture`。新增的三个测试依赖该函数（`#[cfg(unix)]`），未按既有约定连同用例收口到 `cfg(unix)`。修法见提交 `78de175d`；复跑后全绿。

## 边界

- 未改真实保存事务；写入 / 备份 / 核对 / 回滚时序与 0.2.10 相同。
- 核对判据仍是「运行目录 vs 那次写入内容」；用户改过配置时会如实报不一致。
- 触屏与 Windows WebView2 实屏未复核。
- 安装包延续未做操作系统代码签名 / Apple 公证。

