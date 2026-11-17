from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code');BASE=ROOT.parent/'wl2-r6-typed-ready-receipt-20261117-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));sha=lambda b:hashlib.sha256(b).hexdigest();git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(n,v):
    with (ROOT/n).open('x',encoding='utf-8') as f:json.dump(v,f,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip();assert load(ROOT/'source-commit.json')['head']==head
runs={};scenes=[];records=[];traces=[];inputs=[];rust=[];drifts=[]
for name,rc in [('library01',101),('library02',101),('library03',101),('library04',0),('product01',0)]:
    r=load(ROOT/name/'result.json');assert r['exitCode']==rc;runs[name]=dict(rawExit=rc,profile=r['profile'],elapsedMs=r['elapsedMs'])
    if name.startswith('library'):
        log=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig');m=re.search(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',log)
        if name=='library01':assert m is None
        elif name=='library02':assert m and tuple(map(int,m.groups()))==(53,1,0)
        elif name=='library03':assert m and tuple(map(int,m.groups()))==(54,1,0)
        else:assert m and int(m[2])==0 and int(m[3])==0
        if m:runs[name].update(passed=int(m[1]),failed=int(m[2]),ignored=int(m[3]))
        runs[name]['artifact']=load(ROOT/f'{name}-artifact.json')
        for line in log.splitlines():
            if 'r6 ' in line and len(line)<2400:traces.append(dict(run=name,trace=line[line.index('r6 '):]))
        for i,s in enumerate(load(ROOT/f'{name}-private-scenes.json')['scenes']):
            assert sha(Path(s['archive']).read_bytes())==s['archiveSha256'];scenes.append(s)
            with zipfile.ZipFile(s['archive']) as z:
                for row in s['files']:
                    if s['source'].startswith('D:') or any(x in '/'+row['path'] for x in ['/.ocxd024-observed-','/.ocxd024-history-']):
                        raw=z.read(row['path']);assert len(raw)==row['bytes'] and sha(raw)==row['sha256'];p=ROOT/'raw-scene-records'/name/f'scene-{i:03}'/row['path'];p.parent.mkdir(parents=True,exist_ok=True)
                        with p.open('xb') as f:f.write(raw)
                        records.append(dict(source=str(Path(s['source'])/row['path']),evidence=p.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
    for row in load(ROOT/name/'source-manifest.json'):
        raw=(ROOT/name/'executed-source'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
        if name in ['library01','library02','library03']:
            if raw!=(CODE/row['path']).read_bytes():drifts.append(dict(run=name,path=row['path'],reason='private-test-field then protected fixture/correct ID-isolated replacement/expired-read added; original executed source preserved'))
            continue
        assert raw==(CODE/row['path']).read_bytes();assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n')
        if name=='product01':inputs.append(row)
        elif row['path'].endswith('.rs'):rust.append(row)
assert load(ROOT/'product01/source.json')['head']==head
for row in load(ROOT/'source-scope-verified.json')['changedFiles']:
    raw=(CODE/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
failed=load(ROOT/'library02-failed-executable.json');assert sha(Path(failed['retained']).read_bytes())==failed['sha256']
failed=load(ROOT/'library03-failed-executable.json');assert sha(Path(failed['retained']).read_bytes())==failed['sha256']
artifacts=[]
for n in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{n}.exe';raw=p.read_bytes();artifacts.append(dict(name=n,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
write('product01-artifacts.json',artifacts);write('raw-scene-records.json',records);write('directed-traces.json',traces);write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,testRust=rust,earlierRunDifferences=drifts,gitLineEndingsExplicitlyEquivalent=True,preexistingSchemaNotManuallyEditedOrStaged=True))
matrix=load(BASE/'final-status.json')['matrix'];matrix.append(dict(id='ActualLiveReadyReadLane',result='native-read-component-pass/actual-private-typed-gate-compiled-not-executed',fact='Fresh actual Source/retained NativeAction origin and actual ServiceObservation plus canonical uniqueReady/Released/Open/no pending/noStop/noqualification gate admits read only. CapturedTrees keeps BOTH complete read views until original custody/anchors/current nonartifact/all8/children and typed Source/Ready final barrier checked; no DELETE rights, claim/lease/canonical mutation/new snapshot. Failed partial views retained and poisoned. Exactly3catalog leaves have file-shape DATA allowance, never artifact policy/restore qualification; Grok/nonartifacts exact full work generation. Actual old native cutoff expiry read succeeds<=5s, original journal mutation TimedOut still; invalidData nonartifact edit/extra dirs/wrong-kind catalog/Grok/same-bytes differentID negatives. Actual Owner/private gate and public Status/dispatcher remain unexecuted.'))
requirements=load(BASE/'next-closed-requirements.json')['requirements']
for r in requirements:
    if r['id']=='TypedAdapterAndLiveReadyReads':r['requirement']='Internal actual liveReady read gate and complete readonly BOTH-tree/native nonartifact baseline checks now implemented/compiled and native storage positives/negatives tested; old expiry remains mutation Timeout, no DELETE/claim/newphase/snapshot. Actual ServiceObservation/Owner/source factory execution not yet tested. Wire typed v3 capture/Ready/preStop/Closed/public Status and original successor history, then ordinary fixedofficial production. Current marenben login still pending, no controller/admin substitution.'
write('next-closed-requirements.json',dict(requirements=requirements,startOpen=False,actualProductionExecuted=False,typedClosedQualified=False))
cleanup=dict(scenes=len(scenes),removed=0,failedAndPendingRetained=len(scenes),crossVolumePendingRootsRetained=sum(s['source'].startswith('D:') for s in scenes),deletionAttempted=False,historicalRejectedDeletionRetried=False,overallCleanupComplete=False,zipIsNTFSMetadataClone=False,reason='All native custody/failed partial-view/canonical scenes pending full actual Closed; keep scenes and first failed EXE after archive/hash validation. No account/Profile/PATH/policy mutation.')
write('cleanup.json',cleanup)
first=dict(run='library01',rawExit=101,kind='MSVC E0616 new test accessed private WorkFrame.archive, no executable/test attributed',correction='Keep privacy, test actual parent membership before/after instead.',later=[dict(run='library02',rawExit=101,passed=53,failed=1,kind='new positive artifact std::fs::write inherited unprotectedDACL; original private-file check correctly refused',correction='Create synthetic positive via PrivateRecord; isolate replacement nativeID and require InvalidData; add old expired cutoff read/unchanged mutationTimeout.'),dict(run='library03',rawExit=101,passed=54,failed=1,kind='actual temp read view reused shareWrite/delete observer; raw32 write-probe assertion got Ok',correction='Keep original denial assertion; new temporary ordinary file read requests READ_DATA/READ_CONTROL/noDELETE with shareRead only; directory/coord sharing preserved. Old open_node/claim/mutation code unchanged, no retry. Accurate failed source/EXE/scenes retained.')])
status=dict(time=datetime.now().astimezone().isoformat(),contract='TEAM16/IMP142/r6',sourceHead=head,sourceTree=tree,inheritedCode='7c0bc016b4daedada6fad3f2a0884ae40e7b7fc3',inheritedDocs='d75d9a8f750d80335ab408abd0774cb1c46d44b7',fetchedMain='fac8573e55441fe514919c63c7f1d65181ff3577',fetchedGovernanceMain='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,artifacts=artifacts,matrix=matrix,limits=requirements,firstError=first,cleanup=cleanup,allAccepted=False,startOpen=False,typedClosedQualified=False,actualProductionExecuted=False,actualTypedReadyReadFactoryExecuted=False,productionDispatcherUsesV3=False,productionStatusSampleJoined=False,W2=False,dualSID=False,published=False)
write('final-status.json',status)
report=['# r6原Ready完整双树只读校验','',f'源码{head}/tree{tree}；承接7c0bc016b/d75d9a8f7，TEAM16/IMP142/r6及新fetch共同源不变。','', '| 运行 | raw | 过/败/忽略 | ms |','| --- | --- | --- | --- |']
for n,r in runs.items():report.append(f"| {n} | {r['rawExit']} | {r.get('passed','—')}/{r.get('failed','—')}/{r.get('ignored','—')} | {r['elapsedMs']} |")
report+=['','首library01 E0616/raw101无测试/新EXE：测试触私有WorkFrame.archive，保privacy改真实父目录membership不增快照断言。library02 raw101/53过1败：正例新artifact普通write继承未protectedDACL被原private文件检查拒绝，只修合成创建；samebytes换对象反例移保存文件到根外，PrivateRecord新原路径，membership/bytes相同仅ID不同，要求InvalidData而非上游权限/超时。原失败source/EXE/88scene保留。',
 '', 'library03 raw101/54过1败：临时view复用原observe模式，sharing允许写入，原raw32断言遇Ok；保留断言并改专用无DELETE读data/control普通文件shareRead，目录/协调文件sharing不变。准确source/EXE/90scene保留；不通过删拒写断言改变目标。',
 '', 'Actual内部Ready只读gate需fresh原Source/retainedNativeAction身份/typed ServiceObservation/canonicalReady/Released/Open与无pendingStop/content资格；不新动作、不延旧变更截止。完整BOTH-root read views只读无DELETE，原件custody/anchors/全部非产物work ID/bytes/8/children/Grok严格核，source/typedReady前后核毕才释放临时视图；错误保留准确部分视图与poison。无claim/lease重取、journal事件或snapshot目录创建。三已声明catalog叶仅允许文件形状DATA差异，目录同名拒绝，不凭它签policy/restore。',
 '', '原生测试完整两树持有时config写raw32、claim仍可取、journal序列不变；config edit/unknown emptydir/artifactdir/Grok edit/samebytes differentID均InvalidData，原custody字节保持。实际旧fixture10秒截止后BOTH-tree读取<=5秒可通过，原journal变更check仍TimedOut，准确序列不变。不是actual普通Owner/typedReady/privatefactory正例；公共Status/typed adapter/后继和真实原包仍未接。',
 '',f'{len(scenes)}scene私有ZIP/hash/membership核验后全部留，{len(records)}份canonical/快照/history另收存，ZIP非NTFS克隆，不重试历史拒绝。本轮不新建账号/Profile/PATH/策略或改官方日常；{len(inputs)}产品输入/{len(rust)}Rust核工作/Git（CRLF明确等价），预存schema未手改/暂存。14处原变更/完整Stop/typed传输保持。Start拒绝，原生产副作用/初始化/ownerreceipt/原缺席退根/最大容量/普通MSI/CAP-trust/历史清理未完，不旧raw5/空GUI/W2/双SID/发布。',
 '', '| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
with (ROOT/'report.md').open('x',encoding='utf-8') as f:f.write('\n'.join(report)+'\n')
print(json.dumps(dict(head=head,passed=runs['library04']['passed'],scenes=len(scenes),rawRecords=len(records),productInputs=len(inputs),testRust=len(rust),actualFactoryExecuted=False)))
