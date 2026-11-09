# Windows剩余项本机定向进展

用户已授权本机连续负责双SID、提升、安装器与第18项的定向修复/验证，继续不core-sweep/全量、不碰日常配置、不发布。[范围和先行假设](../../../../docs/03-开发实施/0.2.23-contracts/windows-local-followthrough-20261109.md)。

## 双普通SID服务与清理：本轮通过

产品采用第13项原sealed程序：source fdf5d4db551f71cbf493dc97b428f8b9a9364a7a/build42631183420/artifact13681803354/lifecycle.3。新增本机测试入口不改产品，测试源码5ad443222585f1bcf395d70703e45200206614d4。最新Git已在代理恢复后拉取确认；第13项此前待推的a6f1bbb40及范围文档f60a4b865已补推。

two-sid-01前置失败原件保留：New-LocalUser的Description52字符超过Windows48上限，账户0/应用Start0；没有产品运行。只缩短测试描述、首错立即落盘、无账户时不枚举全机残留，然后在新目录two-sid-02首次执行产品。

真实本机Windows Server2025 Datacenter/build26100、RDP会话2，两个不同临时普通SID同桌面原生窗口并行，实际未提升、真实Profile、sandbox:false。不同SID并非用同用户多会话替代。

| 项目 | 用户A（SID尾1014） | 用户B（SID尾1015） |
|---|---|---|
| 唯一Start回执 | 1次/成功1 | 1次/成功1 |
| 原30秒启动 | 15973ms | 16992ms |
| 原生服务PID/端口 | 52368/57052 | 95764/58541 |
| 原10秒正常Stop及实际客户端还原 | 7245ms/通过 | 7864ms/通过 |
| 服务原句柄清理前退出 | exit0 | exit0 |
| 账户/Profile/原目录 | 全删 | 全删 |

两边先经实际Desktop process_action和原保护记录/HMAC/健康/原生监听证明各自Start，再同时observe；正常Stop B后A仍为52368/57052且认证健康，随后正常Stop A。两个Stop均1次，不是强制清理冒正常停止。原30/10/5秒、归属/权限/就绪/还原/退出断言不变。控制器protocolPassed/boundedTwoSidPassed true，desktopAclRestored/programUnchanged true，residualProcesses空；独立读回账户和CIM Profile均0、原目录及服务PID消失、原端口无监听。

四项原证据逐账户见two-sid-02/user1和user2：action-001.raw.json/action-002.raw.json，start-runtime-step.json/stop-runtime-step.json，runtime-record-native.json，overlap-runtime-step.json；A另有survivor-runtime-step.json。各自user1-result.json原数组trace保留原顺序；original-exits-before-cleanup.json含完整镜像/SID/创建时间及真实exit0。账户清理和环境在controller-finished.json、post-cleanup.json、host-environment.json；verified-summary.json为独立断言摘要。

这补齐本轮双普通SID服务互不干扰及双账户清理证明，不覆盖旧失败、不自动关闭所有第18项分支，不代表控制台提升、同SID/IPC/凭据或安装器整项通过。以下项继续分开核对。

## 尚未完成

### 后继失败与接续（23:55，Asia/Shanghai）

elevated-peer-01：新临时A加入管理员组后，凭据启动实际仍为medium/type3，执行器在启动A应用前拒绝；两边服务Start均0。取消时B管理器在父首次后代快照之后启动，导致bootstrap清理而管理器漏入账本，B账户/Profile被正确保留。后续按environment.json原生PID/镜像/SID/创建时间精确回收Manager及后代，manager-recovery.json证明账户/Profile/目录删除、残留空；原controller-finished.json失败不覆盖。对应测试修复dc235ad3b：每次进程准入立即原子保存账本，Manager/driver启动前和供应期间检查取消，取消等待期持续观察后代。不改产品或原预算。

elevated-peer-02：用户同意协助正常UAC，取得actual elevated/type2的临时A与Desktop；但WebView2后代实际属于登录RDP用户SID尾1001，GetOwnerSid返回0，非期望临时用户1018。原SID断言正确拒绝，两边服务Start0；B响应取消后无服务调用并完成账户清理。两个跨令牌WebView在后续只读检查时已退出，没有按进程名终止。A无残留进程但Windows仍报告Profile Loaded，账户/Profile保留；已禁用该精确临时账户ocxaccu16625a3be3并移除其管理员组身份，未强制卸载用户hive。retained-account-containment.json明确该阻点。此跨用户UAC/RDP配方不冒充真实Administrator控制台，转用console会话1的Administrator500及Sandbox保护日常配置。WebView官方维护者讨论仅作为原理参考，链接定位在webview-elevation-primary-references.json，不采用未证实flag或降低SID检查。

installers-build/run42642616981：原生日志确认新源码NSIS/MSI均已成功生成，但新增封存入口读取含合法空package-lock键的policy时，普通ConvertFrom-Json失败，没有上传安装原包；原失败日志ZIP7193字节/SHA256 aa12820756021c001e5aae4709bcec0e278aaed765b431dc634b8556dbb386e3原样保留。修复0302b6608仅改-AsHashtable并始终保存已生成原包；新run42645465791成功，新artifact13687777600/ZIP618204696字节/SHA256 57db0b25ef98f725c825fbe8b03c774cfd33c6846f3dd232b4796cbbd768a9d6及4成员来源清单全部核验，产品appsTree仍a4acf4ad2c0fb25420471d50a7d8e11897362068。未自动改产品语义。

本机NSIS已完成默认安装/实际供应/同版本重装runtime和用户数据保留/卸载数据保留，全部原生exit0；中文自定义路径后续仍在执行。MSI本机default首次返回1625，原生日志明确Non-assigned apps are disabled for non-admin users；本机策略只读、不修改、不重试该失败配方。已另派同一原包的临时Windows MSI定向验证，使用仓库原有临时Server客户端等价policy夹具并恢复，不跑全量，不把临时环境结果冒本机已安装。

控制台供应fixture构建42647303001/同源2da679b9e成功；工具artifact13688781911/1734447字节/SHA256 08a014318e68d4b79389e8e02022078e891d019301bc93bf6e013123a13d8619，工具自身SHA256 f0af58e3990cb02ab92c0e957dc9b4962ae1e5c0a6111a120f1a7810bb07e545，产品appsTree与资源descriptor摘要均匹配。只调用现有offline provisioner，不启动服务或增加产品命令。console-01已在真实SID500/session1启动隔离任务，先由产品初始化Sandbox，再原生供应；尚未有服务Start回执。额外小进程退出码夹具曾被自动审批blocked by policy拒绝，未换方式重试；语法检查通过，原服务退出码仍须实际console retained handle证明。

- 提升/控制台场景；必要时保留环境限制，不用普通令牌冒充。
- 固定端口冲突、同SID/IPC/凭据等剩余实际边界，按已有证据和diff选测。
- 新同源NSIS/MSI构建及隔离账户下实际安装/保留/更新/卸载。已派仅构建任务42642616981/source9ae6b14f71a006e85e53a3c9f327feccccb19d1e，不执行全量/安装/测试，等待新制品后在本机处理。
- 第13项遗留三个编译/下载缓存删除曾被自动审批blocked by policy拒绝，不绕过，仍按原清单单列。

另一次额外提权只读进度观察也遭自动审批blocked by policy，命令未执行；没有换机制绕过，实际结果仅用既有控制器回执。目录各有evidence-manifest.json，原字节大小/SHA256已核验。

## 2026-11-10 00:12 接续核验（Asia/Shanghai）

上节23:55为当时快照；当前控制台、NSIS、隔离MSI均已结束，以下为新回执，不覆盖旧失败。代码9859977d6c588526992cde87ffa6adffb350952e，产品仍为fdf5d4db551f71cbf493dc97b428f8b9a9364a7a/appsTree a4acf4ad2c0fb25420471d50a7d8e11897362068/lifecycle.3。以下三个批次通过即停止独立重验。

| 范围 | 实际结果 | 清理/限制 |
|---|---|---|
| console-01 实际Administrator500/session1/high/type1，Sandbox | 唯一Start10718ms，PID8132/49331，正常Stop及还原6716ms，原服务句柄exit0 | private删除、临时任务删除且lastResult0，日常配置摘要不变；不等于默认Profile/DPAPI验收 |
| nsis-01 本机新普通SID1020，未提升 | default/custom中文路径各安装/原供应/同版本重装/卸载均exit0，runtime及用户哨兵按要求保留 | 账户/Profile/目录删除，ACL恢复，残留进程空；不启动服务或同步 |
| msi-01 本机新普通SID1021 | 首次实际安装exit1625；日志Non-assigned apps are disabled for non-admin users | 本机Server策略只读不修改，不重复失败；账户/Profile/目录删除，ACL恢复，残留空 |
| msi-isolated-01 临时托管Windows，新普通SID1003/type1但elevated:false | run42649281070成功；同一原包default/custom各安装/供应/同版本重装/卸载exit0，runtime/用户哨兵保留 | 账户/Profile/目录删除，ACL恢复，残留空；既有Server客户端等价夹具策略精确还原，AlwaysInstallElevated未改；不能记为本机MSI安装通过 |

隔离MSI输入build42645465791/source0302b66082e2c0314e7911f1f9c771d265dddfdf/artifact13687777600。输出证据artifact13688898895/131967字节/SHA256 d6e3f12e03dea24adc06d6969c91d911121ecb36f498df34827f624c1d75e82e，原ZIP和展开件均已留存。各批evidence-manifest逐文件bytes/SHA256核验；大制品/下载分段和私有缓存不入Git，仅来源/原摘要。fixture工具源2da679b9e与产品appsTree、descriptor一致，program构建42631183420和installer构建42645465791的来源并列，不冒充相同EXE字节。

unchanged-boundary-source-proof.json证明旧run42593238885到当前的IPC/instance/data_root/keychain/credentials/windows_user_scope对象完全相同，可复用此前crossPipe真实access-denied、同SID重复实例被拒且原窗口/元数据保持、peer业务数据拒绝等已过范围。DPAPI仅synthetic subscope own-roundtrip/peer-denied，不升级为完整凭据或同SID多会话验收。

Profile1018仍Loaded:true，无已证明残留进程；ocxaccu16625a3be3仍禁用且已移出Administrators。禁止强卸hive/删Loaded Profile或重登日常会话；这仍是未解决的清理阻点。三个先前被审批拒删的缓存目录也继续单列，不能宣称全机已清净。

H-W6待定向补证：实际Administrator控制台high + 当前15119 RDP未提升、不同SID且两会话Active，同时运行固定端口冲突及Stop peer保持。复用同源sealed程序，新增测试适配器只迁移source/runtime pin、捕获原生action原文和真实退出码，原动作预算/归属/就绪/还原断言不变。A一次auto Start；B一次预期fixed-port conflict（原30秒无自有记录且typed port_conflict），A同PID端口健康；B再一次auto成功；双方observe；Stop B后A保持，再Stop A。这里A1次成功Start，B2次Start尝试中1次预期拒绝、1次成功，不能写成每方仅一次总尝试。此为尚缺跨桌面/冲突证明所需前置，不重跑已通过孤立批次。Sandbox保护日常配置，不新增产品或维护包行为。

## cross-session-01反证与H-W7（2026-11-10 00:23）

真实console500/session1/high/type1与RDP1001/session2/未提升type3均Active。A Start13324ms/PID81400/50106，B固定50106 typed port_conflict及原30秒无自有记录通过，A原代次健康；B auto Start13788ms/PID93216/57395，双方observe同代次健康。RDP正常Stop一次返回lifecycle_stop_unknown，process_action6797ms，原HMAC健康已证实，内部失败阶段未保留，因此原因仍未知。A正常Stop尚未执行，Stop后peer保持未取得；原样失败保留，不能写H-W6通过。

控制器首错后finish只按自有SID/image/FILETIME/保留句柄清理：两边private删除、daily摘要保持，临时任务删除/lastResult0，原PID和监听缺席；这是失败夹具清理，不冒正常Stop或exit0。本旧适配器未采集清理前真实退出码，明确缺证；后继已补原句柄读码。原四项见cross-session-01/verified-summary.json、每动作action.raw.json、invoke-trace.json、runtime-step.json、reply和finished/task-cleanup。全部manifest摘要保留。

H-W7先补固定内部失败阶段/毫秒序列，仅app.log，不改变公开失败、归属、实际还原、Stop逻辑、HTTP认证或30/10/5秒。另在POST响应读回后加一个固定阶段，以区分传输失败与服务端返回未确认。诊断源4d783585645206c52b49bce8656475d26940c1d7，先构建新Desktop及同源供应fixture，不用旧二进制猜新诊断。下一轮只补受影响跨会话启停协议；没有新阶段证据就停止。未更改维护包，未重验安装器/已通过独立单用户。

Profile1018只读事件显示23:33原加载事件，仍Loaded；没有卸载或新持有者线索，不凭猜测强卸/删账户。托盘六处相关Git对象与2c6基线全部相同，tray-boundary-source-proof.json允许复用旧双方隐藏重开及原句柄正常退出范围，不声称本轮新交互。

### H-W7新源构建已通过，实际运行仍待（00:40）

代码4d783585645206c52b49bce8656475d26940c1d7/appsTree d2facb41f19ce1ef6930c743460c4f00010bfc12；build42653885333完成。原program artifact13692037223/304979005字节/SHA256 cf5c5e319102b921ab204dfb990214363a829df7428f603bf3ada41d971a0ad6，9成员全部核验，Desktop b0f671ef6406a1af2daae14ab93cb8b4c55ada3162c88c74c454291e4f87e5a7区别于旧程序。原绑定3/认证状态与Stop9用例实际通过，未跑全量。lifecycle.3及ec14运行包摘要保持。

同源fixture build42653892698成功，工具EXE SHA256 91bc8038a7ed34a2d796308a20f229437e04ae8fa019fe9696dfa9e88f45870f，来源/appsTree/descriptor核验。两构建所有ZIP均由指定代理下载并核对API摘要；watcher最后汇总时因整型字典key导致ConvertTo-Json错误，发生于全部下载解包之后，downloads-verified.json单列，没有重派构建或重下。原失败状态不抹去。

下一cross-session-02启动准备已采用新源码程序和新fixture。H-W7固定阶段诊断外，在失败清理前采集原句柄真实退出码、RDP自己新鲜Sandbox客户端路由字段/摘要，不追加产品动作。只读正式task查询的首次HResult格式化有符号转换错误亦保留，r2以原生COM错误80070002证明精确opencodex-proxy任务不存在；没有修改任务/策略。此只读结果本身不证明Stop通过。HTTP close边界仅作待证源码线索，不写为原因或修复。

## cross-session-02：新源码有界通过，停止追加该场景（2026-11-10 00:47）

产品及适配器4d783585645206c52b49bce8656475d26940c1d7/build42653885333，fixture同源build42653892698，lifecycle.3字节保持。实际控制台Administrator SID…500/session1/high/type1，RDP15119 SID…1001/session2/未提升type3，两会话全程Active。15119虽属于Administrators但实际令牌未提升，不冒称非管理员组账户。Sandbox保护日常配置，不能替代默认Profile/全凭据/同SID多会话验收。

| 原四项 | 控制台 | RDP |
|---|---|---|
| 启动回执原文/数量/唯一性 | start-d816243c/action.raw.json；成功Start1、正常Stop1，无额外成功回执 | conflict+start/action.raw.json共2次Start尝试：预期port_conflict1、成功1；正常Stop1 |
| 启动trace/分类 | 13399ms，PID94072/61175，官方状态/受保护record/HMAC/原生SID及token一致；失败分类空 | 12692ms，PID24676/51582；同样全部一致，预期固定peer冲突单独typed port_conflict |
| 停止trace/真实原退出码 | 原9621ms完成正常Stop及实际还原，reply-5和original-exits-before-cleanup原服务exit0 | 原7862ms，reply-4同样还原/原服务exit0；新鲜客户端三路由字段/opencodex provider/原端口均清除 |
| 清理 | private已删、daily摘要保持、角色全部已准入代次退出；无新账户/Profile | 同左；不删除现有日常RDP/console账户；task-cleanup证明临时任务移除/last0，原PID/端口空 |

原public action DTO没有PID/port字段，因此不伪造“原回执含PID”；一致性由同一动作目录runtime-step/官方状态/受保护record/HMAC及reply中native identity/token关联。verified-summary.json逐条独立断言原文、成功数量/唯一性和所有PID/端口一致性。原30/10/5秒保持，所有get_status_snapshot调用<=5秒。双方正常Stop后原服务句柄exit0与清理前数组另核验，不用事后PID缺席推退出码。

RDP对控制台61175固定冲突经原30秒无自有记录；控制台原94072/61175保持。RDP随后自动51582，双方同代次健康；Stop RDP后console reply-4仍94072/61175/healthy，再正常Stop console。controller固定冲突/overlap/peerSurvivedNormalStop/passed均true。按用户要求本场景已过即停，不重复孤立或本场景。

必须保留因果限制：本轮产品仅新增固定失败阶段日志，并未改变Stop/HTTP/维护包；因本轮通过而没有失败阶段可用于确定cross-session-01的旧lifecycle_stop_unknown原因。旧失败仍真实、原因未证实，不能说诊断补丁治愈它；HTTP close仅源码待证线索，未实施放宽。当前有界通过也不是整版或稳定性证明。Profile1018仍Loaded、账户已禁用且移出管理员组，继续作为独立清理阻点；原42545767548 token-pointer失败和技术原型采用限制保持，不发布。

## 用户决定：Mac发版与本机尾项并行（2026-11-10 01:03）

用户已要求代码交Mac发版、本机继续跟进。代码4d783585645206c52b49bce8656475d26940c1d7及先前文档04336c43ab024fa28f2d1d7008f068c412b70e8c已远端核验；最新发布交接入口为docs/03-开发实施/0.2.23-contracts/windows-to-mac-release-handoff-20261110.md。本机Profile/缓存清理和旧Stop首错持续归因不再由Windows线单独挂住Mac发布；未完成事实和已知风险保持，Mac仍负责真实最终来源/签名/制品/其它产品门。本线没有执行发布，未声称Mac已收件或已发行。

本机已继续做精确只读核对：Profile1018仍Loaded且SID/SID_Classes两个HKU hive都存在，ProfileList State0及RefCount缺席没有被误判成未Loaded；最后100条Profile事件仅原加载记录，没有新安全卸载线索。profile-background-20261110.json和决策回执单列；未重跑已过产品、没有强卸hive或注销日常会话。后续按正常释放的新状态或新的失败固定阶段继续。

## 本机尾项继续（2026-11-10 01:16）

新原生WTS快照减少旧全机CIM盲区：正常RunAs快照478进程无SID1018匹配，仅0/4 owner未知；Profile仍Loaded。后继真实high/type2的LSA只读API成功发现SID1018两条原23:33:23 type2/session0登录记录，但不能因此证明hive持有者或强卸安全。LSA短命helper在CIM准入竞态失败，原退出码缺证，整轮采集不标过；原结果与失败分别保留。没有新产品运行、没有Profile加载/卸载/日常会话操作、没有重试审批拒删缓存。Mac发布交接保持，本机原Loaded:false清理门仍未满足。

详见docs/03-开发实施/0.2.23-contracts/windows-local-tail-profile-followup-20261110.md和本目录local-tail-20261110-0110原件/manifest/verified-summary；暂无线索可安全进一步变更，等待真实释放状态或明确持有者证据，不盲重跑。
