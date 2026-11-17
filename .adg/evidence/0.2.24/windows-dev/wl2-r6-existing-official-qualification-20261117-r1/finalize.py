from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code');DOCS=ROOT.parents[4]
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));sha=lambda b:hashlib.sha256(b).hexdigest();git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(n,v):
    with (ROOT/n).open('x',encoding='utf-8') as f:json.dump(v,f,ensure_ascii=False,indent=2)
source=load(ROOT/'source-commit.json');head=source['head'];assert git('rev-parse','HEAD').decode().strip()==head
remote=git('ls-remote','origin','refs/heads/codex/024-windows-native').decode().split()[0];assert remote==head
write('source-push.json',dict(head=head,remoteHead=remote,verified=True))
runs={};inputs=[];rust=[]
for name in ['library01','library02','library03','product01']:
    r=load(ROOT/name/'result.json');runs[name]=r
    assert r['exitCode']==(101 if name=='library01' else 0)
    for row in load(ROOT/name/'source-manifest.json'):
        raw=(ROOT/name/'executed-source'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
        if name in ['library03','product01']:
            assert raw==(CODE/row['path']).read_bytes()
            assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n')
        if name=='product01':inputs.append(row)
        if name=='library03' and row['path'].endswith('.rs'):rust.append(row)
assert load(ROOT/'product01/source.json')['head']==head and '--features' in runs['product01']['arguments']
stage=load(ROOT/'staged-input.json');public=Path(stage['public'])
for row in stage['files']:
    raw=(public/'input'/row['name']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
for row in stage['scripts']:
    raw=Path(row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
assert not (public/'out01/launch-result.json').exists()
artifacts=[dict(**r,sourceHead=head,sourceTree=source['tree'],features=['custom-protocol'],profile='debug') for r in stage['files'] if r['name'].endswith('.exe')]
write('product01-artifacts.json',artifacts);write('final-source.json',dict(head=head,tree=source['tree'],productInputs=inputs,testRust=rust,preserved=source['preserved'],gitCRLFExplicitlyEquivalent=True,untrackedSchemaNotManuallyEditedOrStaged=True))
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
base=load(ROOT.parent/'wl2-r6-ordinary-profile-preflight-20261117-r1/final-status.json')
base['sourceHead']=head;base['sourceTree']=source['tree'];base['productionSourceHead']=head;base['productionSourceTree']=source['tree'];base['artifacts']=artifacts;base['runsThisRound']=runs;base['actualProductionExecuted']=False;base['startOpen']=False;base['allAccepted']=False
base['ordinaryExistingBaselinePrepared']=True;base['ordinaryBaselineReceipt']='../wl2-r6-ordinary-profile-preflight-20261117-receive-r1/result.json'
base['productionArtifactReference']='product01-artifacts.json'
for row in base['matrix']:
    if row['id']=='OrdinaryDesktopExistingBaselinePreparation':
        row['result']='actual-original-ordinary-desktop-pass/raw0/synthetic-baseline-only'
        row['fact']='Received actual SID1003/session7/elevationType1/noJob/KnownFolder Profile and10native object fixed8 setup results.20intent/result plus prior Absent roots explicitly retained. DailyPath unchanged; no official initialization/NativeLease/Closed authority. See ../wl2-r6-ordinary-profile-preflight-20261117-receive-r1/result.json'
base['preparationOrdinaryDesktopIgnoredPendingUser']=False;base['preparationDirectedDataPassed']=1;base['qualificationDesktopPending']=True;base['desktopLauncher']=str(public/'launch-r6-official-existing.cmd')
base['matrix'].append(dict(id='ExactExistingOfficialQualificationEntry',result='compiled/debug-native-admission-staged/pending-ordinary-desktop-execution',fact='Public execute_async still always calls unconditional require_durable_capture; private execute_admitted factoring preserves original full Source/LaunchPlan/Token/Lease/custody/content/typedReady/NormalExit/SupervisorAbsent/finalClosed sequence. Exact debug native qualification can only run via existing one-shot inherited pipe bootstrap in actual I/ocxd.exe under original ordinary SID1003/session7/noJob/KnownFolder/task I-D-H. Rechecks exact10 prepared original objects/IDs/all8/bytes; fixed bundle held denywrite while hash+regular canonical extraction, shared program_image/selection/entry admission reused. Missing synthetic N/L/K created in setup ledger with prior absence separate, never official initialization. Empty-provider/noAutoRefresh config forbids business/model requests. Actual broker retains original source images/Owner/Job and failures indefinitely; no forced cleanup/native proof from DATA. Planned Start/Status/Stop/successor/Restart actions use original budgets; actual execution still pending. Real GUI official Owner/manager handoff and firstAbsent remain unproven.'))
base['firstErrorThisRound']=dict(nativeCompile='library01 E0596 immutable Owner diagnostic borrow passed to native family_state requiring mutable original retained Owner; raw101/no tests/no new EXE. Fixed same original guard mutable, library02 and03 pass.',nativeTest=None,receiver='Previous ordinary baseline receiver direct private-file PermissionError preserved in receive-r1; same-SID hash export used without ACL change or native rerun.',tool='Patch initially mixed already-fixed rustfmt context and refused; source reread and narrower correct patch applied. One JS string parser failure before command invocation; no native state affected.')
base['cleanupThisRound']=dict(ordinaryBaselineRetained=True,publicInputRetained=True,newNativeSceneCount=0,newOfficialOwnerCount=0,officialServiceStarted=False,historicalRejectedDeletionRetried=False,overallCleanupComplete=False)
base['timeThisRound']=datetime.now().astimezone().isoformat()
write('final-status.json',base)
for n,args in [('public-acl',['C:/Windows/System32/icacls.exe',str(public)]),('session',['quser'])]:
    r=subprocess.run(args,capture_output=True);(ROOT/f'{n}.stdout.log').write_bytes(r.stdout);(ROOT/f'{n}.stderr.log').write_bytes(r.stderr);write(f'{n}.json',dict(arguments=args,rawExit=r.returncode));assert r.returncode==0
report=f'''# r6实际原包existing-root内部验证入口

源码{head}/tree{source['tree']}，仅11Windows允许文件。公共execute_async始终传入原require_durable_capture拒绝；私有debug限定通道复用同一execute_admitted原完整Source/LaunchPlan/Lease/原件custody/content/typedReady/Stop/退出/监督/finalClosed路径，无fakeOwner、序列化proof、环境bool或公共Start放行。release不编入资格入口。

原普通marenben基线已在846a68007真实接收，当前精确SID1003/session7/KnownFolder/noJob/原I-D-H及10原对象path/kind/ID/all8/bytes重核。固定原包bundle同一拒写native句柄hash再解包，普通文件/路径/完整有界集严格检查，独立private完整原字节复制不改官方源；shared program_image/selection/entry实现复用。setup缺席N/L/K仅create-new带意图/结果/原缺席记录，不当官方初始化。无provider/自动refresh关闭，禁止业务/模型请求。broker始终保留原镜像handle/Owner/Job/全部失败现场，未强杀或Drop补Stop。

library01首E0596/raw101无测试/新EXE；library02 10过，library03新增完整基线locator反例后11过/raw0/0失败0忽略。最终custom-protocol MSVCdebug产品raw0/{runs['product01']['elapsedMs']}ms，{len(inputs)}输入/{len(rust)}Rust逐字对工作/Git（CRLF明确等价）。共享trait/HMAC/来源/claim/typed退出/监督/closure等11完整源与完整recovery_state_at/Status接收尾段逐字保持。

已准备{public}/launch-r6-official-existing.cmd；实际普通桌面还未启动这个原包通道，不冒实际生产通过。入口计划真实Start1/Status同5/Stop1/Start2/Restart2/Stop3全部原预算；失败保留原broker与nativeOwner，不重复或改未知为通过。真正GUI官方Owner、manager交接、原包副作用及finalphase/current反例、最大容量、官方初始化/ownerreceipt、原缺席Codex安全退根、普通MSI/CAP-trust/历史清理仍未完。公开Start/allAccepted=false。无官方/日常配置/PATH/策略修改、业务/模型、旧raw5/空GUI/W2/双SID/发布。

| 制品 | SHA256 |
| --- | --- |
'''
for a in artifacts:report+=f"| {a['name']} | {a['sha256']} |\n"
with (ROOT/'report.md').open('x',encoding='utf-8') as f:f.write(report)
p=DOCS/'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md';heading='## 2026-11-17 r6真实原包existing-root内部验证入口'
assert heading not in p.read_text(encoding='utf-8-sig')
with p.open('a',encoding='utf-8') as f:f.write(f'\n\n{heading}\n\n源码{head}/tree{source["tree"]}已推送，仅11Windows允许文件。证据.adg/evidence/0.2.24/windows-dev/{ROOT.name}/。公共execute_async恒调用原durable拒绝，私有debug原生通道只接受原普通marenben/session7/noJob/KnownFolder/精确I-D-H及既有10对象path/kind/ID/8/bytes；通过原one-shot继承pipe进入实际I/ocxd.exe，非环境bool/JSON proof。固定bundle同一拒写handle hash+正规全文件解包，复用shared program_image/selection/entry；N/L/K初始化仅本fixture原缺席setup带intent/result，不是产品首次初始化。原来源/lease/custody/content/typedReady/normal/supervisor/Closed原断言和原预算全走，公共Start不放行。\n\n首library01 E0596/raw101未测/无EXE，实际family_state需同原Owner可变观察，修后library02 10过/library03 11过raw0；custom-protocol产品MSVCdebug raw0/{runs["product01"]["elapsedMs"]}ms，{len(inputs)}输入/{len(rust)}Rust逐字核工作/Git。共享四trait/HMAC/原native关闭及完整recovery_state_at/Status接收尾段保持。入口{public}/launch-r6-official-existing.cmd已备，尚需用户原桌面运行，实际生产仍false；不拿普通基线准备1过替代原包。成功/失败均保原broker/Owner/Job/source与scene，无强杀/Drop补0。\n\n实际GUI官方owner/manager切换/原包副作用/finalphase-current反例/全容量/初始化与ownerreceipt/原缺席Codex安全退根、普通MSI/CAP-trust及历史清理全部未完保持。基线/本Public输入留，未创建新nativeOwner或启动服务，未旧拒绝删除重试。Start/allAccepted/typedClosedfalse，无官方/日常/PATH/策略/业务/模型/旧raw5/空GUI/W2/双SID/发布。\n')
print(json.dumps(dict(head=head,artifacts=len(artifacts),productInputs=len(inputs),testRust=len(rust),ordinaryDesktopPending=True,actualProductionExecuted=False)))
