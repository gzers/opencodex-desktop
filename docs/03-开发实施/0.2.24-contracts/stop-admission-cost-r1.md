# 0.2.24 停止协调边界的完整镜像校验耗时

revision: 1；2026-11-12。归IMP-34 P24-04，承接[TCP归属](macos-tcp-ownership-r1.md)。代码main c025dd83d（固定阶段耗时）、d7b4921b8（完整清单合并）。accepted=false，非冻结候选，未开放MutationPermit或实际apply。

## 假设与真实分段

前批Stop9996ms接近原10000ms。先只增加client-stop-recovery固定阶段及单调毫秒数，不删校验、不打印路径/令牌/原件内容。在准确构建回执提取测试binary并核对副本摘要后，用全新合成Profile及原样官方2.99供应，走真实生产Start/Status/Stop。

第一轮Stop外部10200ms（内部日志10196ms），保留超时和exit101；服务已正常exit0且4个捕获TCP端口缺席，独立索引已RestorePrepared/revision6，但协调末尾超时仍不当成功。停止后的协调总5983ms：进入原件处理前2518ms，原件/计划/索引到3741ms，退出校验又2242ms。两端校验约4760ms，中段约1223ms。记录只用于该轮瓶颈判断，不外推为通用性能分布。

## 产品实现

program_selection::read_complete在每次调用中执行一次真实完整目录inventory，按相对路径、文件字节/大小/SHA/执行位及空目录核对treeSha256；同一inventory中的六个关键角色再逐一核对各自独立摘要：package.json、官方入口、Node、npm、Bun、Bun launcher副本。完整tree摘要一致不能免除任一关键角色摘要。current、manifest、稳定入口、更新/修复/卸载pending、包名/版本/bin和可执行校验继续。

官方协调前及原生工作后原来的“read关键文件，再verify完整目录”改消费上述单次完整准入；两端仍分别做全新全目录读取，无跨调用缓存、无用mtime替代内容、无降低限额。原生中段的锁/owner/路径/SQLite VFS复核与原件/阶段/plan检查均未删。BorrowedWork后置校验仍保留原入口，不扩大本批写集。原30/5/10秒与官方包不变，Windows生命周期未执行。

## 定向验证

30项debug相关检查通过：选择11、完整树3、官方协调6、原生协调10；2个要求显式原包环境的ignored不计通过。新增4项函数内部覆盖全六关键角色：内容变化后重新计算tree也仍须满足独立角色摘要；普通数据/空目录/执行位/软硬链接改变、current/manifest/稳定入口/pending变化均拒绝且保留原件；成功后第二次调用不复用第一次结果。编译、格式、全目标严格Clippy通过。

第二个全新Profile复用并完整重核第一轮未变官方镜像，准确binary摘要与30项检查一致。Start12287ms、Status1232ms、Stop9997ms；4项TCP捕获/exit0/全缺席，四阶段/独立索引revision6/plan/stage与35项内容引用核对，新collector865ms Stopped+Required且三门槛false。停止后协调5256ms，两端约3910ms、中段1350ms；有减少但总Stop仍几乎无余量，不声称已解决完整恢复预算。两轮同一原件链格式/真实失败断言保留，不重跑旧现场。

## 交付优化级别复核

实际release配置（含LTO/opt-level=s）构建完成，同组30项通过；构建回执binary SHA bcb0e3f259312a9982b6525b2f1c8d266a0ef86dfdb8fc8d27495964dd5b3dba与运行副本一致。第三个全新Profile完整重核同一原样官方镜像：Start18962ms、Status2295ms、Stop10048ms（内部10045ms），明确超时。原内核exit0/4项TCP全缺席已确认，但停止后协调到prepare-plan阶段耗尽预算，独立索引State::Stopped/revision5、未登记plan；其余原件/阶段35项内容引用仍可读。新collector1406ms Stopped+Required且三门槛false，完整断言仍exit101/accepted=false。

release没有证明预算闭合，不能把局部校验优化、debug勉强未超限或编译通过说成生产停止恢复通过。三轮的环境/调度差异未隔离，不用单轮时间差宣称稳定收益；下一批须根据新假设继续定位动作准入/全目录遍历的成本或改用等强度实现，不重跑本轮旧现场、不延长原预算。

## 证据与后继

本轮证据归.adg/evidence/0.2.24/stop-admission-cost-r1/，含先写假设、两轮准确build/source/binary、原超时/原完整失败、固定阶段日志、独立索引/帧/plan/stage及内容引用读回。不输出动态令牌原件；原runtime-port仅保留PID/port投影。

三轮自有进程镜像为空，完整Darwin TCP表framing核对三个原主端点缺席；额外TCP缺席只引用各轮准确产品trace，不伪称以未导出的次要端口表独立回放。九个精确新协调数据库各取/释放EXCLUSIVE后，与各自sidecar、自有程序/Profile/双客户端树/原件blob/index/stage/测试副本及临时根一并清理；保留的证据逐摘要核对后删除重复临时回执。最终manifest逐文件长度与SHA核验。实际catalog/cache/audit恢复仍false，完整受控进程集合/持续写者许可、显式恢复及原全范围继续；旧revision4首错、WindowsW0/W1/W2、NAME、System代理、官方发布验证及固定候选均未闭合。本批无新.app/DMG或发布，不关闭整版目标。
