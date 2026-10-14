用于bd2查询夹具失败的Windows实时诊断准备，默认只编译与检查范围，不录制、不跑Node/测试。旧轮询错过活Node，新工具先建立唯一命名的内核会话，再按原ETW QPC和进程/线程世代筛选自有测试家族；不写整机ETL，不序列化其他调度参与者ID、SID、完整命令行或无关路径。

审阅发现并修正两个采集器缺陷：提前读取session.Source会启动普通session，导致后续EnableKernelProvider拒绝；TraceEvent3.1.21通用Stop会重设全局CPU采样间隔。现在先Enable再访问Source，停止用ControlTraceW按唯一自有会话名完成，不调用通用Stop/自动StopOnDispose；返回该会话最终loss计数。写入线程异常会保留错误并排空队列，收尾逐阶段尝试，任何阶段失败不能跳过其他清理。

本机Add-Type编译与53项准备合同通过（38scope/7生命周期/8原生布局及会话名），摘要/目录/普通权限拒绝检查保持。独立错误模型验证旧调用顺序会失败，修正顺序通过；纯检查从未创建ETW会话或跑Node/测试。abc3a91 push CI38561752521、PR CI38561759502均completed/success，每轮Mac900/Windows627通过、忽略6/7，各35组，五必需job成功/main-only build skipped。PR实际检出16c781ea整tree与head相同，检出和制品身份仍分别记录。这些CI不执行C#采集器或53项本机纯准备检查，不能替代管理员录制和真实事件覆盖。

唯一录制入口需明确RecordApproved与实际管理员控制器，保留原库一次完整执行、默认并发/capture和2s/4s/5s/8s，不预热、不自提升、不改防护/全局环境。原失败PR实际checkout31e7748d整tree与bd2相同，但测试EXE不同于成功push，来源和待批准命令在docs/governance-main封存。管理员录制尚未批准或执行，失败场景取证、真实事件覆盖、排空和根因均不称通过。

草稿只新增test/smoke三个工具文件；产品和原夹具不变。不自动合并、出包或发布，原CI失败、性能failed及全部原预算/门禁保留。
