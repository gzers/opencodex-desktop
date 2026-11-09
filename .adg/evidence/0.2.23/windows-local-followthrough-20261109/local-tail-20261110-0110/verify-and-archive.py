from pathlib import Path
import json,hashlib,shutil,datetime,subprocess
root=Path(__file__).resolve().parent
repo=Path(r'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\docs-governance')
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
ordinary=read(root/'query-result.json');raised=read(root/'elevated/query-result.json');lsa=read(root/'lsa/query-result.json');transport=read(root/'lsa/query-finished.json')
sid='S-1-5-21-587408064-856152790-2583328096-1018'
for q in [ordinary,raised,lsa]:assert q['profile']['sid']==sid and q['profile']['loaded'] is True
for path in [root/'query-finished.json',root/'elevated/query-finished.json']:
 q=read(path);assert q['exit']=={'originalHandleExited':True,'exitCode':0} and not q['timedOut'] and q['resultAvailable']
all_wts=raised['wts'][1];assert all_wts['Succeeded'] and not all_wts['Matches'];assert len(all_wts['Unknown'])==all_wts['UnknownOwnerCount']==2
assert sorted(x['Pid'] for x in all_wts['Unknown'])==[0,4]
assert lsa['token']['elevated'] is True and lsa['token']['elevationType']==2
assert lsa['lsa']['EnumerationSucceeded'] and lsa['lsa']['Status']==0 and lsa['lsa']['ReadFailures']==0
assert len(lsa['lsa']['Matches'])==2 and all(x['Sid']==sid and x['LogonType']==2 and x['Session']==0 for x in lsa['lsa']['Matches'])
assert 'failure' in transport and 'exit' not in transport
summary={'observedUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'scope':'Local tail only; no product action or release gate change','profileSid':sid,'profileLoaded':True,'ordinaryWts':{'enumerated':ordinary['wts'][1]['Count'],'unknown':ordinary['wts'][1]['UnknownOwnerCount'],'matchingSidProcesses':len(ordinary['wts'][1]['Matches'])},'runAsWts':{'enumerated':all_wts['Count'],'matchingSidProcesses':0,'unknown':all_wts['Unknown'],'tokenEvidence':'RunAs requested; actual token not captured for this WTS helper; do not borrow later LSA token'},'matchingLogonReferences':lsa['lsa']['Matches'],'lsa':{'enumerated':lsa['lsa']['Enumerated'],'readFailures':0,'noSid':lsa['lsa']['NoSid'],'actualToken':lsa['token'],'elapsedMilliseconds':lsa['elapsedMilliseconds'],'apiReturnVerified':True,'wholeCollectorPassed':False,'collectorFailure':transport['failure'],'originalExitCode':'unknown; child exited before CIM identity admission; do not infer exit0 from result file'},'causalLimit':'Matching historical logon-session data does not prove which process retains the profile, a hive handle or an unload reference; no forced unload justified','cleanupAllowedNow':False,'restorationGate':'Same exact SID/path, non-Special and Windows Loaded:false, plus exact owned-process cleanup identity; earlier protected-process unknowns are never assumed absent','noNewAccountOrProfileLoad':True,'noNewProductRun':True,'noStopFixBasedOnGuess':True,'previousCorePassedScopeUnchanged':True}
(root/'verified-summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
refs={'checkedDate':'2026-11-10','sources':[{'title':'WTSEnumerateProcessesExW','url':'https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/nf-wtsapi32-wtsenumerateprocessesexw','used':'Native API level0 structs, current/all-session filtering and WTS buffer release; null/unknown SID retained'}, {'title':'LsaEnumerateLogonSessions / LsaGetLogonSessionData / SECURITY_LOGON_SESSION_DATA','url':'https://learn.microsoft.com/en-us/windows/win32/api/ntsecapi/nf-ntsecapi-lsaenumeratelogonsessions','related':['https://learn.microsoft.com/en-us/windows/win32/api/ntsecapi/nf-ntsecapi-lsagetlogonsessiondata','https://learn.microsoft.com/en-us/windows/win32/api/ntsecapi/ns-ntsecapi-security_logon_session_data'],'used':'Documented read-only logon-session prefix and status/free-buffer handling; only synthetic matching SID fields output'}, {'title':'UnloadUserProfile / LoadUserProfileW','url':'https://learn.microsoft.com/en-us/windows/win32/api/userenv/nf-userenv-unloaduserprofile','related':['https://learn.microsoft.com/en-us/windows/win32/api/userenv/nf-userenv-loaduserprofilew'],'used':'UnloadUserProfile requires returned profile hive handle and token; do not invent missing original handle or substitute an unrelated registry key; open handles must close first'}, {'title':'CreateProcessWithLogonW','url':'https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createprocesswithlogonw','used':'Profile loading/logon context for earlier credential bootstrap; not proof of present retaining owner'}]}
(root/'primary-references.json').write_text(json.dumps(refs,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
assert not any((root/p/'private').exists() for p in ['','elevated','lsa'])
checks={'observedUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'scope':'Own readonly helper sessions only; no app/service/temporary identity created','ordinaryHelperExit':read(root/'query-finished.json')['exit'],'wtsRunAsHelperExit':read(root/'elevated/query-finished.json')['exit'],'lsaHelperOriginalExit':'not proved by transport; prior exact filename-filter Win32_Process snapshot showed no remaining local-tail helper, which is absence-only and not exit code','noProductActions':True,'noHiveMutation':True}
(root/'local-query-cleanup.json').write_text(json.dumps(checks,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
e=repo/'.adg/evidence/0.2.23/windows-local-followthrough-20261109';out=e/'local-tail-20261110-0110';assert not out.exists();shutil.copytree(root,out)
entries=[dict(path=p.relative_to(out).as_posix(),bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in sorted(out.rglob('*')) if p.is_file()]
(out/'evidence-manifest.json').write_text(json.dumps(dict(archivedUtc=datetime.datetime.now(datetime.timezone.utc).isoformat(),source=str(root),files=entries),indent=2)+'\n',encoding='utf-8')
report='''# 本机尾项继续：Profile登录引用只读定位

2026-11-10 01:10～01:16，Asia/Shanghai。用户已明确要求Mac发版与本机尾项并行，本轮继续仅本机安全排查；不增加发布前置、不改产品/维护包/日常配置、不重跑已过启停，不删除先前审批拒绝的缓存。

## 新观察

临时账户ocxaccu16625a3be3、SID S-1-5-21-587408064-856152790-2583328096-1018、Profile C:\\Users\\ocxaccu16625a3be3仍Loaded:true、非Special，账户禁用和去管理员身份保持。两个HKU hive此前实证仍在。

H-TAIL-1：原全机串行CIM归属盘点曾停滞，本轮用30秒外部包络执行原生WTS只读快照。普通查询308ms，RDP当前184进程未见临时SID，全机479进程299 owner未知；不把未知当缺席。正常RunAs的同一只读查询308ms全机478进程，SID1018匹配0；仅PID0和System PID4 SID未知，RDP183进程owner未知0。RunAs helper原句柄exit0，实际token未在该helper额外采集，不能借用下一LSA token替代。相关Application卸载事件有界查询无该SID匹配；Loaded并未释放。

H-TAIL-2：在上述新线索下只读查询LSA登录会话，实际helper elevated:true/type2，有1255条会话、读取失败0、无SID1；只输出SID1018的两条type2/session0记录，登录时间为2026-11-09T15:33:23.5058434Z和15:33:23.5073517Z（本地23:33:23），均对应原elevated-peer-02启动时间。未输出其它用户/凭据/Token handle/原始LogonId。

这证明“没有观察到同SID进程时，仍有该SID历史登录会话记录”。它没有证明哪个进程持有Profile、hive句柄或卸载引用，不能单靠登录记录宣称根因查明。当前仍未满足原Loaded:false清理门。

## 采集缺陷保留

LSA API status0及匹配结果已先落盘，但外部执行器在短命helper退出后做CIM身份准入时出现“找不到”失败（line189），因此该轮query-finished没有真实原退出码，整轮采集器不记全过。原失败/实际token/API结果分别保留；不重跑已得到的LSA观察，不用结果文件推断exit0。其它两WTS查询原句柄exit0、无超时。最后精确文件名前缀的进程快照未见本轮helper；这只证缺席，不补原退出码。

## 当前安全边界与下一动作

微软原生UnloadUserProfile要求LoadUserProfile返回的原hProfile及对应token，并要求注册表句柄释放。此前CreateProcessWithLogon/Profile启动没有保留可用于当前调用的原加载句柄；禁止以RegOpenKey的新句柄冒充、重新启用账户取token、强制卸hive、注销日常会话或重启系统。也不终止“可能持有引用”的系统服务。原Profile、精确SID与禁用状态继续保留，等待Windows正常释放的新状态或有确切持有者证据再按原门处理。

旧cross-session-01 Stop首错仍无新的失败阶段，cross-session-02已通过即停；没有基于HTTP close猜测改变代码或重跑产品。Mac代码4d783585645206c52b49bce8656475d26940c1d7和发布交接保持，本轮只提交本机证据/结论。没有自动后台任务/轮询器或重启安排；后续接续从本记录与实际新状态开始。

[逐项断言及采集限制](../../../.adg/evidence/0.2.23/windows-local-followthrough-20261109/local-tail-20261110-0110/verified-summary.json)、[原件与脚本manifest](../../../.adg/evidence/0.2.23/windows-local-followthrough-20261109/local-tail-20261110-0110/evidence-manifest.json)。主来源及具体用途另列primary-references.json，不从官方API说明外推本机根因。
'''
report=report.replace('\\\\','\\')
(repo/'docs/03-开发实施/0.2.23-contracts/windows-local-tail-profile-followup-20261110.md').write_text(report,encoding='utf-8')
with (e/'README.md').open('a',encoding='utf-8') as f:f.write('''
## 本机尾项继续（2026-11-10 01:16）

新原生WTS快照减少旧全机CIM盲区：正常RunAs快照478进程无SID1018匹配，仅0/4 owner未知；Profile仍Loaded。后继真实high/type2的LSA只读API成功发现SID1018两条原23:33:23 type2/session0登录记录，但不能因此证明hive持有者或强卸安全。LSA短命helper在CIM准入竞态失败，原退出码缺证，整轮采集不标过；原结果与失败分别保留。没有新产品运行、没有Profile加载/卸载/日常会话操作、没有重试审批拒删缓存。Mac发布交接保持，本机原Loaded:false清理门仍未满足。

详见docs/03-开发实施/0.2.23-contracts/windows-local-tail-profile-followup-20261110.md和本目录local-tail-20261110-0110原件/manifest/verified-summary；暂无线索可安全进一步变更，等待真实释放状态或明确持有者证据，不盲重跑。
''')
f=repo/'docs/03-开发实施/0.2.23-contracts/windows-to-mac-release-handoff-20261110.md'
with f.open('a',encoding='utf-8') as s:s.write('\n## 本机后台后继01:16\n\nProfile未释放，WTS/LSA只读观察定位到两条该临时SID历史登录会话，未证明当前hive持有者；短命查询采集缺陷和原退出码缺证分列。无产品改动或新验收要求，不影响Mac收件发布。详见[本机尾项记录](windows-local-tail-profile-followup-20261110.md)。\n')
print('Archived',len(entries),'raw evidence files; cleanup remains guarded')
