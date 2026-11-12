# 0.2.24 无运行记录的停止诊断准入修正

revision: 1；2026-11-12。归IMP-34 P24-03/04，承接[状态路径修正](status-context-paths-r1.md)、[恢复状态重建](recovery-readonly-view-r1.md)及[树清单](client-tree-inventory-r1.md)。代码main 70d2a2778dc937dd41402b03c8a0c266fb060ff8。新采集器Stopped/Required诊断问题本批闭合；整版accepted=false，客户端实际文件恢复未完成。

## 两个确切回归

status-context-paths移除了错误的paths.config→HOME映射，并在明确Stopped时清运行端口/PID。这是正确展示，但admit_managed_cli_without_record仍依赖mapped.facts.opencodex_home及mapped.pid，形成两种错误：

- 官方2.99正常停止报告只有paths.config=getConfigPath()，没有独立HOME字段，映射后HOME未知，旧准入拒绝；首次collector于是保留Loading。前批真实原包新采集器Required但Loading正是此触发。
- 原停止payload含残留非空PID时，展示映射会清掉它，旧准入因此可能漏拒；不能拿已经清洁过的UI展示当原始进程缺席证明。

修改前已执行既有managed_cli_admission_tests，3项中1通过2失败，原stdout/stderr及binary摘要保留；不是仅凭源码推测。前批误把目录HOME也作为paths.config的正例，本批按官方字段角色纠正为只有实际config.json文件才能位于paths.config；独立HOME字段仍必须是目录HOME本身，不兼容错误的文件/目录混用。

## 生产行为

无记录CLI诊断准入直接读取原payload各字段：opencodexHome/opencodex_home/home必须逐项等于启动上下文H；paths.config必须等于H/config.json，不能仅取parent猜HOME。至少一份明确身份字段存在，所有已提供别名都必须合法且一致。Windows保留原受控路径等价比较，Mac按原绝对路径比较，不扫描其它HOME或旧安装目录。

原ready/isReady/proxy.running若存在，必须明确false；true及错误类型均拒绝。原pid/processId/process_id/proxy.pid若存在，必须null，任何整数/字符串/零/空字符串/false均不当缺席。映射的运行态仍须Stopped，不能接受另一用户的running报告或被官方全局发现误带来的endpoint。运行/就绪观测仍走原受保护记录和同连接认证，不用CLI全局fallback授予运行权限。

此次只修诊断消费，不签发停止/恢复许可。原程序来源核验、无运行记录前后复核、在途变更门槛、记录出现时拒绝CLI fallback、5秒查询预算以及普通Stop/30/10秒预算保持；不能因为状态可读就跳过完整进程/监听/文件写者证明。UI路径映射继续不把config.json当HOME，已停止运行端点继续空，原保存端口不改。

## 验证

31项相关检查通过：4项准入（其中包含全部PID别名和5种非null值反例、字段角色/矛盾路径/错误类型/外部运行报告）、19项状态映射、3项显式D/H路径与去抖/失败、5项状态DTO。最终编译、格式及全目标严格Clippy通过。一次只读sed定位用了错误工作目录，仅工具输出保留；未改生产/测试判定，也未冒独立stderr。

一轮新的隔离原包产品链使用准确最终源码：Start15245ms、Status1092ms、正常Stop及计划登记8412ms，原30/5/10秒不变；索引RestorePrepared/revision6、四schema2树帧/原件及plan原摘要与原stage身份核对。新collector重建查询782ms、runtime=stopped、client_recovery=required，D/H等于固定启动上下文，运行port/PID空，start/stop/restart三门槛false，reopenedRecoveryGate=true。没有在旧失败会话重拍原件或重跑旧目录。

这证明前批新collector误拒停止诊断的具体回归已修复；不证明此前独立的正常Stop后revision4登记失败同因或已消失，该项继续保留。实际catalog/cache/audit尚未恢复，完整原assert仍exit101/accepted=false，未调用apply或创建MutationPermit工厂。config.toml原字节保持/恢复和Grok备份缺席的窄断言不外推完整客户端还原。

## 证据、清理与后继

.adg/evidence/0.2.24/stopped-cli-admission-r1/含原假设/修改前失败、最终纯逻辑/原生结果、准确源码/binary摘要、真实新快照/原索引/四阶段/tree/plan/stage、独立引用核对及manifest。最终源码逐文件与真实产品轮一致，没有用别的代码的通过结果。证据仅本轮明确合成配置，不包含日常runtime secret/admin token或实际用户完整配置。

完整Darwin TCP表framing确认原主端口缺席，自己的test/helper/Node/Bun镜像进程为空；新N/K/history数据库逐一取得/释放排他后删除。程序副本/profile/索引/原件/stage/测试binary及核对后的重复证据根已清理。内核隔离只允许loopback及自有写集，拒绝日常HOME/外网；未改日常配置/Shell/正式安装/官方2.99，无模型或真实WebDAV业务请求，未构建新.app/DMG或发布。

后继仍须完整客户端可恢复字节与实际写回、完整进程集合/监听/写者许可、Grok/外部Agent/首次目录/跨卷/flags/通用图、显式继续/回退及旧revision4首错。WindowsW0 HOME/Profile/W1/W2、NAME、复杂System代理、官方发布验证和固定候选按IMP-34继续，目标active，不将局部回归闭合当整版完成。
