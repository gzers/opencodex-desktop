# 0.2.24 单次Restart的原停止预算

revision: 1；2026-11-12。归IMP-34 P24-04，承接[成功终态与再次启动](client-session-close-r1.md)。代码main82e2d21f1。当前Mac单次真实Restart及最终Stop在本批静态图原包场景通过；诊断accepted=false，不等同整版或候选验收。

## 已核实的预算缺口与修正

此前普通Stop使用入口10秒截止时间，但Restart传入30秒总截止时间，stop_lifecycle_verified在较晚的原件/PreStop之后才截取自身最多10秒；停止前置耗时可能不计入那段10秒。前批completion min原StopProof.deadline只能限制较晚的Proof起点，不能补回更早前置。不能因为外层Restart允许30秒就把停止收尾也放宽。

managed_lifecycle_deadlines现在只按原动作入口Instant创建一次outer与stop截止时间：Stop10秒；Restart总30秒、内部Stop10秒；Start仍原runner30秒。managed_program_identity/source准入、Active.load、pre_stop、官方stop/原服务和父normal0、捕获TCP缺席、补偿/完整树/两稳定锁释放/索引关闭均使用同一stop_deadline，不在途中从now重新加10秒。返回成功前再检查deadline，固定lifecycle-stop-phase日志记录从原入口到所有Stop工作完成的毫秒数/原10000预算/动作类型，不输出路径、令牌或回执秘密。新Start仍只用原外层剩余时间，无延长或额外30秒。

同一共享停止边界也传给非Mac平台，未修改Windows独占模块，不因此宣称其MSVC/原生预算已通过；后继Windows受影响验收仍需覆盖。原样官方2.99未改，无业务调用或自动迁移。

## 定向检查

21项相关函数通过：managed lifecycle4、process9、orchestration8；一个显式原包native入口ignored不计通过。新增入口预算断言：Restart outer-start=30秒、stop-start=10秒，若前置已耗7秒，仅余3秒，不可用当前now再加10秒；Stop的outer与stop相同。保留原进程动作状态机/成功不冒spawn/并发与失败等原断言。编译/格式/全目标严格Clippy通过。本批没有重跑此前无依赖变化的36项索引/完整性或原Windows r5。

## 单次真实Restart验证

先写假设，一轮全新合成Profile，原样官方2.99/tgz/完整镜像固定来源，准确build与21项测试binary SHA 8286e2b4d119e0e90940f203f46a896fa014153dc0ae275e45843edb1b81c5ec和运行副本一致。测试调用真实process_action_with_runner，明确OCXD024_SINGLE_RESTART=1，未把Stop→Start两个独立用户命令伪装成一次Restart。前批repeat-session开关本轮不启用。内核sandbox拒绝日常HOME及外网，仅loopback/自有根和精确协调DB及原globalL借用。

| 项目 | 实测 |
| --- | --- |
| 初始Start | 8529ms，Started/真实ready |
| 初始Status | 232ms，原PID/port/ready一致 |
| 单次Restart总计 | 14996ms，原30秒内，结果Started/Running |
| Restart中完整旧Stop阶段 | 固定产品trace8449ms，独立原10秒内；含原件/还原/终态关闭 |
| 旧/新PID | 10551→10591，实际新运行记录不同代次；新端口62448 |
| Restart结束索引 | 新会话Ready/revision10，旧会话Restored/revision7保留 |
| 最终普通Stop | 9004ms（内部Stop阶段trace8997ms），原10秒内，成功Stopped |
| 最终索引/原件 | Restored/revision14；七路径原字节/原缺席全true |
| 重开状态 | 1298ms，Stopped+NotRequired/canStart=true、无PID/port |

两个实际Stop段均有原服务与父normal0、4捕获TCP缺席、真实三项apply/原后像归档、整树Protection及两稳定锁重获释放、完成回执/索引commit/readback的完整trace；stop-phase-trace-verification.json逐项核对8449/8997均小于10000。普通Stop及Restart总时间不因本批验证放宽。

独立SQLite核对schema3与events1..14全在，两套不同ticket/Originals各有独立References、四frame/Scope/Live/plan/completion/outcome、content对象长度/0400/SHA一致。最终原生Codex/Grok整树额外只读快照SHA匹配最终completion；未扫描目录猜完成或改恢复baseline。原集成函数exit0/resultpassed，外层诊断acceptedfalse保持。

## 证据与清理

.adg/evidence/0.2.24/native-restart-budget-r1/保留先写假设、准确源/build/binary、21检查、真实Restart参数/日志、双会话索引14事件/两份原件及完成参考/最终原生树、原Stop段时间核对和manifest。没有runtime secret/admin token。

两个实际服务主端点由两套Live获取并在完整Darwin TCP表framing中确认缺席；额外TCP仅引用准确产品trace，不编造未导出端口表的独立清理回放。自有服务/launcher/Node/Bun/test与读取helper均缺席，3精确新外部协调DB各取/释放EXCLUSIVE后连同自有程序/Profile/双客户端blob/index/stage/archive/control副本与重复回执根清理。既有globalL原inode/字节/0600/owner保持、独立排他取得释放且无新sidecar，不删旧对象。日常配置/系统设置/官方运行包未改，无真实模型/WebDAV业务请求、无发布。

本批没有新release/app/DMG，完整时间预算的一般配置/大树/首次缺根等范围仍待原计划；崩溃显式恢复、活跃刷新、完整生命周期集合/其它监听、旧revision4首错、WindowsW0/W1/W2、NAME、复杂System代理、官方发布验证/README与固定候选继续。不能把这轮当前静态场景Restart通过写成整版0.2.24可发布。
