# 缺包请求与准备结果取证交接

Windows 冻结候选 e663369226337c9a9cad594ccd3cafd1719c3711，构建 45973489918，制品 15187611477。复核远端 main 与治理合同仍固定于 TEAM84。待证伪判断为现有普通桌面采集可从同一次缺包确认还原请求、实际 HTTP 响应及准备终态。

- `apps/desktop/ui/src/features/environment/preparation.ts:83` 的重试入口将失败准备的 attemptId 和 downloadConfirmed:true 传给后端。`apps/desktop/tauri/src/modules/runtime/preparation.rs:79` 核对原 ID 并生成新的准备 ID。两者代表前序缺包状态和获准的新尝试。
- `apps/desktop/tauri/src/modules/runtime/bundle_download.rs:44` 发起 HTTPS GET。第 83 行把实际非成功响应汇总成「官方地址未提供完整运行资源」。`apps/desktop/tauri/src/modules/runtime/startup_bundle.rs:247` 的 app.log 记录终态文本，没有请求 ID、确认事实或响应状态。普通账户只读采集可以留住这条首错，关联字段缺失。历史 HEAD 结果另存，当前实机 GET 尚未采集。
- 启动时 `startup_bundle::start` 用空 transfer；档案存在直接走本地校验和供应。档案缺席且 transfer 为空时在 GET 入口之前返回 bundle_archive_missing。这是冻结源码的控制流核对，离线桌面结果仍待实测。

请共享集成线用现有沙箱诊断路径记录：原缺包 ID、受理后生成的 ID、确认布尔值，首个实际 GET 及每次响应的状态码或连接错误类别，以及同一新 ID 的准备终态。无请求时可由同一记录的完整边界核验暂缓和重开。只保留阶段、单调耗时和结果类别，省略地址查询参数、响应正文和凭据。可与 [首次 Status 计时交接](first-status-trace-readback-r1.md) 一起进入下一同源候选。

当前目标重新要求实际离线准备。TEAM84 原回执仍按其合同记录 skipped。在当前 RDP 环境仅发现 Hyper-V 服务运行，Get-VM 对当前账户拒绝访问，未证实有独立离线 Windows 桌面。Full 仍在普通桌面运行，日志待同账户采集。Missing 暂缓、重开与确认下载仍待桌面取证。首次 Status、可信空闲及 30／5／10 秒正向启停全部保留原判定，accepted:false。
