---
id: PERF-OPENCODEX-DESKTOP-01
object_kind: performance.baseline
state: verified
title: macOS aarch64 Release 性能基准
summary: 记录 TASK-57 采集的启动到窗口就绪、运行态 RSS 与 app bundle 尺寸；这是本地候选基准，不宣称已签名发布。
source_refs:
  - TASK-OPENCODEX-DESKTOP-57
  - RCP-OPENCODEX-DESKTOP-57
  - 4d97949178f47f0b4d1d70cebe4bb91731d23d5e3a16b17a7c1d1a634b3ba4bb
---

# PERF-OPENCODEX-DESKTOP-01 macOS aarch64 Release 性能基准

## 结论

| 指标 | 结果 | 方法 | 判定 |
|---|---:|---|---|
| 启动到窗口就绪 | 687 ms | `open` 前后毫秒差 + Quartz on-screen layer 0 轮询 | 通过 |
| 运行态常驻内存 | 113.3 MiB | 窗口稳定 2 秒后 `ps rss` | 通过 |
| app bundle 尺寸 | 7.0 MiB / 6,303,744 bytes | `du` 读取 `OpenCodeX Desktop.app` | 通过 |

## 环境

- 平台：macOS arm64（`Darwin-arm64`）
- 构建：`aarch64-apple-darwin` Release app bundle
- 代码：`4d97949178f47f0b4d1d70cebe4bb91731d23d5e3a16b17a7c1d1a634b3ba4bb`
- 方法：warm quit 后启动；Quartz 轮询 owner 为 `OpenCodeX Desktop` 且 layer 为 0 的窗口；窗口稳定 2 秒后采集 RSS。
- 机器可读证据：`.adg/work/evidence/task57-perf.json`
- 视觉证据：`.adg/work/evidence/task56-release.png`

## 候选限制

1. 这是单次本机基准，不是统计性发布性能结论。
2. 生产签名与公证未执行；公开发布前必须单独授权。
3. app bundle 不包含官方面板素材；生产面板承载方式仍受 R-17 发布前冻结门禁。
4. 本记录只支持候选验证，不构成公开发布决定。
