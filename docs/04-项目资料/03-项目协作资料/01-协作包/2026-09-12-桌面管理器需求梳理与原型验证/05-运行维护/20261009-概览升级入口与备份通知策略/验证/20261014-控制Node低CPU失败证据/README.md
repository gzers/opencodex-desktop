# 控制 Node 低 CPU 与未进入脚本的失败证据

2026-10-14，Mac后台，无原生实例/性能测试或新包。源码诊断bd2f6eb10d58115235e35d575507bb69f44666da，production与51a相同。

PR CI38537166560 completed/failure：Mac35组/900通过/6忽略；Windows库523通过/7失败/4忽略，后续suite未执行。七失败为version success/nonzero/timeout、metadata success/nonzero/overflow/timeout，全部未建立夹具父子句柄。WindowsAPI诊断代码已真实编译并执行，不代表查询/清理测试通过；fmt/Clippy前置步骤通过，产品门禁仍失败。实际PRcheckout见原checkout日志，不把关联head当独立head制品。完整REST/日志ZIP大小SHA/CRC、安全条目与原job字节压缩记录已核验，失败不重跑或删除。

## 控制进程原始快照

各组分别创建独立std::process控制Node，不加入生产queryJob。其同一保留句柄/创建FILETIME在before-query、query-returned、observer-returned三个时点一致，API错误均null；WaitForSingleObject0返回258（WAIT_TIMEOUT）、GetExitCodeProcess259（STILL_ACTIVE）并exitFILETIME=0，共同证明这些时点进程未结束。不可仅按259判断存活，真实PID/handle/creation证据均保留在analysis与原日志。

| 自有control场景 | 原observer结束经过ms | user CPU ms | kernel CPU ms | control-started标记 |
|---|---:|---:|---:|---|
| metadata/nonzero | 5657 | 0 | 0 | 缺失 |
| metadata/overflow | 5401 | 0 | 15.625 | 缺失 |
| metadata/success | 5464 | 0 | 31.25 | 缺失 |
| metadata/timeout | 5418 | 0 | 15.625 | 缺失 |
| version/nonzero | 5051 | 0 | 15.625 | 缺失 |
| version/success | 5128 | 31.25 | 31.25 | 缺失 |
| version/timeout | 5153 | 31.25 | 15.625 | 缺失 |

七组的control标记在三个时点全缺，累计CPU远小于约5秒wall time；该证据支持“独立进程进入JS前/初期执行进展很低”，反对只归因生产Job清理/取消的解释，但不证明线程为何等待。仍缺加载器/线程wait-chain、调度/磁盘/系统保护工具时间窗的原始证据，不能认定Defender、CPU抢占、Node错误或产品缺陷。JS require/module启动/本地写标记都属于尚未区分的阶段，低CPU不等于没有I/O或没有运行任何原生初始化。

query实际墙钟结束部分超过2秒，仍低于原<4秒清理上限；host调度与同步spawn也影响可观察时间。原2秒截止、5秒observer/8秒watchdog、清理/exit结果和默认并发保持；观测增加小开销，不能作正常资源/性能通过样本。

## 下一证据与边界

Windows固定bd2源码，在现有原生性能采样之外后台执行原夹具，绑定测试EXE/Node路径摘要，收集限定自有PID/时间窗的线程/调度或ETW过程，区分进程创建后初始化等待与读写迟延；保留默认并发/截止/断言及失败。只观察自身测试进程，不按名称结束系统或其他用户进程；不启动第二个desktop、不调整防护策略/全局环境、不执行FZ50包生命周期。已有进程快照来自已结束runner，不能拿其PID在用户宿主定位目标；本机新PID必须重新按句柄与创建时间核验。

push38537161445最后仍in_progress，MAC/前端已成功、Windows后端仍处置中，原run_rest保留；不是静默重派或新候选。后续若push成功，只记其对照，不解除本PR真实失败。当前只有产品51a、诊断bd2源码和旧aae包不同身份，无新candidate dispatch。PR4已同步实际失败draft/open/unmerged，maineae未合并。

原macOSCPU+1.817629pp/RSS+117.008965MiBfailed、全部启动/闲置/主线程/缓存原始计数/真实周期/双平台安装保留/事件备份矩阵不变。原TASK3878字节/revision2/in_progress/pending、空acceptance结果、RCPoverallfailed/goalactive保持；Windows原唯一实例不操作、Mac原生先协调Windows实例0。无真实数据迁移、日常替换、FZ50脚本、稳定核心晋升、合并/签名/发布或完成证书。
