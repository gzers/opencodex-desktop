# TEAM84 环境诊断失败的源码复核与共享交接

候选 e663369226337c9a9cad594ccd3cafd1719c3711，Actions 45973489918，Windows 制品 15187611477。待证伪判断是界面提示能区分未取得报告与取得含失败项的报告。实机点击「环境诊断」后出现「Doctor 暂不可用；已保留上次结果。」[截图](raw/full-reopen-doctor-unavailable-user.png)与[同账户采集回执](raw/collect-20261127T1447152648524Z.json)未记录这次调用的原始退出码、限时分支或报告是否生成。界面功能失败已确认，底层首错未知。复算随包运行归档、官方归档及两份上游源码成员的摘要见[原始复核](raw/doctor-source-readback-r1.json)。本轮只读源文件，未运行官方 Doctor 或发送探测请求。

- 取得报告后仍可能被判不可达。官方 2.99 的归档成员 package/src/cli/dispatch.ts 第 339、352 行先运行 Doctor，存在 FAIL 时返回 1。桌面 apps/desktop/tauri/src/infrastructure/official_cli_source.rs 第 97 行将所有非零退出映射为不可达并丢弃 stdout。apps/desktop/ui/src/features/diagnostics/useDiagnosticsController.ts 第 56 行将错误显示为通用提示。
- 官方探测可能长于桌面等待时间。官方归档成员 package/src/cli/doctor.ts 第 385、764、1611 行显示常规 Doctor 会进入 WHAM 探测，单次等待上限为 8 秒。共享桌面来源第 91 行给 wait_with_output 的等待限时为 5 秒，第 51 行使用 kill_on_drop(false)，超时分支未显式回收。现场是否命中超时、是否遗留进程均未知。

请 Mac 集成线接手共享 Doctor 来源与界面。记录同一次调用的关联标识、入口解析、子进程启动、耗时和超时、原始退出码、是否取得完整报告、脱敏或解析失败。日志只需阶段、时间、退出码和结果分类。按官方退出 1 的契约处理完整报告，并通过现有验证和脱敏后展示，保留报告表达的 FAIL。超时返回可诊断的分类并回收本次子进程，维持路径、环境与脱敏边界。用本地无网络夹具覆盖退出 0、携带完整报告的退出 1、失败且无可用报告，以及超时和子进程回收。共享修复进入同源候选后，Windows 在新的普通账户沙箱实机复验。

首次 Status 使用独立的 5 秒截止时间，仍需同次前后端计时与冷启动可信空闲证明，见 [Status 复核](first-status-trace-readback-r1.md)。当前仍拒绝 Start。Start／Status／Stop 预算为 30／5／10 秒。历史失败现场保留。accepted:false。
