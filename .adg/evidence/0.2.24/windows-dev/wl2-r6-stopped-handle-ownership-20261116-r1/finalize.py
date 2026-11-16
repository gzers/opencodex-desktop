from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=ROOT.parent/'wl2-r6-content-canonical-20261116-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,v):
    with (ROOT/name).open('x',encoding='utf-8') as f:json.dump(v,f,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
assert load(ROOT/'source-commit.json')['head']==head
runs={};scenes=[];records=[];traces=[];rust=[];inputs=[]
for name,rc in [('library01',101),('library02',0),('product01',0)]:
    result=load(ROOT/name/'result.json');assert result['exitCode']==rc
    runs[name]=dict(rawExit=rc,elapsedMs=result['elapsedMs'],profile=result['profile'])
    if name.startswith('library'):
        log=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig')
        m=re.search(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',log)
        if name=='library01':assert m is None
        else:
            assert m and int(m[2])==0 and int(m[3])==0
            runs[name].update(passed=int(m[1]),failed=int(m[2]),ignored=int(m[3]))
        runs[name]['artifact']=load(ROOT/f'{name}-artifact.json')
        for line in log.splitlines():
            if 'r6 ' in line and len(line)<2400:traces.append(dict(run=name,trace=line[line.index('r6 '):]))
        for i,s in enumerate(load(ROOT/f'{name}-private-scenes.json')['scenes']):
            assert sha(Path(s['archive']).read_bytes())==s['archiveSha256'];scenes.append(s)
            with zipfile.ZipFile(s['archive']) as archive:
                for row in s['files']:
                    if s['source'].startswith('D:') or any(m in '/'+row['path'] for m in ['/.ocxd024-observed-','/.ocxd024-history-']):
                        raw=archive.read(row['path']);assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
                        p=ROOT/'raw-scene-records'/name/f'scene-{i:03}'/row['path'];p.parent.mkdir(parents=True,exist_ok=True)
                        with p.open('xb') as f:f.write(raw)
                        records.append(dict(evidence=p.relative_to(ROOT).as_posix(),source=str(Path(s['source'])/row['path']),bytes=len(raw),sha256=sha(raw)))
    for row in load(ROOT/name/'source-manifest.json'):
        raw=(ROOT/name/'executed-source'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
        if name=='product01' or name=='library02' and row['path'].endswith('.rs'):
            assert raw==(CODE/row['path']).read_bytes(),row['path']
            assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n'),row['path']
            (inputs if name=='product01' else rust).append(row)
assert load(ROOT/'product01/source.json')['head']==head
for row in load(ROOT/'source-scope-final.json')['changedFiles']:
    raw=(CODE/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
artifacts=[]
for name in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe';raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
write('product01-artifacts.json',artifacts);write('raw-scene-records.json',records);write('directed-traces.json',traces)
write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,testRust=rust,gitLineEndingsExplicitlyEquivalent=True,preexistingSchemaNotEditedOrStaged=True))
matrix=load(BASE/'final-status.json')['matrix'];matrix.append(dict(id='ActualStoppedOriginalHandleOwnership',result='component-ownership-pass/actual-guard-driver-compiled-not-executed',fact='CapturedTrees keeps Option of exact2trees; actual original Stop restoration/history then journal retains owned trees before any fallible close guard. Original unchanged close driver uses native retained inventory and timely native completion; actual Stop/source/content/normal/supervisor/current guards before and after. Missing local slot rejects later mutations; poison propagates into actual handoff. Actual original files deny writes raw32 after transfer/refused mutation. Duplicate transfer retains both exact object sets and no close event. No typedClosed or production acceptance.'))
requirements=load(BASE/'next-closed-requirements.json')['requirements']
for r in requirements:
    if r['id']=='OwnershipClosure':r['requirement']='Original owned2tree transfer to retained journal and original native handle close driver are now joined to actual private Stop restore/history path; new native ownership/duplicate/missing-slot counterexamples pass. Actual production execution and full exact production inventory roles/counts remain unproved; process/supervisor completed histories and typed Closed/Status/dispatcher remain unjoined. No Drop success or serialized native permit.'
write('next-closed-requirements.json',dict(requirements=requirements,startOpen=False,typedClosedQualified=False,actualProductionExecuted=False))
cleanup=dict(scenes=len(scenes),removed=0,failedAndPendingRetained=len(scenes),crossVolumePendingRootsRetained=sum(s['source'].startswith('D:') for s in scenes),deletionAttempted=False,historicalRejectedDeletionRetried=False,overallCleanupComplete=False,zipIsNTFSMetadataClone=False,reason='All produced native tree/canonical scenes are pending or incomplete without qualified Closed; preserve exact original scenes after verified private content archive. Compile-failure round produced no scenes.')
write('cleanup.json',cleanup)
first=dict(run='library01',rawExit=101,testsExecuted=False,newExecutableAttributed=False,errors=['E0609 test Option iterator','E0499 tree borrow before check_working in open_prepared_working','E0499 tree borrow before check_working in release_prepared'],correction='Unwrap retained test tree slot; borrow field only after existing full check_working. Original assertions/deadlines/native close body unchanged. Source/log/result preserved.',toolRecorderError='Static transformation initially expected12 closures, found11, aborted before code writes; corrected explicit observed11. Not a product/runtime failure.')
status=dict(time=datetime.now().astimezone().isoformat(),contract='TEAM16/IMP142/r6',sourceHead=head,sourceTree=tree,inheritedCode='d29dbc36d567d4344407a07d652311f2bfa140d9',inheritedDocs='01fb86369b8176a28c02010c6afd560e24703dd7',fetchedMain='fac8573e55441fe514919c63c7f1d65181ff3577',fetchedGovernanceMain='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,artifacts=artifacts,matrix=matrix,limits=requirements,firstError=first,cleanup=cleanup,allAccepted=False,startOpen=False,typedClosedQualified=False,actualProductionExecuted=False,productionDispatcherUsesV3=False,productionStatusSampleJoined=False,W2=False,dualSID=False,published=False)
write('final-status.json',status)
report=['# r6原件所有权接入受限Stop关闭','',f'源码{head}/tree{tree}；承接d29dbc36d/01fb86369，TEAM16/IMP142/r6及既定共同源不变。','', '| 运行 | raw | 过/败/忽略 | ms |','| --- | --- | --- | --- |']
for n,r in runs.items():report.append(f"| {n} | {r['rawExit']} | {r.get('passed','—')}/{r.get('failed','—')}/{r.get('ignored','—')} | {r['elapsedMs']} |")
report+=['','首轮MSVC raw101/E0609+两E0499，未生成/归属新EXE且未测试；树Option转换一测试iterator及两处字段借用过早，仅修正确原槽访问/缩短借用，不改变原check_working/断言/超时。转换记录器最初expected12实际11，在写源前失败，改准确11；错误原文与首编译source/log/result保留。',
 '', 'CapturedTrees保管准确两树Option，实际Stop qualified restoration/history后，把原所有权移入journal先留存，再做同一actual source/Owner/NormalExit/Absent/StopAccepted/routes/content/canonical/end guard和原native close driver；同段保留completion后再核guard。原close body及逐handle Held/ConfirmedClosed/Unknown规则不变，未用Drop作成功。所有本地树变更入口先拒空槽，错误poison传到实际handoff。重复接收先拒绝，两个原集合都不被替换或drop。',
 '', '新增实际文件counterexamples证明转移后写入raw32，缺槽变更失败后原件仍同bytes/raw32且journal保管poison现场；重复转移两个不同树集合均raw32，未写关闭事件/未签正常退出。原生产guard只编译未执行，不能以组件代actual ordinary fixedofficial Stop预算/TypedClosed。完整inventory roles/counts、process/supervisor完成history、同次Status5/dispatcher/后继、原包副作用/初始化/ownerreceipt/原缺席Codex退根、普通MSI/CAP-trust/历史清理仍待，Start拒绝。',
 '',f'{len(scenes)}场景完整私有ZIP/hash/membership核验，{len(records)}份原canonical/快照/history单独收存；全部pending/incomplete scene留存，未删除/重试历史拒绝。ZIP不代表NTFS身份安全克隆。{len(inputs)}产品输入/{len(rust)}Rust实源核工作/Git明确CRLF等价，preexisting schema未手改/暂存。原预算/unknown拒绝/官方日常策略保持，不raw5/空GUI/W2/双SID/发布。',
 '', '| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
with (ROOT/'report.md').open('x',encoding='utf-8') as f:f.write('\n'.join(report)+'\n')
print(json.dumps(dict(head=head,passed=runs['library02']['passed'],scenes=len(scenes),rawRecords=len(records),productInputs=len(inputs),testRust=len(rust),typedClosed=False)))
