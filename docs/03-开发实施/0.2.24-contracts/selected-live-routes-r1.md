# 0.2.24 明确选择清单的动态发现恢复

revision: 1；2026-11-12。归IMP-34 P24-04，承接[显示别名静态路由](static-display-alias-routes-r1.md)。main8fd4bbf45；accepted=false、非冻结候选。本批覆盖live发现且已有非空selectedModels的渠道，不等于开放发现/全部一般图完成。

## 独立声明依据

固化官方2.99 tgz SHA b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c核验，按具体路径只读提取build-entries/retained-sync/provider-models/slug-codec源码并记录SHA与锚点，不改发行包。

retained-sync为非disabled且selectedModels.length>0的渠道建立selectedModelsByProvider；build-entries再通过同一routedSlug/slugEquivalenceKey将最终provider条目限制在该集合。空清单没有建立限制，意味着开放列表，不能与Some(empty)混同。因此非空用户选择本身能在任何发现请求之前声明有限路由ID，live模式并不必然使声明未知；未选择完整名单才仍然未知。

先加定向测试在原e6619359d产品算法上实际失败：明确Live选择claude-opus-5-5和vendor/gemini-3.5-pro仍返回None，原stdout/stderr与准确binary摘要保留。随后只将declaration的liveModels约束改为布尔或省略，并要求动态/省略模式的selectedModels非空；ID仍从原JSON选择清单独立编码，不从发现返回值、Ready或cache学习。静态模式原行为保持，非法类型/空动态清单/null/冲突及所有未支持图继续拒绝。

原artifact policy的精确集合检查未改：每个声明必须实际出现在Ready中，所有新增Ready slug必须在独立声明中，外来行或发现缺项仍拒绝全批恢复。原Scope与执行时配置复核、同阶段连续性/权限、父子正常退出/实际捕获TCP、四锁持续排他、原件/plan/CAS、完整树/两稳定锁及原30/5/10保持。不能用一个provider前缀代替完整路由授权，也不把未知发现作不存在。

## 合成发现夹具与严格首错

新增lifecycle_loopback_models_fixture.rs只被测试模块编译：系统分配127.0.0.1端口，仅接受GET /models或/v1/models，返回固定4个合成ID（3已选、1未选），不提供推理/业务接口。配置中models故意是stale-known-model，选择清单为3个其它ID，以证明使用真正发现结果而非静态旧列表。记录请求数量及有界方法/去query路径，不记录headers/凭据/body。线程/监听由Drop与显式finish清理。

初夹具JSON宏的array.map表达式编译失败，原输出保留，修为先构造变量。第一新Profile发生1次未识别本地请求/连接，0成功发现；旧记录没有真实首行，不能补造它是某种URI/方法。该轮Ready目录为空，恢复精确集合门正确拒绝，Stop仍保留Required/索引Stopped revision5，未实际补偿，acceptedfalse保持。

随后精确官方buildModelsRequest只读调用给出GET /v1/models/no query，不能冒它为第一轮实际wire。测试夹具增加方法/path诊断，先正规解析origin或absolute request target并去query，再按相同GET与精确path准入；不新增其它业务入口。新Profile第二轮才有真实GET/models正例。产品声明算法、恢复断言、原超时均不因夹具修正放宽；成功不覆盖第一轮失败。

## 验证与实际结果

28项相关函数通过：session11、artifact policy17；2个显式manual入口ignored不计通过。新增live=true/省略均使用非空清单、empty/null动态未知、非法live类型拒绝；原碰撞/外来条目/原件/权限/审计等断言保持。编译/格式/全目标严格Clippy通过。

实际第二轮准确binary SHA 617af4879cbeb7e1167c069a6c4989a690a9417c4e152501e16858d7f5abc5bb与28检查、实际运行副本一致，未改官方镜像由首次现场完整再核验；新的合成Profile，日常HOME/外网被OS sandbox拒绝。只有本轮loopback合成发现，不访问真实渠道或发送模型推理/同步请求。

- 本地发现：1次GET /v1/models，unexpectedOrBusinessRequests=0，fixture listenerClosed=true。
- Start6591ms、Status232ms、完整Stop8815ms，原30/5/10不变；原服务/父normal0与4捕获TCP全缺席，真实三项apply及原后像归档、整树Protection/两稳定锁释放和独立终态均完成。
- before-start scope声明3个真实slug，实际Ready恰好3项；unselected-live-fixture与stale-known-model均没有进入目录。显示别名/Fast开启仍保持真实ID，未从发现返回集合扩大声明。
- 索引schema3/Restored revision7、七路径原字节/原缺席全true；重开1256ms Stopped+NotRequired/canStart=true、无PID/port。原集成函数exit0/resultpassed，外层diagnostic acceptedfalse，不冒整版/候选验收。
- 两轮独立四frame/Scope/Live/index及每轮35项内容引用SHA/0400/长度核对；第二轮plan/completion/outcome、三项归档旧后像原字节分别等于Stopped帧。额外当前Codex/Grok原生完整快照SHA匹配完成receipt。第一轮revision5/空Ready/拒绝与错误统计全部保留。

## 证据和清理

.adg/evidence/0.2.24/selected-live-routes-r1/留存假设、旧算法失败、官方源摘要/过滤锚点、夹具编译错误、两轮准确build/source/binary/首错与实际发现统计、原声明和目录对照、原件/plan/receipt/整树与manifest，不含runtime secret/admin token或真实密钥。

两个服务主端点及两个合成发现监听端口均经完整Darwin TCP表framing核对缺席；额外服务TCP仅原准确产品trace，不伪独立重放未导出表。自有服务/launcher/Node/Bun/test/夹具线程和读取helper均退出；6精确新外部协调DB逐个EXCLUSIVE重获释放后，与两套自有程序/Profile/双客户端blob/index/stage/archive/control副本和重复回执/少量源码提取目录清理。既有globalL inode/字节/0600/owner保持、独立排他重获释放、无新sidecar，原文件保留。无日常/系统/官方配置修改或真实业务请求，无新release/app/DMG或发布。

开放动态发现/空选择与初选流程、retain/default/disable过滤、自定义/组合图、首次缺根/外部Agent/大树/跨卷等一般范围、显式中断恢复/活跃刷新/完整生命周期集合及旧revision4首错仍待完成。WindowsW0/W1/W2、NAME、复杂System代理、官方发布验证/README/固定候选继续，目标active。
