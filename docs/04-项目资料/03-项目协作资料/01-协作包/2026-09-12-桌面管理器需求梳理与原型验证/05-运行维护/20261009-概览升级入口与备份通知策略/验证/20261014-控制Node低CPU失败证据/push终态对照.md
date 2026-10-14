# bd2 push 终态与失败对照

2026-10-14；独立追加，原PR失败封存及其“push进行中”快照不改。

push38537161445现completed/success，独立headbd2f6eb1；Mac35组/900通过/6忽略、Windows35组/627通过/7忽略，五必需job成功、main-only build条件跳过。完整REST/日志ZIP/CRC/SHA、安全条目和actual checkout独立核验，原三七个测试在本轮均通过；PR38537166560库523通过/7失败/4忽略保持，不因该成功解除。

成功push七control均在query返回时已有control-started标记。版本success/nonzero的observer结束约706ms；metadata success/nonzero/overflow约143–165ms；timeout约2.02–2.03s。各自user/kernel累计CPU见新analysis，不混入失败runner样本。两轮同SHA没有放宽截止/并发/断言，也没有手动重跑求绿；不同runner的执行差异仍需失败线程/调度/初始化/文件I/O证据解释。

成功version/success的control在marker已写时user CPU仍显示0ms，说明此累计计时值有采样/计量精度限制，不能用CPU=0认定没执行任何用户代码。此前失败“低CPU”只描述原API观测值；5秒仍active且标记缺失是联合线索，不推断精确阶段或Defender/loader/scheduler原因。新成功对照进一步限制了单指标归因。

本轮两个bd2CI均已结束；代码bd2保持固定，不继续盲目加脚手架或改产品求绿。下一步Windows按[后台诊断提示](Windows后台诊断提示.md)取得限定自有PID的线程/调度/I/O原证据，现有唯一desktop验收与性能采样不干扰。新产品源码51a/诊断bd2与旧候选aae身份分开，尚无新candidate dispatch。

PR保持草稿、main不合并；原性能failed、全部预算/八门禁/FZ50许可、真实数据/日常版和原TASKrevision2/in_progress/pending/空验收不变。Mac仅后台，原生前仍需Windows实例0，goalactive；不签名、发布、稳定通道/核心晋升或关闭证书。
