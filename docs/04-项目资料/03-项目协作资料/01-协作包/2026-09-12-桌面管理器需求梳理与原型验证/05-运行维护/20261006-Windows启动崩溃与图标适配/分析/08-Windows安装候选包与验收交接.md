# Windows 安装候选包与验收交接

2026-10-06，Asia/Shanghai。承接 [07 的交付门禁](07-macOS原生UI与安装回归及合并打包门禁.md)。用户随后请求“给我发行我在 windows 安装”，发行授权已收到；正式门禁尚未齐备，先准备 Windows 可安装候选制品，用于补齐实际安装验收。

## 来源与执行

应用实现基线仍为 feature/0.1.8-windows-fixes@9262a5bedbb1a5f454f32afb7418b959fa8c721f。新增 feature/0.1.8-windows-package，只添加候选出包工作流及安装 smoke，不改应用实现。最终候选出包提交为 cffca3afc29b8b251f3f14535f997b5d08560cb2。

工作流固定 Tauri CLI 2.11.4，生成并归档 Cargo.lock，以同一锁文件运行 fmt、clippy、Windows 后端回归及 NSIS/MSI 构建。临时 Actions runner 实际安装每种包，验证安装后 EXE 与构建 EXE 经唯一包类型标记修改后的预期 SHA-256 完全相同、版本为 0.1.8、日志源码提交与当前 HEAD 相符；沿用原有无 HOME、隔离根、Vue 首屏及 Tauri invoke 门禁。随后卸载并检查主程序已移除。只有这些步骤成功才上传安装包；证据在失败时也保留。

首轮 a2b346f2f19230f2acb60761269edd622fdfd99c 的完整 CI 通过，候选工作流在出包命令解析阶段失败：npm 的 PowerShell Tauri 入口将 --locked 当作 Tauri 自身参数，未生成或上传安装包。保留 [失败任务 37409082551](https://github.com/gzers/opencodex-desktop/actions/runs/37409082551)。后续用原生 node 直接调用 tauri.js，参数探针确认 Cargo 接收到 build / --locked / --bins / tauri/custom-protocol / --release，再重跑全流程。没有去掉依赖锁或跳过门禁。

第二轮 0f8ea8eb 的 [完整 CI 37409930674](https://github.com/gzers/opencodex-desktop/actions/runs/37409930674) 通过，[候选出包 37409930678](https://github.com/gzers/opencodex-desktop/actions/runs/37409930678) 成功生成两种包，但安装身份检查失败并阻断上传。NSIS 已卸载，未宣称首屏通过。检查固定 CLI 所用 tauri-bundler 2.9.4 的 bundle.rs：打包时把唯一 __TAURI_BUNDLE_TYPE_VAR_UNK 原位替换为 NSS/MSI，随后恢复 target/release 中的原 EXE。因此原始与安装后摘要不应相同。7a00b125 按该规则构造预期字节并比对完整 SHA-256，不忽略任何其他变化。额外改动、错误标记、缺失/重复标记及未知包类型均必须失败，身份回归 5 项在本机通过。两次失败记录保留为历史。

第三轮 7a00b125 的 [候选出包 37411592656](https://github.com/gzers/opencodex-desktop/actions/runs/37411592656) 中，NSIS 新装、精确字节身份、原生首屏与卸载全部通过；MSI 安装日志显示 INSTALLDIR 被此前 NSIS 的登记位置覆盖，文件实际安装到旧测试目录，指定目录检查失败并阻断制品上传。MSI 随后卸载。固定 bundler 的 MSI 模板搜索 HKCU 下记忆位置，NSIS 模板只有删除应用数据时才移除该位置登记。本轮仅测试应用卸载，不删除用户数据。因此 cffca3af 改为在全新 runner 先测 MSI，再由 NSIS /D 指定独立位置；两种安装门禁均保留。此轮是安装测试顺序问题，不写成 MSI 应用首屏通过。

最终任务：[候选出包 37412506074](https://github.com/gzers/opencodex-desktop/actions/runs/37412506074)、[完整 CI 37412506075](https://github.com/gzers/opencodex-desktop/actions/runs/37412506075)。候选任务已完成为 success，MSI 与 NSIS 均通过新装、最终 EXE 首屏和卸载。完整 CI 第二次执行已完成为 success。

完整 CI 37412506075 首次执行在 https_scheme_reaches_the_transport_layer 失败：本机 TLS 端点首字节为 None，预期 Some(22)，原因未确认；同一提交的锁定出包回归通过。保留首次失败日志与任务快照，在不改测试、不跳过断言的情况下仅重跑失败 job；第二次执行的 Windows 后端 335 项通过（5 项原有忽略），release EXE 构建与无 HOME 原生首屏均通过。完整 CI 的前端 375 项 / 79 文件、CDP 7 项、macOS 后端 574 项（5 项原有忽略）、发布工具 29 项也通过。main 限定的 build job 正常跳过。单次重跑通过不构成 TLS 原因已修复的结论。

## 可下载安装包与证据

[下载 Windows x64 候选包](https://github.com/gzers/opencodex-desktop/actions/runs/37412506074/artifacts/11390500494)（需登录 GitHub）。下载 ZIP 并解压，普通安装运行 OpenCodeX Desktop_0.1.8_x64-setup.exe；需要 MSI 时选择另一文件。制品有效期至 2026-10-20 12:22:24（Asia/Shanghai）。过期后应重新核验构建，不把旧链接作为长期下载入口。

| 文件 | 字节数 | SHA-256 |
|---|---:|---|
| OpenCodeX Desktop_0.1.8_x64-setup.exe | 2880768 | b07fac2b82aa02134081d85298da504de20edb08ac511173e8ce197231e5e6b6 |
| OpenCodeX Desktop_0.1.8_x64_en-US.msi | 4026368 | 68a927eff84c680a0a5175f931858e72a5a974ff6700def0df41edf8c91f952b |

已从 Actions 下载制品并逐字节计算摘要，与 runner 的 installer-verification.json 一致。安装器 ZIP artifact 摘要为 c17efd53d4f9fb7b4547712c2ce94a38d3826cda7718cb4dc58dc7e656287beb；它与表中单文件摘要分别代表不同对象。MSI / NSIS 安装后 EXE 摘要分别为 ecfa04d4fe73150f99831af6df5fc4a490ef598cb3b823cefd3e5065d869dac5 / 096bc61aa77376636ac03208d9dcad0adf6e70dfc37ae288b28cd2574d0134d1，两者完全符合唯一 bundle 标记变换后的预期。安装/卸载退出码均为 0，卸载后主程序已移除。

两次安装均在无 HOME、独立数据根观察 30 秒，主窗口保持响应；Vue 已挂载，关于页回读为 Windows / 0.1.8，Tauri 命令与沙箱初始化通过，启动来源严格匹配 cffca3af。下载的 MSI/NSIS 截图均为 1028×729，人工查看为正常概览与未接入引导。runner 是 Windows Server 2025（win25-vs2026，镜像 20260925.250.1），不据此宣称最终包的 Windows 普通/管理员矩阵通过；06 的双权限证据针对原 release EXE。

出包工具为 Rust/Cargo 1.99.0、Node 24.21.0、npm 11.19.0、Tauri CLI 2.11.4；归档 Cargo.lock 的 SHA-256 为 2a03998419a8a197bd5703d5f0bf109ec800bda12cd13d621584b1cd9f5b9dcf。锁定依赖的 Windows 后端 335 项通过、5 项原有忽略，身份检查 5 项通过；fmt、clippy 与前端生产构建通过。

机器证据见 [WIN-018-INSTALLER-20261006](../../../../../../../../.adg/evidence/WIN-018-INSTALLER-20261006/manifest.json)：保留历次失败、最终构建/CI 日志、工具链、锁文件、安装身份、首屏 JSON/截图与制品元数据；不提交安装器二进制、沙箱缓存或依赖目录。文本日志按原 UTF-8/UTF-16 编码解码，归档为 UTF-8，去 ANSI、转 LF 并去行尾空白，manifest 同时登记输入摘要和归档摘要；JSON、PNG 和锁文件保留原始字节。

## 交付边界

候选包为未做 Windows 代码签名的 x64 NSIS/MSI，先供安装验收。一般安装选择 NSIS 的 *-setup.exe；MSI 留给需要该格式的部署场景。下载后核对制品 SHA-256 与验证记录；Windows 若显示 SmartScreen，按既有未签名测试包说明处理。

Actions runner 的新安装、首屏和卸载不代替用户设备 0.1.7 升级、Win10/11 双权限、多 DPI、快捷方式/任务栏图标缓存、数据保留、完整代理生命周期或签名 OTA。07 中这些门禁继续开放；不能把可下载候选包写成正式发行验收通过。

没有合并 main、创建版本发行标签、公开 GitHub Release 或修改 latest.json / beta 通道；Latest 继续为 0.1.7。日常 Mac 0.1.7 未被替换或停止。实现及测试流程提交在 feature 分支，本文和机器证据只提交 docs 分支。
