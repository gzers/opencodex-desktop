# r6逐节点原生动作与v3意图结果绑定

TEAM16/IMP142/r6，源码285be9fada2461cc7c386f6e2228c6ca1804a572/tree16f6ec340cfaedeafeb4b6e3c0c5d94ae254f0e1。承接5a7cccaaa/ce8c28c37，完整计划与单journal沿前批，不改变官方/四trait/共享算法/预算/日常策略。

| 运行 | 原退出码 | 过/败/忽略 | ms |
| --- | --- | --- | --- |
| library01 | 0 | 14/0/0 | 219237 |
| library02 | 0 | 15/0/0 | 136831 |
| product01 | 0 | None/None/None | 156873 |

本轮无非预期首错；原controlled故障全部保留为原例。完整两树plan和当前原普通Token/peer Binding核验后，NodeMove/WorkCreate/WorkRetire/OriginalRestore均先不可覆盖intent sync/readback及current同ID CAS，才借同actual claim/NLK/epoch/key/原deadline执行native回调；随后同原native对象全树读回，结果登记准确ID和8投影。新工作security沿创建结果校验退休，不从manifest猜工作代次。expected摘要绑定全plan和准确位置/内容/分代规则，实际新ID/8投影不能预测，单独由完成记录保存原生读回。

三份原文件Codex+Grok全树往返，30个canonical事件（6个锚点snapshot+24个文件intent/result）；目录及两稳定对象不动，原ID/内容/all8/完整children恢复，工作对象准确原handle退休。固定Bun claim-only在组合开始/最后native变更后仍raw73，显式释放后raw0；各native节点后另有实际SQLite竞争拒绝。旧r5 raw5根移动与空GUI不重跑。该组件没有生产Owner、正式阶段或Closed资格。

四个组合中断分别位于intent/current已落而native未动、native已移而结果未写、结果phase落而CAS前失败、结果current真实单字节部分写并sync。日志与原树/custody实际位置一致，tree和journal同时poison，后续调用拒绝，不catchup。新实际lease即使同路径/Token/ID也因epoch不同拒绝继承许可，明确新段协议仍需实现。

首轮14过后复核识别工作文件seed→writer→seal重开窗口，补原子私有CREATE_NEW工厂：仅新对象使用已有PrivateDescriptor/准确TokenUser+SYSTEM protected DACL，不修现有ACL；返回准确RW/DELETE创建句柄后立即存pending_work，任何后续失败均保留该对象。新增空创建后/实际1字节写+sync后/完整写+sync读回而发布前故障；原件custody不动，创建ID/all8/已写字节摘要及原pending记录保留，第二writer raw32。成功路径也不再关闭重开工作对象。最终15过。新all8事件不提供旧v2或前批pending的转换/接管；旧记录只作定位/原失败证据。

下一段仍需lease释放Prepared/原生结果、重新取得的phase/current权限，真实Ready/PreStop/Stopped封存、原包允许副作用及工作区可写窗口、source/route/history/successor、完整Stop与typed Closed、broker原Owner/H-S-D。当前生产适配入口仍v2，公共Start保持无条件拒绝；原缺席Codex安全退根blocked单列，不能借现有根通过代首次空环境。不W2/双SID/业务模型/发布。完整继承状态表归final-status.json。

最终产品MSVC custom-protocol debug bins及输入/最终测试Rust对准确源码字节核验；前端32份复用；不签release预算/冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop | 77688832 | 86ec8065fb72158df494beaf56ad2682bd72fd02aeb729a8210a8d53d4b06ad1 |
| ocxd | 5398528 | 6409aa3203c8dd4cffd1e9332d9889293af522029987cceee2b59ffb40bee6ef |
| ocx | 306176 | 6328242d66c8ad254276133d37381bc2621b0c8062be7b0480bdba9d9612d4a6 |

88场景私有ZIP内容核验；工作部分写原生观察和D场景phase/current原字节另归pending-work-native-observations/journal-scene-records，包含单字节current。10处本轮已完成自有纯/原语场景先核全部文件和目录后清理；78失败/pending场景原树及25处D日志根保留，三层前批任务父目录沿用。完整native回调正例虽原件恢复，正式release/phase未闭合，测试journal仍保持pending，不删除或冒Closed。ZIP不是NTFS身份/安全克隆。旧失败broker/账户/Profile/Public拒读与自动删除拒绝不重试，普通MSI/CAP-trust/overallCleanup未代签。
