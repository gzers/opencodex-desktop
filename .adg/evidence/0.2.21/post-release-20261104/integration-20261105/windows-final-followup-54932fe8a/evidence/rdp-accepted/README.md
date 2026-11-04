# RDP 环境接受后的材质性能复验

2026-11-05，用户明确“rdp也可以接受了”。后续接受本机RDP作为材质性能验收环境，不再要求物理桌面；原帧时间与停绘预算没有豁免。旧TASK、回执、原始失败作为当时事实保留，本次另建WINDOWS-RDP任务绑定AC-P21-02，不关闭整版验收。

固定产品bd7c0fe8c、同一GNU EXE SHA256 76dc1049f4b6c3995993b2c1457ae06e1d3627669e20bd8c62ffdd98054b778a，在Windows Server2025/26100、WebView2/156、Intel UHD770 D3D11、DPI192、RDP下复测。每个前台场景30秒，B=16.667ms，p95≤33.333ms、over6B=0沿用原预算。

| 场景 | p95 ms | 超100ms帧 | 结果 |
| --- | ---: | ---: | --- |
| 高档概览静止 | 162.6 | 140 | 失败 |
| 高档详情滚动 | 144.0 | 107 | 失败 |
| 高档公共弹窗 | 171.7 | 159 | 失败 |

后台最小化后2秒draws=0，strategy.animated=false；恢复前台后2秒draws=15，animated=true，Mica恢复，检查通过。此前31.5/28.8ms样本仍保留在raw/prior-static-scroll.json，不能用该单次较快样本覆盖本轮失败或断言稳定提升。此次接受RDP没有改变预算，也没有把环境接受等同于性能通过。

首轮启动使用旧discovered运行源，实际解析到宿主10100页面，已立即停止并记为无效样本；未向日常服务提交操作或配置写入。正式采样前，隔离manager-state/runtime.json固定为只读status-reviewed.exe，无runtime-port记录时报告stopped。候选已按精确路径、PID10444和创建时间确认后退出，对应9444端口清理。历史误停日常服务事件仍在父目录isolation-incident.json，不在本次改写为通过。

本次只提交文档/证据，代码HEAD b2cb82cde不变。MSVC本地正式构建、正式Codex默认adapter的安全隔离、切源快速重开失败及其他集成门保持原未完。最终集成者据最新用户环境决定回写共享IMP/DMD；本线不改共享事实。
