# Windows代码交Mac发版，本机尾项并行跟进

2026-11-10，Asia/Shanghai。用户最新明确要求：“这些并非发布必要条件把，推送相关代码给mac发版了，同时你本机在这里做着吧”。此决定覆盖Windows此前将本机尾项统一写为“整项尚未闭环”的排队含义：Windows交付Mac现有代码与真实通过证据，Mac继续整版发布流程；本机Profile清理、缓存残留和旧Stop首错的后续归因转为Windows本机并行跟进，不再由本线单独追加为Mac发版前置。

这不是抹去失败或关闭未完成清理，也不豁免最终发布源/签名/制品/其它产品AC；若Mac当前正式候选出现新的启动、停止、归属或还原失败，仍按真实候选结果处理。Windows本轮不执行发布命令，不替Mac宣称已收到或已发布。

## Mac直接收件入口

| 内容 | 分支/固定来源 |
|---|---|
| 代码，已经push且远端核验 | codex/0.2.23-integration，**4d783585645206c52b49bce8656475d26940c1d7** |
| 代码appsTree | d2facb41f19ce1ef6930c743460c4f00010bfc12 |
| 运行包 | 2.97.0-desktop023.lifecycle.3，15413397字节，SHA256 **ec14baaee354c6729d1d2545ab9165d207b14a3a4239f354662163087eeb0739** |
| Windows完整证据文档基线 | docs/governance-main，**04336c43ab024fa28f2d1d7008f068c412b70e8c**；其后含本文件的提交为最新发布交接说明 |
| 证据范围 | [最终有界结果及原四项](windows-local-followthrough-result-20261110.md)、[逐批原件索引](../../../.adg/evidence/0.2.23/windows-local-followthrough-20261109/README.md) |

Mac拉取上述两分支，先读本文件，再读最终有界结果。已有代码都在集成分支，不另造Mac补丁包，不要求Windows回滚或重复通过项。Windowsdebug custom-protocol构建只作已验证来源与行为证据，正式发行包仍由Mac既有整版发布流程按最终源码/目标平台构建、核对和签名；不把调试ZIP当正式发行物。

本批实际产品修复61080a1bc644e5abb4ef45794344a7185542b74a处理Desktop扩展HOME与Bun普通DOS HOME等价比较失配；d3b34a234前继维护包修复扩展本地盘符private-record guard。旧原始签名tuple、Job摘要、User/SYSTEM严格ACL、租约、HMAC、进程身份和真实客户端还原退出判据保留。

02912eb80/4d7835856仅补停止失败阶段日志和测试清理前原句柄读码；没有放宽或重写Stop、HTTP或运行包。Mac不能把后继通过写成该诊断补丁证明旧首错根因已经消除，也不应把尚未证实的HTTP close猜测直接做入发行代码。

## 可用Windows通过证据

- 普通user1：Start15351ms、正常Stop及实际客户端还原8727ms，原Bun/Node/cmd exit0，账户/Profile/目录清理通过。
- two-sid-02：两个新普通SID服务同时健康、正常停止B后A保持、双正常Stop及原exit0、双账户/Profile/目录/ACL清理通过。
- cross-session-02：实际Administrator500/session1提升type1和15119/session2未提升type3，两会话Active。console Start13399ms/正常Stop及还原9621ms，RDP12692/7862ms；各成功启动回执唯一，RDP另有一次明确port_conflict预期拒绝。双方原服务exit0、独立端口、重叠健康、Stop B后A原PID/端口/健康保持；private/任务/进程端口清理和daily摘要保持。原30/10/5秒不变。Sandbox边界和管理员组/未提升区别均保留。
- 原生命周期绑定3及认证状态/Stop9项通过；485份逐文件封存证据的Git字节已核验。
- NSIS本机两路径、MSI同一原包在临时托管Windows两路径，安装/供应/同版本重装保留/卸载定向通过。MSI本机Server策略1625失败单独保留，未改本机策略；隔离成功不伪写成本机成功，也不扩为跨版本升级/签名验收。
- 托盘/IPC/单实例/凭据按已封存Git对象和真实原证据复用范围交接；原42545767548 token-pointer失败及后继技术原型范围保留，不冒称维护包采用修复。默认Profile/完整凭据/同SID多会话不由Sandbox新造通过结论。

## Windows本机并行跟进，当前不阻塞Mac收件发版

1. 精确临时账户ocxaccu16625a3be3/SID S-1-5-21-587408064-856152790-2583328096-1018，Profile C:\Users\ocxaccu16625a3be3，仍Loaded。账户已禁用并移出Administrators。2026-11-10 01:03只读核验HKU SID和SID_Classes均仍加载；ProfileList State0/RefCount缺席不能冒充已卸载。最新100条Profile事件中仅有此前加载证据，未得到安全释放线索。待Windows正常释放后才走原同SID/原路径/非Special/未Loaded的清理门；不强卸hive、不注销/重启日常会话。旧已准入进程的清理回执保留；最终全局owner盘点曾停滞并按自有进程原生身份终止，未完成不补零。
2. cross-session-01的lifecycle_stop_unknown旧首错原因尚未证实；cross-session-02同一严格场景通过即停，不盲重跑。当前固定阶段日志保留用于未来有新失败线索时定位；只有对应新证据才修最小受影响处。
3. 三个旧缓存目录先前自动审批blocked by policy拒删仍保留，准确路径见最终有界结果及earlier-scratch-final-observation.json，不绕过。与之独立，本轮四个新下载分段已在校验原ZIP后删除，原制品和原证据保持。

Mac不需等待上述本机尾项完成才接代码继续发布。本线继续承担这些事项，未完成状态不改成已完成；不扩大为core-sweep/全量、不动日常配置、不跑真实业务同步。

## 本机后台后继01:16

Profile未释放，WTS/LSA只读观察定位到两条该临时SID历史登录会话，未证明当前hive持有者；短命查询采集缺陷和原退出码缺证分列。无产品改动或新验收要求，不影响Mac收件发布。详见[本机尾项记录](windows-local-tail-profile-followup-20261110.md)。
