---
id: REL-OPENCODEX-DESKTOP-07
object_kind: release.candidate
state: verified
title: 0.1.7 能耗与内存优化稳定发布验收
summary: 0.1.7 已公开并晋升稳定 Latest；双平台构建、完整回归、三个更新包旧公钥验签与匿名端点一致性通过。后台绘制停止及面板闲置回收有验证，完整前台动画短测未证明 CPU 降幅；长期内存与 Windows 实机仍有验证边界。
source_refs:
  - IMP-OPENCODEX-DESKTOP-17
  - DMD-OPENCODEX-PERF-017
  - REL-OPENCODEX-DESKTOP-06
  - ed99a59af25e49dbed21820d3300ebdf3273e86a
---

# 0.1.7 能耗与内存优化稳定发布验收

用户明确授权按文档治理新建分支、开发测试、合入并发版。本轮通过 Git / GitHub Actions 实际发布，project-governance 保存事实与关闭证据。发布时间为 **2026-10-04T19:43:57Z**（北京时间 **2026-10-05 03:43:57**）；记录日期以 UTC 为准。

## 发布结果与身份

- 代码分支 `codex/0.1.7-performance` 已快进合入并推送 `main`，源码及 `v0.1.7` 均为 `ed99a59af25e49dbed21820d3300ebdf3273e86a`，基线为 `4c5afb5189c02cba6a84b7e0faebbdbf3153d66d`。
- GitHub Release **403177453** / [v0.1.7](https://github.com/gzers/opencodex-desktop/releases/tag/v0.1.7) 已公开，`draft=false`、`prerelease=false`，稳定 Latest 从 v0.1.6 晋升至 v0.1.7。
- `promote-stable/v0.1.7` 显式触发稳定晋升；没有改写历史版本标签或重建已发布的旧资产。
- macOS arm64 与 Windows x64 的正式包均由版本标签构建。已从公开 macOS 更新包内只读核验 version/build 为 0.1.7，主二进制含上述提交；SHA-256 为 `9a13f20faf20d51103d185fbf82a220812fe7d8e5fe7b119f2690643a4b23ed7`。

## 交付范围

前台可见概览保留原动画、shader、DPR 和帧率；切页、退出激活、隐藏、最小化停止装饰续帧，返回时相位连续。缓存与复用减少不变参数计算、相同 SVG 属性写入和 uniform 上传；不降低效果来换取性能数据。

周期、超时、退避、面板 TTL 与字节限额统一配置。稳定运行时轻量健康/就绪检查，完整诊断缓存 60 秒；托盘事件驱动、低频退避兜底与请求合并；隐藏未编辑面板满 5 分钟只回收子 WebView，重开恢复路径与位置，代理服务继续运行。诊断输出及日志尾读有字节上限，超时回收本次诊断子进程。

具体实现、未交付候选方案与运行边界见 [IMP-17](IMP-OPENCODEX-DESKTOP-17.md) 及 [实施与验证结果](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261004-能耗与内存性能分析/分析/04-0.1.7实施与验证结果.md)。

## 验证门禁

- 本地前端 78 文件 / **373 通过**，类型检查及生产构建通过；后端 **566 通过、5 忽略、0 失败**，格式及 clippy 通过；发布脚本 **29 通过**。
- [main CI 37227977433](https://github.com/gzers/opencodex-desktop/actions/runs/37227977433)：frontend、backend、backend-windows、release-tools、build 均成功。
- [Release 37228658880](https://github.com/gzers/opencodex-desktop/actions/runs/37228658880)：macOS arm64、Windows x64 均成功。
- [Publish Release 37229374830](https://github.com/gzers/opencodex-desktop/actions/runs/37229374830)：签名、资产身份、公开端点和 Latest 验证成功后晋升稳定通道。
- 集成线另行匿名下载固定清单与稳定 `/releases/latest/download/latest.json`，内容相等、版本 0.1.7；API 确认同一个公开 Release 为 Latest。重新下载三个不同更新包，大小和 GitHub SHA-256 元数据一致，并使用 **v0.1.3 标签中的原公钥**逐个完成真实 minisign 验签。

| 更新资产 | 字节数 | SHA-256 |
|---|---:|---|
| `OpenCodeX.Desktop_aarch64.app.tar.gz` | 3911087 | `dee09c460c62d11dd5c1481868438c8a4050ec7b264746bee6b123b18f4b1d4c` |
| `OpenCodeX.Desktop_0.1.7_x64_en-US.msi` | 3940352 | `647d5753cf9e7c4e3c1b16297e93ef21614fd7ff5945d0df2c9af9e26863b994` |
| `OpenCodeX.Desktop_0.1.7_x64-setup.exe` | 2844720 | `e41965cbe2647ad0d1ffe9402de7db31758ed2fe5c613d837b7d6c06a95c7511` |

公钥未改变，解码文本 SHA-256 为 `30190687026fdc7b7e223884315baa52f6d42c1c37d447b8631e9597c1ecdfee`。旧公钥验签通过证明签名兼容，不等于旧客户端实机安装/重启已验收。

## 性能结论与适用边界

隔离 macOS GUI 验证未编辑夹具面板闲置回收、重开位置恢复及编辑后超过 TTL 仍保留文字。后台/最小化短时 CPU 接近空闲，自动化覆盖后台无装饰续帧和暂停恢复。但同效果前台短测为 0.1.6 的 34.329%、0.1.7 的 36.057%，**未证明前台降幅**；旧版隐藏也已受操作系统暂停，不能据两版隐藏差异给出节能比例。

面板 WebContent 释放是已观察事实；共享内存快照不能相加成全应用总量。没有两小时/多设备或持续固定业务负载验证，未排除长期泄漏，也未测量真实功率、续航或 GPU 硬件利用率。合成面板验收不能代替官方面板全部编辑流程；停止/无可信 PID 状态仍逐次完整 CLI，连接元数据可能缓存 60 秒；编辑过的面板采用保守驻留保护。

本地最终候选 App 有明确 0.1.7 / ed99a59 启动身份及隔离 GUI smoke，公开发布资产有独立验签与版本核验。本轮**未安装替换日常 App、未重启官方代理**。日常桌面 PID 44788 与官方代理 PID 18504、启动时间、2.69.0 身份连续，只读 `/healthz`、`/readyz` 正常；不把日常 0.1.6 的运行观察记成 0.1.7 已接管的实测。

Windows 后端 CI、构建与公开更新包验签通过，Windows GUI 安装/重启未覆盖，超时回收仅直接子进程，后代未实测。OTA minisign 签名不等同于 Apple 开发者签名/公证或 Windows Authenticode。

## 兼容与回退

沿用现有更新公钥与偏好 schema，没有新增数据迁移或官方运行时包升级。保留 v0.1.6 及其资产作为回退来源；稳定降级需另外授权并核验，不通过本次正常晋升入口强行降级。已安装的新版本不会因 Latest 回退而自动降级。

证据入口：[本轮证据索引](../../.adg/evidence/PERF-017-20261005/manifest.json)，关键原始证据为 `public-release-verification.json`、`publish-diag.txt`、`release-build-jobs.json`、`publish-jobs.json`、`local-source-identity.json`、`cpu-summary.json`、`panel-edit-protection.json` 和 `daily-runtime-continuity.json`。治理 TASK/RCP/CERT 保存于 `.adg/work/objects/`，长期实施与需求结论保存在 Markdown。
