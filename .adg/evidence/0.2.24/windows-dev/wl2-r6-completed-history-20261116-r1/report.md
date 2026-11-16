# r6跨动作历史候选与原预算成本诊断

上次已推源码d20103fee084a3e63f728bf7c6664af65405e456保持不变。本地未提交候选tree 958c0deb01aa56c43dda74c68cbca05c31b96884；candidate.patch和准确执行输入可完整复核。候选未通过完整原预算，未作为源提交/推送。实际fetch与网络恢复证据0182e060d已推，TEAM16/IMP142/r6不变。

| 运行 | 新编译 | profile | raw退出 | 过/败 | ms |
| --- | --- | --- | --- | --- | --- |
| library01 | True | debug | 0 | 27/0 | 187444 |
| library02 | True | release | 101 | 0/3 | 773903 |
| library03 | True | release | 101 | 30/7 | 1247100 |
| profile01 | False | release | 0 | 1/0 | 10265 |
| profile02 | False | release | 101 | 31/6 | 145657 |
| product01 | True | debug | 0 | None/None | 195617 |

首library02原优化profile/原10秒fixture raw101/0过3败，后继library03 raw101/30过7败，上游恢复或原后验截止失败均保留。已归档EXE同Hash直接诊断profile01单例raw0/1过：及时交易完成后真实等原截止过期，新5秒只读成功，原期限append仍TimedOut且不发布。这个独立观察不冒充完整验收；同EXE、原37筛选/单线程/原断言完整诊断profile02仍raw101/31过6败，新完整/负例scope未过。所有原失败保持，不能按单例成功替代。

本地候选实现私有非serde/nonClone CompletedOriginals，实际closes、canonical结果/current、最后整树与journal后验全部在原截止内才构造；固定原截止/完成时点/准确current ID与session/manifest/binding/plan/action/claim/epoch/tip/当前原字节/recovery/history/handle摘要。新读取用原调用者when/end最大5秒，前后核同receipt与完整原journal比较、归档/retired对象和及时lease历史。原check_inner/action cutoff不改，写路径仍原截止，不存在deadline setter。当前只能称候选/单例观察，不签typedClosed或生产Status。

恢复助手与append相邻重复journal遍历做了一次职责收敛；完整原journal比较体逐字相同，append写前/写后原native检查保持，实际move完整BOTH-tree before/after、原语affected-tree before/after、原claim/NLK、旧fault位置与断言保留，guard证据在案。但完整预算仍失败，不能声称该改动完成了性能收口。

CPU/I/O诊断是只读采样实际自有子进程，没有调优先级/改策略/停服务/清缓存。完整profile02 145657ms墙钟、136859.375ms子CPU（94%单核），其中kernel95609.375ms，1369872 readOps/21488996 otherOps；同期间系统总体忙约10.8%/20逻辑CPU。数据不支持归咎全机繁忙或凭空放宽超时。下一步采用r6原已允许的单phase有界原子节点事件批处理，减少durable record/full-chain native核验成本；原文件/安全/内容/children/未知拒绝、intent-before-native/result-after-readback及准确中断前缀不缩。无需用户改变日常配置/策略或给更大预算。

候选未通过完整检查，源分支不创建新commit、不推其实现；本轮仅分离文档原始证据提交推送，工作区保留7个候选文件供继续实现。虚拟tree来自独立临时Git index，正常index未改，临时index已移除；tree不是commit SHA，精确patch和原编译输入同工作源/原Git对象行尾等价核验。product01为本地候选MSVCdebug编译，不是已通过/已推源码的制品或发布。

共184场景均私有ZIP核验，6完成纯数据scene核fresh准确路径/全文件hash/目录membership后清理；178失败/pending及59个D根保留。4119份canonical/快照/history原字节另归raw-scene-records；ZIP非NTFS身份/安全克隆，旧拒绝清理不重试。原失效窗口全部保留，无catch-up。

Start无条件拒绝，实际Source/Owner/typedStop/原process与supervisor历史/完整inventory角色/生产Status/typedClosed/后继仍未完；原缺席安全初始化/退根、普通MSI/CAP-trust/历史Profile/Public/失败broker清理保持未完。无旧raw5/空GUI/相同1314/W2/双SID/发布或官方/日常配置/令牌/策略修改。目标继续active，不把性能失败当外部权限阻塞，也不标完成。

| 本地候选制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop | 77765120 | 9a9755794858dcc1763ebb762b759339628bb46ab8ce9b60fbd0298743e6d090 |
| ocxd | 5398528 | 90e1899ba85186e6ac50076cc4f1fb9eb106dc877dcc3c31598a7208af8ed548 |
| ocx | 306176 | f7d06c2ae97aaa95f0414a6641b87bc4d21b8a6b7c28e7a011bae19bc0348657 |
