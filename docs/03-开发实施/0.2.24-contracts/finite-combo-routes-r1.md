# 0.2.24 有限目标组合的恢复路由声明

revision: 1；2026-11-12。归IMP-34 P24-04，承接[普通自定义模型](custom-model-routes-r1.md)。代码main c12c8d03090c7c850a2e8ccc869fcbdc9a88ddf3。accepted=false、非冻结候选；本批组合支持只用于已有组合配置的正常恢复，不作为渠道名称修复或能力替代方案，P24-NAME未关闭。

## 原样官方规则

固化官方2.99 tgz SHA b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c核验，只读提取combos/types/index/identifiers和catalog aggregation/routed-gather/build-entries并留摘要锚点。官方comboPublicModelId为trim后的alias或combo/id；deriveComboCatalogModel要求全部成员顺序/目标对应且上下文/模态信息充分，输出上下文最小值、模态与推理档位交集。普通组合公开条目有自己的slug，不能将它当provider显示名。最终disabledModels会过滤公开qualified slug；combo选择不受另一provider的selected白名单误约束。

原实现将非空combos全部判None；先加2函数实际得1正例失败/1拒绝反例通过，原binary/输出保留。新add_combo_declarations使用启动前原JSON，处理有限目标组合：canonical combo/id或合法单/双段公开alias、按官方64字符id段语法；每target必须对应本次已独立声明provider/model原生ID，目标重复/缺失/嵌套/未界定来源拒绝。非空组合与physical provider combo或同名provider冲突拒绝，公开ID与任何原路由/其它组合相同拒绝，总路由仍限4096、组合限1024、每组目标限256。

nativeAlias接管（包括true）、bare OpenAI原生家族alias等复杂影子图保持None；不复制官方native知识库来猜授权。普通alias无效类型/超过一斜线/保留combo命名空间拒绝，null/空白alias沿官方canonical规则；组合策略与元数据仍由官方配置校验/派生决定，Desktop不执行请求或改能力。

组合目标声明是有限授权来源，不是从Ready学模型。实际派生缺项/未知能力导致目录没生成组合时，原精确Ready集合门仍拒绝恢复，不能因预先配置ID就接受任意内容。原original/Scope/Live/CAS/阶段连续性、四锁/正常退出/捕获TCP、补偿/整树/终态和原30/5/10均保持。最终disabled过滤沿前批完整公开slug编码等价，不扩大为前缀或任意别名匹配；更复杂canonical-disabled/nativealias情况未核验不称完整支持。

## 检查与原包实测

35相关函数通过：session18、artifact policy17；2个显式manual入口ignored不计执行。新增canonical/qualified alias/disabled组合声明与遮蔽/保留命名空间/坏alias/nativealias/空缺重复嵌套target反例。编译/格式/全目标严格Clippy通过，旧正例失败保持原目录，不降原恢复断言。

准确binary SHA 204e73c4c0a46f2df70fb64440a3989ed18108b5bb22054608c774224adbc263与35检查和新Profile实际副本一致。官方原样2.99完整供应/镜像核验，静态2个目标alpha-fixture/beta-fixture，显式custom元数据分别196608/text+image/low+medium和131072/text/medium+high；配置一个2目标failover组合mixed、一个公开Public/fixture的单目标named组合、一个Public/hidden被禁用组合。合成无用Key和loopback1端点，不运行发现/模型推理/业务同步请求。

- Start8689ms、Status227ms、完整Stop8701ms，原30/5/10不变，原集成函数exit0/resultpassed，外层诊断acceptedfalse。
- startup-scope启动前独立声明与Ready实际4条路由精确相等：Public/fixture、combo/mixed、native-fixture/alpha-fixture、native-fixture/beta-fixture；Public/hidden未进入目录。
- mixed实际目录为131072上下文、text、仅medium推理档位，符合两目标交集；公开单目标别名仍196608、text/image、low/medium。actual-combo-capability-proof.json留实际字段，不把组合降能力方案冒NAME透明改名。
- 原父子normal0/4捕获TCP缺席、三件实际补偿/原后像归档、七目标原字节全true、完整树Protection及两稳定锁释放和独立Restored revision7均完成；重开1264ms Stopped+NotRequired/canStarttrue、无PID/port。
- 独立四frame/Scope/Live/index/plan/completion/outcome及35内容引用核对；另当前完整Codex/Grok原生快照SHA与完成receipt逐字相等。未用成功覆盖旧误拒或改变原catalog样例。

## 证据和清理

.adg/evidence/0.2.24/finite-combo-routes-r1/保留假设、原正例失败/反例通过、固化官方源摘要/锚点、准确build/source/binary、35检查、真实4路由与能力交集、原件/index/完成参考/最终树与manifest。没有runtime secret/admin token或真实Key。

原服务主端点经完整Darwin TCP表framing确认缺席，额外TCP仅原准确产品trace，不假独立重放未导出列表。自有service/launcher/Node/Bun/test和读取helper均缺席，3精确新外部协调DB各EXCLUSIVE重获释放后与自有程序/Profile/全部双客户端blob/index/stage/archive/control测试副本及少量源提取/重复临时根清理。原globalL inode/字节/0600/owner保持、独立排他取得释放、无新sidecar，原文件保留。未改日常/系统/官方配置，无真实业务、新release/app/DMG或发布。

nativeAlias/原生账户/未声明target/开放发现等一般图、外部Agent/首次缺根/大树/跨卷及全客户端，显式中断恢复/活跃刷新/完整生命周期集合/其它监听与旧revision4首错继续。WindowsW0/W1/W2、NAME、复杂System代理、官方发布验证/README/固定候选仍在IMP-34，目标active。
