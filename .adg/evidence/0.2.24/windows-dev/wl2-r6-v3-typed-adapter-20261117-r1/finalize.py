from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code');BASE=ROOT.parent/'wl2-r6-live-ready-read-20261117-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));sha=lambda b:hashlib.sha256(b).hexdigest();git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(n,v):
    with (ROOT/n).open('x',encoding='utf-8') as f:json.dump(v,f,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip();assert load(ROOT/'source-commit.json')['head']==head
runs={};scenes=[];records=[];inputs=[];rust=[]
for name in ['library01','product01']:
    r=load(ROOT/name/'result.json');assert r['exitCode']==0;runs[name]=dict(rawExit=0,profile=r['profile'],elapsedMs=r['elapsedMs'])
    if name=='library01':
        log=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig');m=re.search(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored',log);assert m;runs[name].update(passed=int(m[1]),failed=0,ignored=0,artifact=load(ROOT/'library01-artifact.json'))
        for i,s in enumerate(load(ROOT/'library01-private-scenes.json')['scenes']):
            assert sha(Path(s['archive']).read_bytes())==s['archiveSha256'];scenes.append(s)
            with zipfile.ZipFile(s['archive']) as z:
                for row in s['files']:
                    if s['source'].startswith('D:') or any(x in '/'+row['path'] for x in ['/.ocxd024-observed-','/.ocxd024-history-']):
                        raw=z.read(row['path']);assert len(raw)==row['bytes'] and sha(raw)==row['sha256'];p=ROOT/'raw-scene-records'/name/f'scene-{i:03}'/row['path'];p.parent.mkdir(parents=True,exist_ok=True)
                        with p.open('xb') as f:f.write(raw)
                        records.append(dict(source=str(Path(s['source'])/row['path']),evidence=p.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
    for row in load(ROOT/name/'source-manifest.json'):
        raw=(ROOT/name/'executed-source'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256'];assert raw==(CODE/row['path']).read_bytes();assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n')
        if name=='product01':inputs.append(row)
        elif row['path'].endswith('.rs'):rust.append(row)
assert load(ROOT/'product01/source.json')['head']==head
for row in load(ROOT/'source-scope.json')['changedFiles']:
    raw=(CODE/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
artifacts=[]
for n in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{n}.exe';raw=p.read_bytes();artifacts.append(dict(name=n,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
write('product01-artifacts.json',artifacts);write('raw-scene-records.json',records);write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,testRust=rust,gitLineEndingsExplicitlyEquivalent=True,preexistingSchemaNotManuallyEditedOrStaged=True))
matrix=load(BASE/'final-status.json')['matrix'];matrix.append(dict(id='PublicTypedFirstGenerationV3Dispatch',result='compiled-public-wiring/directed-locator-and-existing-native-components-pass/actual-production-unexecuted',fact='Root WindowsOriginals export now exclusively new v3 adapter; no v2 fallback/migration. Nonserde/nonClone phase tags locate original single retained CaptureSlot by private UUID/actual Owner.scope and exact stage; fail poisoned. Capture/Ready/readonly repeated Ready/preStop/restore+route+finalClosed use existing actual factories. PreStop takes SAME original nonClone ServiceObservation from Owner and installs it before fallible native prepare, exact original action and official accepted tuple preserved. Closed tag issued only after actual internal full finalStop and original cutoff read. Owner stop receives original ActionWindow, rawStatus keeps deterministic end-5s start, new actions keep original window. Successor explicitly REFUSES pending exact v3 predecessor/native namespace protocol; public Start gate unchanged. No actual Owner/Closed factory positive has executed.'))
requirements=load(BASE/'next-closed-requirements.json')['requirements']
for r in requirements:
    if r['id']=='TypedAdapterAndLiveReadyReads':r['requirement']='Public first-generation ClientOriginals now exclusively dispatches to retained v3 capture/Ready/preStop/completeClosed, actual nonClone observation and private stage/source locator. Native full custody/read/route/final factories registered, no v2 fallback; public Status/Recovery invokes typed same-original-deadline checks. Actual production Owner/Closed/Status positive remains unexecuted, phase/current fault/budget/sideeffect proof still needed. Successor currently explicitly refuses until exact v3 predecessor set/native namespace references and retained old Owner histories integrated; do not simply relax empty namespace.'
write('next-closed-requirements.json',dict(requirements=requirements,startOpen=False,actualProductionExecuted=False,typedClosedQualified=False,productionDispatcherUsesV3=True,successorRefused=True))
cleanup=dict(scenes=len(scenes),removed=0,failedAndPendingRetained=len(scenes),crossVolumePendingRootsRetained=sum(s['source'].startswith('D:') for s in scenes),deletionAttempted=False,historicalRejectedDeletionRetried=False,overallCleanupComplete=False,zipIsNTFSMetadataClone=False,reason='All directed native custody/canonical scenes retained after private archive/hash checks, no new ordinary login/official service run/cleanup bypass.')
write('cleanup.json',cleanup)
status=dict(time=datetime.now().astimezone().isoformat(),contract='TEAM16/IMP142/r6',sourceHead=head,sourceTree=tree,inheritedCode='14b3198a8cfe0ce92c4db405def22aa7725f7e6c',inheritedDocs='4aea903c368ff65e2b8203585393c38d8487433c',fetchedMain='fac8573e55441fe514919c63c7f1d65181ff3577',fetchedGovernanceMain='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,artifacts=artifacts,matrix=matrix,limits=requirements,
 firstError=dict(currentProductOrTestError=None,toolError='Initial apply_patch matched pre-rustfmt timeout line and refused before edits; reread actual multiline code, applied exact narrower patch. No native retry/permission bypass.',inheritedLatest=load(BASE/'final-status.json')['firstError']),cleanup=cleanup,
 allAccepted=False,startOpen=False,typedClosedQualified=False,actualProductionExecuted=False,productionDispatcherUsesV3=True,productionStatusSampleRegistered=True,productionStatusSampleJoined=False,actualProductionStatusPositive=False,successorAccepted=False,W2=False,dualSID=False,published=False)
write('final-status.json',status)
report=['# r6首代公共typed生命周期分派接入v3','',f'源码{head}/tree{tree}；承接14b3198a8/4aea903c3，TEAM16/IMP142/r6不变。仅7Windows允许文件。',
 '',f'library01 MSVCdebug raw0/{runs["library01"]["passed"]}过0败0忽略，{runs["library01"]["elapsedMs"]}ms；产品custom-protocol bins raw0/{runs["product01"]["elapsedMs"]}ms。本轮无非预期产品/测试失败；prepare patch初格式marker不匹配写前拒绝，原工具记录保留。',
 '', 'WindowsOriginals根导出切到唯一新v3 adapter，私有nonClone/nonserde tag只定位当前slot/UUID/原Owner.scope/准确阶段，失败poison；原v2只保历史源码，不fallback/转换。capture/Ready/重复Ready完整readonly/preStop/完整Stop route finalClosed接已有actual工厂，原Source/Job/typedHMAC/normal/supervisor/内容/claim/锁与预算谓词未删。preStop取原Owner真实同一个typed ServiceObservation装adapter后才fallible prepare；complete接受tuple必须同该对象代次。',
 '', '原Stop窗口内完成重读精确等于Store.stop_deadline和Journal.original end；Status frozen end-only接口按end-5s定位确定最早样本起点，不创建now+5。新Start/Stop/Restart读调用者真实原ActionWindow，Stop签名携原window，不把旧截止延长。共享trait/HMAC及完整recovery_state_at/StatusReceipt/同次receiver尾段逐字保持。typedReady来源、原恢复/关闭等14原文件函数核对通过。',
 '', '新增stage/ID/scope/failed纯DATA反例只比locator，不构造fakeOwner/Closedtag或原生permit，实际public capture/Ready/Closed/Status生产正例仍未执行。后继当前明确拒绝，必须先接准确v3 predecessor集合/namespace历史/原Owner与新capture引用，不放宽空namespace断言或旧文本采用。公开Start仍无条件拒绝，原包副作用/初始化/ownerreceipt/原缺席Codex安全退根/最大容量/普通MSI/CAP-trust/历史清理保持未完。',
 '',f'{len(scenes)}scene私有ZIP/hash/membership核验后全部保留，{len(records)}份canonical/快照/history另收存，ZIP非NTFS克隆，无删除/历史拒绝重试。产品{len(inputs)}输入/{len(rust)}Rust逐字核工作/Git（CRLF明确等价），预存schema未手改/暂存。未改官方/日常/PATH/账户/策略，未业务/模型，未旧raw5/空GUI/W2/双SID/发布。',
 '', '| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
with (ROOT/'report.md').open('x',encoding='utf-8') as f:f.write('\n'.join(report)+'\n')
print(json.dumps(dict(head=head,passed=runs['library01']['passed'],scenes=len(scenes),rawRecords=len(records),productInputs=len(inputs),testRust=len(rust),v3PublicDefault=True,actualFactoryExecuted=False)))
