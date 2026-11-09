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
