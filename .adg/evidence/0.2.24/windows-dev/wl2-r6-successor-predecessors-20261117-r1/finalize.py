from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=ROOT.parent/'wl2-r6-v3-typed-adapter-20261117-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(n,v):
    with (ROOT/n).open('x',encoding='utf-8') as f:json.dump(v,f,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
assert head==load(ROOT/'source-commit.json')['head']
remote=git('ls-remote','origin','refs/heads/codex/024-windows-native').decode().split()[0];assert remote==head
write('source-push.json',dict(branch='codex/024-windows-native',head=head,remoteHead=remote,verified=True))
runs={};scenes=[];records=[];inputs=[];rust=[]
cleanup=load(ROOT/'data-scene-cleanup.json')
removed={x['source'] for x in cleanup['targets'] if x['removed']};assert len(removed)==6
for name in ['library01','library02','library03','product01','product02']:
    r=load(ROOT/name/'result.json');assert r['exitCode']==0
    runs[name]=dict(rawExit=r['exitCode'],elapsedMs=r['elapsedMs'],profile=r['profile'],arguments=r['arguments'])
    if name.startswith('library'):
        log=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig')
        m=re.search(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored',log);assert m
        runs[name].update(passed=int(m[1]),failed=0,ignored=0,artifact=load(ROOT/f'{name}-artifact.json'))
        for i,s in enumerate(load(ROOT/f'{name}-private-scenes.json')['scenes']):
            assert sha(Path(s['archive']).read_bytes())==s['archiveSha256']
            s['removedAfterVerifiedCompletedDataTest']=s['source'] in removed
            if s['source'] in removed:assert not Path(s['source']).exists()
            else:assert Path(s['source']).exists()
            scenes.append(s)
            with zipfile.ZipFile(s['archive']) as z:
                assert sorted(z.namelist())==sorted(x['path'] for x in s['files'])
                for row in s['files']:
                    raw=z.read(row['path']);assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
                    if s['source'].startswith('D:') or any(x in '/'+row['path'] for x in ['/.ocxd024-observed-','/.ocxd024-history-']):
                        p=ROOT/'raw-scene-records'/name/f'scene-{i:03}'/row['path'];p.parent.mkdir(parents=True,exist_ok=True)
                        with p.open('xb') as f:f.write(raw)
                        records.append(dict(source=str(Path(s['source'])/row['path']),evidence=p.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
    for row in load(ROOT/name/'source-manifest.json'):
        raw=(ROOT/name/'executed-source'/row['path']).read_bytes()
        assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
        if name in ['library03','product01','product02']:
            assert raw==(CODE/row['path']).read_bytes()
            assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n')
        if name=='product02':inputs.append(row)
        if name=='library03' and row['path'].endswith('.rs'):rust.append(row)
assert '--features' not in runs['product01']['arguments']
assert runs['product02']['arguments'][runs['product02']['arguments'].index('--features')+1]=='custom-protocol'
assert load(ROOT/'product02/source.json')['head']==head
artifacts=[]
for n in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{n}.exe';raw=p.read_bytes()
    artifacts.append(dict(name=n,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
write('product02-artifacts.json',artifacts);write('raw-scene-records.json',records)
write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,testRust=rust,gitLineEndingsExplicitlyEquivalent=True,preexistingSchemaNotManuallyEditedOrStaged=True))
matrix=load(BASE/'final-status.json')['matrix']
matrix.append(dict(id='V3RetainedOrderedPredecessorsAndSuccessor',result='compiled-public-wiring/directed-data-native-namespace-and-action-windows-pass/actual-production-unexecuted',fact='Single immutable manifest now binds exact ordered prior native completed reference DATA and prefix digests. Private nonserde/nonClone NativePredecessors only derives from actual retained completed CaptureSlots and exact Owner.previous. Actual one-shot same-window ticket retains every old slot/Closed locator/typed observation/source scope; new capture checks original outer/Stop ends/source/ref digest/order. Every current stage boundary reads actual old histories and exact namespace; no adopting nonempty/oldv2/textClosed or reconstructing authority from DATA. New actual Owner positive remains unexecuted, public Start unchanged refusing.'))
requirements=load(BASE/'next-closed-requirements.json')['requirements']
for row in requirements:
    if row['id']=='TypedAdapterAndLiveReadyReads':row['requirement']='Public v3 first-generation and ordered retained successor adapters are wired and compile. Original action observations now borrow SAME caller ActionWindow; Status end-only receiver unchanged, Stop predecessor reads use original sub10 even while Restart outer30 remains. Actual ordinary fixedofficial capture/Ready/Closed/Status/successor positive, phase/current faults and latency remain unexecuted; private locator DATA/native namespace tests cannot substitute. Public Start remains refused.'
write('next-closed-requirements.json',dict(requirements=requirements,startOpen=False,actualProductionExecuted=False,actualSuccessorQualified=False))
clean=dict(scenes=len(scenes),removedCompletedDataScenes=6,retainedNativeScenes=len(scenes)-6,rawRecords=len(records),deletionAttempted=True,historicalRejectedDeletionRetried=False,overallCleanupComplete=False,zipIsNTFSMetadataClone=False,reason='Only6 completed synthetic predecessor locator DATA fixture roots removed after all-source/hash/membership/archive verification. Native custody/pending/canonical/counterexample scenes remain retained; no historical denied deletion retried.')
write('cleanup.json',clean)
first=dict(currentProductOrTestError=None,sourceInspectionFinding='Before production, Ready observations found shared between Status and actions. Corrected by passing original caller ActionWindow to action-only observe/check_ready, preserving original Status and full recovery receiver bytes.',executionSetupError='Product01 requested unknown build-custom mode; driver defaulted to normal bins without custom-protocol. Raw successful default build and exact args retained as nonqualifying product01; correct actual-gui-build mode product02 is the sole final custom-protocol artifact producer.',cleanupDisplayError='OrderedDictionary projected with Select-Object printed null preview fields; immutable JSON receipt contains6 exact sources/archives/removed=true/error=null, independently verified. No cleanup rerun.',inherited=load(BASE/'final-status.json')['firstError'])
status=dict(time=datetime.now().astimezone().isoformat(),contract='TEAM16/IMP142/r6',sourceHead=head,sourceTree=tree,inheritedCode='fb9a64ff79319383f015850d54903866bc6756d7',inheritedDocs='7e428aef7b3a89b851ad5e21ae7096b8cb9faa9b',fetchedMain='fac8573e55441fe514919c63c7f1d65181ff3577',fetchedGovernanceMain='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,artifacts=artifacts,matrix=matrix,limits=requirements,firstError=first,cleanup=clean,ordinaryDesktopSessionActive=True,ordinaryAccount='marenben',ordinarySession=7,allAccepted=False,startOpen=False,typedClosedQualified=False,actualProductionExecuted=False,productionDispatcherUsesV3=True,successorWired=True,actualSuccessorQualified=False,W2=False,dualSID=False,published=False)
write('final-status.json',status)
lines=['# r6保留完整前驱链与一次性后继接线','',f'源码{head}/tree{tree}；TEAM16/IMP142/r6；仅13Windows允许文件。',
f'三轮MSVC库分别{runs["library01"]["passed"]}/{runs["library02"]["passed"]}/{runs["library03"]["passed"]}过，全部raw0/0失败0忽略。首原语批与后继接线批实源分开保留；最终动作Ready窗口修正后63过。',
'', '单一manifest新增准确有序前驱DATA（原session/目录ID/8投影摘要/current ID/安全/原字节/final lineage/前缀摘要），NativePredecessors私有nonserde/nonClone工厂只从实际完成slot/Owner.previous读取。后继票据绑定原newActionWindow双截止、原来源/旧scope/ref摘要，所有旧槽、Closed定位、原typed观察和Owner保留。新capture重核同票据与完整namespace；阶段前后原deadline内核全历史/精确成员，不采用文本/旧v2或未知兄弟。',
'', '实际生产/真实原Owner/后继正例仍未执行，纯DATA与native locator目录不构造Closed或NativePredecessors。共享四trait/HMAC/来源/原生退出/监督/14完整源片段和完整recovery_state_at/Status接收尾段保持。原Start门无条件拒绝；原缺席根/官方初始化与副作用/ownerreceipt/全容量/实际H-S-D/普通MSI/CAP-trust及历史清理未完。',
'',f'product01模式名误写导致默认非custom构建，原参数与raw0保留不充目标产物；product02正确custom-protocol MSVCdebug bins raw0/{runs["product02"]["elapsedMs"]}ms。目标{len(inputs)}输入/{len(rust)}Rust核工作/Git（CRLF明确等价）。本轮无产品/测试首错，源检查发现的Ready Status/Action窗口混用在真实生产前修正并增加反例；所有继承首错保留。',
'',f'{len(scenes)}scene私有ZIP成员/原字节摘要核验，6纯DATA成功fixture清理，其余{len(scenes)-6}原生现场保留；{len(records)}份canonical/快照/history另收存。ZIP不克隆NTFS元数据，未重试任何旧删除拒绝。marenben session7已Active，实际普通原包入口仍待准备，无服务启动。',
'', '未改官方/日常配置/PATH/账户/策略，不业务/模型，不旧raw5/空GUI/W2/双SID/发布。',
'','| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:lines.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
with (ROOT/'report.md').open('x',encoding='utf-8') as f:f.write('\n'.join(lines)+'\n')
print(json.dumps(dict(head=head,passed=runs['library03']['passed'],scenes=len(scenes),removed=6,rawRecords=len(records),productInputs=len(inputs),testRust=len(rust),actualProductionExecuted=False,startOpen=False)))
