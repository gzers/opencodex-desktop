---
id: REL-OPENCODEX-DESKTOP-15
object_kind: release
state: verified
title: 0.2.10 状态展示统一化发布
summary: 0.2.10已公开并晋升stable/Latest；PR18合并main ed023c2be、tag v0.2.10；23个后端phase收敛为7类结论、状态区3处→1处，前端823项与门禁18项及双平台CI/制品签名核验通过。
source_refs:
  - ed023c2bea8b2f19e6fa482de06b9f1ab838924b
  - https://github.com/gzers/opencodex-desktop/pull/18
  - https://github.com/gzers/opencodex-desktop/releases/tag/v0.2.10
---

# 0.2.10 状态展示统一化发布

2026-10-25，Asia/Shanghai。用户指出模型页同一屏三处状态互相矛盾，并问「能不能用这个状态编码做统一状态展示」；分析后确认根因是**没有状态码**，随后以目标模式贯通文档→开发→测试→发布。

## 源码

工作树 `ocx-022-model-selector`、分支 `fix/0.2.10-status-unification`，PR [#18](https://github.com/gzers/opencodex-desktop/pull/18) 合并 main：`ed023c2bea8b2f19e6fa482de06b9f1ab838924b`，tag `v0.2.10`。

## 七步实施

| 步 | 交付 |
| --- | --- |
| 1 | `lib/saveStates.ts`：23 phase → 7 类结论 + 终态；tone 五类；客户端 5 态单一文案源 |
| 2 | `components/ui/UiStatus.vue`：只接语义 tone，复用 `.state-pill` + `.state-*` |
| 3 | 模型页 3 处 → 1 处（表格起始 y 307 → 263，原型 256） |
| 4 | 扩展页 / 详情头部 / 列表 tone 收编（`data-tone` 10 → 8，余下为检查判定） |
| 5 | 补 `.tag.warning` / `.tag.success` |
| 6 | `qa/status-contract.mjs` + 单测门禁 |
| 7 | UI 规范 §41 + 数据与状态 §5.9 |

## 顺带修的两处真实缺陷

1. 进入模型页时保存结果被请求两次（`latestPolicy()` 与通知 target 各拉一次）。
2. 状态框与客户端状态同时提示「待重启」时，同一句话出现两遍。

## 验收

| 层 | 结果 |
| --- | --- |
| 前端类型 | `vue-tsc --noEmit` 通过 |
| 前端单测 | 159 文件 / 823 通过 / 10 跳过 |
| 前端门禁 | `gate:release` 18 / 0 / 0 |
| 后端 | workspace + integration 39 套件 / 1141 项 |
| fmt / clippy | 绿 |
| CI（PR #18） | frontend / backend / backend-windows / windows-smoke / release-tools 全绿 |
| 出包 | release 40198473154（双平台均 success） |
| 公开 | publish 40199577060 + promote-stable 40199738970 |

## 制品与签名核验

`v0.2.10`（非草稿、非预发布，Latest=`v0.2.10`）：dmg / app.tar.gz(+sig) / x64-setup.exe(+sig) / x64_en-US.msi(+sig) / latest.json。

- `darwin-aarch64` sha256=`923adc8d…6b38495`
- `windows-x86_64`（msi）sha256=`2e33ed94…22be02d`
- `windows-x86_64-nsis`（exe）sha256=`bf0e229f…eaa158fa4`
- `Target: v0.2.10; mode: stable; previous Latest: v0.2.9` → `Verified public release v0.2.10; Latest=v0.2.10`

## 边界

- 未改真实保存事务本身；写入、备份、核对目录与回滚时序与 0.2.9 相同。
- 概览页检查判定文字色（ok/bad/busy）属另一语义域，未并入词表。
- 触屏与 Windows WebView2 实屏未复核。
- 安装包延续未做操作系统代码签名 / Apple 公证。

