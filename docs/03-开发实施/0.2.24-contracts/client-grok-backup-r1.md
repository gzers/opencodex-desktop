# Grok首次备份的可逆恢复与独立官方配置锁

revision: 1；2026-11-12，IMP-34 P24-04，代码main d5970bc55。承接[首次客户端诊断](client-first-use-r1.md)、[首次空配置](client-empty-config-r1.md)。accepted=false，非冻结候选；不修改官方2.99、日常配置或真实业务。

## 原始问题与新的恢复对象

client-first-use-r1保存的旧原生失败：官方Start首次复制原Grok config到config.toml.bak-opencodex，普通Stop还原config但保留新backup。完整原件读回因额外文件正确拒绝，不能把这个文件扩为稳定锁之外的残留例外，也不能仅按名字unlink。

新GrokHistory从一次已验证的原件frame前缀读取profile/.grok的root identity/保护及明确普通文件记录。Original backup必须Absent且Original config存在；Ready backup必须与Original config字节长度/SHA和Protection一致，实际PreStop若有及Stopped须与Ready backup记录完全相同。当前backup Image（真实identity/bytes/Protection）和.grok根必须匹配该独立来源，才形成GrokBackup。不从现在的目录输出猜权限，不接受“内容看着像”但未记录的备份，根重定向/新权限/替换/历史变化均拒绝保全。没有记录的原缺席对象后来出现也拒绝，不给旧baseline补文件。

schema6计划新增可选grokBackup，绑定精确Stopped/current Image和原.grok root。Codex三项始终是0..2，首次空config可选3，Grok backup可选4；按实际indices迭代而非用枚举位置重新编号。只有backup时顺序0/1/2/4，两者都有时0..4，不能因此把Grok操作落到Codex目录。只接受当前profile下已固定.grok，原root和私有journal须同卷，EXDEV无copy兜底。旧schema没有此字段则原完整Grok余项仍严格要求，不自动迁移或收养文件。

## 同一个可逆状态机，独立的目标根

第五对象复用原Pending的全组预检、state/pair/archived-generation识别、每步load和当前权限检查、原生RENAME_EXCL、native_gap后核验、create-only冲突记录和一次可证明的反向保全。Complete把这个准确已记录backup移入私有stage再归档old-4，原路径必须Absent；Rollback移回同一个文件对象，不能覆盖新目标。rearm仍用同object_id和原生对象链、逻辑新plan，原撤销记录不变。失败保全，不按名字删文件或放宽安全属性。

plan.load按原frame重新验证backup内容和根来源；缺历史阶段只能沿已有明确当前RecoveryObservation/sourceHead使用实际历史+新当前观察，不合成PreStop/Stopped。确认摘要包含GrokBackup，独立当前计划准备仍不修改目标，后继另行Continue。准备/继续/撤销/重准备均原10秒。全组review包含稀疏真实index对应的target/stage/archive，任何变更都不能沿旧确认。

recovery remainder只在已admit的第五对象明确交给该对象引擎时投影掉这个准确backup，其它Grok文件和root保护仍比原件。Rollback完整读回同时要求plan终态与停止后backup对象一致，不给启动许可；最终Complete完整Grok树仍直接严格匹配Original，backup没有永久例外。完整Codex、两稳定锁真实释放及ReadbackProof/索引CAS照旧。

## 官方配置锁按目标独立持有

未改官方的withConfigWriteLock以canonical config.toml.ocx-write.lock序列化Grok注入/清理，Codex的该锁路径不同。Desktop管道holder现仅接受已核验Codex根或当前profile/.grok两个固定target，仍验证官方包版本/来源、canonical根和HOME/CODEX_HOME/OPENCODEX_HOME，未把上游源码复制或打补丁。实际Grok补偿必须借Grok锁，Codex可选空配置必须借Codex锁，两者都有时同时持两把。

同进程NativeLease仍拥有原claim/N/L/K；每次写前后再检查各配置holder的原child未退出和同锁Image、当前native许可/原deadline。holder只阻塞父pipe，不靠计时器解锁；release消息/官方finally/独立回执/exit0与锁缺席均是成功必要条件。忙或不安全直接拒绝，不等待竞争者以绕过源身份；任何一把释放未确认仍失败。按反序正常释放，专用helper的失败清理不能当正常业务完成。父退出EOF的既有验证承接client-empty-config-r1，未从PID存活替代锁证明。

这些advisory锁只覆盖协作的官方写者；Codex/Grok客户端本体和不协作进程仍需精确对象检查、冲突保全和后续完整进程范围工作，不能声称已证明所有writer静默。

## 验证与当前边界

[本批89文件与manifest](../../../.adg/evidence/0.2.24/client-grok-backup-r1/)保留假设、精确源/二进制、测试/原生trace/全部原件和清理。65个相关后端函数通过：新增Grok6（含真实child exit91交换/归档/终态重开）、旧restore38、完整终态1、completion7、Grok4/tree9。完整终态现在普通原配置/首次空配置/首次Grok备份/二者同时出现×refs3/2/1共12情形，撤销→完整Rollback读回/独立终态→rearm→Complete/Restored覆盖；旧原件和缺阶段文件保持。已有backup不移除，内容/identity/根变化、PreStop外部编辑和native gap替换分别拒绝并保全。首闭包lifetime及部分move编译错误按hypothesis.json和chat原输出保留说明；不冒称另有原始日志。最终fmt/编译/strictClippy过，无UI改变所以不重复浏览器检查。

真实新隔离Profile用原样核验官方2.99：Start9025ms/Status234ms/Stop9189ms，原30/5/10秒不变。driver exit0/result passed，完整恢复index8 Restored。Original backupAbsent，Ready/PreStop/Stopped同一已记录副本等于Original config；schema6精确root/Stopped Image与old-4归档、Original/Scope/Live/phase/plan摘要核对，最终backup缺席，原Grok config与Agent保持。独立完整当前Codex/Grok原生树SHA匹配completion；正确实际Program上下文Stopped/NotRequired/无PID/port/仅Start。Stop仅约0.8秒预算余量，单样本不是最终候选/一般树性能保证；先前失败目录保持不可变。

同现场独立无服务动作，用真实官方writer进程验证：仅Grok guard时Grok locked而Codex获准；两guard并存时两边locked；释放Grok仅放行Grok，释放Codex才放行Codex。两锁和claims实际缺席，不用只有tasklist/PID活性推断已清理。原生source.json逐路径摘要与最终提交文件相同，native执行副本SHA等于当前编译测试binary；没有把旧证据当新源码执行证明。

先归档逐字核验，再确认自有进程/主端口缺席和两配置锁/claims缺席；3精确新协调DB真实EXCLUSIVE重获释放后删除，隔离程序/Profile/全部夹具及重复临时根清理。globalL inode/字节/权限/owner保持且排他重获释放，无sidecar，unitTMP空。OS sandbox实拒日常HOME及外网，只有loopback，无真实业务/模型请求；未修改日常/官方/系统配置，未构建新.app/DMG或发布。

当前仍未完成整个.codex缺根归属、缺Ready启动中断、一般客户端/图/路径/完整进程集合及官方刷新协调、旧revision4首错、Windows产品/W1/W2、NAME/复杂System代理、官方发布验证/README/版本和固定候选。Windows本轮fetch无新回交，不扩大本批完成范围，目标继续active。
