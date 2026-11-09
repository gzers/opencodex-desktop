# Windows本机定向结果与保留阻点

2026-11-10用户后继决定：本机Profile/缓存清理及旧Stop首错持续归因转Windows后台，不再由本线单独挂为Mac发版前置；已推代码与原通过证据交Mac继续整版发布。旧失败及未完成状态保留，正式来源/签名/其它产品AC不由本说明代签。最新派发及角色边界见[Mac发布交接](windows-to-mac-release-handoff-20261110.md)。


2026-11-10，Asia/Shanghai。用户已扩大授权到双SID、安装器及第18项剩余失败的定向修复/验证，沿代码codex/0.2.23-integration和文档docs/governance-main回交；不core-sweep/全量、不改日常配置、不跑真实业务同步、不发布。通过项已停止追加运行。

## 当前结果及因果边界

代码完整SHA **4d783585645206c52b49bce8656475d26940c1d7**，appsTree d2facb41f19ce1ef6930c743460c4f00010bfc12。真正修复第13项的代码61080a1bc644e5abb4ef45794344a7185542b74a：Desktop扩展本地HOME与Bun普通DOS HOME比较不等，导致在官方Stop之前拒绝；只对等价比较使用dunce::simplified，原签名tuple/Job摘要和权限/租约/退出还原断言保留。源fdf5d4db551f71cbf493dc97b428f8b9a9364a7a已经通过普通user1启停/真实还原/清理。维护包前继lifecycle.3的扩展本地路径guard修复来源d3b34a234，当前版本仍2.97.0-desktop023.lifecycle.3，15413397字节/SHA256 ec14baaee354c6729d1d2545ab9165d207b14a3a4239f354662163087eeb0739，本轮后继没有改变包字节。

本轮最晚产品提交02912eb80仅在失败app.log追加既有固定阶段/耗时，并标记已读完POST响应；公开DTO、Stop逻辑和原30/10/5秒不变。测试后继4d7835856固定新制品来源并在清理前记录原句柄真实退出码。本轮cross-session-02通过，**不能推断诊断补丁修好了cross-session-01的lifecycle_stop_unknown**；旧首错阶段未知，本轮通过也没有失败阶段可作归因。HTTP Connection close只为源码线索，未证实、未改代码。依用户“已通过则停止”停止该场景追加，不以再跑次数证明已修。

| 场景 | 有界实证 | 清理/限制 |
|---|---|---|
| 原第13项普通user1 | ocxu1-20261109-223434 Start15351ms/Stop及实际还原8727ms，Bun/Node/cmd原句柄exit0 | 账户/Profile/目录删除；不再独立重验 |
| 双新普通SID | two-sid-02：A15973/7245ms、B16992/7864ms，双服务重叠，Stop B后A保持，均exit0 | 两账户/Profile/目录/ACL清理通过；同RDP桌面的两个不同真实SID，不冒称两个桌面 |
| 实际控制台独立 | console-01：Administrator500/session1/high/type1，10718/6716ms、exit0 | Sandbox、daily摘要保持、private和临时任务删；不再独立重验 |
| 实际控制台+RDP | cross-session-02：两会话Active、不同SID；console13399/9621ms、RDP12692/7862ms，双exit0/实际还原，fixed conflict/重叠/Stop peer保持全过 | Sandbox；15119属管理员组但实际未提升type3；private/任务/进程端口清净，daily保持；不等于默认Profile/完整凭据/同SID多会话通过 |
| NSIS | nsis-01本机新普通SID：default/中文自定义路径安装、实际离线供应、同版本重装、卸载均exit0，runtime和用户数据保留 | 账户/Profile/目录及ACL均清理 |
| MSI本机 | msi-01首次原生exit1625：Non-assigned apps are disabled for non-admin users | Windows Server策略只读、不改、不重复该失败；账户/Profile/目录清理 |
| MSI临时托管Windows | 同一原包run42649281070：default/中文自定义路径安装/供应/同版本重装/卸载exit0，runtime及用户数据保留 | 原有客户端等价Server夹具策略精确恢复，AlwaysInstallElevated未改，账户/Profile/目录清理；不能记成本机MSI安装成功 |

安装/重装保留结果是上述受影响路径的定向证明，不扩为跨版本升级/签名/发行整套验收。IPC/单实例/凭据可复用范围由unchanged-boundary-source-proof.json固定Git对象；旧实际crossPipe拒绝、peer数据拒绝和同SID重复启动拒绝不改写，synthetic DPAPI只保留其子范围。托盘六处相关Git对象自原2c6基线完全相同，可复用旧实际双方隐藏重开及原句柄托盘退出；本轮未伪称新托盘交互。原42545767548 token-pointer失败和其后技术原型范围保留，不写成维护包采用修复。

## 最晚一轮原四项证据

独立证据[根目录与逐批说明](../../../.adg/evidence/0.2.23/windows-local-followthrough-20261109/README.md)，最晚批次[cross-session-02独立核验摘要](../../../.adg/evidence/0.2.23/windows-local-followthrough-20261109/cross-session-02/verified-summary.json)。原失败cross-session-01及此前elevated-peer-01/02均未覆盖。

1. 原回执原文：各角色*/action.raw.json；console共Start1/成功1/正常Stop1；RDP共Start2，其中typed port_conflict1和成功1，正常Stop1。每成功回执唯一。public action本身无PID/port字段，不能伪造“回执含PID”；同动作目录runtime-step、官方status、受保护record/HMAC及reply原生SID/image/FILETIME/token逐项核对一致。console服务94072/61175、RDP24676/51582。
2. 启动trace：各start目录invoke-trace/owned-health-observations/runtime-step及reply。两成功Start无失败分类；RDP conflict单独port_conflict，原30秒无本侧记录，console原代次保持。每次get_status_snapshot均在原5秒内。
3. 停止trace：各stop目录原action/invoke-trace/runtime-step、console reply-5、RDP reply-4；Stop及真实客户端还原9621/7862ms。双方restore.originalServiceExit及original-exits-before-cleanup中服务原句柄均exit0；不是事后PID消失推退出码。RDP新鲜客户端读回三路由字段及opencodex provider、原端口均清除。Stop RDP后console reply-4仍94072/61175/healthy，再正常Stop console。
4. 账户/Profile与自有清理：本批仅两个现有真实桌面账户各自Sandbox，不创建或删除日常账户/Profile。finished证明两private移除、daily六文件存在性/摘要保持、准入代次清理；task-cleanup lastResult0/任务移除、原PID与端口空。两个新普通SID及安装器临时账户的删除结果在各自controller-finished/account-cleanup。保留Profile1018见下节。

## 新源码制品与原件

Desktop build42653885333/artifact13692037223，ZIP304979005字节/SHA256 cf5c5e319102b921ab204dfb990214363a829df7428f603bf3ada41d971a0ad6；9成员核验、Desktop b0f671ef6406a1af2daae14ab93cb8b4c55ada3162c88c74c454291e4f87e5a7确与旧程序不同，原绑定3/认证状态Stop9实际通过。供应fixture同源build42653892698，EXE SHA256 91bc8038a7ed34a2d796308a20f229437e04ae8fa019fe9696dfa9e88f45870f，appsTree/descriptor匹配。它只调用原offline provisioner，不增加产品命令。全部下载经127.0.0.1:3060；构建/ZIP/API来源/原日志及逐文件manifest见stop-phase-build。watcher汇总整型字典key失败和只读HResult格式化首错均独立保留，发生后未重派产品或下载。

安装器同源appsTree a4acf4ad2c0fb25420471d50a7d8e11897362068/source0302b66082e2c0314e7911f1f9c771d265dddfdf/build42645465791；artifact13687777600、ZIP618204696字节/SHA256 57db0b25ef98f725c825fbe8b03c774cfd33c6846f3dd232b4796cbbd768a9d6。它与最后诊断Desktop的字节/来源分开列，不冒充一个制品。最晚诊断只触及Stop日志，不重跑已过安装器。

## Windows本机并行跟进项

- 临时账户ocxaccu16625a3be3，SID **S-1-5-21-587408064-856152790-2583328096-1018**，Profile **C:\Users\ocxaccu16625a3be3**，Windows仍报告Loaded:true。已禁用并移出Administrators。最终只读报告列实际观察的残留PID/镜像/SID及归属盘点完成度（停滞的全机只读盘点已按原生身份停止，未完成不补零），不能把不可读当缺席。没有新的安全卸载线索，未强卸hive、未删除Loaded Profile、未注销/重启日常会话。第18项不能因此整项关闭。
- cross-session-01旧lifecycle_stop_unknown根因仍未证实；当前有界场景通过即停，保留固定失败诊断供新线索出现时使用。未建立根因消除或稳定性证明。
- 此前三个编译/ZIP缓存目录删除曾遭自动审批blocked by policy，未绕过也未重试：D:\ocx13-analysis-20261109-220054\binding-check\target；D:\ocx13-stop-build-20261109-220455\program.zip.parts；同根build-logs.zip.parts。准确原件在第13项task-scratch-cleanup.json。与之独立，本轮四个新下载分段已在核验原ZIP后删除，new-download-range-cleanup.json留证，原制品和证据保留。

原IMP-31/32/33和P23-08/09继续in_progress/未发行，不签整版通过证书，不发布。当前无需盲重跑；若再有旧Stop首错，先读固定阶段与原退出/还原证据，再选最小受影响步骤。Loaded Profile应等待Windows正常释放后的真实新状态，再走同SID/路径/非Special/未Loaded的原清理门，不改安全判定。
