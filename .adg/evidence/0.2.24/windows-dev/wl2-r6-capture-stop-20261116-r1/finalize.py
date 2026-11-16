from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
DOCS=ROOT.parents[4]
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,value):
    with (ROOT/name).open('x',encoding='utf-8') as out:json.dump(value,out,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
runs={};scenes=[];inputs=[];rust=[];records=[];traces=[]
for name,passed in [('library01',6),('library02',16),('product01',None)]:
    result=load(ROOT/name/'result.json');assert result['exitCode']==0
    for row in load(ROOT/name/'source-manifest.json'):
        raw=(ROOT/name/'executed-source'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
        if name=='product01' or name=='library02' and row['path'].endswith('.rs'):
            assert raw==(CODE/row['path']).read_bytes()
            assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n')
            (inputs if name=='product01' else rust).append(row)
    if passed is not None:
        text=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig');m=re.search(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',text);assert m and tuple(map(int,m.groups()))==(passed,0,0)
        traces.extend(dict(run=name,trace=line[line.index('r6'):]) for line in text.splitlines() if 'r6' in line and len(line)<3000)
        for i,scene in enumerate(load(ROOT/f'{name}-private-scenes.json')['scenes']):
            assert sha(Path(scene['archive']).read_bytes())==scene['archiveSha256'];scenes.append(scene)
            if scene['source'].startswith('D:'):
                for row in scene['files']:
                    p=Path(scene['source'])/row['path'];raw=p.read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
                    target=ROOT/'raw-scene-records'/name/f'scene-{i:03}'/row['path'];target.parent.mkdir(parents=True,exist_ok=True)
                    with target.open('xb') as out:out.write(raw)
                    records.append(dict(source=str(p),evidence=target.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
    runs[name]=dict(rawExit=0,profile=result['profile'],passed=passed,failed=0 if passed is not None else None,ignored=0 if passed is not None else None,elapsedMs=result['elapsedMs'],artifact=load(ROOT/f'{name}-artifact.json') if passed is not None else None)
assert load(ROOT/'library01-artifact.json')['sha256']==load(ROOT/'library02-artifact.json')['sha256']
assert load(ROOT/'product01/source.json')['head']==head
write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,finalTestRust=rust,gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyModifiedOrStaged=True))
write('stop-boundary-traces.json',traces);write('raw-scene-records.json',records)
artifacts=[]
for name in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe';raw=p.read_bytes();artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
write('product01-artifacts.json',artifacts)
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
matrix=load(ROOT.parent/'wl2-r6-capture-ready-20261116-r1/final-status.json')['matrix']
matrix.extend([
 dict(id='V3StorePreStopCallChain',result='compiled/adapter-lane-wired-not-invoked',fact='Actual CaptureSlot Ready generation -> original Source/routes+typed ServiceObservation -> original Stop/Restart NativeAction binding -> full official lease reacquire -> BOTH native PreStop observation/archive/reference -> typed PreStop phase -> durable teardown intention -> precise work observer Resume -> explicit original release -> Source/Ready postcheck before prepared flag. Actual Source/Owner path not executed; dispatcher unchanged.'),
 dict(id='V3CoreStoppedSampling',result='compiled-not-invoked/Slot-continuation-incomplete',fact='CapturedTrees requires exact retained PreparedStop generation+original deadline, actual Source/Owner/NormalExit/SupervisorAbsent/StopAccepted/record/listener; then complete lease/new BOTH Stopped snapshots/native phase and original policy series selector. Returns data context while actual lease remains installed on caller stack for same critical-section restoration. Not exposed as public successful Stop/Closed; full CaptureSlot stopped transaction/retirement/restore is next work.'),
 dict(id='PreparedStopDataBoundary',result='directed-counterexamples-pass',fact='Exact pid/port/record digest and unchanged original absolute deadline; wrong generation/digest/port, shortened or extended end, invalid action/zero IDs/bad digest/expired locator refuse. Locator alone creates no native proof. Reuses strict action/PreStop/Stopped progression cases and actual shared original-window tests.'),
])
cleanup=load(ROOT/'cleanup.json')
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract='TEAM16/IMP142/r6',baselineMain='fac8573e55441fe514919c63c7f1d65181ff3577',baselineDocs='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,artifacts=artifacts,matrix=matrix,firstUnexpectedError=None,
 testScope={'library01':'6 data/window cases only; new PreparedStop locator plus existing action progression and actual ActionWindow origin rules, NOT actual Owner/Source production execution','library02':'16 exact same debug EXE: affected Capture native components/source-current native pins/original windows/action progression; no fake typed official proofs','newProductionFactories':'compile and static integration only; not invoked','unchangedReleaseSeriesEvidence':'prior snapshot-series release41/6 passes and original debug failures retained, not rerun'},
 limits=['Production dispatcher/old v2 tags still not switched; actual Source/Ready/PreStop/Stopped factory invocation unproven','CapturedTrees Stopped observation leaves lease on caller stack; full CaptureSlot stopped restoration transaction still needed, no call may report successful Stop or release implicit Drop as success','Source-declared config/auth/Grok changes and dynamic artifact/new-directory retirement/restoration not implemented; current strict unknown refusal remains','Whole native original restoration, immutable original history, typed Closed/successor/real same-owner Status5 and original ordinary Owner/GUI H-S-D not yet combined','OriginalAbsent Codex initialization/safe retirement blocked; ordinary MSI/CAP-trust/all historical cleanup gaps unchanged'],productionDispatcherUsesV3=False,actualSourceOrPreStopFactoryExecuted=False,actualStoppedFactoryExecuted=False,allAccepted=False,startOpen=False,productionExecuted=False,officialOwnerCreated=False,W2=False,dualSID=False,published=False,
 cleanup=dict(archivedScenes=len(scenes),archivedFiles=sum(len(s['files']) for s in scenes),archivedBytes=sum(f['bytes'] for s in scenes for f in s['files']),completedOwnScenesRemoved=len(cleanup['removed']),failureAndPendingScenesRetained=cleanup['failedAndPendingScenesRetained'],crossVolumePendingRootsRetained=cleanup['crossVolumePendingRootsRetained'],nativeMetadataCloned=False,historicalRejectedDeletionRetried=False,newAccounts=0,newProfiles=0,newPublicRoots=0,newGUIProcesses=0,newOfficialServices=0,dailyPathConfigPolicyChanged=False,overallCleanupComplete=False))
write('final-status.json',status)
report=f'''# r6同Store PreStop与Stopped采样调用链

TEAM16/IMP142/r6，源码{head}/tree{tree}；承接f842e9b8d/40134823c。只Windows允许集，原trait/预算/8投影/未知拒绝、固定官方及日常配置保持。

| 运行 | profile | 原退出码 | 过/败/忽略 | ms |
| --- | --- | --- | --- | --- |
'''+ '\n'.join(f"| {name} | {r['profile']} | {r['rawExit']} | {r['passed']}/{r['failed']}/{r['ignored']} | {r['elapsedMs']} |" for name,r in runs.items())+'''

本轮无非预期首错。library01六项为数据与原窗口边界（新增PreparedStop locator＋原action/阶段和原共享ActionWindow规则），不是新实际Source/Owner生产正例。library02复用library01准确debug EXE，执行前后SHA及全部构建输入字节一致，Capture实际native组件/原Source-current pin/原动作窗/阶段进度受影响回归16过0败；actual direct invocation参数在run-command/result，source.json保留生产该EXE的Cargo参数，不冒作第二构建。原release41和debug超时证据按未变组件复用，旧raw5/空GUI/1314/MSI/无关官方读取不重跑。

实际调用链从原WindowsOriginals专用v3 prepare_v3_stop进入CaptureSlot，同一Store准确Ready pid/port/record摘要、原Source/current/image/routes和typed ServiceObservation先复核；原Start截止可以已结束，只通过actual Source/Owner生成的专用NativeAction桥进入新Stop/Restart原窗，不用旧journal+新裸deadline偷续。CapturedTrees在同活lease下观察双根→PreStop snapshot intent/current、实际同卷archives和双根引用→typed PreStopVerified→durable TeardownPrepared，随后准确工作observer Resume保留原档案→显式释放原4域→Source/typedReady/原新窗后验。成功才设置core和Store preparedStop/end；失败保留同installed Store/原树/journal并poison，不旧v2 fallback。官方teardown调用必须在该临界段release后，不在持claim时等官方自取同锁。

Stopped核心接线要求精确原PreparedStop定位/原deadline，原Source/actualOwner、NormalExit、SupervisorAbsent、StopAccepted、record/listener原生后验；fresh完整lease取得后双根Stopped观察/archives/reference与typed StoppedVerified，再取原生产Ready/PreStop/Stopped内容series上下文。成功只标core stopped_observed并返回DATA，actual lease仍安装在caller栈供同一临界段准确退休还原；该方法没有对外发布Stopped/Closed，也不在成功路径Droplease推定完成。完整CaptureSlot stopped transaction和恢复/Closed调用仍未接，不把compiled核心当整组Stop成功。

PreparedStop是额外定位数据，不是原生许可：错pid/port/digest、长短不同绝对end、非Stop/Restart、零值/非法digest/过期均拒绝。实际所有native方法仍重查typed证明/Source/claim。没有假Owner、fake ServiceObservation/NormalExit/Absent/StopAccepted来签生产正例。本批所有新生产factory仅编译未实际调用，dispatcher及旧v2 tags未切，公共require_durable_capture无条件拒绝。

剩余：actualSource/原普通Owner生产调用、完整v3 Stage+Stop transaction、config/auth/Grok源声明非产物与动态artifact/new-directory的准确退休还原、原policy内容和native恢复权限组合、全部原件恢复后NormalExit/监督/端点/record/source/原history与typedClosed/后继/真Status5/原Owner-GUI-H-S-D。原缺席Codex初始化/安全退根仍blocked单列，不能留目录/两锁冒Absent。普通MSI/CAP-trust/历史Profile/Public/旧broker缺口保留。不W2/双SID/模型业务请求/发布。

最终MSVC custom-protocol debug产品输入和最终testRust对工作源/Git blob行尾等价逐份核验，前端32份复用；不是release预算或冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
'''+ '\n'.join(f"| {a['name']} | {a['bytes']} | {a['sha256']} |" for a in artifacts)+f'''

本轮{len(scenes)}native/数据场景私有内容ZIP核验，D canonical/current原字节另归raw-scene-records；{len(cleanup['removed'])}处已完成纯数据场景核全部文件与目录后清理，{cleanup['failedAndPendingScenesRetained']}处失败或prepared custody/原件还原未闭合pending、{cleanup['crossVolumePendingRootsRetained']}处D日志根保留。ZIP不是NTFS身份/安全克隆，历史删除拒绝/账户/Profile/Public/旧broker均不重试，overallCleanup=false。完整继承状态表归final-status.json。
'''
with (ROOT/'report.md').open('x',encoding='utf-8') as out:out.write(report)
with (DOCS/'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md').open('a',encoding='utf-8',newline='\n') as out:out.write(f"\n\n## 2026-11-16 TEAM16/r6 Store PreStop及Stopped采样接线\n\n源码 {head}/tree{tree}；证据 .adg/evidence/0.2.24/windows-dev/{ROOT.name}/。原CaptureSlot Ready→actualSource/route/typedReady新StopAction→完整lease→双根PreStop/teardown intent→Resume→显式release，preparedStop成功后才发布。Stopped核心要求原NormalExit/Absent/StopAccepted/record/listener/原窗，freshlease下双根Stopped/context，lease留caller供后续同临界段恢复；完整Slot恢复/Closed尚未接，不对外签Stop成功。新生产链编译未实跑；六数据/窗口定向及准确同EXE16受影响回归过，本轮无非预期首错。产品MSVCdebug bins0/{runs['product01']['elapsedMs']}ms，源/EXE/完整表/清理归report/final-status。\n\n生产dispatcher/原typed正例/动态nodes与config/auth/Grok/退休还原/Closed/history/后继/实际原Owner-H-S-D待；原缺席退根blocked且Start拒绝，不W2/双SID/发布，不旧raw5/空GUI/1314重跑。\n")
print(json.dumps(dict(head=head,tree=tree,productInputs=len(inputs),finalRust=len(rust),scenes=len(scenes),rawFiles=len(records),artifacts=artifacts)))
