# TEAM85 冷启动空闲判定交接

主线及冻结候选 9a452c37d3005119042d81ee1ae8f62c4920c4d0，治理基线 fa336d097。原普通账户现场和实机回执见 [result.md](result.md)。仍为 accepted:false。

## 固定源码中的冷启动缺口

- `apps/desktop/tauri/src/infrastructure/status_source.rs:156` 先读取托管运行程序身份，识别固定 2.99.0 后转到 Windows 专用 Status。运行身份读取失败也可能直接返回 Unreachable。
- `apps/desktop/tauri/src/infrastructure/windows_official_lifecycle024.rs:96,364,574` 显示唯一允许创建 owner 槽的生产入口是 Start。Status 只查已有槽并要求其中有 owner。缺槽、槽锁竞争、槽内无 owner 均不可达。在这个分支中，只有已持有 owner 且已证实正常退出的一类状态能返回 stopped。即使内置环境正常，通过托管身份检查的新管理器也没有冷启动空闲成功路径。
- `apps/desktop/tauri/src/modules/status/mod.rs:686` 只在状态采样有空闲证据时投影 can_start。真实 Full 就绪后重新采样仅用 218 ms 即返回不可达，Start 保持禁用。这个返回符合源码中的冷启动缺口。当前发布候选没有暴露具体拒绝阶段，单凭回执不能断定是前置身份检查还是 owner 查询先拒绝。
- `apps/desktop/tauri/src/infrastructure/official_cli_source.rs` 将来源、工作目录、子进程启动、等待错误及非零退出折叠为 Unreachable，`apps/desktop/tauri/src/modules/doctor/mod.rs:30` 再映射 NotConfigured。Doctor 3 ms 的表层错误未提供这些分支的证据。

## 请集成线确认共享语义与改动归属

Windows 负责原生侧只读冷启动证明和实机复验。请在治理交接中明确共享 Status 如何承接「从未启动且已确认空闲」，以及 Doctor 原因分类由谁修改。建议复用已存在的空闲动作门控，由原生可信证据驱动。区分冷启动空闲与本轮正常 Stop 的退出证明。若现有接口已可表达该契约，请指出确切调用点与字段。

Status 的 5 秒原截止时间涵盖身份检查和空闲证据采集。对固定安装根、当前 SID 与同一隔离 HOME 的安全判断须使用真实可核验信息。单凭内置包就绪、无进程内 owner 或两条记录均缺席仍不足以证明空闲。权限不足、记录不确定、只出现 PID、监听归属不清、时间耗尽均保持未知与 Start 禁用。Start 在自身 30 秒窗口内重新验证前置条件。Stop 保持 10 秒及正常退出证据语义。

建议候选补充首次 Status 的调用、耗时和首个拒绝阶段，Doctor 保留脱敏错误类别及非零退出类别。复验使用新的普通账户隔离根，保留本轮 Full。缺包交互可并行进行，原生 HTTPS 的请求证据单独核验。

## 普通账户定向回执

[原始回执](../team85-status-doctor-20261128-r1/status-doctor-20261128T0440590869063Z.json) SHA-256 为 `44699d92d2fb767af748fa8d8297be8d287e9fb060425e669633328ef01c4af3`。marenben 的原 SID 在 RDP session 3 执行，探针退出码 0，总耗时 2219 ms。身份解析确认 2.99.0，耗时 240181 µs。直接调用无 Owner 的 Windows Status 在 82 µs 返回 Unreachable，完整和轻量入口分别在 239159 µs 和 242886 µs 返回 Unreachable。此为已确认的原生复现，首次 GUI IPC 未捕获。

相同 SID 下 Doctor 的工作目录 `<Full>/home` 返回 PathNotFound。受控入口、当前运行时选择文件和 `<Full>/data/opencodex/home` 均存在。固定源码在派生子进程之前检查工作目录，因而该现场在预检即被拒绝。尚未验证 Doctor 通过预检后的退出码及输出。请共享集成线回复上节归属及冷启动证据契约，下载仍需同源受控测试输入。保留现有 Full 原现场，本条仅归档回执。

## 现有共享接口与定向回归

固定候选已有可复用的空闲投影。`apps/desktop/tauri/src/modules/status/mod.rs:797` 将 `CollectError::ManagedHomeIdle` 转成 `Stopped` 与 `idle_observed: true`，并清空 PID 和端口。`snapshot` 在第 686 行以最近采样的空闲证据计算 `can_start`。该投影可承接 Windows 冷启动的可信空闲结果，无须先新增 UI 状态或改变 Stop 返回值。请集成线确认 Windows 向该接口提交结果所需的原生证据及写集。

2.99.0 的 Windows 路径在 `status_source.rs:158` 提前交给原生 Status，之后的通用无端点分支不会运行。原生 `runtime_records_absent` 只检查两项记录的缺席，不能直接用来签发空闲。实现时保留 PID、文件访问错误、锁竞争、未知监听归属和超时的拒绝行为，并在 Start 自己的动作窗口中重新验证，避免使用过时的 UI 采样开放启动。

[本轮原生回归原日志](../team85-status-doctor-20261128-r1/native-regression.log) 已覆盖 PID-only、端口设置写锁以及原截止时间，6 项通过、4 项隔离场景跳过。其中写锁夹具用 750 ms 子窗口，验证超时且未发布端口设置、未写 HOME、未创建 owner。PID-only 断言保留原 PID 内容且数据根没有新写入。普通账户实机两项仍按 [状态表](result.md) 记录为 blocked，复用现有回归即可。

## 首次 Status 与 Doctor 普通桌面回执补充

固定源码仍为 9a452c37d。新 Full 在 marenben 普通令牌、RDP session 3 启动，管理器 PID 108100。完整包 attempt f1ea9eda-70af-409f-9497-d31914d7e229 达到 supplied，随后 discover_environment 3236.9 ms 返回 ready。首次 Status 241.1 ms 返回 unreachable 与 canStart:false，287.5 ms 内 UI 显示未知、重试、诊断，Start 缺席。重试 257.6 ms 结果相同。5 秒内返回与未知门禁已实测，可信空闲和真实启停仍阻塞。

Doctor 先因隔离夹具的用户工作目录缺席于 1.6 ms 返回 code 1。仅给新夹具创建该目录后进入命令等待，7564 ms 返回 code 7。请集成线确认 Doctor 默认工作目录及诊断错误归属。最小需求是让只读诊断选择经过校验的隔离工作目录，并区分来源解析、入队、创建进程、进程退出、stdout EOF 与超时。源码的 5 秒等待窗口保留，IPC 总耗时单独记录。固定源码丢弃 stderr 且 kill_on_drop(false)，超时后的子进程退场尚无证据。

外层采集器记录 observerExitCode:null 和 Observer exited unsuccessfully。IPC 原始记录 sentinelVerified:true、firstError:null、failures:[]，其摘要与外层记录一致。两份原件均保留，采集器退出码仍未知。观察器及父进程已退出，管理器仍打开。未执行 Start，失败现场保留。

原件及生成入口均为逐字节归档，由接收目录复算 SHA-256 后与归档副本比对。

- [status-doctor-desktop-r2-20261128T0504445053024Z.json](raw/status-doctor-desktop-r2/status-doctor-desktop-r2-20261128T0504445053024Z.json) 6b4995188002ec69d65845dad5ec8490101b0cca73375f5ac7201db3e8709438
- [status-doctor-desktop-r2-20261128T0504445053024Z-ipc.json](raw/status-doctor-desktop-r2/status-doctor-desktop-r2-20261128T0504445053024Z-ipc.json) d5105dcc79f804dc0d02143d809f3db73215e54f3bc73ec6f40e9b5498735443
- [observe-status-doctor.mjs](raw/status-doctor-desktop-r2/observe-status-doctor.mjs) f1d755b17cb47c42321d49b1a2ad62f33dc7fc7c6cb743ace1108b760ecafbf6
- [run-status-doctor-desktop.ps1](raw/status-doctor-desktop-r2/run-status-doctor-desktop.ps1) 9ce94d034504a07fec74d5b6b9144381244181b505868e8dde300ca641e2a1df
## Doctor 等待阶段与集成接口

Windows MSVC 本地无网络探针在同一 5 秒窗口内分别观测子进程退出和 stdout EOF。正常夹具两者均在 4 ms 完成。慢子进程在 5000 ms 仍未退出，最终为 6256 ms。继承管道的夹具中直接子进程 8 ms 退出，5002 ms 时 stdout 仍未关闭，6262 ms 才收到 EOF。三个断言通过，夹具进程残留为零。首次编译因 Future trait 未导入失败，修正后通过。保留[源文件、编译日志与原始回执](raw/doctor-wait-phases-r1/sha256.json)。候选实机超时阶段仍未知。

随包 official-runtime.tgz 摘要为 b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c。源码 package/bin/ocx.mjs 用继承 stdio 启动 Bun，package/src/cli/doctor.ts 的 probeWham 默认会访问远端 usage 接口，单项超时 8000 ms。源码未找到可用的本地专用选项。两条机制都可能产生实机的等待超时。现有记录无法确认该次请求是否发出、实际等待阶段或子进程退场。

请 Mac 集成线确认三个共享改动的归属和接入提交。Doctor 使用验证过的隔离工作目录，默认诊断提供本地结果并明确划分主动网络检查，记录队列、创建进程、子进程退出、stdout EOF 与超时阶段。保留原 5 秒等待窗口，另记 IPC 总耗时及进程退场结果。Windows 待确认接口后复验，避免用真实远程请求定位阶段。冷启动可信空闲与缺包受控下载的接入需求沿用前文。
