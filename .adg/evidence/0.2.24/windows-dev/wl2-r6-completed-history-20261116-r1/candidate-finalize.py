from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
OLD=ROOT.parent/'wl2-r6-handle-handoff-20261116-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,value):
    with (ROOT/name).open('x',encoding='utf-8') as out:json.dump(value,out,ensure_ascii=False,indent=2)
candidate=load(ROOT/'candidate-tree.json');head=git('rev-parse','HEAD').decode().strip()
assert head==candidate['baseCommit']=='d20103fee084a3e63f728bf7c6664af65405e456'
tree=candidate['localCandidateTree'];runs={};scenes=[];records=[];product_inputs=[];test_inputs=[]
for name in ['library01','library02','library03','profile01','profile02','product01']:
    result=load(ROOT/name/'result.json');is_profile=name.startswith('profile')
    rc=result['rawExit'] if is_profile else result['exitCode']
    expected={'library01':0,'library02':101,'library03':101,'profile01':0,'profile02':101,'product01':0};assert rc==expected[name]
    text=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig');m=re.search(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',text)
    runs[name]=dict(rawExit=rc,newCompilation=not is_profile,elapsedMs=result['elapsedMs'],profile='release' if is_profile else result['profile'],
        passed=int(m.group(1)) if m else None,failed=int(m.group(2)) if m else None,ignored=int(m.group(3)) if m else None,
        diagnosticOnly=is_profile,sourceProducer=result.get('producer') if is_profile else None)
    if name!='product01':
        runs[name]['artifact']=load(ROOT/f'{name}-source-link.json') if is_profile else load(ROOT/f'{name}-artifact.json')
        for i,scene in enumerate(load(ROOT/f'{name}-private-scenes.json')['scenes']):
            assert sha(Path(scene['archive']).read_bytes())==scene['archiveSha256'];scenes.append(scene)
            for row in scene['files']:
                if scene['source'].startswith('D:') or any(marker in '/'+row['path'] for marker in ['/.ocxd024-observed-','/.ocxd024-history-']):
                    p=Path(scene['source'])/row['path'];raw=p.read_bytes();assert sha(raw)==row['sha256'] and len(raw)==row['bytes']
                    target=ROOT/'raw-scene-records'/name/f'scene-{i:03}'/row['path'];target.parent.mkdir(parents=True,exist_ok=True)
                    with target.open('xb') as out:out.write(raw)
                    records.append(dict(source=str(p),evidence=target.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
    if not is_profile:
        for row in load(ROOT/name/'source-manifest.json'):
            raw=(ROOT/name/'executed-source'/row['path']).read_bytes();assert sha(raw)==row['sha256'] and len(raw)==row['bytes']
            if name=='product01' or name=='library03' and row['path'].endswith('.rs'):
                assert raw==(CODE/row['path']).read_bytes(),row['path']
                blob=git('show',f"{tree}:{row['path']}");assert raw.replace(b'\r\n',b'\n')==blob.replace(b'\r\n',b'\n')
                (product_inputs if name=='product01' else test_inputs).append(row)
assert (runs['library01']['passed'],runs['library02']['failed'],runs['library03']['passed'],runs['library03']['failed'],runs['profile02']['passed'],runs['profile02']['failed'])==(27,3,30,7,31,6)
assert load(ROOT/'profile01/result.json')['programSha256']==load(ROOT/'profile02/result.json')['programSha256']==load(ROOT/'library03-artifact.json')['sha256']
assert len(load(ROOT/'native-traces-profile01.json')['observations'])==1
assert len(load(ROOT/'recovery-native-traces-profile02.json')['observations'])==12
write('final-source.json',dict(sourceBaseCommit=head,localCandidateTree=tree,isSourceCommit=False,sourceCandidateAccepted=False,
    sourceCandidatePushed=False,candidatePatchSha256=sha((ROOT/'candidate.patch').read_bytes()),productInputs=product_inputs,
    finalTestRust=test_inputs,sourceInputsMatchLocalTreeWithExplicitCrlfEquivalence=True,normalGitIndexUnmodified=True,preexistingSchemaNotManuallyModifiedOrStaged=True))
artifacts=[]
for name in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe';raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceBaseCommit=head,localCandidateTree=tree,
        sourceCommit=None,profile='debug',features=['custom-protocol'],candidateAccepted=False,sourceCandidatePushed=False))
write('product01-artifacts.json',artifacts);write('raw-scene-records.json',records)
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
matrix=load(OLD/'final-status.json')['matrix']
matrix.extend([
 dict(id='CompletedHistoryCandidate',result='implemented/compiled/direct-single-case-only/full-budget-failed',fact='Private timely transaction receipt captured after actual closes/canonical result-current/final postconditions, receipt-gated fresh5s read shares exact old full native comparison body; original mutation guard retained. Single exact-EXE diagnostic case proves original-expired fresh read and expired append refusal once. Full new13-case scope NOT passed, most new negative native points remain unexecuted due upstream timeouts. Not accepted or source-committed.'),
 dict(id='AffectedBudgetRegression',result='failed/source-candidate-kept-local',fact='library02 raw101/0pass3fail; library03 raw101/30pass7fail. Exact same latest EXE profile01 raw0/1pass, profile02 original full37-filter scope raw101/31pass6fail. Earlier failures remain failures. Native safety comparisons/old fault positions/assertions/original10s unchanged. Single success cannot establish robust complete scope.'),
 dict(id='NativeCostDiagnostic',result='measured/no-machine-load-excuse',fact='Full sameEXE profile child136859.375CPUms/145657wallms (~94% one core), kernel95609.375ms,1369872 readOps/21488996otherOps, global sampled busy~10.8%/20logical CPUs. Counters diagnose CPU/native-query work; no measured basis to blame whole-machine load, increase budgets, lower predicates or change daily policy. Next r6-permitted bounded node phase batches target record count/canonical validation cost.'),
 dict(id='RemoteFetchRestored',result='confirmed/previous-local-evidence-pushed',fact='Original verified TLS fetch raw0/1247ms; actual remote TEAM16/IMP142/r6 unchanged at fac8573e5/578230717. Prior TLS failure evidence and recovery recheck pushed as0182e060d; no disabled certificate verification or permanent network setting changes.'),
])
cleanup=load(ROOT/'cleanup.json');counts=load(ROOT/'cleanup-counts.json');assert cleanup['removed']==6 and cleanup['failed']==0
profile=load(ROOT/'profile-outcome.json')
first=dict(run='library02',rawExit=101,passed=0,failed=3,kind='bounded-original-deadline at upstream recovery before most new receipt tests',
    subsequent=[dict(run='library03',rawExit=101,passed=30,failed=7),dict(run='profile01',rawExit=0,passed=1,diagnosticOnly=True),dict(run='profile02',rawExit=101,passed=31,failed=6,diagnosticOnly=True)],
    noFailedRunRelabelledPass=True,rootCauseKnown=False)
status=dict(time=datetime.now().astimezone().isoformat(),contract='TEAM16/IMP142/r6',sourceHead=head,lastPushedSource=head,localCandidateTree=tree,
    sourceCandidateAccepted=False,sourceCandidateCommitted=False,sourceCandidatePushed=False,sourceWorktreeDirtyIntentionallyRetained=True,
    lastAcceptedSourceEvidence='7d0d567e19b7b3cbc799dcd85ec3a43ad4fca0f4',networkRecoveryEvidence='0182e060d72ed08ed3355e238307bba493b714e9',
    fetchedMain='fac8573e55441fe514919c63c7f1d65181ff3577',fetchedGovernanceMain='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,
    candidateArtifacts=artifacts,previousAcceptedArtifacts=load(OLD/'product01-artifacts.json'),matrix=matrix,firstError=first,guards=load(ROOT/'guard-verification.json'),
    measuredCost=profile,fullScopePassed=False,allAccepted=False,startOpen=False,typedClosedQualified=False,productionExecuted=False,W2=False,dualSID=False,published=False,
    cleanup=dict(**counts,removed=6,historicalRejectedDeletionRetried=False,overallCleanupComplete=False),
    nextAction='Continue within r6 single-phase bounded node-batch allowance to reduce per-node record/full-chain native verification cost, retaining exact native intent-before-move/result-after-readback/prefix/claim/NLK/10s. No user policy or timeout changes requested.',
    limits=load(ROOT/'next-production-requirements.json')['requirements'])
write('final-status.json',status)
report=['# r6跨动作历史候选与原预算成本诊断','',f'上次已推源码{head}保持不变。本地未提交候选tree {tree}；candidate.patch和准确执行输入可完整复核。候选未通过完整原预算，未作为源提交/推送。实际fetch与网络恢复证据0182e060d已推，TEAM16/IMP142/r6不变。','', '| 运行 | 新编译 | profile | raw退出 | 过/败 | ms |','| --- | --- | --- | --- | --- | --- |']
for name,r in runs.items():report.append(f"| {name} | {r['newCompilation']} | {r['profile']} | {r['rawExit']} | {r['passed']}/{r['failed']} | {r['elapsedMs']} |")
report+=['', '首library02原优化profile/原10秒fixture raw101/0过3败，后继library03 raw101/30过7败，上游恢复或原后验截止失败均保留。已归档EXE同Hash直接诊断profile01单例raw0/1过：及时交易完成后真实等原截止过期，新5秒只读成功，原期限append仍TimedOut且不发布。这个独立观察不冒充完整验收；同EXE、原37筛选/单线程/原断言完整诊断profile02仍raw101/31过6败，新完整/负例scope未过。所有原失败保持，不能按单例成功替代。', '', '本地候选实现私有非serde/nonClone CompletedOriginals，实际closes、canonical结果/current、最后整树与journal后验全部在原截止内才构造；固定原截止/完成时点/准确current ID与session/manifest/binding/plan/action/claim/epoch/tip/当前原字节/recovery/history/handle摘要。新读取用原调用者when/end最大5秒，前后核同receipt与完整原journal比较、归档/retired对象和及时lease历史。原check_inner/action cutoff不改，写路径仍原截止，不存在deadline setter。当前只能称候选/单例观察，不签typedClosed或生产Status。', '', '恢复助手与append相邻重复journal遍历做了一次职责收敛；完整原journal比较体逐字相同，append写前/写后原native检查保持，实际move完整BOTH-tree before/after、原语affected-tree before/after、原claim/NLK、旧fault位置与断言保留，guard证据在案。但完整预算仍失败，不能声称该改动完成了性能收口。', '', 'CPU/I/O诊断是只读采样实际自有子进程，没有调优先级/改策略/停服务/清缓存。完整profile02 145657ms墙钟、136859.375ms子CPU（94%单核），其中kernel95609.375ms，1369872 readOps/21488996 otherOps；同期间系统总体忙约10.8%/20逻辑CPU。数据不支持归咎全机繁忙或凭空放宽超时。下一步采用r6原已允许的单phase有界原子节点事件批处理，减少durable record/full-chain native核验成本；原文件/安全/内容/children/未知拒绝、intent-before-native/result-after-readback及准确中断前缀不缩。无需用户改变日常配置/策略或给更大预算。', '', '候选未通过完整检查，源分支不创建新commit、不推其实现；本轮仅分离文档原始证据提交推送，工作区保留7个候选文件供继续实现。虚拟tree来自独立临时Git index，正常index未改，临时index已移除；tree不是commit SHA，精确patch和原编译输入同工作源/原Git对象行尾等价核验。product01为本地候选MSVCdebug编译，不是已通过/已推源码的制品或发布。', '', f'共{len(scenes)}场景均私有ZIP核验，6完成纯数据scene核fresh准确路径/全文件hash/目录membership后清理；{counts["failedAndPendingRetained"]}失败/pending及{counts["crossVolumePendingRootsRetained"]}个D根保留。{len(records)}份canonical/快照/history原字节另归raw-scene-records；ZIP非NTFS身份/安全克隆，旧拒绝清理不重试。原失效窗口全部保留，无catch-up。', '', 'Start无条件拒绝，实际Source/Owner/typedStop/原process与supervisor历史/完整inventory角色/生产Status/typedClosed/后继仍未完；原缺席安全初始化/退根、普通MSI/CAP-trust/历史Profile/Public/失败broker清理保持未完。无旧raw5/空GUI/相同1314/W2/双SID/发布或官方/日常配置/令牌/策略修改。目标继续active，不把性能失败当外部权限阻塞，也不标完成。', '', '| 本地候选制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
with (ROOT/'report.md').open('x',encoding='utf-8') as out:out.write('\n'.join(report)+'\n')
write('unused-draft-finalizer.json',dict(file='finalize.py',executed=False,reason='Earlier draft required all new native cases and latest library run pass. Those conditions were not met. candidate-finalize.py is the actual report generator and records failures/incomplete coverage.'))
print(json.dumps(dict(sourceHeadUnchanged=head,localCandidateTree=tree,sourceCommitted=False,runs={k:v['rawExit'] for k,v in runs.items()},
    productInputs=len(product_inputs),finalTestRust=len(test_inputs),scenes=len(scenes),rawRecords=len(records),removed=6,accepted=False)))
