from pathlib import Path
from datetime import datetime
import hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parent;DOCS=ROOT.parents[4];CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));sha=lambda b:hashlib.sha256(b).hexdigest();git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(n,v):
    with (ROOT/n).open('x',encoding='utf-8') as f:json.dump(v,f,ensure_ascii=False,indent=2)
source=load(ROOT/'source-commit.json');producer=source['head'];head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
assert producer=='3afd4e59dc61c01a9d63053b498a222b91597634' and head=='625101e7bfa9dd60e5753361ebe9dce9561969d8'
remote=git('ls-remote','origin','refs/heads/codex/024-windows-native').decode().split()[0];assert remote==head
write('source-push.json',dict(branch='codex/024-windows-native',head=head,remoteHead=remote,productProducerHead=producer))
inputs=[];rust=[];deltas=[]
for name in ['library01','product01']:
    result=load(ROOT/name/'result.json');assert result['exitCode']==0
    if name=='product01':assert load(ROOT/name/'source.json')['head']==producer and '--features' in result['arguments']
    for row in load(ROOT/name/'source-manifest.json'):
        raw=(ROOT/name/'executed-source'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
        assert raw.replace(b'\r\n',b'\n')==git('show',f"{producer}:{row['path']}").replace(b'\r\n',b'\n')
        now=(CODE/row['path']).read_bytes()
        if raw!=now:
            assert row['path']=='test/windows024-native/archive-r6-failed-preresume.ps1'
            deltas.append(dict(run=name,path=row['path'],compiledSnapshotSha256=sha(raw),currentArchiveScriptSha256=sha(now),reason='Standalone read-only archive script updated AFTER frozen production Rust build, separately committed625101e7b; no Rust/normalStop/source gate changed. Product stays attributed3afd4e59d.'))
        if name=='product01':inputs.append(row)
        if name=='library01' and row['path'].endswith('.rs'):rust.append(row)
assert git('diff','--name-only',producer,head).decode().splitlines()==['test/windows024-native/archive-r6-failed-preresume.ps1','test/windows024-native/archive-r6-process-facts.cs','test/windows024-native/probe-r6-archive-process-facts.ps1']
artifacts=[]
for n in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{n}.exe';raw=p.read_bytes();artifacts.append(dict(name=n,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=producer,sourceTree=source['tree'],profile='debug',features=['custom-protocol']))
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
write('product01-artifacts.json',artifacts)
write('final-source.json',dict(head=head,tree=tree,productProducerHead=producer,productProducerTree=source['tree'],productInputs=inputs,testRust=rust,scriptOnlyAfterBuildDeltas=deltas,preserved=source['preserved'],fixedSourceShaLiteralsUnchanged=True,gitCRLFExplicitlyEquivalent=True))
scenes=load(ROOT/'library01-private-scenes.json')['scenes']
assert len(scenes)==2
for s in scenes:
    assert sha(Path(s['archive']).read_bytes())==s['archiveSha256'] and Path(s['source']).exists()
    with zipfile.ZipFile(s['archive']) as z:
        for row in s['files']:assert sha(z.read(row['path']))==row['sha256']
probe1=load(ROOT/'archive-probe01/result.json');probe2=load(ROOT/'archive-probe02/result.json')
assert not probe1['passed'] and probe2['passed'] and probe2['moduleEnumerationPath'] is None
assert all(v['forcedCleanup'] and v['rawExit']==242 and v['originalHandlesClosed'] and not v['resumed'] for v in [probe1,probe2])
for name in ['archive-r6-process-facts.cs','probe-r6-archive-process-facts.ps1','archive-r6-failed-preresume.ps1']:
    p=CODE/'test/windows024-native'/name;target=ROOT/'final-archive-source'/name;target.parent.mkdir(exist_ok=True);target.write_bytes(p.read_bytes())
for name in ['archive-r6-failed-preresume.ps1','archive-r6-process-facts.cs']:
    p=Path(load(ROOT/'archive-kernel-staged-input.json')['public'])/name
    assert p.read_bytes()==(CODE/'test/windows024-native'/name).read_bytes()
base=load(ROOT.parent/'wl2-r6-existing-official-qualification-20261117-r1/final-status.json')
base.update(sourceHead=head,sourceTree=tree,productionSourceHead=producer,productionSourceTree=source['tree'],productionArtifactReference='product01-artifacts.json',artifacts=artifacts,runsThisRound=dict(library01=load(ROOT/'library01/result.json'),product01=load(ROOT/'product01/result.json')),actualProductionExecuted=False,startOpen=False,allAccepted=False,qualificationDesktopPending=False,qualificationFirstActualRun='../wl2-r6-existing-official-qualification-20261117-receive-r1/result.json',qualificationFirstActualResult='failed-raw2-before-resume',qualificationOriginalBrokerRetained=dict(pid=54692,filetime='134393577413203335',rawExit=None),qualificationOriginalLauncherRetained=dict(pid=50580,filetime='134393577919362322',creatorResumed=False),archiveDesktopPending=True,archiveDesktopLauncher='C:/Users/Public/ocx-r6-failure-archive-20261117-r2/archive-r6-failed-preresume.cmd')
for row in base['matrix']:
    if row['id']=='ExactExistingOfficialQualificationEntry':
        row['result']='actual-desktop-preparation-pass/first-native-capture-raw2-before-resume/repair-compiled-not-rerun'
        row['fact']='Actual c870bd81f first run received: fixed bundle/full image/NLK fixture setup pass, Start9842ms fails source-path os2 before original client custody or creator resume; originalbroker54692 and suspendedNode50580 retained. Current Windows source references fixed to npm-prefix/PACKAGE_SUBPATH (Bun unchanged), 9affected tests and custom-protocol product pass; no same-original-instance retry/Owner adoption. Original failure and complete raw22 records at receive-r1.'
base['matrix'].append(dict(id='CorrectFixedOfficialSourcePackagePaths',result='actual-fixed-file-native-counterexample-and-repaired-pin-pass/product-compiled/production-retest-blocked',fact='Owner derives actual package root once using shared PACKAGE_SUBPATH; audit4/five role callsite source hashes/CLI/sentinel all use it. Actual fixed299 prefix source reproduces raw2; correct package native pins/IDs/all8/exact hashes pass; wrongprefix/wronghash/sentinel mismatch and raw-role/query assertions still refuse. All original source SHA literals unchanged; prefix node_modules/bun/bin/bun.exe unchanged; sharedHMAC/trait/lease/closed/current budgets and full recovery_state_at/Status tail preserved.'))
base['matrix'].append(dict(id='ReadOnlyFullFailedSceneArchive',result='r1-failed-before-copy/r2-kernel-helper-proven/ordinary-r2-pending',fact='Original r1 archive script .NET MainModule path lookup fails on uninitialized suspended Node before copying files. New query-only OpenProcess QUERY_LIMITED_INFORMATION|SYNCHRONIZE/GetProcessTimes/QueryFullProcessImageName helper has no resume/terminate/Owner method; original PID/FILETIME/session/path comparisons retained with normal Windows case-insensitive image spelling. Controller-owned never-resumed cmd probe reproduces null MainModule, kernel identity matches actual retained creation. Probe01 failed only WINDOWS-vs-Windows literal case; raw1/forced242 original cleanup kept. Probe02 passes/raw0 with own original handle forced242 explicitly distinct from normalStop. No marenben or historical40588 process terminated. Full ordinary archive r2 still needs desktop.'))
base['firstErrorThisRound']=dict(inheritedActualProduction='Original c870bd81f Start9842ms os2; created-suspended retained, not overwritten or marked success.',directedSourcePathTests=None,archiveOrdinary='r1 .NET MainModule/GetFullPath invalid path before copying any files; original error and scripts retained.',archiveProbe='probe01 strict string spelling mismatch C:/WINDOWS vs native C:/Windows; kernel FILETIME/session/image correct. Native same-Windows-path comparison fixed, original raw/time/absence requirements unchanged; probe02 pass.',network='Source push3afd4e59d succeeded; immediate ls-remote received Empty reply from server, subsequent ls-remote verified latest625101e7b. No native rerun.',cleanup='Automatic approval rejected recursive removal of2new completed source-only fixtures with reported reason blocked by policy. Command never executed. No alternate deletion or retry; both sources and verified archives retained.')
base['cleanupThisRound']=load(ROOT/'source-only-scene-cleanup.json')
base['cleanupThisRound'].update(marenbenFailureSceneAndProcessesRetained=True,fullMarenbenArchivePending=True,controllerProbeOriginalHandlesClosed=True,controllerProbeForcedRaw242=True)
base['nextRequiredDecision']='After exact same-SID complete archive, existing loaded c870 diagnostic host has no control channel and keeps old Owner/Job. A rebuilt process must not adopt oldOwner/PID or mint NormalExit/Closed. Need an explicit controlled cleanup/new-independent-scene plan before any new production run; source fix itself does not authorize retry or change old failure. Preserve unknown/forced distinctions and public refusal.'
base['timeThisRound']=datetime.now().astimezone().isoformat();write('final-status.json',base)
report=f'''# r6实际包目录修复与失败现场只读归档

生产修复源码{producer}/tree{source['tree']}；代码分支最新{head}/tree{tree}仅后续只读归档脚本/C#与probe，未再改Rust。原首轮失败源c870bd81f保留，原Node50580与broker54692均不动，原Job不可从新进程接管；未重跑首轮/未恢复官方launcher。

把npm前缀与真实包目录分开：Owner统一给出prefix/node_modules/@bitkyc08/opencodex，四audit源与D角色CLI/sentinel/五类callsite绑定使用实际包目录；Bun仍prefix/node_modules/bun/bin/bun.exe。原所有固定SHA、角色/argv/rawExit断言不改。9定向测试raw0/0失败0忽略，真实固定包旧路径raw2反例复现，正确四原生文件pin/ID/8投影/原摘要与五callsite摘要读回通过，错hash/prefix/sentinel及原role/query拒绝保持。publicStart/共享HMAC/四trait/lease/normal/supervisor/Closed与完整recovery_state_at/Status接收尾段原样。custom-protocol MSVCdebug产品raw0/{load(ROOT/'product01/result.json')['elapsedMs']}ms，{len(inputs)}产品输入/{len(rust)}Rust对producer Git逐字核（CRLF明确等价）；当前archive脚本独立6251改动未计作产品重编，差额明确。

只读归档首r1在未初始化Node的MainModule路径处失败，files/dirs均空，原日志/脚本留。新kernel helper实际只读原进程FILETIME/Win32镜像/session/退出状态，无terminate/resume/Job/Owner构造。controller自有挂起cmd验证原模块枚举返回null；首probe因Windows目录大小写DATA比较失败，原错误留，修正常Windows同路径拼写比较后probe02通过。两probe均从未Resume，均由本次原创建handle受控forced242清理且原handle显式关闭，不冒NormalStop，不动marenben/旧40588。新r2归档入口已准备，尚待原桌面运行，仍无进程终止/删目录或改策略。

两新source-only scene均私有ZIP完整成员/hash核；递归删除被自动审批以blocked by policy拒绝，未执行且不换方法重试，两源留。完整marenben失败场景ZIP尚待；ZIP不是NTFS元数据克隆。现loaded首轮宿主无后续控制接口，不能外部PID接管成旧Owner或正常Stop；完整归档后仍须明确受控清理与新独立测试方案，公开Start/allAccepted/生产/typedClosed保持false，其余原包副作用/初始化/ownerreceipt/原缺席Codex安全退根/全容量/实际GUI官方owner/普通MSI/CAP-trust/历史清理未完。

无官方包/日常配置/PATH/策略修改、真实业务/模型、旧raw5/空GUI/W2/双SID/发布。

| 制品（producer3afd4e59d） | SHA256 |
| --- | --- |
'''
for a in artifacts:report+=f"| {a['name']} | {a['sha256']} |\n"
with (ROOT/'report.md').open('x',encoding='utf-8') as f:f.write(report)
p=DOCS/'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md';heading='## 2026-11-17 r6真实包目录修复及归档内核观察'
assert heading not in p.read_text(encoding='utf-8-sig')
with p.open('a',encoding='utf-8') as f:f.write(f'\n\n{heading}\n\n生产修复3afd4e59dc61c01a9d63053b498a222b91597634/tree{source["tree"]}、后续archive脚本625101e7bfa9dd60e5753361ebe9dce9561969d8已推送，证据.adg/evidence/0.2.24/windows-dev/{ROOT.name}/。统一Owner实际包目录prefix/PACKAGE_SUBPATH用于audit4与D角色CLI/sentinel/callsite，Bun仍prefix/node_modules/bun/bin/bun.exe，全部固定SHA/原roles/argv/rawcodes不改。实际固定包旧prefix raw2复现，正确包目录原生pin/ID/8/摘要及五角色source摘要通过；9定向raw0/0失败0忽略。共享HMAC/trait/lease/native退出/监督/Closed和完整recovery_state_at/Status尾段保持；正确custom-protocol产品MSVCdebug raw0/{load(ROOT/"product01/result.json")["elapsedMs"]}ms，{len(inputs)}输入/{len(rust)}Rust核producer Git，archive独立脚本差额明列，不冒重编。\n\n首归档r1 .MainModule未初始化路径失败，尚无文件/目录归档；原错误/脚本保留。kernel query-only helper无terminate/resume/Job/Owner，实际controller自有挂起cmd证明FILETIME/镜像/session可核且MainModulenull。首probe仅Windows目录大小写DATA比较失败，原错留；同Windows路径比较修后probe02过，两次从未Resume且只用自己原创建handleforced242关闭，不冒NormalStop/普通验收、不动marenben/40588。r2只读归档入口C:/Users/Public/ocx-r6-failure-archive-20261117-r2/archive-r6-failed-preresume.cmd待原桌面运行。\n\n两新source-only scene私有ZIP/hash核后，删除请求被自动审批以blocked by policy拒绝，命令未执行、两源与归档留，不换方式重试。marenben首失败broker54692/原挂起Node50580及全scene继续留，loaded宿主无控制通道，新程序不能接管旧Job/Owner；全归档后仍需明确受控清理/新独立夹具方案，不直接重试或签Closed。Start/allAccepted/production/typedClosedfalse，全部原初始化/原缺席安全退根/GUI官方owner/容量/普通MSI/CAP-trust/历史清理未完保持，无官方/日常/PATH/策略/业务/模型/旧raw5/空GUI/W2/双SID/发布。\n')
print(json.dumps(dict(head=head,productProducer=producer,passed=9,productInputs=len(inputs),testRust=len(rust),marenbenArchivePending=True,cleanupDeletionRejected=True,actualProductionExecuted=False)))
