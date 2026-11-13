# 围绕历史目录投影的完整双遍观察 r1

revision: 1；2026-11-13；IMP-34 P24-04；main 236ed43dd；accepted=false。承接[历史计划去重](client-plan-observation-r1.md)。本批把上层两次已含前后读的历史入口，合并为围绕同一目录投影的完整前验与后验；公开单次历史读取和 current/Stop/移动入口的语义保持。

## 观察顺序与引用

上层 vault_history 原先按每代在投影前后各调用一次 open_closed_history，每次又完整读取两遍，产生四套归档核验。本次保持两循环：全部 generation begin 并核各代scope → exact vault names/current-attempt投影 → 全部 generation finish 再核最终scope → current directory/成员/manager复核 → 外层独立index/Head/events/binding/slots复核。任何一代失败整体失败，不在部分finish后返回聚合成功。

begin 仍validate source/closed/规范路径ID/独立movement/completion，使用原完整Plan load及exact source Head比较；完整读取archive prefix/每帧来源及内容/root/protection/Original/tree/Grok/scope/live，核实际released tree、独立intent及重建readback SHA。finish消耗不可clone/serde的观察对象，独立重开全部来源和完整Plan，再次exact source及原Plan摘要比较，重读完整archive/树/Grok/intent/readback，并比原树摘要/回执。返回的是最终认证的Originals。

不通过callback暴露未完成Originals：begin只提供scope需要的data/profile/client目录事实，返回值只能为io<()>。未完成对象只保存独立ClosedRefs（含原内存namespace）、路径、初Plan/树摘要与小intent/receipt字节，不保留fd/Plan/tree。没有从磁盘或摘要重建namespace，没有跨观察/动作cache，原每帧认证不减少。每个完整pass代码由原历史入口共用，公开open_closed_history仍在单次调用中执行两个完整pass；current路径不使用这个上层投影合并。

资源边界：每代begin/finish临时pins随调用释放，不随512代保留完整祖先链FD；不缓存整树或Plan。scope前后仍逐代核；结束时重新开路径并验证独立frame中的identity/保护，不能靠仍持有旧fd让替换路径被忽略。成员枚举仅与索引准确名单比较，不学习新attempt或slot。

## 验证

107个唯一受影响函数通过（3新例、完整夹具比较1、其它历史39、索引56、多代8；5项ignore不计），strict all-target Clippy/fmt/diff通过。

3个新增反例验证两pass之间篡改Plan、receipt、Original frame、archive权限、namespace均拒绝且不修复；scope投影错误不返回可完成对象、零写入；真实独立子进程同时保留32个不完整观察，fd初始/最终70、循环内不累计。FD计数由父精确--exact/单线程子进程完成，避免默认并发测试掩盖或误报句柄数量。

同一binary/同一独立索引的完整schema2四帧夹具，reference→current→current→reference：原参考约180.66/173.85ms，当前约90.56/91.64ms；完整pass4→2、frame16→8、full provenance20→10、plan load4→2。保留一套完整前验与一套独立后验，去掉的是重叠重复的两套，不是把每帧检查换成摘要cache。只说明合成debug观察48.61%的小样本变化，不直接当作原包Stop或整版通过。

证据 .adg/evidence/0.2.24/client-history-projection-r1/ 保留开跑前假设、4源与实际librarybinary身份、反例/FD子进程记录、原与当前计数、正反对照及清理。初FD计数用例在本轮显式串行运行通过，但没有默认并发隔离保证；review后改为独立进程，旧日志不冒最终隔离证明。原历史/Plan/索引/多代/中断反例继续验证。

## 后续与原失败

前轮原官方双轮第二Stop12878ms/RestorePrepared16失败证据保持。新原包轮应以“历史遍历减半后，第二normalStop应回到原10秒内”为可证伪假设，使用冻结源码/实际releasebinary、全新自有Profile及原沙箱/精确新协调文件清理，独立保留前后所有原字节，不重复失败轮或增加预算。该结果单独记录，不把本合成观察当fixed.app/安装器/全版本验收。

原30/5/10及Restart30/Stop子预算、根中断/动作回执容量边界、缺Ready/一般客户端图路径/完整进程集合和刷新、NAME/复杂代理/Windows W1W2/发行README和固定候选等IMP-34全范围保持。没有原官方2.99补丁、旧版兼容/自动迁移、日常配置/Shell/安装app或真实业务WebDAV/模型请求，发布另依有效授权。
