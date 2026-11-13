# 0.2.24 显示别名与独立静态路由声明

revision: 1；2026-11-12。归IMP-34 P24-04，承接[当前正常恢复](client-session-close-r1.md)。代码main e6619359d。accepted=false、非冻结候选；本批扩展静态图恢复中的显示别名/省略开关，不等同通用图或P24-NAME闭合。

## 确认的原样官方语义

从固化024bundle中读出原样官方2.99 tgz，SHA b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c匹配。只在自有根展开源码和原bundle的6722个普通导入文件/Bun（本次所选范围无符号链接），每个官方src文件逐字匹配原tgz；不修改发行包或产品安装，不把测试展开冒Desktop供应链验收。全部输入长度/SHA保留。

源码defaultAliasesEnabled为provider.defaultAliases ?? config.defaultModelAliases ?? false；effectiveModelAliases只给已有模型产生displayName，普通provider的catalogModelSlug仍是provider/encodeRoutedModelId(id)。内斜线编码成连字符，显示别名不成为磁盘slug。组合模型的真正alias另有语义，当前仍不在此静态provider补丁授权范围。

原样官方pure函数实际执行4组：省略全局/渠道开关、全局开启、渠道覆盖关闭、用户modelAliases。global-on下内置opus/g3p、自定义custom-sol显示确实生效，但三条磁盘slug四组始终不变。另调用expandFastRow确认对外模型列表加--fast；它不是这条Codex磁盘目录链的额外路由，不能盲目把Fast选择器塞进artifact恢复声明。Fast能力字段本身保持官方职责，不覆盖或降级。

测试入口test/official299-native/display-alias-route-proof.mjs直接import未改包的default-aliases/catalogModelSlug/deriveEntry/expandFastRow，未跑gather/provider discovery或模型请求；OS sandbox拒绝外网/日常HOME，两个拒绝探针实际通过。原函数/import不替换，不抄一套默认别名regex到Desktop。

## 产品修正与不变约束

declaration此前要求global/defaultAliases显式false、modelAliases必须空，导致纯显示设置或省略默认字段就返回None，普通Stop在准备恢复处被拒。现允许布尔或省略的defaultModelAliases/defaultAliases和有界字符串modelAliases；disabled=true渠道不为本次选中图声明路由，非法disabled类型仍拒绝。声明永远按实际provider键与selectedModels编码，不从alias或显示名生成ID，不采用Ready/cache学到新路由。

缺providers/selectedModels、非法字段/控制字符/过长ID、编码碰撞、动态liveModels及原customModels/combos/disabledModels等未支持图继续拒绝。每个声明必须在实际Ready中存在且所有新Ready路由必须独立声明的原完整精确集合检查保持；没有把白名单改为前缀匹配或跳过未知行。原件/归属/四锁/预算/实际补偿和完整树安全读回不变。省略Fast按官方默认开启合法，非法Fast类型拒绝；不因这个开关添加未存在磁盘路由。

本批只修复有证据的静态别名限制，不把自定义显示名问题混成P24-NAME已经解决。任意中文等非直接合法运行键、派生组显示和官方持续刷新保持仍是NAME剩余原范围。

## 验证

27函数通过：session10、artifact policy17；2个显式manual入口ignored不计通过。新增开关组合与省略、手动别名、disabled渠道、非法类型反例；原精确Ready集合/外来行/内容与权限变化/审计范围等断言保持。编译/格式/全目标严格Clippy通过。本批未重复无依赖变化的正常重启/索引完整性/Windows r5。

新Profile真实官方2.99静态3模型配置：defaultModelAliases=true，渠道defaultAliases省略，gpt-6-sol手动alias custom-sol，Fast=true；三模型是claude-opus-5-5、gpt-6-sol、vendor/gemini-3.5-pro，只有固定合成无用Key和loopback1端点，不使用真实模型或业务同步。准确测试binary SHA b9ccfca67db1237ca605c8f125cb4f2d9474cf27b7a8d8180694a1aded7880ce与27检查及运行副本一致。

- Start9054ms、Status232ms、完整Stop8872ms，原30/5/10不变。
- 独立startup-scope启动前声明3个真实slug，与Ready目录3个slug逐项相等：native-fixture/claude-opus-5-5、native-fixture/gpt-6-sol、native-fixture/vendor-gemini-3.5-pro；显示名实际是native-fixture/opus、native-fixture/custom-sol、native-fixture/g3p。没有给display alias或--fast虚构恢复授权。
- 原服务/父normal0、4捕获TCP缺席、真实catalog/cache/audit补偿/后像归档、完整树Protection及两稳定锁释放、独立completion/IndexRestored revision7均通过；7路径原字节/缺席全true。
- 重开1283ms，Stopped+NotRequired/canStart=true、无PID/port。原完整集成函数exit0/resultpassed，诊断acceptedfalse；不是整版或一般图验收。
- 独立四frame/Scope/Live/plan/完成receipt/outcome/index与35项内容引用核对；额外原生Codex/Grok完整快照SHA匹配完成回执，原件不重写。

## 留证与清理

.adg/evidence/0.2.24/static-display-alias-routes-r1/保存先写假设、固化tgz源与6722输入SHA、官方源码锚点、pure4组和Fast实际回执、源码/build/binary/27结果及真实原包完整恢复/独立路由显示对照、原件/完整树和manifest。不输出runtime secret/admin token。

原服务主端点在完整Darwin TCP表framing中确认缺席，自有服务/launcher/Node/Bun/test/pure探针进程均缺席；其它TCP仅准确产品trace，不伪独立重放未导出端口表。3精确新协调DB排他取得释放后，与自有程序/Profile/全部双客户端blob/index/stage/archive/control测试副本清理；pure导入副本/Bun/官方src约414MB及重复临时回执根在逐摘要保存证据后清理。原globalL inode/字节/0600/owner保持、独立排他取得释放、无新sidecar。无日常/系统/官方修改或业务请求，没有新release/app/DMG或发布。

一般动态/retain/default选择语义、customModels/combos/disabled过滤与别名碰撞图、外部Agent/首次缺根/大树/跨卷等完整范围、显式中断恢复/活跃刷新/完整生命周期集合及旧revision4首错继续；WindowsW0/W1/W2、NAME、复杂System代理、官方发布验证/README/固定候选仍在IMP-34，目标active。
