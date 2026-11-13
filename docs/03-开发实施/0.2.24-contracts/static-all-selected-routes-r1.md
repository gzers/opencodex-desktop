# 0.2.24 静态全选名单及禁用模型的恢复声明

revision: 1；2026-11-12。归IMP-34 P24-04，承接[明确动态选择](selected-live-routes-r1.md)。main ee1d607bd3a8effa076fece0fc8bacf2c2c6e849；accepted=false、非冻结候选，不作为全部一般图或整版通过。

## 来源与规则

从固化bundle读取原样官方2.99 tgz，SHA b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c匹配，只读提取provider-models/build-entries/slug-codec并记录摘要和锚点，未改发行包。官方configuredIds为去重models与retainModels；仅静态models为空才以defaultModel补种子。最终selectedModels非空才形成选择限制，因此空或省略selectedModels在静态图中代表完整模型名单，不是“什么都不选”。disabledModels再按完整slugEquivalenceKey移除对应路由，bare native禁用不扩大到另一渠道同名路由。

先加3个新函数于原产品算法，均实际失败并留原binary/输出：静态名单并集与默认种子；空/省略选择；跨渠道精确禁用及编码等价。随后调整declaration，未采用Ready/cache/发现返回值作授权来源。

新的有界ID列表解析限制数组4096项/每ID1024字节/非空无控制字符，类型错误/null/未知对象拒绝。非空选择仍优先；只有liveModels=false时空选择可取完整配置名单，models非空时defaultModel不凭空变成额外名单成员，retain重复ID去重。动态/省略live模式仍要求非空选择，保持开放图未知。

disabledModels必须同样合法数组，按一斜线provider保持、内ID斜线编码成连字符的官方等价比较；路由编码碰撞仍先拒绝，再过滤disabled，不能借禁用隐藏歧义。全局禁用不当作前缀授权，另一provider同ID保持。combos/customModels及其它未支持图仍未知；原exactReady集合、配置/原件身份/权限与四锁、正常退出/捕获TCP、补偿/整树/终态和原30/5/10预算均不放宽。

## 检查与实际原包

31项相关函数通过（session14/artifact policy17），2个显式manual入口ignored不计执行。3个原失败新例通过，原显示别名/动态明确选择/编码碰撞/非法类型/外来Ready路由和原件策略检查保持；编译/格式/全目标严格Clippy通过。

新Profile真实未改官方2.99配置：static models含claude-opus-5-5与gpt-6-sol，retainModels含重复gpt-6-sol与vendor/gemini-3.5-pro，selectedModels为空（全名单）；disabledModels精确禁用native-fixture/gpt-6-sol。默认显示别名/Fast保留，只有合成无用Key与未运行loopback1地址，不执行动态发现/模型业务请求。

准确binary SHA 2a66bee7c7fc3dd63aaf064e3af978f094a4035e92f47e3ec952f5cd418ce778与31检查及实际副本一致，原包完整供应/镜像核验保持。

- Start9066ms、Status230ms、完整Stop8663ms，原30/5/10不变，原集成函数exit0/resultpassed，外层诊断acceptedfalse。
- 启动前scope声明与Ready实际目录都是2项：native-fixture/claude-opus-5-5和native-fixture/vendor-gemini-3.5-pro；禁用模型没进入目录，retain-only模型确实存在，重复模型不制造重复slug。配置选择/名单的无凭据投影与实际目录对照另存static-filter-proof.json。
- 原父子normal0/4捕获TCP缺席、实际catalog/cache/audit补偿/原后像归档、七目标原字节/原缺席全true、完整树Protection及两稳定锁释放与独立Restored revision7均完成。
- 重开1247ms，Stopped+NotRequired/canStart=true、无PID/port；四frame/Scope/Live/index/plan/receipt/outcome及35项内容引用核对，额外原生Codex/Grok最终快照SHA逐字匹配完成receipt。

## 证据、清理与未完

.adg/evidence/0.2.24/static-all-selected-routes-r1/保存假设、3旧算法失败、官方源摘要/锚点、准确build/source/binary、31结果、真实名单/禁用对照、原件/索引/完成参考/最终树和manifest。无runtime secret/admin token或真实密钥。

原服务主端点经完整Darwin TCP表framing确认缺席，额外捕获TCP只引用准确产品trace，不假独立重放未导出端口表。自有服务/launcher/Node/Bun/test和读取helper均缺席，3精确新外部协调DB逐个EXCLUSIVE重获释放后与自有程序/Profile/双客户端blob/index/stage/archive/control副本及源码提取/重复临时回执清理。原globalL inode/字节/0600/owner保持、独立排他取得释放、无新sidecar，原文件保留。日常/系统/官方配置未改，无业务请求、新release/app/DMG或发布。

本批只是同一声明器常用静态名单/禁用规则覆盖，不代替开放动态/初选、自定义/组合、原生账户/保留复杂图、外部Agent/缺根/大树/跨卷等范围。显式中断恢复、活跃刷新、完整生命周期集合/其它监听与旧revision4首错仍待；WindowsW0/W1/W2、NAME、复杂System代理、官方发布验证/README及固定候选继续，目标active。
