from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent
PREVIOUS=ROOT.parent/'wl2-r6-completed-read-windows-20261117-r1'/'finalize.py'
# Reuse only the original artifact/source/raw-record verification operations.
# Final outcome/matrix/limits below are this batch's own observed facts.
original=PREVIOUS.read_text(encoding='utf-8')
prefix=original[:original.index("matrix=load(BASE/'final-status.json')['matrix']")]
edits={"BASE=ROOT.parent/'wl2-r6-native-stop-history-20261117-r1'":"BASE=ROOT.parent/'wl2-r6-route-pin-close-20261117-r1'",
 "('library01','debug',50),('library02','debug',51),('library03','release',8)":"('library01','debug',56),('library02','debug',56),('library03','release',load(ROOT/'source-commit.json')['directedPassed']['library03'])",
 'source-scope-verified.json':'source-scope-final.json',
 'later explicit TimedOut mapping and actual elapsed-window test; first exact executed source retained':'later explicit historical native-route/canonical-summary join; first exact executed source retained'}
for old,new in edits.items():assert old in prefix;prefix=prefix.replace(old,new)
exec(compile(prefix,str(PREVIOUS)+'#verified-prefix-adapted','exec'),globals())
write('recorder-provenance.json',dict(source=str(PREVIOUS),sourceSha256=sha(PREVIOUS.read_bytes()),prefixSha256=sha(prefix.encode()),edits=edits))
route=load(ROOT/'actual-route-observations.json')
for r in route['alternateStreamExports']:
    raw=(ROOT/r['evidence']).read_bytes();assert len(raw)==r['bytes'] and sha(raw)==r['sha256'] and raw==Path(r['source']).read_bytes()
matrix=load(BASE/'final-status.json')['matrix'];matrix.append(dict(id='CanonicalRouteCloseInActualFinalStopLane',result='compiled-actual-factory/DATA-and-native-candidate-pass/production-positive-unexecuted',fact='Actual internal Stop stores original RoutePin Option; moves exact original into retained route candidate BEFORE prepare. Actual completed exit/supervisor/source/content/action guard checks candidate original routes. Same final closure intent binds full route facts/inventory digest, then canonical RouteHandlesCloseIntent -> actual explicit primary-last closes -> RouteHandlesClosedVerified -> StopClosedVerified -> actual final native seal. No claims/client trees mutated by route release. Exact original native close parent remains unchanged. Historical read compares native route completion against canonical route summary. Actual new final/route factories unexecuted; component facts do not qualify public Closed.'))
requirements=load(BASE/'next-closed-requirements.json')['requirements']
for r in requirements:
    if r['id']=='OwnershipClosure':r['requirement']='Actual internal Stop now retains exact original route candidate before fallible IO and joins full route-close intent/native explicit primary-last close/result into same final canonical seal under actual exit/absence/source/content/action guard. Native candidate and strict DATA tests pass, actual ordinary fixedofficial Stop/route/final-positive and dedicated phase/current fault windows/budget remain unexecuted. Public typed adapter/Status/dispatcher/successor remain unjoined.'
    if r['id']=='CanonicalContentAndStatus':r['requirement']='Content/final/route canonical chain, private actual completion guards and internal same-sample5s/original-action history reads are implemented/compiled. Route completed native history must match exact canonical digest in future reads; original native close receipt kept as final parent. Full current3980<=4096 only116bytes remaining, cannot silently increase or omit facts. Actual source/Owner/closure-specific fault/latency evidence and publicStatus/dispatcher/successor still required; Start stays refused.'
write('next-closed-requirements.json',dict(requirements=requirements,startOpen=False,typedClosedQualified=False,actualProductionExecuted=False,canonicalRouteCloseRegistered=True))
cleanup=load(ROOT/'cleanup.json');assert cleanup['removed']==3 and cleanup['failed']==0
counts=load(ROOT/'cleanup-counts.json')
cleanup=dict(**counts,removed=3,alternateStreamExports=len(route['alternateStreamExports']),deletionAttempted=True,historicalRejectedDeletionRetried=False,overallCleanupComplete=False,zipIsNTFSMetadataClone=False,reason='Only3 completed pure own-read-handle candidate scenes removed after fresh exact membership/hash/native path checks; all canonical/pending/protected-Unknown/ADS originals retained. ADS bytes separately exported; ZIP not ADS/native metadata clone.')
write('final-cleanup.json',cleanup)
status=dict(time=datetime.now().astimezone().isoformat(),contract='TEAM16/IMP142/r6',sourceHead=head,sourceTree=tree,inheritedCode='b8e06be268182c517a19706dd974961b7e26e57a',inheritedDocs='be0ff1fff329efda4f8550f37e6a1cf738677d3e',fetchedMain='fac8573e55441fe514919c63c7f1d65181ff3577',fetchedGovernanceMain='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,artifacts=artifacts,matrix=matrix,limits=requirements,
 firstError=dict(currentProductOrTestError=None,toolError='prepare_route_slot.py exact static formatting marker assertion failed before any code write; corrected to actual multiline .content_start.check_completed_history marker, original no-code-write failure noted here',inheritedLatest=load(BASE/'final-status.json')['firstError']),cleanup=cleanup,
 allAccepted=False,startOpen=False,typedClosedQualified=False,actualProductionExecuted=False,actualFinalStopSealExecuted=False,actualCanonicalRouteFactoryExecuted=False,canonicalRouteCloseRegistered=True,productionDispatcherUsesV3=False,productionStatusSampleJoined=False,W2=False,dualSID=False,published=False)
write('final-status.json',status)
report=['# r6原路由关闭进入最终canonical链','',f'源码{head}/tree{tree}；承接b8e06be26/be0ff1fff，最新fetch共同源及TEAM16/IMP142/r6不变。','', '| 运行 | profile | raw | 过/败/忽略 | ms |','| --- | --- | --- | --- | --- |']
for n,r in runs.items():report.append(f"| {n} | {r['profile']} | {r['rawExit']} | {r.get('passed','—')}/{r.get('failed','—')}/{r.get('ignored','—')} | {r['elapsedMs']} |")
report+=['','本轮无非预期产品/测试失败，继承首轮route primary先关反例不改。source转换记录器初静态多行marker不匹配，在写源前中止，纠正准确marker；原native primitive/claim/恢复/关闭/内容等17文件函数逐字保持。首56过后加未来native-route/canonical digest读取join，末data56过；各executed-source/EXE分开归属。',
 '', 'Store只在原资格恢复/native原件关闭后消费实际退出/监督完成history，再把准确原RoutePin无IO移入retained候选，所有权在prepare前可达。CompletedStopGuard改借真实route候选，核原H/route事实、source/Owner/action/content/audit/actual退出监督；候选prepare不产生Stop许可。',
 '', '同一closure intent先绑定完整routeCloseDigest，RouteHandlesCloseIntent刷盘/current后才实际primary-last close，读回route完整completion后RouteHandlesClosedVerified，最后StopClosedVerified和显式actualguard final seal。DATA拒绝缺/跳/替换/额外祖先/错ID/role/flag/count，原native关闭前驱准确seq/tip/current/state保持。未来读取核native routehistory与canonical摘要相等，不采用后继live config。完整current3980<=4096，容量余116不可加预算或删事实。',
 '', '原生candidate所生data已通过严格canonical shape核验，partial/protected/postclose/ADS/fullclose独立测试保持。actual新guard/完整Stop-route-finalseal正例及route phase/current专属故障窗口仍未真实执行；不以DATA或synthetic own-file结果当ordinary fixedofficial生产Stop10/typedClosed。公共adapter/Status/dispatcher/后继、原包config/auth/Grok/动态目录副作用、初始化/ownerreceipt/原缺席Codex安全退根/最大batch容量/普通MSI/CAP-trust/历史清理保持未完。',
 '',f'{len(scenes)}scene完整私有ZIP/hash/membership核验后仅3完成纯路由原语scene核fresh准确路径/hash/membership清理，{len(scenes)-3}失败/pending及全部protectedUnknown/ADS原现场保留。{len(records)}份canonical/快照/history另收存，{len(route["alternateStreamExports"])}份ADS原字节另导出且原流保留。ZIP非NTFS或ADS克隆，历史拒绝删除不重试。最终data/release/product对工作/Git明确CRLF等价，产品{len(inputs)}输入/release{len(rust)}Rust，预存schema未手改/暂存。Start拒绝，不改官方/日常，不旧raw5/空GUI/W2/双SID/发布。',
 '', '| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
with (ROOT/'report.md').open('x',encoding='utf-8') as f:f.write('\n'.join(report)+'\n')
print(json.dumps(dict(head=head,passed={k:r.get('passed') for k,r in runs.items()},scenes=len(scenes),rawRecords=len(records),productInputs=len(inputs),releaseRust=len(rust),actualFactoryExecuted=False)))
