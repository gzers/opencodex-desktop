# 0.2.24 原服务代次的TCP监听归属与停止缺席检查

revision: 1；2026-11-12。归IMP-34 P24-04，承接[正常退出](macos-normal-exit-r1.md)、[退出变更](macos-exit-events-r1.md)及[Grok原件](grok-originals-r1.md)。代码main 1a7133e40710e41989b3ed4c7745a823b904e0d9。accepted=false、非冻结候选；本批不开放MutationPermit或实际apply。

## 新的原生事实

macos_tcp_listeners::TcpListeners按当前UID/PID/出生秒微秒读内核完整FD列表，容量/条目数/重复FD/代次变化均严格检查，拒绝截断和不可读。只对socket FD通过libproc取得socket_fdinfo，识别TCP IPv4/IPv6 LISTEN；两次完整观察的socket FD集合及监听对象/FD/family/port必须一致。非socket FD的日志等正常开关不用于推定TCP变化；UDP/Unix等明确不在本TCP集合中。两个稳定观察不是原子全时序快照，未知/变化不发部分通过结果。

没有在Rust复制Darwin大型结构偏移。小型C桥直接使用目标SDK的sizeof/socket_fdinfo命名字段，返回固定16字节object/family/port；编译时静态断言C ABI，Rust repr(C)对应，真实双栈子进程检查证明端口/类型解码正确。cc1.6.0复用原锁定版本作为直接build依赖，不升级包；build.rs按实际target_os=macos才编译C，Windows分支不编译libproc代码。内核opaque socket标识只作本次私有比较，不写公共DTO、磁盘帧或日志。

## 接入普通Stop

原OS归属/服务管理器检查通过后，先订阅同代次ExitWatch，再捕获实际TCP监听；运行记录的主port必须出现在该内核集合，不能凭HMAC响应或配置端口就推断FD归属。无法核对则在发送停止请求前拒绝。随后沿原认证官方Stop，不加信号/端口驱逐或额外超时。

原进程退出、运行记录/管理器等旧门槛之外，现在对每个捕获的不同TCP端口重复使用原全局Darwin TCP表缺席检查；次要端口仍监听或查询未知即保持未确认，不因为主端口已关闭就通过。CompletedLifecycleStop保留实际捕获集合，require_original另核对其Generation等于原退出代次，缺此事实不能用单PID exit0替代。原Stopped预先观察仍没有新退出/监听事实，不能授权客户端补偿。

日志只写捕获/全部缺席的数量，不写socket对象或地址。恢复原件/计划仍沿同一原截止时间；端口被其它程序复用也保守拒绝，不终止其它用户或别HOME进程。此类暂时观察不保证后继没有新监听，真正恢复仍需持续的完整控制集合/写者排他。

## 验证

18项去重定向检查通过：3项新真实FD/Socket子进程检查、5项停止绑定/次要端口谓词、10项既有内核退出回归。真实子进程同时开IPv4及IPv6两个TCP和一个UDP，TCP端口全识别且UDP排除；两观察之间增加第三监听明确拒绝，随后新观察有3项；错误出生代次/已退出PID/原预算拒绝。合成Host仅用于次要端口阻断/查询错误谓词，实际TcpListeners来自当前内核，不能把合成外层Stop元组当服务证书。

原有合成CompletedLifecycleStop测试新增字段漏写导致首编失败，保留stderr后补实际活进程捕获或明确None负例；正例只是引用比较，不是生产许可。正常退出fixture先输出明确就绪字节再订阅，避免初始exec事件时序；缺listener证据仍拒绝。首Clippy复杂返回类型用内部类型别名修正，不禁lint。最终编译/格式/全目标严格Clippy通过。

## 原包来源错误及有效轮次

第一轮继承的临时脚本硬编码旧测试binary089f843b。cc成为直接依赖后，本轮编译产物实际为7a2a7cb1，该脚本仍执行旧文件；新工作树source.json快照不等于该旧binary的源码证明。此错误由本对话发现，原结果/原索引/帧及binary-mismatch.json保留，明确不计入新TCP生产验证，不能拿它的普通启停结果说新代码通过。

修正后从本轮Cargo编译记录得到实际路径，并在执行前对比18项已通过检查的binary SHA，再复制并复核同SHA。第二轮用全新Profile及完整重验的同一不可变官方镜像，不重跑原失败目录。最终9件源SHA与有效产品轮匹配：

- Start13958ms、Status1138ms；正常Stop及原件/计划登记9996ms，原30/5/10秒不变。
- 真实日志先native-tcp-listeners captured count=4；原进程内核exit0后captured-tcp-listeners absent count=4。这些是4个不同捕获端口；所有端口缺席谓词通过才会写该后日志。
- Profile/Grok/Codex四schema4帧/独立索引RestorePrepared/revision6/plan/stage及全部引用字节匹配；Grok原字节恢复，旧备份/Agent保持，新collector794ms Stopped+Required且三门槛false。
- catalog/cache/audit仍未执行补偿，原完整assert保持exit101/accepted=false。没有签发恢复许可或更改完整恢复断言。

第一轮错误binary不用于性能/新TCP通过结论；第二轮9996ms几乎耗尽原十秒，没有给后继apply预留额外预算。该时长只证明此轮组件未超限，不证明余下完整恢复可在预算内完成；关键路径优化仍必需。次要端口缺席证据来自准确产品代码及其固定trace，清理后的独立表只另核对原主端点，不伪称已根据未导出的4项端口列表再次回放。

## 范围限制与清理

TCP集合只覆盖捕获时该原PID拥有的监听，不覆盖UDP/Unix、先前后代、逃逸/转移描述符或捕获后新监听，也不是Job/完整进程树证书。正常Stop原exit0、全部捕获TCP缺席及NativeLease仍不足以关闭任意其它写者；MutationPermit保持无生产构造器，实际恢复未接。Windows平台未执行新原生测试，不冒Mac证据为Windows通过。

.adg/evidence/0.2.24/macos-tcp-ownership-r1/保留SDK头摘要/C桥合同、首编/lint/18项输出、错误binary及正确binary两独立轮次/分类、原索引/phase/plan/stage/内容匹配表、准确源码/binary、manifest和cleanup。没有记录runtime secret/admin token或用户配置凭据；动态普通内容不公开原字节，只验证摘要/安全属性。

Darwin完整TCP表确认两原主端点缺席，自有test/helper/Node/Bun镜像为空；精确新N/K/history数据库逐个取得/释放排他后删除。两轮程序副本/Profile/双客户端全部blob/frame/索引/stage/test binary及核对后的重复回执根均清理，日常配置/Shell/正式安装/官方包未改，无模型或真实WebDAV业务请求，未出新.app/DMG或发布。

后继继续完整进程集合/监听/持续写者许可与关键路径优化、实际客户端补偿、显式继续回退及原完整范围；旧revision4首错、外部Agent/一般路径、大树链接/flags/跨卷/通用图、WindowsW0/W1/W2、NAME、复杂System代理、官方发布验证和固定候选保持IMP-34未完，目标active。
