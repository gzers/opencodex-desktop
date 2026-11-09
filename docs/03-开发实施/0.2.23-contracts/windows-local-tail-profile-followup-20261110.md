# 本机尾项继续：Profile登录引用只读定位

2026-11-10 01:10～01:16，Asia/Shanghai。用户已明确要求Mac发版与本机尾项并行，本轮继续仅本机安全排查；不增加发布前置、不改产品/维护包/日常配置、不重跑已过启停，不删除先前审批拒绝的缓存。

## 新观察

临时账户ocxaccu16625a3be3、SID S-1-5-21-587408064-856152790-2583328096-1018、Profile C:\Users\ocxaccu16625a3be3仍Loaded:true、非Special，账户禁用和去管理员身份保持。两个HKU hive此前实证仍在。

H-TAIL-1：原全机串行CIM归属盘点曾停滞，本轮用30秒外部包络执行原生WTS只读快照。普通查询308ms，RDP当前184进程未见临时SID，全机479进程299 owner未知；不把未知当缺席。正常RunAs的同一只读查询308ms全机478进程，SID1018匹配0；仅PID0和System PID4 SID未知，RDP183进程owner未知0。RunAs helper原句柄exit0，实际token未在该helper额外采集，不能借用下一LSA token替代。相关Application卸载事件有界查询无该SID匹配；Loaded并未释放。

H-TAIL-2：在上述新线索下只读查询LSA登录会话，实际helper elevated:true/type2，有1255条会话、读取失败0、无SID1；只输出SID1018的两条type2/session0记录，登录时间为2026-11-09T15:33:23.5058434Z和15:33:23.5073517Z（本地23:33:23），均对应原elevated-peer-02启动时间。未输出其它用户/凭据/Token handle/原始LogonId。

这证明“没有观察到同SID进程时，仍有该SID历史登录会话记录”。它没有证明哪个进程持有Profile、hive句柄或卸载引用，不能单靠登录记录宣称根因查明。当前仍未满足原Loaded:false清理门。

## 采集缺陷保留

LSA API status0及匹配结果已先落盘，但外部执行器在短命helper退出后做CIM身份准入时出现“找不到”失败（line189），因此该轮query-finished没有真实原退出码，整轮采集器不记全过。原失败/实际token/API结果分别保留；不重跑已得到的LSA观察，不用结果文件推断exit0。其它两WTS查询原句柄exit0、无超时。最后精确文件名前缀的进程快照未见本轮helper；这只证缺席，不补原退出码。

## 当前安全边界与下一动作

微软原生UnloadUserProfile要求LoadUserProfile返回的原hProfile及对应token，并要求注册表句柄释放。此前CreateProcessWithLogon/Profile启动没有保留可用于当前调用的原加载句柄；禁止以RegOpenKey的新句柄冒充、重新启用账户取token、强制卸hive、注销日常会话或重启系统。也不终止“可能持有引用”的系统服务。原Profile、精确SID与禁用状态继续保留，等待Windows正常释放的新状态或有确切持有者证据再按原门处理。

旧cross-session-01 Stop首错仍无新的失败阶段，cross-session-02已通过即停；没有基于HTTP close猜测改变代码或重跑产品。Mac代码4d783585645206c52b49bce8656475d26940c1d7和发布交接保持，本轮只提交本机证据/结论。没有自动后台任务/轮询器或重启安排；后续接续从本记录与实际新状态开始。

[逐项断言及采集限制](../../../.adg/evidence/0.2.23/windows-local-followthrough-20261109/local-tail-20261110-0110/verified-summary.json)、[原件与脚本manifest](../../../.adg/evidence/0.2.23/windows-local-followthrough-20261109/local-tail-20261110-0110/evidence-manifest.json)。主来源及具体用途另列primary-references.json，不从官方API说明外推本机根因。
