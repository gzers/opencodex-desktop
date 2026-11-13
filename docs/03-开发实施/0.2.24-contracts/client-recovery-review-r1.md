# 中断恢复整组现场只读核对

revision: 1；2026-11-12。归IMP-34 P24-04，承接[显式收尾](client-explicit-finalize-r1.md)和[收尾界面](client-recovery-ui-r1.md)。代码main 82764eabc56bb70e332a1879ec738ba9d15a86af。accepted=false，非冻结候选；未新增部分继续/回退执行器或按钮。

## 当前行为

RestorePrepared预览在原共享观察锁内重新读取原独立plan及四阶段原件，整组核对3个目标、对应暂存和归档的原生身份、内容与保护属性；intent必须为本plan的精确原文，terminal只能为本plan的固定Complete/Rollback。无terminal的已知中间态可以核验，但不产生写许可。缺intent却已发布、未知/替换inode、损坏terminal、归档变化均拒绝并保留现场。两次读取之间有变化亦拒绝，不收养新基线。

核对指纹只绑定当前固定计划、精确对象与根身份，不在DTO或日志输出原字节。Complete终态另逐项核对原内容与保护；Rollback核对Stopped后像。确认metadata收尾仍须原完整树、两稳定锁、当前HOME停止/原两PID及端口缺席、完整程序身份与独立索引关闭；不能用磁盘terminal重建历史正常退出证明。

预览同一观察只做一次整组核对，复用其中已核对的原字节结果，消除重复读取；保留全部不同事实门及新动作原10秒。已登记Restored后不再将正常用户改动与旧中断后像比较，不重开恢复或重放旧计划。执行失败新增仅阶段/耗时的运行诊断，不包含路径、密钥、配置或确认摘要。

## 实际验证与原失败

- 23项Mac原生文件事务测试含新增整组指纹/零写入、缺intent/坏terminal/同字节替换归档拒绝；原11个实际子进程exit91发布中断边界均能只读识别。合并重复读取后再验这23项通过。另21会话和6读回检查通过，复用未受影响结果；最终格式、编译及全目标严格Clippy通过。
- 首Clippy多余借用失败保留。隔离Profile真实原样官方2.99：Start8771ms、Status225ms、Stop8795ms；7文件原字节/缺席均恢复，按预设在索引关闭前中断，原exit101/RestorePrepared revision6/Required不改写。
- 第一独立收尾拒绝；加阶段日志后只重验保留现场，定位close-index耗时10624ms（测试外测10630ms），原10秒拒绝及revision6保留。未将其记为现场冲突通过或扩大预算。
- 合并重复核对后，仅重验同一保留现场的显式收尾，未重装/重启/重跑服务或已过反例：8898ms，Restored revision7、重开预览仍Restored，正常用户合成cache修改仍Restored且内容保留；测试退出0。原错误attempt/revision/digest、外部config编辑及陌生runtime记录拒绝已在第一轮通过，后继未重复。
- 独立保存原失败索引与成功索引/7事件、完整receipt摘要、4帧/plan及35内容引用/15原内容blob；另逐项核对7客户端文件原字节/缺席。没有另跑最终全树导出，原生完整树/锁读回证据以该实际收尾为准。

证据[本批目录](../../../.adg/evidence/0.2.24/client-recovery-review-r1/)与manifest按字节核对后保留；失败explicit/与explicit-trace/均不被成功explicit-delivery/覆盖。外网与日常HOME由OS sandbox拒绝，仅允许loopback；无模型/业务请求。自有服务/启动进程和原主端口缺席，3精确新协调DB排他取得/释放后连同自有程序/Profile/原件/stage/archive/测试副本清理；既有global L的inode/字节/权限/owner保持，排他重获释放且无sidecar。未改官方/日常配置，未构建新.app/DMG或发布。

## 后继范围

部分继续/回退仍需新的原生执行权限和用户确认；本批只读指纹不代替它们。全客户端/一般图/完整进程集合、旧revision4首错及Windows、NAME、System代理、官方发布验证和固定候选仍按IMP-34推进。Windows远端仍b2e89665d/0eef59a01，无新回交；本批不重跑W2。
