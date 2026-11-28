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
