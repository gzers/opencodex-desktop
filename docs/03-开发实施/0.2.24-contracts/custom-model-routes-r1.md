# 0.2.24 普通自定义模型行的恢复声明

revision: 1；2026-11-12。归IMP-34 P24-04，承接[静态全选与禁用](static-all-selected-routes-r1.md)。代码main ccb6d16d5ebca28229047d3c237bc2b561e14a01。accepted=false、非冻结候选；本批纳入普通customModels覆盖/新增，combos、原生账户复杂图、开放发现及其它一般范围仍未完成。

## 原样官方语义与独立来源

从固化bundle读官方2.99 tgz，SHA b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c核对，按具体路径只读提取routed-gather/build-entries/config-schema/slug-codec并记录SHA/锚点，不改官方包。routed-gather按真实provider/modelId构建customModels，用canonical routedSlug覆盖同slug的发现行或新增自定义行，元数据/显示名不改变id。最终build-entries仍以渠道非空selectedModels和全局disabledModels过滤普通自定义路由；只有真正combo条目有另一种豁免，本批不混用。

declaration旧实现将所有非空customModels直接判未知，还接受空对象型customModels；先加两函数实际失败，原准确binary/输出保留。新逻辑要求有界customModels数组（最多4096行）、有效已存在provider和有界非空modelId，独立输入来自启动前JSON，绝不由Ready/cache补齐授权。每provider收集原生ID：同模型可普通覆盖，不多授权一个slug；无非空选择限制的静态完整名单可加入新增custom id；非空选择仍只声明选中项。disabled过滤继续最终执行。

普通名单与自定义ID统一检查编码：不同原生ID编码成同slug时一律拒绝，即使该custom未选或最终disabled也不悄悄绕过；重复custom声明、空/非字符串ID、缺失provider/无效custom容器拒绝。明确disabled provider不会因此被重新启用。自定义能力字段由官方原样解析/投影，Desktop不自行改写，不用缩能力、combo别名或显示名冒真实路由。原exactReady集合/执行配置复核/所有原件与权限/正常退出/四锁排他/完整树/终态和原30/5/10不变。

对有额外原生状态/影子行/发现过滤的配置，声明与实际Ready不符仍保持恢复待处理，不泛化为全部custom图已支持。开放live无选择仍未知，combos不因custom支持而放行，P24-NAME剩余也不闭合。

## 检查与真实官方包

33项相关函数通过：session16、artifact policy17；两个显式manual入口ignored不计执行。新增普通覆盖/新增、选择优先、禁用、编码碰撞、重复/缺渠道/坏类型反例；原选择并集/显示别名/动态有限选择与精确集合、原件权限/审计反例保持。编译/格式/全目标严格Clippy通过。

一轮新Profile真实原样官方2.99完整供应/镜像核验，准确binary SHA 2f0acf9b9318a862578193c0e54bff2be09a813902f6efa9f64e77c28caedcc7与33检查及执行副本匹配。配置只有静态名单与合成无用Key/未使用loopback1地址，无发现/模型推理/业务同步请求。custom配置分别覆盖gpt-6-sol、新增vendor/custom-native、禁用custom-disabled，selectedModels空代表全静态名单，已有显示别名/Fast设置继续。

- Start8550ms、Status240ms、完整Stop8578ms，原30/5/10不变，原集成函数exit0/resultpassed，外层acceptedfalse。
- before-start声明与实际Ready都只有native-fixture/gpt-6-sol及native-fixture/vendor-custom-native，custom-disabled确实缺席。
- 原样官方实际目录中覆盖行为Custom base、context196608、text模态及low/medium两档；新增行为Custom added、context262144、text。字段逐项核对actual-custom-route-proof.json，不声称对未显式声明的其它能力作完整比较。
- 原服务/父normal0、4捕获TCP缺席、实际三件apply/原后像归档、七路径原字节全true、完整树Protection/两稳定锁及Restored revision7均完成。
- 重开1261ms Stopped+NotRequired/canStarttrue，无PID/port。独立frame/Scope/Live/index/plan/completion/outcome及35内容引用核对，额外当前Codex/Grok完整原生快照SHA逐字匹配completion。

## 证据与清理

.adg/evidence/0.2.24/custom-model-routes-r1/保留假设、2旧算法失败、官方源摘要/锚点、准确source/build/binary、33检查、实际原声明/目录ID/能力投影、原件/index/plan/receipt及最终树与manifest。没有runtime secret/admin token或真实密钥。

原主端点经完整Darwin TCP表framing确认缺席，其它TCP仅准确产品trace，不假独立重放未导出列表。自有service/launcher/Node/Bun/test与读取helper均退出；3精确新外部协调DB各重获/释放EXCLUSIVE后连同自有程序/Profile/双客户端blob/index/stage/archive/control副本和少量源码提取/重复临时根清理。原globalL inode/字节/0600/owner保持、独立排他取得释放、无新sidecar，旧文件保留。无日常/系统/官方修改、真实业务请求、新release/app/DMG或发布。

一般组合/开放发现/原生账户图、外部Agent/首次缺根/大树/跨卷等全客户端、显式中断恢复/活跃刷新/完整生命周期集合与其它监听及旧revision4首错继续；WindowsW0/W1/W2、NAME、复杂System代理、官方发布验证/README/固定候选仍在IMP-34，目标active。
