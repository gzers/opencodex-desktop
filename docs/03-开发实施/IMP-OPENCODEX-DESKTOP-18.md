---
id: IMP-OPENCODEX-DESKTOP-18
object_kind: implementation.change
state: completed
demand_ids: ["DMD-OPENCODEX-DESKTOP-MANAGER"]
title: 0.1.8 Windows 启动与运行适配修复
summary: Windows 适配与 CDP 续修已推送至 9262a5b；对应 CI 和普通/管理员 PowerShell 7 首屏复验通过。未合并、未制作安装器或发布，后续集成门禁保留。
completion_summary: "原始 feature 开发、Windows 本机测试及提交完成；远端前端、macOS 后端与 Linux 发布工具通过，Windows final EXE smoke 在 CDP target 获取阶段失败。原始完成状态不代表续修或发布门禁通过。"
---

# 0.1.8 Windows 启动与运行适配修复

2026-10-06，Asia/Shanghai。用户明确授权“目标模式完成开发、测试。再提交”，并指定后续将在另一设备合并分支、打包。本轮实施分支为 **`feature/0.1.8-windows-fixes`**，从 **`main@5a0859ad2e6d44f4ca5751eedfb5a1b7eb8c5d09`** 创建；文档与证据仅在 `docs/governance-main` 登记。维护依据为 [MNT-OPENCODEX-DESKTOP-20261006-01](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261006-Windows启动崩溃与图标适配/README.md)。

本轮交付目标是**修复开发、运行测试和可继续集成的提交**。0.1.8 为候选代码版本；本轮不合入 main、不创建版本标签、不生成 NSIS/MSI/DMG、不发布或晋升通道。为验证启动而编译 release EXE，不等于制作安装器或发布制品。

## 实施选择

| 问题 | 最终实现 |
|---|---|
| WIN-01 | 通过 `dirs` 与基础设施适配器解析平台家目录，拒绝空或相对路径；Windows 不再把 HOME 缺失等同于未配置。Windows release 主程序使用 GUI 子系统，后台 CLI 不弹控制台 |
| WIN-02 | 从现有应用 PNG 重生成 16/24/32/48/64/128/256px ICO；新增全部尺寸解码与有效图形占比测试；Windows 托盘使用彩色应用图标，macOS 保留模板 |
| WIN-03 | Windows 后端真实回归、Common Controls v6 测试清单、无 HOME 的最终 EXE 冒烟和 WebView 首屏/命令检查；特性分支 CI 执行测试，不生成安装器 |
| WIN-04 | 受控候选覆盖 `.local/bin`、NVM Desktop `.nvmd/bin`、用户 npm/Node 目录、明确的 NVM/Node 前缀与 Program Files；不搜索宿主 PATH，保留显式 > 托管 > 发现优先级；Windows 指引不再推荐 Homebrew |
| WIN-05 | Windows 托管入口为 `ocx.cmd`，UTF-8 代码页、禁用延迟展开、绝对 Node 与脚本路径；受控 PATH 使用平台分隔符，清空环境后只补明确允许的系统/用户目录；保留官方命令退出码 |
| WIN-06 | 本轮不实现 Windows IPC；关于页返回真实平台，Windows CLI 设置明确未支持，禁用开启并由后端拒绝启用请求 |

实测进一步处理了 Windows 的目录句柄释放时序：前缀原子重命名仅对共享/访问冲突有界重试 2 秒，不先删除目标，超出窗口仍明确失败。真实联网安装与离线导入均经过版本探针及来源回读；没有把“文件存在”当作安装成功。

共享修复包括：实例取得锁之后才清空身份文件，避免竞争失败者破坏已有实例身份；家目录脱敏兼容 Windows 路径；路径断言按 `PathBuf` 比较；TLS 回归通过本机握手字节验证，避免依赖英文系统错误和固定关闭端口。

最终核验还发现构建来源缓存缺口：工作树的符号 HEAD 内容在提交时不变，仅跟踪 HEAD 文件会保留旧提交号。`build.rs` 现在同时跟踪 HEAD、当前分支引用和 packed-refs；Windows 冒烟会比对源码提交与启动日志，不接受陈旧构建身份。

九态动画的生产实现未改变。黄金样本从优化前固定提交 `4c5afb5189c02cba6a84b7e0faebbdbf3153d66d` 重新捕获，按 1e-6 数值精度比较，避免不同平台数学库的最后几位序列化差异。Windows 的优化前与当前 36 个采样帧已作对照，不以当前实现自生成结果替换旧基准。

## 验证与交接

代码提交为 `536a791`（运行适配）、`d3b86a7`（ICO）和 `785e80dada6928c9643ee69f16056d414cd7304e`（测试门禁）。前端 375 项、Windows 后端 331 项通过；格式、clippy、前端类型与生产构建通过；真实官方 2.78.0 联网安装与离线导入往返通过。最终 EXE 的无 HOME 隔离首屏与 Tauri 命令检查通过，启动日志对应 `785e80d`，七档嵌入 ICO 均与来源像素一致。

完成状态仅指原始 Windows 设备上用户授权的开发、测试和提交。初始交接时 GitHub 认证阻断推送，因此提供 Git bundle；随后用户已推送特性分支。2026-10-06 核实 CI run `37397271349`：前端、macOS 后端、Linux 发布工具通过；Windows 编译与后端回归通过，但 final EXE smoke 无法取得主 WebView CDP target。macOS 原生 UI/安装回归仍未由后端 CI 替代。

用户随后授权“回写文档、目标模式开发、提交推送、交 Windows 测试”。续作保持首屏渲染门禁，在 Mac 修改与验证通用部分，在 Windows 复验原生行为；复核、实施与交接结果见 [05-Windows CI 首屏门禁复核与修正](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261006-Windows启动崩溃与图标适配/分析/05-Windows-CI首屏门禁复核与修正.md)。本记录的原始 `completed` 状态不宣称续作已经通过 Windows 门禁。

续修提交 `d2c3801` 显式配置 smoke WebView2 浏览器参数及隔离数据目录；`9262a5bedbb1a5f454f32afb7418b959fa8c721f` 保留首屏门禁并补齐 CDP 与进程诊断，两笔均已推送特性分支。本机前端 375 项、Rust 574 项通过（另 5 项原有忽略），CDP 协议 7 项及 PowerShell 语法检查通过；未执行 Windows 原生 smoke。新 CI run `37403376481` 已启动，当前证据不声明 Windows 门禁通过。续修机器记录见 [.adg/evidence/WIN-018-CDP-20261006](../../.adg/evidence/WIN-018-CDP-20261006/manifest.json)。

2026-10-06 Windows 复验追加：保持代码为 `9262a5b`，重新编译 release EXE（SHA-256=`d02e2d373043a113aa56989c45a7557e3f232fa1228eec3cd1b7acfd73e8c30c`）。PowerShell 7.6.5 普通/管理员组的实际提升 token 分别为 false/true，两组均在无 HOME、独立沙箱中通过 Vue 首屏、Tauri invoke、构建来源和截图门禁；协议 7 项、Windows 后端 335 项通过（5 忽略），格式/Clippy 与前端构建通过。CI run `37403376481` 已完成为 success，Windows 首屏步骤明确通过。以上更新前段历史“待复验”快照，不改写原 CI 失败。细节见 [06-Windows 双权限首屏复验](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261006-Windows启动崩溃与图标适配/分析/06-Windows普通与管理员首屏复验.md)，机器证据见 [.adg/evidence/WIN-018-CDP-WINDOWS-20261006](../../.adg/evidence/WIN-018-CDP-WINDOWS-20261006/manifest.json)。本轮只测试和文档回写，日常 0.1.7、特性代码和 main 均未改动，未制作安装器或发布；安装升级与原生 UI 等后续门禁保持。

实际检查、制品身份、提交和边界见维护专题的 [04-0.1.8开发测试与交接](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261006-Windows启动崩溃与图标适配/分析/04-0.1.8开发测试与交接.md)。机器证据保存在 [.adg/evidence/WIN-018-20261006](../../.adg/evidence/WIN-018-20261006/manifest.json)。

本机是 Windows Server 2025 x64，不推导为全部 Windows 10/11 的安装验收。安装器升级、桌面快捷方式缓存、多种 DPI、真实官方代理服务生命周期和 OTA 仍由后续集成阶段执行。macOS 原生验证不能用 Windows 本机通过结果替代。

CHANGELOG 依仓库归属仅在 main 维护；用户合并时补写 `[0.1.8]` 的实际发布范围与未覆盖项，再完成出包和版本交付。0.1.7 已安装实例保持原样，本轮不替换日常应用或停止其代理。
