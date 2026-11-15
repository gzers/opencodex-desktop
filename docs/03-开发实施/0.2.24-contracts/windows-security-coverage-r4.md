# Windows有界访问安全与原件恢复合同 r4

revision: 4；2026-11-15 Asia/Shanghai；归DMD-POST023 AC-P24-08、IMP-34 revision140、TEAM revision14。共同代码 **main 2ebdd3ae0423903e9e713108a09a165ac36ead37**，Windows实现基线 **codex/024-windows-native c47953ccfe4c66a976285eacffd691e3a3b0c01b**；文档输入为含本合同的docs/governance-main实际SHA。r2四trait及原30/5/10/Restart30含Stop子10预算不变；本文明确替代r3的“所有新普通会话都必须完整SD/SACL”前提及相应持久格式/验收，其余r2/r3写集/归属/失败保留约束继续。

## 明确采用的决定及代价

用户把完整SD/SACL安全覆盖合同交本对话评估，并要求明确决定/后继合同。本对话采用 **windows-bounded-access-v1：完整读取并恢复下表所列访问安全投影，不承诺完整审计SACL恢复**。这是显式产品范围调整，不是Windows遇到1314后自行退到0x7，不把partialDiagnostic改名为完整证明。

完整方案没有已证实可用的原普通环境。本人拥有文件、SD看似无审计ACE、普通桌面noJob，均不自动授予ACCESS_SYSTEM_SECURITY。非管理员账户若原本经特殊安全策略授予且token已有效启用SeSecurityPrivilege，理论上可能满足，但不是默认普通用户基线，也没有本项目正例。本轮不构建这种特权环境，不启用权限/提升或替换令牌/借系统服务/更改日常策略或ACL；继续把完整SACL作为所有新会话必需条件，会使已测普通环境始终拒绝Start，不能靠修读取API解决。

新策略在原TokenUser/elevated=false下用READ_CONTROL核验访问安全投影，每个必需投影不可读仍拒绝。**完整审计SACL、整体SACL保护/继承控制及未请求扩展安全信息明确ExcludedUnknown**，不报告Absent、相同或已恢复。同fileID原对象保管可避免主动复制重建隐藏信息，但不是其未被系统/其它有权限主体修改的证明。需要全量审计SD一致性的场景不属于新策略可验收范围，仍须拒绝/另议完整方案，不称“恢复全部安全属性”。

决定允许Windows实现新策略；**c479当前生产Start仍拒绝**。须先完成新native捕获/整树保管/恢复/Closed及H/S/D组合和定向实测，不得删除预检或require_durable_capture便直接启动。

## Git回交复核边界

已fetch代码c47953ccf/证据fe1a54f6e299b96a6542c5b87ae384643bb9d163，消费handoff及原wl1/wl2的shared-minimum-request.json。最新请求实际在wl2-current-reference-20261115-r1/environment-decision-request.json及mac-decision-handoff.md；用户选择在wl2-sd-contract-decision-20261115-r1/user-decision.json。不存在同名新shared-minimum-request，不捏造。

docs净差3601文件限11个新轮次与handoff，全部清单payload逐项校验；最新final-source及library02/product01各39份Rust执行输入对c479核对。可离线重放结果归.adg/evidence/0.2.24/windows-wl2-safety-receive-r4/receive-result.json。这是来源/范围与证据复核，不是5573行实现的完整安全审计；本次不把尚未完成的Windows生产实现整合到main，保持Windows独占分支，后继c479合入本次main共享合同。最终生产集成另按净差审查，旧main EXE不能冒c479产物。新控制记录/目录仍同时满足W0原TokenUser/SYSTEM私有DACL、原生pin与完整性规则；有界投影不替代私有权限核验，也不放宽现有访问主体。

| 部分 | 可接收事实 | 仍不可宣称 |
| --- | --- | --- |
| H普通桌面 | marenben SID1003/session7；空broker22216/FILETIME134391493883001324、原创建句柄、bootstrap/挂起noJob，跨fixturemanager gen1..4 | actualGUI/officialOwner/production均false；最终broker rawExit为null不能填0；旧101/manager32保留 |
| O移动/阶段/引用 | 单文件同对象移动、排他CAS、阶段先刷盘后pointer、partialwrite与PendingOnly拒绝测试有进展 | 整树manifest/custody/workcopy/实际move意图及引用、restore/Closed未完；文本Closed/PartialDiagnostic不授资格 |
| S/D | 保留token/image/父链/argv、精确supervisor原语与sentinel来源增量 | 全生产family/阶段proof与新Stop组合未完，raw非零不改写 |
| 最新构建 | c479 clean MSVC debug bins0/156748ms，107warnings，最终lib8项 | 非release预算/严格Clippy/签名候选/整组验收；中间输入不冒最终源 |
| 清理 | H成功两根22文件98,829,764字节先归档后清 | overallCleanupComplete=false；失败根、现有账户/Profile、7文件670,767字节旧树及普通MSI等不关闭、不绕过删除拒绝 |

## 精确安全覆盖

新会话固定scopeId=windows-bounded-access-v1，不由IPC/导入文件/用户JSON切换，不从full读取失败自动fallback。所列全部投影必需。

| 查询组件 | SECURITY_INFORMATION | 核验/恢复断言 |
| --- | --- | --- |
| owner、primary group、DACL | 0x1、0x2、0x4 | SID/ACL完整有效原生内容；ACE顺序/掩码/继承位/条件保持，null/empty/absent区别；owner/group默认位和DACL present/defaulted/protected/auto-inherit控制位保持 |
| mandatory integrity label | 0x10 LABEL | 独立查询文件label ACE/SID/策略位；进程integrity不替代文件标签 |
| resource attributes | 0x20 ATTRIBUTE | 独立读取资源属性ACE原始属性/值，不删条件 |
| CAP scope | 0x40 SCOPE | 独立读取SYSTEM_SCOPED_POLICY_ID_ACE；成功确认未设置才可Absent |
| process trust label | 0x80 PROCESS_TRUST_LABEL | 独立读取trust-label ACE；无法读取/核验即拒绝 |
| access filter | 0x100 ACCESS_FILTER | 独立读取access-filter ACE及条件/策略数据；同上 |
| 完整审计SACL、全局SACL控制 | 不请求0x8 SACL或0x10000 BACKUP | ExcludedUnknown；不保存假的空SACL，不比较/设置整体SACL，不承诺所有审计元数据相等 |
| 未请求/未来扩展 | 未知 | 不宣称覆盖未来OS能力；必需投影内结构/API支持无法确定则CoveredUnknown→拒绝，不过滤后继续 |

合并mask **0x000001f7**，访问 **READ_CONTROL=0x00020000**。先逐投影QuerySecurityAccessMask确认原生需求恰为READ_CONTROL（0或要求额外权限均不准入），再对原文件/目录保留handle执行GetSecurityInfo(SE_FILE_OBJECT)逐项与合并查询，不能先full失败再降级。内容/目录读取、rename所需data/DELETE等另行最小请求，不请求ACCESS_SYSTEM_SECURITY/WRITE_DAC/WRITE_OWNER捷径。Windows记录实际OS build、API、各请求mask/结果/原码；不支持或是否支持无法确认→拒绝。QuerySecurityAccessMask仅生成需求，不单独证明数据已返回。

owner/group/DACL/label/attribute/scope的READ_CONTROL依据微软SECURITY_INFORMATION文档；trust/filter的额外要求须在同版本Windows原API/SDK及定向实测核验，不因为Mac常量声明就冒普通实机正例。逐项成功且有效才算覆盖，不能只测试0x7。**本策略8项全部成功/整树原生正例仍未取得**，不保证所有普通文件可用；ACL拒绝、占用、加密/reparse/硬链/ADS/不支持文件系统等原门保持。每个必需flag在目标OS是否支持无法确认时也不能把空响应当Absent。

依据：[SECURITY_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-information)、[QuerySecurityAccessMask](https://learn.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-querysecurityaccessmask)、[File security and access rights](https://learn.microsoft.com/en-us/windows/win32/fileio/file-security-and-access-rights)。2026-11-15读取摘要与下载字节SHA归reference-and-decision.json；这些资料不替Windows逐投影实测。

快照验证SD revision/有效结构/有界长度，保存每投影SID/ACL/ACE有效原生内容及present/null/empty状态，不memcmp含指针/未初始化padding，不排序ACE/丢条件，不只存SDDL/hash。只比较各投影定义控制位；合并响应全SACL present/保护位不作完整审计证明。逐项/合并须同原object/parent且前后相互一致；混合时代、权限拒绝、结构未知或范围内漂移均拒绝。重复读取仅本窗口事实，不是全局原子性保证；各阶段继续原CAS/协调重验，不延时。

**范围内未知拒绝，范围外明确不保证**。诊断分列securityScopeId、coveredStatus、auditSaclAssurance=excluded-unknown、fullSecurityProven=false。只有真实新native证明可令coveredStatus=verified，旧partialDiagnostic不得换标签。结果称“声明访问安全范围已核验”，不得称“完整SD恢复成功”。

## 原件恢复断言与失败

文件/目录/缺席、原卷fileID、完整原声明树的内容/认证/路由、父根身份及其安全投影、原非reparse/链接/ADS范围均不缩。仍是**同卷原对象custody**，D跨卷只放私有引用，禁止copy/delete替原对象。工作副本用于运行，恢复移动准确原对象回准确目标，再核内容/fileID/全部8投影；原根缺席回缺席，原根存在保留原根和全树身份。不能省额外生成物/凭据，不能以根ID代所有子文件恢复。

禁止SetSecurityInfo修原ACL/label/SACL凑相等。same-volume rename可能涉及的继承变化也须原生往返读回证实，不能只凭API名称保证。移动意图先持久化、原句柄/父pin/排他和失败poison保持；排他分享不等于阻止一切安全写。excluded审计信息即使同fileID也不加相等断言；不主动改写不能宣称不存在不可见副作用。

- 必需投影open/query/结构/OS覆盖失败（包括5/1314/50/87/缓冲漂移）在首次原件变更前拒绝capture/官方Start，记录准确API/mask/原Token/原码，不修权限重试。audit未请求而ExcludedUnknown是采用范围外，不是错误后兜底。
- 阶段间范围内安全/内容漂移、未知新增文件、替换、缺custody/stale reference→Required/保留现场；不得Stopped/Restart/第二Start/Auto重试，不能删工作副本/custody/phase抹冲突。
- 旧Full-v1事务/类型不降级；无未决事务才可新v2 capture。旧/未知pending只定位和受限显式恢复，不扫旧目录/自动迁移/删原件。
- ClosedEligibility绑定scope与同SID/Profile/D/H/I/source/session/generation，由native完整退出/listener/record/supervisor缺席和全树读回共同生成；本次仅安全保证范围变化，进程/完整内容/真实GUI/预算断言不弱化。文本Closed或hash/fixture绿灯不签资格。

## 持久格式与类型

新 **windows-client-originals-v2** phase/manifest与 **windows-client-current-v2** reference，严格schema；owner-rendezvous-v1、pipe帧schema1/QPC不因共享CONTRACT_VERSION=2自动升级。v1原件格式不转换。

manifest/每phase/current共同绑定 **securityPolicyDigest**。manifest固定securityPolicy={scopeId:windows-bounded-access-v1, queryAccess:0x00020000, includedMask:0x000001f7, requiredProjections:[0x1,0x2,0x4,0x10,0x20,0x40,0x80,0x100], excludedAssurance:audit-sacl-and-unrequested-security-unknown}。拒绝缺失/重复/非规范顺序/未知scope/错误mask/扩大排除，原prevDigest/sequence/binding/manifest链保持。

每原节点securityProjectionSnapshot绑定原ID/存在性/scope及每必需投影的requestedMask/nativeStatus/SD revision/有效control/present-null-empty状态/有效原生结构与摘要。敏感SID/条件结构留原SID私有保管，公开trace只状态/hash；OS/build仅诊断/准入事实，不替native归属。不放假的auditEmpty/fullSecurityProven=true。

当前CurrentReference(Reference<Full>)不能直接换PartialDiagnostic。新增固定BoundedAccessV1原生查询器与字段私有、不可serde/Clone的实际捕获/恢复proof，不接受请求端传bytes/bool构造成功许可；真实v2 capture/manifest/policyDigest/current排他CAS一起绑定。PartialDiagnostic仅cfg(test)且不可转换；旧descriptor/Full函数保持full语义，不用同名函数偷偷改覆盖。decode只给待核数据/PendingOnly，记录重新定位后须native复验才可后继。

阶段create-only/sync/readback先于current CAS；partialwrite/额外phase/poison不补通过。新格式只新代次，旧/未知pending保持，不兼容迁移/默认清空/自动catch-up。先纯测试范围/schema，再native单对象/整树证明；旧Full1314、partial CAS历史不追溯变通过。

## Windows执行与写集

在c479上合入精确main2ebdd3ae0423903e9e713108a09a165ac36ead37，不reset。main本批只给共享声明，不含c479全部实现；docs承接本合同提交，只读正式DMD/IMP/TEAM。最终源码/EXE必须记录Windows实际合入后的SHA/tree，不拿main旧产物代验。

r2/r3单写集保持：windows_client_originals024/**允许独立bounded查询/manifest/v2/binding/phase/reference及整树捕获恢复；原Windows lifecycle/owner-host/S/D/ocxd内部接线和test/windows024-native/**继续。共享本批常量只消费，再变范围/trait签名、其它平台/安装器/根workflow/官方包须带最小理由回交。Mac本批不并行写Windows文件。

先新Token逐投影/拒绝反例、v2纯解析和真实单对象/整树custody/CAS；并行H真实GUI/官方owner/S-D剩余组合。只有全树恢复/Closed及H/S/D前提就绪才开放新Start成功门。新mask有新假设可跑一次，旧full1314不重跑；空H桌面noJob组件证据复用，真实GUI和source变化的路径新验。

依赖齐后单SID、原官方2.99和合成Profile跑受影响生产闭环。原30/5/10/Restart30含Stop子10、来源/HMAC/真实Job/listener/owner未知拒绝保持。debug/partial样例不当release预算/最终候选。官方bundle/descriptor/内tgz摘要沿r2/r3不改。

## 受影响验收表

| ID | 必须验证 | 拒绝/边界 |
| --- | --- | --- |
| B1 普通权限 | 原普通token/elevated=false；8项单独+combined查询/原码/OS支持/QuerySecurityAccessMask；不请求Full/SACL权限 | 仅0x7、失败fallback、必需Unknown/未请求就Absent拒绝；fullSecurityProven恒false |
| B2 组件边界 | audit=ExcludedUnknown；带label/条件DACL/resource/CAP/trust/filter的受支持合成对象比较相同/变化，缺正例的项如实unverified不能用空文件代签 | 错mask/删投影/未知结构/unsupported/拒绝/范围内漂移必须拒绝 |
| B3 v2纯合同 | 8项集合/schema/scope/policyDigest及phase/current同源；缺失/重复/旧schema/篡改/poison反例 | Full-v1/PartialDiagnostic不转v2，旧pending保留 |
| O1 原件全树 | 单文件和原存在/缺席Codex/Grok整树、D跨卷、工作副本→准确原对象恢复：内容/ID/8投影/父根；两锁有限原例外 | copy重建ID、修ACL、覆盖外改、未知生成物、权限/链接/ADS不明；单文件不代整树 |
| O2 中断并发 | move前意图/刷盘/CAS，各move/teardown/restore边界中断/外改/partialwrite，原pending可定位 | 文本Closed不产资格，失败保留custody/Required无后继 |
| H 真实GUI | 复用noJob前提；实际GUI退出/故障、新GUI认证仍原broker/Job/source/H，旧连接失权 | fixturemanager不代GUI；raw未知不填0，丢creator handle不PID收养 |
| SD生产 | source/owner/阶段supervisor重查、全family/rawexit与真实Stop组合 | 未知helper/丢历史/force/服务root非零拒绝；query1060/1不单签Absent |
| P闭环 | 新scope整树capture→原包Ready→普通Stop≤10完整有界读回→Restart/第二Start/Auto资格，原窗口 | 前提未就绪标blocked不启动，旧W-L1成功不填新scope空白，不W2/双SID/发布 |

每轮先写可证伪假设，一次采全独立项/首错/rawtrace及source/tree/实际EXE/toolchain/profile/SHA，批量修复再轮；诊断accepted:false。只用自有合成对象，不改现有marenben日常Profile/配置、token或机器策略；fixture权限变化限专用对象且记前后，不借此给验收token加权限。残留先原SID可读归档核验后清准确自有对象；所有旧删除拒绝不重试/绕过，普通MSI/历史失败树/Profile/Public未完保持。

本轮交付明确采用的有界合同/共享声明/证据复核/后继允许表。整树custody/恢复/Closed、真实GUI/官方owner和S/D组合仍未完成；当前生产Start拒绝。新scope普通原生正例/最终候选均待实证，范围采用不等于通过。不改官方/日常策略，无真实业务同步/模型请求，不W2/双SID，不发布。
