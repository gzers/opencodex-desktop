from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
PREVIOUS=ROOT.parent/'wl2-r6-retirement-native-20261116-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,value):
    with (ROOT/name).open('x',encoding='utf-8') as out:json.dump(value,out,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
assert head=='55a6d00100a614cbc551b4785424ef5ad7b9e089'
runs={};inputs=[];rust=[];records=[];scenes=[];native=[];traces=[]
for name,passed in [('library01',17),('library02',19),('product01',None)]:
    result=load(ROOT/name/'result.json');assert result['exitCode']==0
    if passed is not None:
        text=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig')
        m=re.search(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',text);assert m and tuple(map(int,m.groups()))==(passed,0,0)
        for line in text.splitlines():
            if 'r6-recovery-native-plans=' in line:native.append(dict(run=name,plans=json.loads(line.split('r6-recovery-native-plans=',1)[1])))
            if 'r6 ' in line and len(line)<2200:traces.append(dict(run=name,trace=line[line.index('r6 '):]))
        runs[name]=dict(rawExit=0,passed=passed,failed=0,ignored=0,elapsedMs=result['elapsedMs'],artifact=load(ROOT/f'{name}-artifact.json'))
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
        if name=='product01' or name=='library02' and row['path'].endswith('.rs'):
            assert raw==(CODE/row['path']).read_bytes(),row['path']
            assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n'),row['path']
            (inputs if name=='product01' else rust).append(row)
assert len(native)==1
write('native-recovery-plan-observations.json',native);write('recovery-traces.json',traces);write('raw-scene-records.json',records)
write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,finalTestRust=rust,gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyModifiedOrStaged=True))
artifacts=[]
for name in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe';raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
write('product01-artifacts.json',artifacts)
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
unchanged=[]
for path in ['apps/desktop/tauri/Cargo.toml','apps/desktop/tauri/Cargo.lock','apps/desktop/tauri/src/modules/runtime/client_artifact_policy.rs','apps/desktop/tauri/src/modules/runtime/official_lifecycle024.rs','apps/desktop/tauri/src/modules/runtime/windows_lifecycle024_contract.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/mod.rs']:
    raw=(CODE/path).read_bytes();assert raw.replace(b'\r\n',b'\n')==git('show',f'67c37937e:{path}').replace(b'\r\n',b'\n')
    unchanged.append(dict(path=path,bytes=len(raw),sha256=sha(raw)))
write('unchanged-build-and-contract-inputs.json',dict(baseline='67c37937e',unchanged=unchanged,releaseProfileChanged=False,budgetsChanged=False,officialExecuted=False))
matrix=load(PREVIOUS/'final-status.json')['matrix']
matrix.extend([
 dict(id='CompleteRecoveryPlanV3Data',result='native-observation/data-pass',fact='Two complete native original/current/archive plans validate exact IDs/all8/children/bytes/parents and same-volume retirement targets. Initial work/current replacement/original objects remain separate. Original-current artifact absence is explicit; originally-absent new artifact is a distinct flag. Missing/duplicate/unknown/crossscope or self-consistently rehashed nonartifact changes reject. Scope permits only exact3 new artifact leaves; no content policy exception.'),
 dict(id='CanonicalRecoveryEventProgress',result='data-counterexamples-pass/native-driver-pending',fact='Recovery plan intent/result and all current retirement/original restore intent/result pairings join ChainEntry/Progress/current bounded summary. Actual lease/action/snapshot/strategy/manifest/plan scope remains bound; pending forbids cross-node/skip/reentry/early restore/old node-work-snapshot-action-lease bypass. All-row completion remains insufficient for anchor release or Closed. Runtime Journal currently never appends these new events; actual same-handle witnesses and fault windows must be integrated next.'),
 dict(id='ExplicitAbsentCurrentOriginalArtifact',result='native-component-data-pass',fact='Captured existing synthetic artifact work was removed before freeze while its original remained under custody handle; actual immutable frame/archive and complete plan record exact original ID/parent for absence. Actual current catalog atomic replacement and unmodified config/Grok work were independently validated. Observation context only; no native Stopped/Source/content qualification and no native recovery move in this new test.'),
])
cleanup=load(ROOT/'cleanup.json');counts=load(ROOT/'cleanup-counts.json');assert cleanup['removed']==7 and cleanup['failed']==0
first=dict(run='driver00',rawExit=1,phase='evidence harness before cargo',fact='Missing library01 evidence directory caused Set-Content source.json failure. Directory created once; no compiler/test/EXE had run. First actual compiler/native run library01 raw0; library02 raw0. Historical debug timeout/raw failures remain unchanged in predecessor evidence.')
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract='TEAM16/IMP142/r6',baselineMain='fac8573e55441fe514919c63c7f1d65181ff3577',baselineDocs='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,artifacts=artifacts,matrix=matrix,firstError=first,executionBoundary=load(ROOT/'execution-boundary.json'),allAccepted=False,startOpen=False,productionDispatcherUsesV3=False,actualSourceOrRecoveryFactoryExecuted=False,productionExecuted=False,officialOwnerCreated=False,W2=False,dualSID=False,published=False,cleanup=dict(**counts,removed=7,historicalRejectedDeletionRetried=False,overallCleanupComplete=False),limits=['native recovery intent/move/result/current fault integration pending','actual typed Stop + source/routes/process/audit/policy/native recovery/history/Closed/Status5/handoff incomplete','official nonartifact sideeffects/dynamic directories not admitted','original-absent Codex safe no-gap initialization and root retirement blocked','ordinary MSI/CAP-trust/historical Profile/Public/failed broker cleanup remain open'])
write('final-status.json',status)
report=['# r6完整退休计划与同链恢复数据进度','',f'源码{head}/tree{tree}；承接67c37937e/568aa9c4e，TEAM16/IMP142/r6。既定Windows写集，原目录/两稳定文件/8投影/审计ExcludedUnknown/预算/未知拒绝不变。','', '| 运行 | raw退出 | 过/败/忽略 | ms |','| --- | --- | --- | --- |']
for name,r in runs.items():report.append(f"| {name} | {r['rawExit']} | {r.get('passed','—')}/{r.get('failed','—')}/{r.get('ignored','—')} | {r['elapsedMs']} |")
report+=['', '首错为driver00证据目录缺失，raw1发生于Cargo前，事实独列；两次实际MSVC定向编译/测试均raw0，历史失败不改通过。library01为初步严格计划/进度17过；library02补显式缺席、保持原规则的自洽坏摘要反例及真实原生冻结双树，共19过。native观察沿原10秒fixture截止，没有新计时、预算延长或断言移除。product01仅custom-protocol debug bins编译，不启动GUI/官方。', '', '完整计划绑定双根原StoragePlan、准确当前冻结元数据与same-volume私有archive引用，以及每普通current对象的原卷/ID/all8/bytes/hash、父ID和准确退休target。新增absentCurrent显式列原本存在、当前缺席的准确artifact；与originalAbsent的新artifact严格区分。原目录/协调对象不可退休，非artifact内容/ID/安全/存在性必须等于原工作代次，额外空目录/节点仍拒绝。DATA scope只容纳原已观察准确三artifact，不给名称豁免或恢复许可证。', '', '同一ChainEntry新增Recovery事件；Progress保管计划/逐当前文件退休/逐原件返回/pending，current只记录有界摘要/计数/pending。PlanIntent→PlanVerified后才允许严格成对逐row进度，全部当前退休后才准原件返回；原nodes同步到8，但无anchor release/全树后验仍不能complete。任一pending/新scope/错序/cross-node/旧node-work-snapshot-action-lease旁路拒绝。运行时尚未调用Journal append这些新事件，不能把纯数据进度当native执行或写盘中断已过。', '', '本轮native正例使用真实既有artifact原件、被修改/原子替换的current catalog、原当前cache缺席和Grok/普通配置。完整原生frame/archive计划由实际live claim核验，缺席/目标篡改拒绝；该用例是Observation组件，未造实际Source/Ready/NormalExit/Stopped/policy/恢复权限，也未执行新恢复移动。原private raw退休/返回仍只被旧native tests调用，边界检查及Start无条件拒绝保留。', '', '后继须接真实双树witness、原内容policy和typed Stopped，再将intent/current→same-handle move→result/current故障窗口逐项实证；最后原根/children/所有原件/退休集合/显式lease释放/postconditions/history/typedClosed。生产dispatcher和原包H-S-D/Status5/后继未齐，Start关闭。原缺席安全退根独立blocked，普通MSI/CAP-trust及历史清理保持。旧raw5/空GUI/相同1314/W2/双SID未跑；无官方/日常配置/权限策略改变或发布。', '', f'本轮51场景已完整私有ZIP核验，7完成纯数据场景按本轮exact路径/全部文件hash/目录membership核验后清理；44失败或pending场景、{counts["crossVolumePendingRootsRetained"]}跨卷D根保留。原生与durable原日志/快照另存raw-scene-records；ZIP不是NTFS身份安全克隆。历史拒绝删除不重试，overallCleanupComplete=false。完整继承状态表归final-status.json。', '', '| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
with (ROOT/'report.md').open('x',encoding='utf-8') as out:out.write('\n'.join(report)+'\n')
print(json.dumps(dict(head=head,runs={k:v['rawExit'] for k,v in runs.items()},productInputs=len(inputs),testRust=len(rust),rawRecords=len(records),scenes=len(scenes),removed=7,production=False)))
