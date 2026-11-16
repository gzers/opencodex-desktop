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
for name,passed in [('library01',3),('library02',13),('product01',None)]:
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
write('native-capture-traces.json',traces);write('raw-scene-records.json',records)
artifacts=[]
for name in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe';raw=p.read_bytes();artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
write('product01-artifacts.json',artifacts)
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
matrix=load(ROOT.parent/'wl2-r6-snapshot-series-20261116-r1/final-status.json')['matrix']
matrix.extend([
 dict(id='InitialCaptureWorkOpen',result='native-component-pass/production-method-wired',fact='Static integration review found CaptureSlot prepared sealed work and released without canonical Open; CREATE_NEW description would block official writes. Actual CaptureSlot now calls prepare->canonicalOpen->Source/unresumed checks->explicit release. Native core20 Open/22release permits real same-byte work writer and keeps original custody; full unchanged preResume-like reacquire/release completes26 records, no official launch.'),
 dict(id='ActualSourceBeforeResumeCaptureLane',result='compiled-not-executed',fact='Original WindowsOriginals v3 lane delegates to CaptureSlot Source/routes/unresumed actual Owner+original Start window, full actual lease reacquire, exact initial work/native generation unchanged proof and explicit release. Unexpected pre-creator edit refuses; old v2 lane/Start gate not switched.'),
 dict(id='ActualSourceReadyCaptureLane',result='compiled-not-executed',fact='CaptureSlot->CapturedTrees original typed ServiceObservation chain reacquires full original coordination, observes BOTH work roots, writes Ready snapshot intent/archives/link, records actual Source/Owner/typedReady phase, resumes exact work descriptions keeping immutable archives, explicitly releases and rechecks Source/Ready before flag publication. No fake actual Owner/Source/typedReady positive; production dispatcher not invoked.'),
 dict(id='InstalledCoreOpenFaults',result='native-component-pass',fact='Open intent/native/result/partial-current failures retain same installed original/work/native journal and Opening prefix; no opened/ready success flag on error. Reentry refuses. Real unexpected work edit before official creator would run refuses unchanged check and preserves original custody+foreign edit.'),
])
cleanup=dict(time=datetime.now().astimezone().isoformat(),archivedScenes=len(scenes),archivedFiles=sum(len(s['files']) for s in scenes),archivedBytes=sum(f['bytes'] for s in scenes for f in s['files']),completedOwnScenesRemoved=0,failureAndPendingScenesRetained=len(scenes),crossVolumePendingRootsRetained=sum(s['source'].startswith('D:') for s in scenes),reason='All retained native capture scenes contain prepared original custody and working objects without final restoration/Closed, or deliberate old pending namespace. Keep originals and immutable private content ZIPs; successful ephemeral current-pin scopes clean via original fixture. No exact task deletion attempted.',nativeMetadataCloned=False,historicalRejectedDeletionRetried=False,newAccounts=0,newProfiles=0,newPublicRoots=0,newGUIProcesses=0,newOfficialServices=0,dailyPathConfigPolicyChanged=False,overallCleanupComplete=False)
write('cleanup.json',cleanup)
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract='TEAM16/IMP142/r6',baselineMain='fac8573e55441fe514919c63c7f1d65181ff3577',baselineDocs='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,artifacts=artifacts,matrix=matrix,firstUnexpectedError=None,staticIntegrationFinding='Sealed prepared CaptureSlot omitted canonical Open, so official work writes would fail sharing before Ready; production sequence corrected, no failed production execution fabricated.',
 controlledFailures=['four installed Open intent/native/result/current fault windows retained','unexpected initial work mutation before creator resumes refuses','old Store pending namespace and reentry refusal reused'],
 limits=['Actual Source/Owner/beforeResume/typedReady factory chain is wired and compiled but not executed; new tests exercise same native core with synthetic original objects and source-free component context only','Production dispatcher and v2 tags still not switched; original durable public Start gate unconditional refusal','Store Stop/new original action/PreStop/Stopped chain, source-declared config/auth/Grok changes, dynamic artifacts/directories, policy-qualified exact retirement/restoration/Closed/history/successor and actual original ordinary Owner H-S-D remain incomplete','OriginalAbsent Codex safe initialization/retirement blocked; previous debug timeout/raw failures, ordinary MSI/CAP-trust/historical cleanup retained'],productionDispatcherUsesV3=False,productionSourceFactoryExecuted=False,nativeReadyFactoryExecuted=False,startOpen=False,allAccepted=False,productionExecuted=False,officialOwnerCreated=False,W2=False,dualSID=False,published=False,cleanup=cleanup)
write('final-status.json',status)
report=f'''# r6实际CaptureSlot初始可写及typedReady接线

TEAM16/IMP142/r6，源码{head}/tree{tree}，承接512ff5de2/527642b02。仅Windows允许集；原trait/期限/Source/route/policy/固定官方/日常配置不变。

| 运行 | profile | 原退出码 | 过/败/忽略 | ms |
| --- | --- | --- | --- | --- |
'''+ '\n'.join(f"| {name} | {r['profile']} | {r['rawExit']} | {r['passed']}/{r['failed']}/{r['ignored']} | {r['elapsedMs']} |" for name,r in runs.items())+'''

本轮无非预期首错。实际接线静态复核发现：CaptureSlot此前prepare后直接release，工作副本保留CREATE_NEW拒写句柄，实际官方Resume会被分享冲突挡住。未编造生产失败；新增同一canonicalOpen/原生observer handoff在release前执行，原件custody/目录与两稳定文件的严格句柄不改变。三新原生定向证明20事件Open/22显式release后实际合成work写可行且原件字节/身份保留，重新取得原域核完整初始work不变再显式释放共26事件；四处Open意图/native/结果/真实current部分写故障保留installed core/原句柄/current及Opening，成功opened/ready不发布，reentry拒绝。官方Creator未运行前外部编辑不是官方声明副作用，完整initial recheck拒绝并保留原custody和外改对象。测试没有制造Source/Owner/typedReady或实际官方launcher。

实际生产调用链已写进CaptureSlot及原WindowsOriginals专用v3 lane：capture先prepare/Open、Source/routes/unresumed actual Owner复核后显式release；before_v3_resume通过原Source/RoutePin/actual unresumed Owner/original Start action窗重新取完整NativeLease，核initial工作全部ID/8投影/bytes/children不变，随后显式release并前后复核；record_v3_ready消费实际typed ServiceObservation和原Source/Owner，完整lease重取→双根观察→Ready快照intent/原native archive/双根link→ReadyVerified→准确work observer Resume保留原档案→显式release→Source/typedReady复核后发布内部ready代次。live lease仍栈内，失败先清理自有范围，installed Store和原像/引用不掉，旧v2 fallback拒绝。

生产链仅编译、未实际调用。公共Start无条件拒绝，dispatcher/v2 tags未切换；不是actualSource/officialReady/原Owner生产验收。旧长序列release41通过与debug超时原失败均复用并保持。当前EXE先跑3新定向，再确认全部执行输入字节/EXE摘要相等直接运行原Capture路径+旧current pin+原动作预算过滤器，13过0败，执行前后hash一致；无重复Rust编译、无修改断言/截止/过滤器。回归实际程序/参数/源归library02/run-command与run-regression脚本，不把重用EXE伪装新构建。产品debug仍在最终源码提交上独立MSVC bins构建并核摘要。

剩余要求：v3 Store真实Stop新动作/PreStop/Stopped全链、完整config/auth/Grok源声明非产物、副作用原policy和动态nodes的准确退休还原、完整NormalExit/监督/端点/record/Source/typed Closed/history/后继、原普通账户originalOwner/H-S-D生产。原缺席Codex初始化/安全退根blocked单列，不能existing-root组件代首次环境。旧普通MSI/CAP-trust/全部历史清理缺口保持。不W2/双SID/真实业务模型请求/发布，旧raw5/空GUI/1314未重跑。

最终产品输入/最终testRust与工作源/Git blob行尾等价逐项核，前端32份复用；custom-protocol debug不是release预算或冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
'''+ '\n'.join(f"| {a['name']} | {a['bytes']} | {a['sha256']} |" for a in artifacts)+f'''

本轮{len(scenes)}处native原场景内容私有ZIP核验，{cleanup['crossVolumePendingRootsRetained']}处D日志根原phase/current字节另归raw-scene-records。全部保留场景仍含prepared原custody/工作对象未完成最终还原或故意old pending，故本批0删除，不伪造Closed或把准备通过当可清理。原scope成功临时current-pin夹具按原fixture清理；未操作历史拒绝删除/账户/Profile/Public/旧broker。ZIP不是NTFS身份/安全克隆，overallCleanup=false。完整继承状态表归final-status.json。
'''
with (ROOT/'report.md').open('x',encoding='utf-8') as out:out.write(report)
with (DOCS/'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md').open('a',encoding='utf-8',newline='\n') as out:out.write(f"\n\n## 2026-11-16 TEAM16/r6 CaptureSlot可写与原Ready调用链\n\n源码 {head}/tree{tree}；证据 .adg/evidence/0.2.24/windows-dev/{ROOT.name}/。修实际capture prepare后sealed句柄遗漏Open，改canonicalOpen→Source/unresumed复验→显式release，before_v3_resume原Owner窗完整initial work复核，typedReady实际工厂双根观察/归档/link/Ready/Resume/release调用链已编译未调用。三新native组件测试通过，准确同debugEXE13受影响回归通过，本轮无非预期首错。产品MSVCdebug bins0/{runs['product01']['elapsedMs']}ms，原始源/EXE/完整表/0删除且全部pending留存见report/final-status。\n\n生产dispatcher/Start拒绝未动，actualSource/Ready正例未实跑；Store Stop/原policy+动态nodes/退休还原/Closed/history/后继/实际Owner H-S-D及原缺席安全退根仍待。不W2/双SID/发布，不旧raw5/空GUI/1314重跑。\n")
print(json.dumps(dict(head=head,tree=tree,productInputs=len(inputs),finalRust=len(rust),scenes=len(scenes),rawFiles=len(records),artifacts=artifacts)))
