# 0.2.24 原启动父进程与服务的双正常退出

revision: 1；2026-11-12。归IMP-34 P24-04，承接[原进程退出](macos-normal-exit-r1.md)、[TCP归属](macos-tcp-ownership-r1.md)和[摘要后端](macos-sha256-backend-r1.md)。代码main e62fc8a35；纯旧夹具临时根修正另为2fc3cb04a。accepted=false、非冻结候选；实际客户端恢复和MutationPermit仍未开放。

## 所补事实

Ready的独立startup-live摘要已绑定服务和启动父进程两代身份，但前批普通Stop只消费服务的正常退出。父进程也可能仍活着、非零退出，或订阅后fork/exec；不能凭服务退出和父PID随后缺席推断两者均正常收口。

macos_process_pair::PairWatch在停止前，从原Live的服务/父UID、PID、出生秒/微秒捕获两份实际ExitWatch；注册前后均核对两代及当前父子关系，同PID、外来父进程、错代次和已退出进程拒绝。不重新发现父进程、不接受外部序列化的退出回执。两者都必须取得本次原生订阅的normal0事件；任一未退保持未完成，任一非零/信号、fork/exec或未知事件保持拒绝。对子进程尚活着的情况仍检查父进程事件，不把父端失败藏在另一端等待后面。等待和Proof核对沿调用方同一原动作绝对deadline，不给父进程另开超时。

PairExitProof是不可序列化的两代正常退出事实，不是完整family/socket/写者证明或文件变更许可；不推定历史逃逸后代、转移/迟到描述符、其它监听与任意外部写者已经不存在。

## 接入当前生产路径

Active::pre_stop在PreStop原件发布及官方停止之前保留PairWatch。原官方认证Stop、原服务内核exit0、主端口及4项实际捕获TCP等原门槛保持。Active::stopped先等双正常退出，再核对原Live的两代，之后才取得原生协调、seal_stopped和登记restore plan。seal_completed_stop/prepare_restore也显式要求该PairExitProof与原Live相符，再沿原CompletedLifecycleStop核对程序/HOME/原件/记录/原deadline，不能拿单服务证明替代父进程事实。

原件或前置不满足时的既有正常Stop回退路径保持：仍只走原认证官方停止并保留原失败，不把缺Pair转成恢复许可。当前Pair/原件有失败也不会清理原会话或假报恢复成功；运行态可由独立已验证Stop观察为Stopped，恢复保持Required、三动作关闭。没有新增强杀、服务signal或端口驱逐。Windows平台未执行此Mac实现；原共享/Windows独占写集未越界。

## 定向验证与首错

最终40个函数通过（含1个helper入口，即39个功能判定）：pair6、原Exit10、Session9、Owner binding15；两个要求显式原包环境的manual入口ignored不计通过。新五类实际OS父子夹具覆盖：服务exit0但父仍活着；无关父/重复PID/错出生代次；任一端非零；父fork或exec时服务仍活着且拒绝永久保持；原传入deadline已过时不接受队列或缓存成功。正例Proof不能换角色或绑定另一父代次。没有用成功桩替代内核事件。

首pair轮1辅助入口通过/5功能失败是夹具默认tempdir落到/private/var，被helper严格/private/tmp断言拒绝，就绪门随后超时；原输出保留。修为显式/private/tmp，另用原binary直接复现被拒的根断言。Clippy指出helper子进程路径未覆盖reap，补普通错误退出的RAII释放/等待；显式exec反例仍由外层夹具保留并释放leaf，不删该反例。后继原Exit组9过1失败同为非直接子进程夹具默认临时根，单独只改其tempdir构造器，原失败不覆盖。产品退出判定未为夹具放宽。最终编译/格式/全目标严格Clippy通过。

## 一轮当前官方包实测

先写假设，只追加一轮受影响生产链；原样官方2.99与完整镜像/descriptor固定，准确最终build binary摘要2a013ff50dd7cee3880f27f3b068a23f6fa84b9cc73b2ec975c054fb68c3015b与上述检查及执行副本一致。全新合成Profile，无真实模型/WebDAV请求，OS sandbox拒绝日常HOME读写及外网。

- Start11317ms、Status231ms、Stop4947ms（内部4943ms），原30/5/10秒保持。
- 原服务exit0及4项捕获TCP全缺席之后，真实固定trace为stop-proof: original-native-launcher-and-service confirmed code=0；双退出与原Live核对约54ms，不外推为固定成本。
- 四schema4 frame、独立索引RestorePrepared/revision6、plan/stage及35项内容引用匹配；新collector371ms Stopped+Required/无PID与port、三门槛false。
- 实际catalog/cache/audit仍未补偿，完整断言保持exit101/accepted=false。只证明已知两代正常退出及当前窄样本预算，不是整版恢复通过。

本批未重构release/app/DMG；前批release摘要后端样本不能代签本批新父进程控制路径的最终release候选，最终固定候选仍须受影响验收。

## 证据、清理及后继

.adg/evidence/0.2.24/macos-launcher-exit-r1/保留先写假设、全部首错/夹具修正记录、最终准确源码/build/binary、当前真实日志、独立索引/frame/plan/stage/内容摘要及manifest。不保存runtime secret/admin token。所有pair及非直接子进程helper命令已缺席；原主端点经完整Darwin TCP表framing核对，额外TCP缺席仅引用准确产品trace，不伪称以未导出次要端口列表独立重放。3个精确新外部协调DB各取/释放EXCLUSIVE后，与自有程序/Profile/双客户端树/原件blob/index/stage/测试副本及重复回执根一并清理，未改日常配置/官方包或系统设置。

完整受控进程集合/持续写者/实际恢复、显式恢复和原通用图/客户端全范围、旧revision4首错、WindowsW0/W1/W2、NAME、复杂System代理、官方发布验证/固定候选继续。双PID证明不能被包装成完整恢复许可，不发版、不关闭目标。
