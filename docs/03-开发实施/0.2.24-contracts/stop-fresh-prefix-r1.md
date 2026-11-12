# 同次恢复核验的完整原件链读取成本

revision: 1；2026-11-12。归IMP-34 P24-04；代码main3e3bb6ab5。承接[历史原件与Stop超时](client-vault-history-r1.md)。accepted=false、非冻结候选。本批普通隔离Stop样本9415ms完成全部恢复/索引，原10秒不变；前10539ms与本批prefix-only10296ms失败保持，不改写失败记录或冒一般路径/最终候选均通过。

## 原因与修改

最新源码逐层核清：Originals.open_for_profile依次frame(Original/Ready/PreStop/Stopped)，每次frame又递归读前驱；Pending.captures逐阶段frame亦重复前驱链，同次四帧核验形成10份frame读取/归档校验。第一批改为每次完整prefix按Original→Ready→PreStop→Stopped新读一次，每阶段全部独立digest/strict decode/schema/context/目录原生身份/保护、capture/树一致性、完整归档blob和前驱摘要逐项核验保持。没有跨调用缓存，任何变更/重新打开/下一mutation前仍全新核验；不改变Original/phase格式或来源规则。open独立输入phase列表按最高index核验完整prefix，不因输入顺序省略已绑定后继。

第一原生轮仍超10秒后，不直接重跑服务：对保留失败现场做一次只读成本probe。实测index observe15ms、open117ms、full4prefix104ms、Pending.open145ms、Pending.review273ms、当前完整两树22ms、session.open202ms、scope/live69ms、完整owner242ms、finalization fingerprint550ms；包含实际安全/归档读取，不将单一成本推成所有平台结论。

核对后同边界尚有两类重复：live先scope核验Original，再frame(Ready)又核验Original+Ready；session.open先scope后live进一步重复scope；Pending.load先read_plan核验Original再captures核验全4帧。最终合并这些同观察重复：live使用scope_binding保留所有profile/HOME/程序选择/原件引用约束，并由该次新鲜Ready完整prefix包含Original；无Live的scope仍完整frameOriginal。session.open有Live时只走一次完整live，无Live仍scope；Pending计划原文/父链严格解析之后load必做新鲜完整4帧，不在此前重复Original。每一操作边界、每步mutation permit/锁/原生退出/端口/源完整镜像及最终全树/回执/index门仍在；无时间窗口重置、跳过归档、缓存成功布尔或官方包修改。

## 反例与验证

105个去重相关功能函数：originals23、restore24、session26、index25、completion7，另只读成本manual入口1项。新prefix测试证明每阶段只访问一次，随后逐一改坏4个frame和3个独立内容blob，每次重新读取/open都拒绝并保留坏字节；故不是一次通过后缓存。phase refs乱序也必须核验最高完整prefix。新held session live反例覆盖Original/Ready/scope/live及归档被改坏后立即拒绝，恢复合成夹具后同API才通过；synthetic Live只检storage admission，不冒真实正常退出。restore相关保留外部写者/未知对象/归档/权限/普通与rearmed各11个真实子进程exit91恢复边界。最终编译/格式/全目标严格Clippy过，UI未改不重复页面验证。

只读probe第一次编译被测试变量root shadow后move触发E0382，cost-build.log保留；仅改helper变量名，cost-build-fixed及成本结果独立保存，不新增产品权限或降低断言。最终成功native binary与build-combined/实际source.json逐字SHA对应，全部产品修改发生于成功构建之前，无后加源码冒用旧binary。

## 两次有假设的原生结果

同类全新隔离Profile、同样原样官方2.99、固定已选动态路由合成GET/models，OS sandbox拒绝外网和日常HOME，只允许loopback；无真实模型或业务请求。

1. prefix-only先验：Start10396/Status267/Stop10296ms，原父子normal0/4捕获TCP缺席/三项补偿通过，全树与锁读回处触及原期限，index6 RestorePrepared/Required，原完整assert exit101/resultfailed。此结果证伪“只改prefix即可解决”的判断，native/原目录保留；未用文件还原或另动作收尾改判Stop通过。
2. 保留同一失败现场仅做只读成本诊断，补上述两项同次重复读取后，才开第二个新Profile的一次普通全链。Start10239/Status271/Stop9415ms（产品入口Stop段9407ms）均原30/5/10内；父子normal0/4捕获TCP全缺席、3项文件原字节/缺席、完整Codex/Grok安全属性、两稳定锁真正释放、完成receipt与独立索引Restored revision7均在原Stop预算完成。函数exit0/resultpassed，重开1020ms NotRequired/canStarttrue。没有追加第二会话/Restart或改超时预算，外层诊断仍acceptedfalse，不冒整版通过。

独立只读native成功两树SHA与已登记completion匹配，4帧/plan/35内容引用/15blob逐项核对；失败轮index6和成功轮index7分别保存，两轮7路径原字节/缺席共14项全部匹配，原后像归档保留。旧10539ms失败按prior-failure-reference.json固化来源摘要，不删除、不覆盖。9415ms只剩585ms余量，不能因此宣称一般大树、其它调用、真实发行构建/Windows或最终候选预算稳妥；后续候选仍须受影响原预算验证。

证据[本批目录](../../../.adg/evidence/0.2.24/stop-fresh-prefix-r1/)与manifest保留原假设/首失败/只读probe/编译错误/后继假设与完整成功、准确source/binary及索引/完整树和原内容。两自有原主端口及进程明确缺席，6精确新协调DB经EXCLUSIVE重获/释放后连同两套Profile/程序/完整原件/stage/archive/test/probe副本清理；失败和成功证据先逐字核验归档，原globalL inode/字节/权限/owner保持、可排他重获释放且无sidecar。unit TMPDIR为空，重复临时根归档验证后删除，工程target保留，无日常/官方/系统或业务改动、新.app/DMG或发布。

## 后继范围

当前同类普通Stop预算样本已修复并过原预算，但一般路径/完整集合/候选预算仍待，不放宽原验收。继续处理真正部分捕获/未登记引用、早期已变化客户端但无plan、首次缺根/完整客户端与路由图、全进程家族/转移监听、旧revision4首错、Windows W0/W1产品接线与最终W2、NAME、复杂System代理、官方发布验证/README与固定候选。Windows远端仍b2e89665d/0eef59a01无新回交；目标active。
