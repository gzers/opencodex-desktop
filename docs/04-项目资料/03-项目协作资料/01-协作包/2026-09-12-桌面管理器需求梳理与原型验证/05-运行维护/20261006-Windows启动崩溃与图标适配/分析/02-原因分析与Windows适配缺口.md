# 02-原因分析与 Windows 适配缺口

源码审阅固定为 **`v0.1.7@ed99a59af25e49dbed21820d3300ebdf3273e86a`**。下列链接指向该提交；文件摘要、行号和观察分类见 [source-review.json](../../../../../../../../.adg/evidence/MNT-OPENCODEX-DESKTOP-20261006-01/source-review.json)。本目录保存分析，不复制实现源码。

## 问题分级

| 编号 | 问题 | 证据分类 | 建议优先级 |
|---|---|---|---|
| WIN-01 | 启动强制读取 HOME，缺少 Windows 家目录回退 | **本机启动对照与源码确认** | P0：阻断启动 |
| WIN-02 | Windows ICO 有效图形过小，与应用 PNG 不一致 | **安装 EXE 资源提取、逐帧像素核对与用户截图** | P1 |
| WIN-03 | Windows 只有编译门禁，缺少最终 EXE 启动验证 | **源码确认**；未重跑 CI | P1 |
| WIN-04 | 默认运行发现沿用 macOS 路径与无扩展名工具名 | **源码与本机工具位置核对**；完整发现流程待验证 | P1 |
| WIN-05 | 托管入口与 npm 环境沿用 Unix 脚本和 PATH | **源码确认**；Windows 安装与启停待验证 | P1 |
| WIN-06 | Windows 本机 IPC 端点未实现 | **源码明确声明并跳过服务** | P2；本版范围待决定 |

## WIN-01：家目录解析是直接启动阻断

[lib.rs:158–164](https://github.com/gzers/opencodex-desktop/blob/ed99a59af25e49dbed21820d3300ebdf3273e86a/apps/desktop/tauri/src/lib.rs#L158) 在 setup 中直接读取 HOME，未设置或为空即返回 `AppError::NotConfigured`。该值是管理器的家目录前置条件错误，不证明用户尚未配置官方 OpenCodex。

本机 HOME 未设置、USERPROFILE 存在；默认启动正好返回上述错误。Tauri setup 失败后发生 panic，而 [Cargo.toml:71–76](https://github.com/gzers/opencodex-desktop/blob/ed99a59af25e49dbed21820d3300ebdf3273e86a/apps/desktop/tauri/Cargo.toml#L71) 的 release 配置使用 `panic="abort"`，因此表现为进程退出。仅给同一 EXE 的子进程补 HOME 后不再复现该启动阻断。

修复应统一采用跨平台家目录来源，明确 Windows 与 Unix 的取值策略，并审查发现、日志等其他读取 HOME 的入口。缺少有效绝对目录时应保留可定位错误，不能回退到当前工作目录或空路径。全局新增 HOME 只能兼容旧包，不能替代产品修复。

## WIN-02：发布 EXE 使用的 ICO 没有跟 PNG 保持一致

[tauri.conf.json 的图标清单](https://github.com/gzers/opencodex-desktop/blob/ed99a59af25e49dbed21820d3300ebdf3273e86a/apps/desktop/tauri/tauri.conf.json) 同时包含 PNG、ICNS 与 ICO。Windows 使用的 [icon.ico](https://github.com/gzers/opencodex-desktop/blob/ed99a59af25e49dbed21820d3300ebdf3273e86a/apps/desktop/tauri/icons/icon.ico) 有七档尺寸，但每档透明边距都过大；安装 EXE 的资源像素与它一致。

[tray_icon.rs](https://github.com/gzers/opencodex-desktop/blob/ed99a59af25e49dbed21820d3300ebdf3273e86a/apps/desktop/tauri/tests/tray_icon.rs) 已对 `tray-16.png`、`tray-32.png` 检查有效图形覆盖率，却没有检查应用 ICO，因此 PNG 或托盘图标测试通过也无法保证 Windows 快捷方式图标正常。

建议从统一应用图标源重新生成多档 ICO，并检查最终 EXE 的实际嵌入资源。视觉基准应与应用 PNG 一致；覆盖率阈值、alpha 阈值与小尺寸可读性同时核对，避免只通过透明边界测试却保留难以辨认的图形。

## WIN-03：编译通过没有覆盖启动

[ci.yml:44–54](https://github.com/gzers/opencodex-desktop/blob/ed99a59af25e49dbed21820d3300ebdf3273e86a/.github/workflows/ci.yml#L44) 的 `backend-windows` 仅执行 `cargo check --workspace --all-targets`。[app_startup_wiring.rs:1](https://github.com/gzers/opencodex-desktop/blob/ed99a59af25e49dbed21820d3300ebdf3273e86a/apps/desktop/tauri/tests/app_startup_wiring.rs#L1) 则通过 `cfg(unix)` 排除了 Windows。

这些检查能发现编译层平台错误，不能执行 Tauri setup，更不能验证安装后的实际快捷方式、WebView 和图标。本次故障需用**去除 HOME、保留有效 Windows 用户目录、启动最终 EXE**的门禁防回归；数据根应使用已有测试沙箱机制隔离。

## WIN-04：默认发现路径不适用于本机

[discovery_paths.rs:4–11](https://github.com/gzers/opencodex-desktop/blob/ed99a59af25e49dbed21820d3300ebdf3273e86a/apps/desktop/tauri/src/types/discovery_paths.rs#L4) 固定寻找 `~/.local/bin/node`、`npm` 和 `ocx`，启动阶段还加入 [Homebrew 候选](https://github.com/gzers/opencodex-desktop/blob/ed99a59af25e49dbed21820d3300ebdf3273e86a/apps/desktop/tauri/src/lib.rs#L169)。[安装依赖发现](https://github.com/gzers/opencodex-desktop/blob/ed99a59af25e49dbed21820d3300ebdf3273e86a/apps/desktop/tauri/src/commands/runtime.rs#L34) 也消费同一 macOS 默认路径。

本机实际 Node 为 `C:\Users\15119\.nvmd\bin\node.exe`，预期的 `.local\bin\node` 不存在。补 HOME 能让管理器启动，但不会使上述发现路径变正确。

建议集中到平台适配层，识别 Windows 的工具名、受控候选目录和 npm 前缀；保留“显式指定 > 托管安装 > 自动发现”的来源优先级。现有受控发现不读取任意 PATH，是否扩展该规则属于后续方案选择，不能在诊断阶段直接改写契约。

## WIN-05：托管启动器与子进程环境仍是 Unix 形式

[install.rs:1642–1655](https://github.com/gzers/opencodex-desktop/blob/ed99a59af25e49dbed21820d3300ebdf3273e86a/apps/desktop/tauri/src/modules/runtime/install.rs#L1642) 生成 `#!/bin/sh` / `exec` 入口。Windows 发布不能仅因 Rust 可编译，就认为该入口能在无 Unix shell 的环境执行。

[install.rs:742–760](https://github.com/gzers/opencodex-desktop/blob/ed99a59af25e49dbed21820d3300ebdf3273e86a/apps/desktop/tauri/src/modules/runtime/install.rs#L742) 清空环境后以冒号拼接 PATH；[兜底路径](https://github.com/gzers/opencodex-desktop/blob/ed99a59af25e49dbed21820d3300ebdf3273e86a/apps/desktop/tauri/src/modules/runtime/install.rs#L53) 也是 Unix 目录。

需要验证 Windows 原生 Node 调用或启动器、npm 入口、路径分隔、含空格及中文路径、必要环境键与参数传递。当前只确认实现缺少对应分支，未实际执行联网安装或官方代理启停，不能记录为已复现安装失败。

## WIN-06：Windows IPC 尚未实现

[ipc/mod.rs:11–15](https://github.com/gzers/opencodex-desktop/blob/ed99a59af25e49dbed21820d3300ebdf3273e86a/apps/desktop/tauri/src/modules/ipc/mod.rs#L11) 明确只编译 Unix 端点；[启动接线](https://github.com/gzers/opencodex-desktop/blob/ed99a59af25e49dbed21820d3300ebdf3273e86a/apps/desktop/tauri/src/lib.rs#L258) 在非 Unix 上不启动 IPC 服务。

这不是本次 HOME 崩溃原因。是否在 0.1.8 实现 Windows IPC，或明确展示未支持状态并延后，应作为范围决定单列；不能将开关存在、`ocxd.exe` 随包安装等同于服务可用。
