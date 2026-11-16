# Windows r4增量接收与路由声明、状态同窗接口 r5

2026-11-16后继：[claim保管r6](windows-claim-anchored-custody-r6.md)已明确新的Windows布局/进度；本合同Route与同窗期限接口继续消费，不要求重冻或复制。旧r5失败/组件边界保持。

revision: 5；2026-11-15 Asia/Shanghai；TEAM15/IMP141。共享后继 **main358e7fc4717787616f049f3bdc1cfe02f6d353f9**（路由提取8e2699a91＋状态期限358e7fc47）；Windows实现输入 **0549a660ee3f77ea243e798ec55b01f52545e492**，实际最新产品源 **6e312f02270f247d2f440ec369385e523a4a8043**。证据分支 **a7baa88e1**完整SHA归同批integrated-source.json。r4固定8投影/审计ExcludedUnknown/v2、原四trait/预算/未知拒绝全部继续；本次不整批合并Windows未完实现，不开放Start或发行。

## 接收结论

消费最新handoff、wl2-working-snapshots的shared-minimum-request.json与wl2-stop-restart-results的shared-status-deadline-request-r2.json（后者替代其第一版，只加recovery_at不足以共享原窗口）。净文档5873文件为14新轮次与handoff；各轮payload清单5854文件57,943,378字节逐一核SHA/长度。最新GUI产品/测试实际输入对6e312源、归档清理脚本对0549核对，原文件换行等价明确。证据回放在.adg/evidence/0.2.24/windows-wl2-shared-receive-r5/verify_receive.py；仅完整性/来源与列举事实复核，不是新增原生执行或万行Windows实现的完整安全审计。

| 项目 | 已接进展 | 未完/限定 |
| --- | --- | --- |
| B1/B2 | 固定8投影/combined及LABEL、条件DACL、resource/filter非空合成反例有进展 | populated CAP/trust创建1314/5仍未满足，不提权或重复相同前提；普通生产整组不因此通过 |
| O树/阶段/引用 | 存在/缺席双根保管恢复、move意图/current CAS、两锁只读、Ready/PreStop工作快照、历史/后继/Closed/Stop接线有组件证据 | 完整官方副作用/NativeLease及真正原包闭环未完成；Start durable gate拒绝，组件/文本Closed不当生产资格 |
| 真GUI＋空broker | marenben SID1003/session7/elevatedfalse，四代真实visible HWND/pageFinished/setup，normal0/hold原GUI故障1/after-failure0/finish0；同原broker31484/FILETIME134392264041593579及原创建句柄持续匹配，observer0 | 是实际GUI空宿主组件通过，officialOwner/Job/production仍false，frontendDomAssertion=false；broker rawexit=null不能补0 |
| 首次GUI失败 | 0c场景observer101、TokenProfile/sandbox错误已诊断并修 | 原失败broker40588退出仍未确认，现场保持，不能按PID强杀 |
| 最新产品 | 6e312 custom-protocol MSVC debug bins0，组件证据分时点 | 0549只加归档脚本无产品重编，不拿0549当实际产品测试源；非release预算/签名/候选 |
| 清理 | 成功Profile子树685文件/291目录/119,851,895字节原SID ZIP核验后删除；输入/回执匹配 | Public20文件178,669,957字节已原SID归档但源保留，控制器12读可核/8拒绝；私有ZIP控制器未独立复读，NTFS元数据不克隆；overallCleanupComplete=false |

所有首失败/中间构建缺失hash/未先假设轮次等原局限不改写。旧删除拒绝、失败根/账户Profile、普通MSI/其它未完保持。真实原包、NativeLease/S-D组合和原预算闭环仍待验，不以真实GUI通过替代。

## A：共享纯路由声明

实际模块 **modules::runtime::client_route_declaration**；API **pub fn declaration(bytes: &[u8]) -> AppResult<Option<Vec<String>>>**。从原macos_client_session私有纯解析及helpers等价提取，Mac原declared_routes继续在pin目录读取后调用，Windows读取其原生固定H/config.json后消费同函数，不再复制代码/猜路由。

- None表示声明无法完整确定，必须阻止依赖完整路由集合的内容判定；Some(empty)才是独立声明明确空集合。Err为严格JSON/边界失败，不能吞成空。
- 原provider选中/static/dynamic/retained/default语义、slash编码及碰撞、display alias不变路由、customModels/combos/disabledModels/fastRows规则保持；原10项测试迁到共享模块，另加原native读取的2MiB上限测试，API在复制/解析前拒超限。
- 没有I/O/网络/发现请求/模型调用/写配置/进程或恢复权限；不从生成的catalog反推路由，不把display name当实际route。错误仍原分类与不含凭据的摘要。
- Windows在启动前固定已核验config字节/身份/摘要与解析结果到新会话声明，Ready/PreStop及恢复判定仍重核同源/原配置未漂移；不能将一次Vec缓存跨H/source/session。调用client_artifact_policy仍须真实原生阶段与完整受控副作用事实，此API不替NativeLease或协调锁。

## B：一次状态采样共用原5秒截止

StatusSource新增fixture兼容默认方法requires_fresh_sample、fetch_at(deadline)、fetch_light_at(deadline)、client_recovery_at(deadline)。requires_fresh_sample默认false，旧fixtures/Mac原生恢复方法保持原路径。Windows managed OfficialStatusSource为true，不在准入前为了选择分支先做额外完整原生读取。

StatusCollector::refresh_at/refresh_light_at在进入采样时用真实Instant::now＋原collect_timeout建立唯一deadline，注入的now只记调度/去抖，不能拿未来测试时间扩原生窗口。fresh来源不读取此前cached payload或recovery来构造新权限，即使在debounce时间内仍做新的有界采样；原polling/事件发出节流保持。流程只有fetch_at或fetch_light_at一次、随后client_recovery_at一次，共用完全相等deadline；没有原来的采样前无期限recovery和采样后另开5秒。

OfficialStatusSource的Windows managed fetch/light将该deadline传到fetch_inner_at→windows_official_lifecycle024::status，并以timeout_at包异步采集；fallback CLI仅剩余预算。client_recovery_at把相同deadline传到Windows已有recovery_state_at。collector在每边界/发布前拒过期，fetch失败/未知或非法status payload不与后续成功recovery拼成新Start许可，恢复状态保持Unknown；取不到真实证明不能清Required。

这不是全局抢占同步IO的能力：native receiver必须协作检查同一deadline/有界锁及等待，不能在函数内先无界扫描再回报Timeout。超时后不能接受晚到成功；取消/等待受控子进程沿原native规则，不用强杀凑普通通过。直接诊断用fetch()仍是独立调用的旧入口，生产collector以新_at入口采样；Windows须验证真实用户status调用确实到此新路径，没有绕过的旧采样链。

**重要合并规则**：main尚未含Windows完整Closed实现，因此main新增同签名recovery_state_at仅为保守桥接，所有成功结果仍只能Required/Unknown，不签NotRequired。Windows当前已有更完整native receiver（检查原Closed/NativeLease/source/原deadline）。合入本main时必须保留Windows原来的实现，不能被main桥接覆盖。只读git merge-tree预演显示唯一内容冲突是infrastructure/windows_official_lifecycle024.rs；该文件main本批唯一净改动为桥接函数。以0549分支的完整原文件为该冲突解法、保留真实recovery_state_at及其它Owner实现，确认文件SHA/后继本线新增差异后合并；其它共享路由/collector/status_source更新正常接。不要用全仓ours/theirs或丢本线修改。

共享时限不是native身份替代；同deadline内的status/recovery各自仍须原owner/source/H/current证明，若有并发切换/动作或旧闭合对象未知，不得以先后两张不同代次成功拼NotRequired。Windows使用既有同H动作/owner锁和原代次复验，不新增共享假proof工厂。

## 可写集与下一步

Windows在0549或其干净后继上合入main358e7fc4717787616f049f3bdc1cfe02f6d353f9，docs承接本合同所在提交。r2/r3/r4允许集继续；新增明确允许消费共享route API、在status_source.rs Windows分支及Windows native receiver落实相同deadline/实际调用测试。共享解析规则、Mac语义、HMAC/官方来源/预算不由Windows另改；真实必要缺陷给最小原因与diff。Mac本批之后不并行改Windows单写文件。

1. 先合并、解决唯一native bridge冲突并MSVC编译；运行路由11项/collector时限6项及已有受影响status回归。pure跨平台用例可以Windows执行，不重复无关CLI/MSI/旧空H/已过原件原语。
2. 将native config pin字节接共享declaration，用原policy评估真实允许副作用；完成NativeLease/官方副作用剩余和原历史/后继。不要把Some(routes)或parse成功当恢复许可。
3. 将Windows真实status/recovery调用绑定同样本deadline，在同一原owner上测试迟到/归属漂移/闭合失效；提供调用trace中的原始sample开始/截止/各阶段耗时及是否进入recovery，禁止nested now+5。
4. 就绪后才打开原允许的单SID原包定向执行条件，采完整受影响Start/Status/Stop/Restart/第二Start/Auto失败后继与GUI官方owner组合。当前Start gate仍拒绝；本次提供接口不授权跳过未完。release预算按原参数，debug结果分列，固定候选前不W2/双SID/发布。

| 项目 | 本批必须验证 | 拒绝/复用 |
| --- | --- | --- |
| Route | 严格重复JSON/2MiB、动态未知/明确空、slash碰撞、自定义/组合/禁用/display/fastRows与原10例等价；nativeH/config前后摘要与声明一致 | None/Err不转空、不从catalog学路由；config改变后不能用旧Vec |
| Sample | fetch/light和recovery同deadline一次；真实采样起点与注入now分开，debounce不cache授权，任一迟到/失败/未知payload拒绝清恢复 | 不以late NotRequired成功/先前Stopped补签；没有新的5秒窗口 |
| Native状态 | 真实receiver保留原Closed/原代次/当前source、锁/调用均同截止；检查expired/错H/source/旧owner/并发动作 | main桥接不能覆盖Windows receiver；host fixture不能代真Closed正例 |
| 生产剩余 | NativeLease/声明官方副作用/完整O闭环与真实GUI+官方owner/S-D原包组合，仍原30/5/10/Restart30含Stop子10 | trueGUI空broker不代官方Job；B2 CAP/trust非空/普通MSI仍未验不隐藏 |
| 清理 | 成功Profile子树已清可复用；Public源8不可读/失败broker40588/所有旧拒绝分列 | 不改ACL/策略、按PID强杀、删除工具替换或把原SIDZIP当控制器独立复读 |

每轮先假设，一次采全独立状态/首错/rawlog/source/实际EXE/SHA/清理，再批量修复；无新假设不重跑旧前提。代码/文档分支分别及时push；Windows范围内连续处理，非共享协议变化不用每处回Mac。无真实模型/业务同步/日常配置策略写，不发布。

## 本机验证边界

共享route11项、status33项（含6新期限/失败反例）、source10项、Mac会话17项通过/2原ignored保持；fmt/严格Clippy通过。原提取首次import置于模块文档之前导致E0753，修正位置后通过；未编造原生Windows运行。route原纯逻辑和10旧测试除格式/公开性及原2MiB界限外做文本等价核对。main新增native桥接只保守返回，Windows MSVC合并后验证仍必须执行。候选/发布/整版NAME等未完仍由IMP保持。
