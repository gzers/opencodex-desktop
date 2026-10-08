---
id: RUN-OPENCODEX-DESKTOP-20261006-01
title: 0.1.7 Windows 启动崩溃与图标诊断
source_refs:
  - MNT-OPENCODEX-DESKTOP-20261006-01
  - REL-OPENCODEX-DESKTOP-07
  - CODEX-THREAD:01a10cec-a70b-7fd3-883b-2bf716d9f539
---

# RUN-OPENCODEX-DESKTOP-20261006-01 运行观察

## 状态与范围

诊断与登记日：**2026-10-06**，Asia/Shanghai。受影响对象为本机安装的 **0.1.7 / Windows x64**。已复现缺少 HOME 的启动崩溃，并完成补 HOME 的进程级对照；已提取安装 EXE 的图标，确认有效图形偏小。**正式修复未实施、未验收，事故未闭合；拟修复与发版版本暂定 0.1.8**。

本记录保存实际运行观察，原因、方案和附件见 [MNT-OPENCODEX-DESKTOP-20261006-01](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261006-Windows启动崩溃与图标适配/README.md)。0.1.7 的历史发布事实继续引用 [REL-07](REL-OPENCODEX-DESKTOP-07.md)，本次不改写该版本发布时的检查结果；Windows 真机缺口以本记录追加。

## 环境与安装身份

| 项目 | 核对值 |
|---|---|
| 系统 | Microsoft Windows Server 2025 Datacenter，10.0.26100，64 位 |
| 安装 EXE | `D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe` |
| FileVersion / ProductVersion | 0.1.7 / 0.1.7 |
| EXE 大小 / SHA-256 | 7,551,488 bytes / `058d924f762b8c1f7b1909e4b63ffb79ed5a4009663a80e5bc9825829798afba` |
| 日志构建来源 | `ed99a59af25e49dbed21820d3300ebdf3273e86a`，与本地 v0.1.7 一致 |
| 用户家目录变量 | HOME 未设置；USERPROFILE=`C:\Users\15119` |
| WebView2 Runtime | 154.0.4258.53，已安装 |
| 日常应用数据根 | `C:\Users\15119\AppData\Roaming\com.gzers.opencodex.desktop` |
| 快捷方式 | 目标与工作目录正确，无参数；IconLocation 原值为 `,0` |

安装器和快捷方式的 SHA-256、版本字段及数据来源见 [observations.json](../../.adg/evidence/MNT-OPENCODEX-DESKTOP-20261006-01/observations.json)。本次未执行完整制品验签或可复现构建核对。

## 实际观察

| 本地时间 / 顺序 | 结果 | 证据与限制 |
|---|---|---|
| 2026-10-05 20:39:05 | 历史应用日志与 Windows Application Error 可见同一 EXE 异常退出 | 该条日志来自此前启动，非本次新执行；未重建用户当时点击过程 |
| 2026-10-06 00:37:47 | 默认环境启动同一安装 EXE，PID=53904，进程退出 | ExitCode=-1073740791（0xC0000409）；stderr 为 Tauri setup 返回 `application is not configured yet`；Windows Event ID=1000 |
| 2026-10-06 00:38:54 | 给诊断进程设置 HOME=USERPROFILE，另启用 RUST_BACKTRACE | PID=19216；12 秒后仍存活，主窗口句柄=2493916，Responding=True；后续诊断检查仍存活。完整渲染与业务流程未验收 |
| 随后只读图标核对 | EXE 的 RT_GROUP_ICON=32512 含七档图标，逐档像素与固定源码 ICO 相同 | 32×32 档有效图形 10×10；截图与量化一致；未修改 EXE、ICO 或系统图标缓存 |

时间来自应用 UTC 日志及 Windows 事件，按 UTC+08:00 转换。日志记录：

```text
2026-10-05T12:39:05.592Z [manager] startup version=0.1.7 commit=ed99a59af25e49dbed21820d3300ebdf3273e86a
2026-10-05T16:37:47.286Z [manager] startup version=0.1.7 commit=ed99a59af25e49dbed21820d3300ebdf3273e86a
2026-10-05T16:38:54.636Z [manager] startup version=0.1.7 commit=ed99a59af25e49dbed21820d3300ebdf3273e86a
```

启动日志写在后续家目录解析之前，不能单独当作完整初始化成功证据。标准错误原文保存在 [startup-stderr.log](../../.adg/evidence/MNT-OPENCODEX-DESKTOP-20261006-01/startup-stderr.log)。

## 已确认与未覆盖

默认启动失败与仅补 HOME 后不再退出，结合固定源码 HOME 缺失即返回错误的路径，确认本次直接启动阻断是 Windows 家目录解析缺口。ICO 资源的图形占比与源码一致，确认快捷方式图标偏小有发布资源自身的原因。

没有修改系统或用户级环境变量、安装文件、快捷方式或正式版本；HOME 仅注入诊断子进程。诊断启动使用日常数据根，应用会初始化结构并写日志，不能描述为沙箱验收或全程无数据写入。未执行官方代理启动、停止、重启、联网安装、更新或恢复流程。

诊断时曾保留补 HOME 的进程继续运行；该事实是当时快照，不保证后续仍存活。原快捷方式仍没有兼容变量补充，本轮没有让其正常启动。

Windows 10/11、完整首屏渲染、不同 DPI 下图标外观、托管安装、运行来源、代理生命周期、CLI/IPC 和 OTA 仍待验证。本记录不包含 0.1.8 修复或验收通过回执。

## 后续

2026-10-08 0.1.9 接续：先以日常 0.1.8 EXE 的独立实例采集菜单祖先样式，确认设置行／卡片的 backdrop root 限制模糊采样；相同菜单移至主壳或仅去除祖先 blur 的原始对照均使穿透文字变为模糊。`feature/0.1.9-windows-appearance@8a8397a61eee676d6b5166bd67a43930eb5e8d44` 随后实施用户三档、静态 WebGL、CSS 软边及原生激活恢复，按该提交重建同一 EXE（SHA-256 `24ea68223d9e4eecc64218ad6f5cf1b025c80bcdbbccbe83c49449015a504657`）。普通权限 DPR 1／1.5／2 和管理员 DPR 1 的完整原生矩阵通过，管理员 1.5 的首屏／画质／菜单通过但性能样本中断，2 未执行。17:39:09 出现 Terminal Services 登录／桌面启动事件；17:46:08 复核本工具 session 2 为 Disc、前台 HWND=0，三次原生激活均失败，保留该环境阻断，未用假焦点或降门禁放行。候选全部退出，日常 0.1.8 的 EXE、偏好摘要与 PID 47476／原启动时间未变；源码和文档仅本地提交，不推送、合并、安装或发布。实际证据、完整表格和接续条件见 [IMP-19](IMP-OPENCODEX-DESKTOP-19.md) 与 [维护分析 12](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261006-Windows启动崩溃与图标适配/分析/12-0.1.9玻璃与画质修复及Windows复验.md)。

2026-10-08 追加现场观察：日常安装已为 Windows 0.1.8 / `cffca3a`，EXE SHA-256=`ecfa04d4fe73150f99831af6df5fc4a490ef598cb3b823cefd3e5065d869dac5`，与 MSI 候选身份一致；日常 PID 47476 自 2026-10-06 14:21:46.6541258+08:00 起连续运行。使用同一 EXE 的隔离实例，系统 API 返回客户端区域动画关闭，页面 reduced-motion=true，保存 high 实际 mid；已确认前台样本中 SVG 静止，WebGL 0 次绘制。只对隔离页面模拟 no-preference 后，约 3 秒取得 5 个不同 SVG 采样状态与 174 次 WebGL 绘制，光场恢复；恢复默认媒体并切 CSS 内存状态后静态光可见，进一步单层及圆角/柔化对照定位 CSS cloud-a 的裁剪硬边。系统减少动态的降档策略、静态首帧门控和 CSS 硬边分别登记；未改源码或日常配置，未排查完下拉玻璃。媒体/样式对照恢复，两组隔离实例退出，日常 EXE、偏好摘要及原进程启动时间未变。详细证据与另一端综合分析输入见 [10-Windows 动效与光场本机诊断](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261006-Windows启动崩溃与图标适配/分析/10-Windows动效与光场本机诊断及综合分析交接.md)。

2026-10-06 10:37–10:38（Asia/Shanghai）首屏续修复验：从 `9262a5b` 重建同一 release EXE，在 PowerShell 7.6.5 普通与管理员权限下各观察 30 秒；实际 `tokenElevated` 为 false/true，均取得 loopback CDP 主 target、Vue 首屏挂载、Tauri 命令成功和截图，启动日志匹配源码提交。两组候选进程及监听结束后已释放。日常 0.1.7 EXE、快捷方式摘要与实例 PID 19216/启动时间未变；未改代码、未合并或发布。实际证据与 CI 完成快照见 [06-Windows 双权限首屏复验](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261006-Windows启动崩溃与图标适配/分析/06-Windows普通与管理员首屏复验.md)。

2026-10-06 追加：用户授权后，`feature/0.1.8-windows-fixes@785e80d` 候选在本机移除 HOME 的独立数据根中保持运行 30 秒，Vue 主界面实际渲染，Tauri 关于、发现、偏好和数据根命令成功；运行日志明确记录 0.1.8 与该提交。本机 Node/npm 被正确发现，未安装官方运行时时展示引导。最终 EXE 七档图标与来源 ICO 解码像素一致，画布边界覆盖率全部 100%；它不是安装快捷方式的多 DPI 验收。候选测试进程由冒烟脚本退出，日常 0.1.7 未替换。结果、摘要与剩余集成门禁见 [04-0.1.8开发测试与交接](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261006-Windows启动崩溃与图标适配/分析/04-0.1.8开发测试与交接.md)。

按维护专题的 [0.1.8 拟修复范围与验证方案](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261006-Windows启动崩溃与图标适配/分析/03-0.1.8拟修复范围与验证方案.md) 准备后续实现。新观察以追加方式记录，不把本次兼容启动改写为正式修复成功。机器证据入口为 [manifest.json](../../.adg/evidence/MNT-OPENCODEX-DESKTOP-20261006-01/manifest.json)。
