# 管理员首次实际录制与采样握手修复

管理员已实际启动随机命名GPU/DesktopComposition实例，2026-11-05本地12:21:23.630开始45秒。ETL原件仅留本机，大小/SHA见diagnostic-summary.json；原collector-result仍为failed，因stop合并超过20秒观察限。后查87260已退出、该命名实例not recording；没有重发stop/cancel或停止应用。原始失败不改写为成功。

本线采样器遗漏PS5 Set-Content UTF8 BOM兼容，JSON.parse异常被空catch吞掉，管理员成功录制时仍等待，未生成scene-started或样本。12:27前按原sampler.json核对PID63992、path、创建时间、owner/session及命令行，停止该确切自有采样器，原管理器75988继续保留；旧入口退役。此次不是用户未执行。

独立本机原生OpenTrace/ProcessTrace消费者完整读取86150096事件，exit0，ETL头EventsLost=0。仅按已知manager/WebView后代与session2 DWM的header PID筛选，317814条中317622为DWM，目标manager/WebView合计192条，没有可重建的目标持续present链。DWM调度间隔与present间隔在JSON列出，但不能替代目标帧时延；kernel事件header PID也不等于工作归属。无完整30秒样本/前台证据，本次仅诊断，不定位为DWM、GPU驱动或产品根因，不宣称性能改善。机器级ETL、原始系统事件及数值指针未上传；聚合结果和摘要回交。较慢的独立tracerpt读盘分析被原生消费者替代，精确核对其PID95112/路径/创建时间/owner后停止这个自有只读辅助进程，非WPR或应用。

代码5fb4e83a9加入可复用采样器和BOM读入兼容，绑定目标identity、随机实例、开始时间及45秒窗；过期明确拒绝，不再静默等候。采集器PS5/PS7统一无BOM UTF8，stop单独观察60秒，超时留下精确PID/create/参数及最终输出，不取消或kill录制。PS5最初Start-Process返回null ExitCode的测试失败保留；缓存原生handle后实际PlanOnly exit0。另有真实PS5两秒子命令强制100ms观察超时，确证最终stdout保留且自然退出，无kill。Node两项测试通过；实际旧PS5 BOM反例可解析并因过期拒绝，未采样。产品apps树和原30秒/B/p95≤2B/over6B0预算未变。

新隔离目标身份仍绑定同一已核验MSVC管理器75988，普通采样器66636在12:30:53.996就绪、等待3600秒，管理员入口为admin-paired-20261105-1228/RUN-ADMIN.ps1。等待真实新录制，不把PlanOnly计为性能通过。正式Codex默认adapter仍缺可证明一次性Windows账号/会话，不操作当前日常MSIX。TASK继续in_progress、RCP总体failed，无CERT；不合并main或发布。
