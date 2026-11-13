# 保留归档重读，合并租约目录元数据 r1

revision: 1；2026-11-13；IMP-34 P24-04；性能实现main5027b0ad4；边界回归main5274b7048；accepted=false，非固定候选。承接 [关闭证明失败](client-exitfix-native-r1.md)。本批没有官方服务轮次。

## 否决的删除候选

root_binding archived RootAbsent 路径中，inspect_retired 与随后 verify_retire_readback 的静态检查确实有包含关系。但直接删除前者会改变后续Plan重读相对于已完成归档观察的位置：Plan在中间被改坏且保持坏状态，旧路径的下一load会拒绝；候选可能使用已加载Plan完成最后读回。每帧gate未减少，并不能证明整体动态时间边界等价。

实验候选的两项静态损坏／计量测试通过，却未覆盖上述边界，所以未提交候选，root_binding／其它候选修改和测试接线已准确恢复到baseline382cf593f。证据 [client-retired-inspection-rejected-r1](../../../.adg/evidence/0.2.24/client-retired-inspection-rejected-r1/manifest.json) 保留开跑前假设、未提交源摘要、原日志与否决原因；不把实验通过计入正式修复通过，不把静态审阅反例冒称已运行错误版本的确定性注入。实验TMP为空，材料逐字归档后清理。

主线新增防回归用例直接验证保留的原边界：先成功inspect_retired，再改坏Plan并恢复400保护，verify_retire_readback必须重新读取并拒绝，篡改原字节保留、根仍缺席。该测试通过，不声称删除候选动态等价。原source/candidate/postcommit及post-close原证明、deadline和回退规则保持。

## 实际采用的元数据去重

NativeLease.check原本在任何claim／owner内容读取之前，连续对实际physical client、profile、home分别recheck。现在用已有PinnedDirectory::recheck_set将三根及共同祖先的saved identity／完整Protection期望做同次双遍联合核验；冲突pin直接拒绝。该方法仅增加crate内可见性、返回()，不构造内容读取／写入权限或可缓存证明。

后续claimGate、物理根重定位检查、每个协调file pin的parent／inode／Protection、owner receipt、连接数量／autocommit、原SQLite VFS和deadline检查原位保留。只合并相邻元数据采样，接受其时间点调整，不删除任何动态内容来源gate。实际diff只读补审无阻断；root_binding与baseline保持一致。

真实自有四SQLite租约check按旧／新／新／旧对照，Protection调用134／106／106／134，native xattr214／170／170／214，owner receipt capture均1。外进程证明四锁检查期间blocked、正常release后unblocked；client/profile/home × mode/xattr/replacement九组变化，两模式均拒绝且connections仍持有事务。两项新增在完整协调回归中计入一次。局部耗时约1.595／1.221／1.149／1.320ms，收益仅几百微秒，不足以单独证明能解决Stop总预算，不能作为立即重跑原包的依据。

## 验证和清理

协调29项、原件pin与帧基础35项、命令4项、保留Plan重读边界1项，共69项不同受影响测试通过；协调6项／命令1项原有忽略保持。最终all-targets Clippy -D warnings、fmt、diff-check通过。未重复整个原件模块或已失败原包轮。

采用实现证据 [client-lease-directories-r1](../../../.adg/evidence/0.2.24/client-lease-directories-r1/manifest.json) 含开跑前假设、ABBA／四锁／反例原日志压缩、五源及测试二进制摘要、actualdiff审阅限制和独立verifier。unit TMP为空，所有原字节压缩解压逐字比较后清理两批证据临时根。工程target与之前原包失败证据保持，无日常配置／Shell／安装app／官方2.99或真实业务请求变更。

下一步继续关闭事务的同次认证成本审查与既定动作canStart/status资格同源接线，先保留所有后置Plan／帧／归档／proof边界再找等价底层读取优化；不为几百微秒收益立即开新原包轮。原第二Stop11428ms／RootRetirementPrepared19失败未关闭。根中断／缺Ready／一般客户端图路径和完整进程集合／刷新／NAME／复杂代理／WindowsW1W2／官方交付验证及固定候选完整范围保持，目标active，未发布。
