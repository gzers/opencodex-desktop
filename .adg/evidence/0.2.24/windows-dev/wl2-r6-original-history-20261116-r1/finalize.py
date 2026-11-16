from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
PREVIOUS=ROOT.parent/'wl2-r6-final-release-20261116-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,value):
    with (ROOT/name).open('x',encoding='utf-8') as out:json.dump(value,out,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
assert head!='b7c90313643342804a55b64efe9ce861126cdb0a'
runs={};inputs=[];rust=[];records=[];scenes=[];traces=[]
for name,rc,passed,failed in [('library01',0,24,0),('library02',101,4,2),('library03',0,25,0),('library04',0,7,0),('product01',0,None,None)]:
    result=load(ROOT/name/'result.json');assert result['exitCode']==rc
    if passed is not None:
        text=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig')
        m=re.search(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',text);assert m and tuple(map(int,m.groups()))==(passed,failed,0)
        runs[name]=dict(rawExit=rc,passed=passed,failed=failed,ignored=0,elapsedMs=result['elapsedMs'],profile=result['profile'],artifact=load(ROOT/f'{name}-artifact.json'))
        for line in text.splitlines():
            if 'r6 ' in line and len(line)<2200:traces.append(dict(run=name,trace=line[line.index('r6 '):]))
        for i,scene in enumerate(load(ROOT/f'{name}-private-scenes.json')['scenes']):
            assert sha(Path(scene['archive']).read_bytes())==scene['archiveSha256'];scenes.append(scene)
            for row in scene['files']:
                if scene['source'].startswith('D:') or any(marker in '/'+row['path'] for marker in ['/.ocxd024-observed-','/.ocxd024-history-']):
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
for row in load(ROOT/'source-scope.json')['changedFiles']:
    raw=(CODE/row['path']).read_bytes();assert sha(raw)==row['sha256'] and len(raw)==row['bytes']
for row in load(ROOT/'execution-boundary.json')['methods']:
    path=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'/row['definition']
    assert sha(path.read_bytes())==row['moduleSha256']
assert rust==[r for r in load(ROOT/'library03/source-manifest.json') if r['path'].endswith('.rs')]
observations=load(ROOT/'native-traces-library04.json')['observations'];assert len(observations)==9
assert {r['observation']['fault'] for r in observations}=={0,*range(70,78)}
assert len(load(ROOT/'native-traces-library02.json')['observations'])==7
write('original-history-traces.json',traces);write('raw-scene-records.json',records)
write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,finalTestRust=rust,debugAndReleaseSameFinalRust=True,gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyModifiedOrStaged=True))
artifacts=[]
for name in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe';raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
write('product01-artifacts.json',artifacts)
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
matrix=load(PREVIOUS/'final-status.json')['matrix']
matrix.extend([
 dict(id='ExplicitRecoveryRootId',result='directed-data/native-pass',fact='Every current-work retirement/original return/anchor release event carries exact rootId bound to native movement and whole immutable plan row. Aggregate events refuse rootId. Missing/duplicate/wrong/cross-root locators reject, old component records retained without conversion/adoption.'),
 dict(id='OriginalHistoryCanonicalContext',result='directed-data/native-component-pass',fact='Same ChainEntry/current history intent/result binds session/strategy/source binding/security/manifest/action/claim/lease/recovery/final release and BOTH original-frame digests. Intent precedes archive filesystem writes, result only after actual retained BOTH-tree/receipt/archive proofs. Current holds bounded context/reference summary. No original native handle release or actual Source/Stop/Closed authority.'),
 dict(id='ImmutableNativeOriginalHistory',result='native-component-pass/original-handles-retained',fact='Exact already-retained returned original handles are duplicated, never reopened/adopted from a path. Actual original bytes/all8/children/IDs and absence remain distinct from same-volume immutable archive metadata/chunk copy IDs. Existing frame archive machinery keeps its old work schema; history has a distinct schema and exact rootId. Controlled retired work is bound by full immutable recovery plan/stopped snapshot reference. Archive directory pin installed immediately before fallible security readback.'),
 dict(id='OriginalHistoryFailurePrefixes',result='native-component-pass/7-injected-faults',fact='Durable intent before copies; actual directory retained before all8 readback; nonempty first chunk; first root complete; both roots complete; immutable history result ahead of current; actual one-byte current. Original handles, pending directory/partial archive, canonical event/current prefix remain retained; poison never catches up. Pre-fix optimized budget failures remain separate raw101 evidence.'),
 dict(id='NativeHistoryExternalMembership',result='native-component-refusal-pass',fact='After successful history, a new synthetic foreign directory in its private archive causes actual native history reread to refuse and poison BOTH native trees. Past canonical successful DATA remains unchanged and cannot become current native qualification or Closed. Foreign directory retained without cleanup or security repair.'),
 dict(id='NativeHistoryOriginalBudget',result='fixed-with-failures-retained',fact='First optimized round4pass2fail reached original10s while new history caller and witness stacked duplicate full-tree barriers. Final orchestration preserves all old native/file/security/archive/lease comparisons and complete before/after barriers on each native history operation, assigns receipt/journal before/after to caller; exact predicate code verified unchanged. Final optimized scope7pass, same cutoff/profile, no deadline renewal/assertion removal.'),
 dict(id='SnapshotHistoryIdentityAliases',result='data/native-regression-pass',fact='Progress tracks all previously linked native archive/metadata IDs; new references cannot reuse them, and original-history references cannot alias parent/custody/original/current/initial work/stopped snapshot or previous stage archives. Corrected synthetic factory stage/sequence IDs retain all old assertions and explicit duplicate-ID counterexample; final fullv3 debug regression25pass.'),
])
cleanup=load(ROOT/'cleanup.json');counts=load(ROOT/'cleanup-counts.json');assert cleanup['removed']==counts['ownCompletedScenes'] and cleanup['failed']==0
first=dict(run='library02',rawExit=101,passed=4,failed=2,kind='original10s bounded-original-deadline',fact='Full and post-completion membership cases timed out under original optimized profile/cutoff. Exact old raw/EXE/source/scenes/history/current retained; seven injected prefixes passed. New history orchestration consolidated duplicate full-tree calls while preserving native before-after and receipt predicates. Final25 data regression/7 optimized targeted pass independently; no deadline reset. Two patch-matching errors recorded separately, no compiler failure.')
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract='TEAM16/IMP142/r6',baselineMain='fac8573e55441fe514919c63c7f1d65181ff3577',baselineDocs='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,artifacts=artifacts,matrix=matrix,firstError=first,executionBoundary=load(ROOT/'execution-boundary.json'),barrierOwnership=load(ROOT/'barrier-ownership-verification.json'),allAccepted=False,startOpen=False,productionDispatcherUsesV3=False,actualSourceOrRecoveryFactoryExecuted=False,productionExecuted=False,officialOwnerCreated=False,originalHandlesReleased=False,typedClosedQualified=False,W2=False,dualSID=False,published=False,cleanup=dict(**counts,removed=cleanup['removed'],historicalRejectedDeletionRetried=False,overallCleanupComplete=False),limits=['actual Source/routes/retained process/audit/shared content policy before frame transfer and production driver permission incomplete','native original/custody handle-release and original process/supervisor completion history/typedClosed pending','Status5/handoff/successor/actual Owner H-S-D/official first-use initialization unaccepted','official nonartifact sideeffects/dynamic directories not admitted','original-absent Codex safe no-gap initialization/root retirement blocked','ordinary MSI/CAP-trust/historical Profile/Public/failed broker cleanup remain open'])
status['canonicalOriginalHandleReleaseQualified']=status.pop('originalHandlesReleased')
status['testHandlesClosedAtScopeEnd']=True
status['rawDropIsNotCanonicalHandleRelease']=True
write('final-status.json',status)
report=['# r6显式rootId与不可变原件历史','',f'源码{head}/tree{tree}；承接b7c903136/e92313aef，TEAM16/IMP142/r6。仅Windows允许集，原目录/两稳定对象/原件/all8/原预算/未知拒绝保持。','', '| 运行 | profile | raw退出 | 过/败/忽略 | ms |','| --- | --- | --- | --- | --- |']
for name,r in runs.items():report.append(f"| {name} | {r['profile']} | {r['rawExit']} | {r.get('passed','—')}/{r.get('failed','—')}/{r.get('ignored','—')} | {r['elapsedMs']} |")
report+=['', '首实际非预期失败library02 raw101/4过2败，完整封存与封存后外来成员场景触及原10秒bounded-original-deadline；七注入前缀通过。原源码/EXE/hash/rawstderr/current/不完整与已发布历史/原场景完整保留，history-timeout-forensics只读核验未补写。新历史caller与witness重叠整树读回被合并为明确职责：每native操作完整BOTH-tree before/after，caller原receipt/journal before/after；旧安全/文件/恢复/lease/archive比较实现未删改，barrier-ownership-verification逐字核验，无新计时或预算放宽。library03最终debug完整v3回归25过，library04新优化scope7过，两者全部最终Rust字节相同。旧合成快照factory不同stage复用ID经静态识别后修为stage/sequence独立ID，保留原断言并补别名反例，未跑已知错误前提。两个补丁匹配失败只归编辑记录，无compiler失败。', '', '恢复/current/original/anchor事件新增explicit rootId，准确绑定原node/current row和native movement；aggregate不得带rootId，缺失/重复/错根/跨根拒绝，不转换旧记录。历史intent/result进入同一ChainEntry/current；上下文绑定原Source/manifest/binding/安全policy/action/claim/lease/恢复计划/最终释放事件和双根原始frame摘要，actual receipt与完整原生BOTH-tree边界围住所有历史操作。', '', '原件仍为准确原路径/ID/8投影/字节/children的保管句柄；历史视图仅try_clone这些实际原handle，不从路径/JSON重建身份。副本用同卷SID私有create-new目录/分块/metadata，区分原件ID与归档副本ID；metadata记录原帧/复制块及exact rootId引用，绑定完整controlled retired集合和Stopped原快照。目录创建后立即留pin、先于易失败all8读取；任何partial目录/块/根都保存在原native树。已有工作归档schema不变，原件history用独立明确schema。历史DATA不是原句柄释放或Closed许可证。', '', '新原生9场景：完整1，intent后/目录创建留pin后/非空块后/首根后/双根后/不可变结果领先current/真实1字节current七注入，及成功封存后外来空目录导致native history拒绝1。每失败保留准确原件/退休集合/partial archive和canonical前缀，拒绝check不catch-up；外改场景保留已成功的历史DATA但两个原生树poison，既不重写过去事实也不接纳外来目录。所有原件句柄保留，未生产签字。副本不能别名parent/custody/original/current/initialwork/旧快照，引用/未知字段/textClosed反例拒绝。', '', '本批仍无actual Source/Owner/NormalExit/content policy生产资格。后继需在current frame转移前接原route配置pin/实际process PID/audit home与原时间区间/独立四阶段原sharedpolicy，再完整原件句柄生命周期、原process/supervisor完成history/typedClosed/同代Status5/管理器交接与后继。官方config/auth/Grok及动态目录副作用未知拒绝；first-use initialization/ownerreceipt未验；原缺席Codex安全无缝初始化/退根独立blocked。Start无条件拒绝，普通MSI/CAP-trust/历史清理未关闭。无旧raw5/空GUI/相同1314/W2/双SID/发布或官方/日常配置改变。', '', f'本轮{len(scenes)}场景完整私有ZIP核验；{cleanup["removed"]}完成纯数据scene核fresh准确路径/全部文件hash/目录membership后清理，{counts["failedAndPendingRetained"]}失败/pending及{counts["crossVolumePendingRootsRetained"]}个D根保留。{len(records)}份canonical/工作快照/原件history原字节另存raw-scene-records，ZIP非NTFS身份/安全克隆。历史拒绝删除不重试，overallCleanupComplete=false。完整继承状态表归final-status.json。', '', '| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
rendered='\n'.join(report)+'\n'
assert rendered.count('所有原件句柄保留，未生产签字。')==1
rendered=rendered.replace('所有原件句柄保留，未生产签字。','原件句柄在测试交易期间保管；测试scope结束按Drop关闭，不能冒充canonical原件句柄释放或Closed，未生产签字。')
with (ROOT/'report.md').open('x',encoding='utf-8') as out:out.write(rendered)
print(json.dumps(dict(head=head,runs={k:v['rawExit'] for k,v in runs.items()},productInputs=len(inputs),testRust=len(rust),rawRecords=len(records),scenes=len(scenes),removed=cleanup['removed'],production=False)))
