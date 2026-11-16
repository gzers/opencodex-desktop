from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=ROOT.parent/'wl2-r6-stopped-handle-ownership-20261116-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,v):
    with (ROOT/name).open('x',encoding='utf-8') as f:json.dump(v,f,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
assert load(ROOT/'source-commit.json')['head']==head
runs={};scenes=[];records=[];traces=[];rust=[];inputs=[]
for name,profile in [('library01','debug'),('library02','release'),('product01','debug')]:
    result=load(ROOT/name/'result.json');assert result['exitCode']==0 and result['profile']==profile
    runs[name]=dict(rawExit=0,elapsedMs=result['elapsedMs'],profile=profile)
    if name.startswith('library'):
        log=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig')
        m=re.search(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',log);assert m and int(m[2])==0 and int(m[3])==0
        runs[name].update(passed=int(m[1]),failed=0,ignored=0,artifact=load(ROOT/f'{name}-artifact.json'))
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
        assert raw==(CODE/row['path']).read_bytes(),row['path']
        assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n'),row['path']
        if name=='product01':inputs.append(row)
        elif name=='library02' and row['path'].endswith('.rs'):rust.append(row)
assert load(ROOT/'product01/source.json')['head']==head
for row in load(ROOT/'source-scope.json')['changedFiles']:
    raw=(CODE/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
artifacts=[]
for name in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe';raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
write('product01-artifacts.json',artifacts);write('raw-scene-records.json',records);write('directed-traces.json',traces)
write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,testRust=rust,allThreeRunInputsMatchSource=True,gitLineEndingsExplicitlyEquivalent=True,preexistingSchemaNotEditedOrStaged=True))
matrix=load(BASE/'final-status.json')['matrix'];matrix.append(dict(id='IndependentCompleteNativeCloseInventory',result='native-directed-and-full-release-closure-pass/production-compiled-not-executed',fact='Private nonserde/nonClone ExpectedHandleInventory independently observes every actual retained tree/original/history-source/stopped-anchor/initial-work/old-work and every old/final4coordination/file/claim-parent/ancestor/strict-parent/bound pin before ownership transfer. Exact ordered role/key/native ID/kind/history flag/count equals actual close batch before intent, just before close and after actual closes. Actual pinned ancestor handles observed read-only without clone/reopen/transfer. Original CloseHandle states/transfer/detach functions unchanged. Native missing/extra/role/different-object/changed-deadline negatives and affected full closure/completion/fault history tests use original windows. Parsing DATA never authorizes close.'))
requirements=load(BASE/'next-closed-requirements.json')['requirements']
for r in requirements:
    if r['id']=='PlanCompleteness':r['requirement']='Independent complete native expected inventory is now checked before/after actual close; actual tree/lease ancestor/bound roles/IDs/kinds/counts are no longer accepted from prefix-only rows. Canonical DATA validator still admits broader tree/lease key shapes outside required original/history-source roles, so strict unknown-role/ordinal grammar remains to harden separately. Full maximal1MiB/512node batch-plan capacity/subgroup support remains open; original bounds and complete inventory must not be weakened.'
    if r['id']=='OwnershipClosure':r['requirement']='Original owned2trees are transferred into retained journal before fallible close; independent complete actual expected inventory now joins original close/completion driver and passes affected release synthetic native closure/history/fault regression. Actual ordinary-desktop production source/Owner/Stop restore/close remains unexecuted, process/supervisor completed history and typedClosed/Status/dispatcher remain unjoined.'
write('next-closed-requirements.json',dict(requirements=requirements,startOpen=False,typedClosedQualified=False,actualProductionExecuted=False))
cleanup=load(ROOT/'cleanup.json');assert cleanup['removed']==12 and cleanup['failed']==0
counts=load(ROOT/'cleanup-counts.json')
status=dict(time=datetime.now().astimezone().isoformat(),contract='TEAM16/IMP142/r6',sourceHead=head,sourceTree=tree,inheritedCode='502858eb56f1a5401684259e7420bc0712c6cb4f',inheritedDocs='f3a72381a50bf8cddb05bc63d1e0d3e72e327078',fetchedMain='fac8573e55441fe514919c63c7f1d65181ff3577',fetchedGovernanceMain='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,artifacts=artifacts,matrix=matrix,limits=requirements,
 firstError=dict(currentRound=None,inheritedLatest=load(BASE/'final-status.json')['firstError'],expectedNativeNegativesPreserved=True),
 cleanup=dict(**counts,removed=12,historicalRejectedDeletionRetried=False,overallCleanupComplete=False,zipIsNTFSMetadataClone=False),
 allAccepted=False,startOpen=False,typedClosedQualified=False,actualProductionExecuted=False,productionDispatcherUsesV3=False,productionStatusSampleJoined=False,W2=False,dualSID=False,published=False)
write('final-status.json',status)
report=['# r6独立完整原生关闭句柄清单','',f'源码{head}/tree{tree}；承接502858eb5/f3a72381a，TEAM16/IMP142/r6及既定共同源不变。','', '| 运行 | profile | raw | 过/败/忽略 | ms |','| --- | --- | --- | --- | --- |']
for n,r in runs.items():report.append(f"| {n} | {r['profile']} | {r['rawExit']} | {r.get('passed','—')}/{r.get('failed','—')}/{r.get('ignored','—')} | {r['elapsedMs']} |")
report+=['','本轮无非预期失败，原raw32/原close Unknown/部分current等反例保持原断言和原始退出记录；继承首错另列final-status。新增ExpectedHandleInventory为private nonserde/nonClone，实际原对象在转移前独立逐项观测，再与close batch准确顺序/key/role/nativeID/kind/history/count全等比较。含原件/history-source/stopped-anchor/initial-work/全部旧work，以及全部old/final lease四coord/claim-parent/ancestor/strict-parent/bound目录句柄，关闭intent前、实际关闭前后同原end检查。父pin新只读接口只观测原handle，无clone/reopen/权限变化。',
 '', '实际NativeHandleBatch close_all、lease transfer、树detach原函数体逐字不变；14处源/函数对比及Start原拒绝门禁保留。新反例覆盖遗漏、多余、错角色、实际不同对象、不同deadline及全部祖先原handle。release受影响关闭/完成history回归沿原操作10秒/历史只读5秒，不增预算；不是ordinary fixedofficial生产证明。',
 '', '当前canonical DATA对额外tree/lease key仍较宽，严格未知角色/ordinal语法仍待；独立native完整清单并不把文本变为native资格。process/supervisor原Stop完成history、typedClosed/Status5/dispatcher/后继、实际原包副作用/首次初始化/ownerreceipt/原缺席Codex安全退根、最大节点/记录容量和普通MSI/CAP-trust/全部历史清理保持未完。Start仍拒绝，不W2/双SID/发布。',
 '',f'{len(scenes)}scene完整私有ZIP/hash/membership核验，12完成独立清单原语场景仅含本次合成文件/目录，fresh准确路径/hash核后清理；{counts["failedAndPendingRetained"]}失败/pending及{counts["crossVolumePendingRootsRetained"]}个D根保留，{len(records)}份原canonical/快照/history另收存。ZIP不是NTFS身份/安全克隆，未重试历史拒绝。全部三轮源输入核工作/Git明确CRLF等价，产品{len(inputs)}输入/最终release{len(rust)}Rust；预存schema未手改/暂存。',
 '', '| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
with (ROOT/'report.md').open('x',encoding='utf-8') as f:f.write('\n'.join(report)+'\n')
print(json.dumps(dict(head=head,passed={k:r.get('passed') for k,r in runs.items()},scenes=len(scenes),rawRecords=len(records),productInputs=len(inputs),releaseRust=len(rust),typedClosed=False)))
