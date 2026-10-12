# 内屏前台保护中断（不计性能验收）

复用 deb740 的唯一隔离测试 PID31501，浅色 / 中档静态 / mesh / 100%。内屏位置已确认；本轮在 90 秒预热内中止，资源采集未启动，资源样本为零。窗口仍可见，只失去前台；原因未确定，不认定软件焦点缺陷。用户此前说明的人为前台切换不作软件问题。

## 实际轮次

- guarded-mid-inner：172 条前台事件，先 ready，最后有效样本 elapsed 18.644822 秒，首次无效 elapsed 18.750805 秒、UTC 02:10:27.817139。frontmost=false，窗口可见；monitor exit 3，resource_collector_exit=null，collection_completed=false。90 秒预热未完成，未产生 summary.json 或资源样本。
- guarded-mid-finder / prearmed：均在 10 秒内未前台就绪，monitor exit 2、not_ready_within_ten_seconds；无资源采样。保存各自原始日志与前后选定设置，没有这两轮的完整首末 UI，不计验收。
- 三轮首末 preferences SHA 均为 aed374985a30e077a7093a4ea5983b5f7b61bce40cee7ccbf5cc957f19e9e8aa。真实画质为 mid；guard-context 中旧 manifest 的 high 原样保留。launch.json 表示已有进程的场景预热起点，不是应用启动或启动性能样本。

## UI、位置与来源边界

内屏位置只读查询 UTC 02:09:33 的结果来自工具对话记录，未保存原始 JSON；placement-transcript-derived.json 明示这一来源。主窗口 x=-1453、y=211、1180×760，完整落在内屏 x=-1728、y=0、1728×1117 内。NSWorkspace 当时确认目标前台；AX 查询返回 -25204 不可用，不解释为相反的焦点结论。只读 Swift 方法保留，没有为补档重新查询或伪造原始结果。

内屏轮 start.jpg / end.jpg 已目视核对为同一浅色概览准备态，OpenCodex 未接入。开始 UI 时间 UTC 02:09:58.193，结束 UTC 02:11:10.090；截图不能证明两者之间持续前台。start.ax.txt 仅为“树未变化 / 概览焦点”的增量，不是完整 AX；end.ax.txt 是完整结束树。Finder 准备轮保留设置页与概览起点 UI，没有补造结束 UI。

代理打开的 Finder test-install 标签已关闭，原有“下载 / 存储 / work”三个标签保留。没有启动新测试实例、退出日常应用、修改系统设置或迁移真实数据；这一轮不再请求或占用前台重试。

来源文件按字节复制并列入 source-map / manifest，run-metrics 从目标前台 JSONL 独立计算。保护工具方法见[前台采样保护工具准备](../20261012-前台采样保护工具准备/README.md)。空日志 / stop marker 留在原临时目录，不提交应用二进制、配置载荷、WebView profile 或其他应用元数据。

## 门禁与下一步

该轮无效，不提供 CPU / RSS 或根因结论，也不覆盖[既有十分钟性能比较未通过](../20261012-deb740-十分钟性能对照/README.md)。前次比较亦缺连续前台与受控配对，只保留其原限制；调查线索尚未形成因果。暂无据此修改产品源码的依据。

后续先取得中档静态 / 高档 CSS / 必要的高档 mesh 持续前台诊断，确定原因后再改源码。正式门禁继续要求受控十分钟配对 CPU 增量 ≤0.5 百分点、RSS 增量 ≤10 MiB，冷 / 热各 ≥20 次、可交互 p95 增量 ≤100ms 且 ≤5%，无 >50ms 主线程任务，以及真实 stable 24h / beta 6h / 面板 24h 周期和睡眠 / 重启 / 离线 / 慢网 / 并发 / 通道隔离。双平台原生、安装、真实更新、事件与签名门禁继续开放；Windows runner 的 elevated token 不计普通 / 管理员双权限通过，macOS ad hoc 副本不计正式签名。

TASK-182 / IMP-20 保持 in_progress；review:scheduler-performance-0110 failed，八项 required checks 未整体闭合，acceptance_results 为空。不晋升核心事实，不替换日常安装，不执行发布。
