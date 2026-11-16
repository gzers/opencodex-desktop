from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
PREVIOUS=ROOT.parent/'wl2-r6-recovery-chain-20261116-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,value):
    with (ROOT/name).open('x',encoding='utf-8') as out:json.dump(value,out,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
assert head!='55a6d00100a614cbc551b4785424ef5ad7b9e089'
runs={};inputs=[];rust=[];records=[];scenes=[];traces=[]
for name,rc,passed,failed in [('library01',101,None,None),('library02',101,19,4),('library03',101,21,2),('library04',0,2,0),('product01',0,None,None)]:
    result=load(ROOT/name/'result.json');assert result['exitCode']==rc
    if name.startswith('library'):
        runs[name]=dict(rawExit=rc,passed=passed,failed=failed,ignored=0 if passed is not None else None,elapsedMs=result['elapsedMs'],profile=result['profile'],artifact=load(ROOT/f'{name}-artifact.json'))
        text=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig')
        if passed is not None:
            m=re.search(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',text);assert m and tuple(map(int,m.groups()))==(passed,failed,0)
        for line in text.splitlines():
            if 'r6 ' in line and len(line)<2200:traces.append(dict(run=name,trace=line[line.index('r6 '):]))
        for i,scene in enumerate(load(ROOT/f'{name}-private-scenes.json')['scenes']):
            assert sha(Path(scene['archive']).read_bytes())==scene['archiveSha256'];scenes.append(scene)
            for row in scene['files']:
                if scene['source'].startswith('D:') or '/.ocxd024-observed-' in '/'+row['path']:
                    p=Path(scene['source'])/row['path'];raw=p.read_bytes();assert sha(raw)==row['sha256'] and len(raw)==row['bytes']
                    target=ROOT/'raw-scene-records'/name/f'scene-{i:03}'/row['path'];target.parent.mkdir(parents=True,exist_ok=True)
                    with target.open('xb') as out:out.write(raw)
                    records.append(dict(source=str(p),evidence=target.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
    else:runs[name]=dict(rawExit=0,profile=result['profile'],elapsedMs=result['elapsedMs'])
    for row in load(ROOT/name/'source-manifest.json'):
        raw=(ROOT/name/'executed-source'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
        if name=='product01' or name=='library04' and row['path'].endswith('.rs'):
            assert raw==(CODE/row['path']).read_bytes(),row['path']
            assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n'),row['path']
            (inputs if name=='product01' else rust).append(row)
assert load(ROOT/'product01/source.json')['head']==head
assert load(ROOT/'unchanged-inputs-and-budget.json')['sameDebugAndReleaseRust']==rust
observations=load(ROOT/'native-traces-library03-library04.json')['observations']
assert sum(r['kind']=='success' for r in observations)==1 and sum(r['kind']=='interruption' for r in observations)==11
faults={(r['data']['fault'],r['data']['restore']) for r in observations if r['kind']=='interruption'}
assert faults=={(i,False) for i in [1,2,44,40,41,42,43]}|{(i,True) for i in [40,41,42,43]}
write('native-chain-traces.json',traces);write('raw-scene-records.json',records)
write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,finalTestRust=rust,gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyModifiedOrStaged=True))
artifacts=[]
for name in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe';raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
write('product01-artifacts.json',artifacts)
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
matrix=load(PREVIOUS/'final-status.json')['matrix']
matrix.extend([
 dict(id='BorrowedNativeRecoveryWitness',result='native-component-pass/private-no-production-permit',fact='Only anchored module constructs a borrowed BOTH-tree native witness, with no serde/Clone/path constructor. Before/after each step verifies actual original/custody/current/retired/initial work observer/historical snapshot and original directories/stable IDs/all8/bytes/children, same actual claim/NLK and original absolute cutoff. No actual typed Source/Stop/content qualification or new production entrypoint.'),
 dict(id='CanonicalNativeRecoveryMutations',result='native-component-pass/debug-timeout-retained',fact='One complete immutable recovery PlanIntent/Verified and same-chain current summary precede six actual retained current-handle retirements and three exact original-handle returns, each intent/current before move and verified result/current only after complete BOTH-tree readback. Actual current catalog replacement and three originally absent artifacts accounted. Every step SQLite claim-only competitor busy. Final raw original lease release and all original/retired/stable released ranges read back; canonical final-release/history/Closed still pending.'),
 dict(id='CanonicalNativeRecoveryFaultWindows',result='native-component-pass/11-injected-cases',fact='Three plan windows and four retirement/four restoration windows: before native, after actual move, after immutable result, real one-byte current. Full actual file position IDs/bytes, exact immutable events/current pending/gap independently verified. Any native/durable error poisons both trees and journal; recheck never writes catch-up. Raw native/D trees retained. Debug all-row/restore timeout failures remain separate; identical Rust/assertions/deadlines with original optimized profile pass only those two tests.'),
])
cleanup=load(ROOT/'cleanup.json');counts=load(ROOT/'cleanup-counts.json');assert cleanup['removed']==10 and cleanup['failed']==0
first=dict(run='library01',rawExit=101,kind='MSVC compiler E0432/E0603',fact='RecoveryWitness reexport crossed private retirement module; no tests or new executable. Fixed by parent observation reexport. library02 19pass4fail: synthetic Stop component called production reacquire requiring real Owner, correctly refused. Dedicated cfg(test) receipt reacquire preserves original10s cutoff and no production requirement changed. library03 21pass2fail at original bounded-original-deadline; failures retained. library04 only two failed tests with same114 Rust inputs/assertions/original10s and original release profile, raw0/2pass. Archive exclusive prefix collision separately recorded, no overwrite/test repeat.')
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract='TEAM16/IMP142/r6',baselineMain='fac8573e55441fe514919c63c7f1d65181ff3577',baselineDocs='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,artifacts=artifacts,matrix=matrix,firstError=first,executionBoundary=load(ROOT/'execution-boundary.json'),allAccepted=False,startOpen=False,productionDispatcherUsesV3=False,actualSourceOrRecoveryFactoryExecuted=False,productionExecuted=False,officialOwnerCreated=False,W2=False,dualSID=False,published=False,cleanup=dict(**counts,removed=10,historicalRejectedDeletionRetried=False,overallCleanupComplete=False),limits=['actual typed Source/routes/process/audit/unchanged-policy qualification and production driver entry incomplete','canonical final original release/stable proof/history/typedClosed/Status5/handoff incomplete','official nonartifact sideeffects/dynamic directories not admitted','original-absent Codex safe no-gap initialization and root retirement blocked','ordinary MSI/CAP-trust/historical Profile/Public/failed broker cleanup remain open'])
write('final-status.json',status)
report=['# r6真实节点恢复与canonical中断窗口','',f'源码{head}/tree{tree}；承接55a6d0010/848509580，TEAM16/IMP142/r6。仅Windows允许集，保留原claim/NLK/目录及两稳定文件/all8/原预算/未知拒绝。','', '| 运行 | profile | raw退出 | 过/败/忽略 | ms |','| --- | --- | --- | --- | --- |']
for name,r in runs.items():report.append(f"| {name} | {r['profile']} | {r['rawExit']} | {r.get('passed','—')}/{r.get('failed','—')}/{r.get('ignored','—')} | {r['elapsedMs']} |")
report+=['', '首library01 E0432/E0603/raw101，为Witness跨private模块导出错误，无新EXE。library02编译后19过4败：测试合成Stop阶段误用生产重取锁入口，原实际Owner要求正确拒绝；仅新增cfg(test)组件receipt原窗重取，不改生产逻辑。library03 21过2败，完整返回及返回中断在原10秒bounded-original-deadline超时，原现场/raw/EXE均保留。library04仅这两失败用例按完全相同114份Rust/断言/原截止与仓库原release参数定向2过，未延长计时/降读回/跳过场景。19既有数据/日志成功及2新中断成功复用。不W2/双SID/候选冻结/发布。', '', 'RecoveryWitness只借实际两树，由anchored内部构造，不能serde/Clone或从路径重建。每节点前后完整核原件/初始work观察/当前frozen/已退休对象/全部历史archive/准确目录children/ID/all8/bytes；同actual claim/NLK连续。新事件movement绑定准确from/to/原卷/nativeID/target parent/expectedBefore/After及完整plan row摘要。私有日志driver在intent/current已持久后才调用实际same-handle move，随后双树后验、result/current。两个driver只被cfg(test)入口调用，生产无恢复许可。', '', '完整正例实际6current退休、3original返回同一不可变canonical链及current；原current catalog为明确合成atomic替换、3originalAbsence文件退休，原根/所有目录/两稳定对象留原位。每步rusqlite claim-only竞争均busy。全树返回后实际rawlease显式释放、全部原件/retired集合/稳定锁512范围另读回；此rawrelease尚未接canonical final-release/history/Closed，current不冒称Closed。合成阶段/无凭据artifact内容不构成实际Source/Ready/NormalExit/contentpolicy资格。', '', '11注入：Plan不可变记录→current前、真实current部分写、plan intent/current后；退休和还原各beforeMove/afterActualMove/beforeCurrentAfterImmutableResult/realPartialCurrent。独立读取全部真实原/工作位置ID/bytes、最后不可变event与current，确认native已动/结果缺席或phase领先current的精确前缀。错误后原journal和两树poison，拒绝检查不补写。7debug中断观察与5release观察组成12精确trace（1完整+11中断），离线再核actual current/所有原event原字节。', '', '真实Source/routes/原进程/audit+原内容policy必须在转移current frame之前完成，才可开放production driver；完整原释放/后验/history/typedClosed/后继/Status5/实际Owner-H-S-D继续未完。官方config/auth/Grok与动态节点副作用未知拒绝。Start无条件拒绝，原缺席Codex安全无缝初始化/退根blocked；普通MSI/CAP-trust/历史Profile/Public/失败broker清理未关闭。无旧raw5/空GUI/相同1314重跑、无官方或日常策略配置改变/业务网络请求。', '', f'本轮{len(scenes)}场景均私有ZIP核验，10完成纯数据场景核准确fresh路径/完整文件hash和目录membership后清理，{counts["failedAndPendingRetained"]}失败/pending及{counts["crossVolumePendingRootsRetained"]}个D根保留。{len(records)}份canonical/快照原字节另存raw-scene-records；ZIP非NTFS元数据/身份克隆。所有历史拒绝删除不重试，overallCleanupComplete=false。', '', '| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
with (ROOT/'report.md').open('x',encoding='utf-8') as out:out.write('\n'.join(report)+'\n')
print(json.dumps(dict(head=head,runs={k:v['rawExit'] for k,v in runs.items()},productInputs=len(inputs),testRust=len(rust),rawRecords=len(records),scenes=len(scenes),removed=10,production=False)))
