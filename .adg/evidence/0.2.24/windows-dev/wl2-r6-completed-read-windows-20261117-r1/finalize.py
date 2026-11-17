from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=ROOT.parent/'wl2-r6-native-stop-history-20261117-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,v):
    with (ROOT/name).open('x',encoding='utf-8') as f:json.dump(v,f,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip();assert load(ROOT/'source-commit.json')['head']==head
runs={};scenes=[];records=[];traces=[];rust=[];inputs=[];drifts=[]
for name,profile,passed in [('library01','debug',50),('library02','debug',51),('library03','release',8),('product01','debug',None)]:
    result=load(ROOT/name/'result.json');assert result['exitCode']==0 and result['profile']==profile
    runs[name]=dict(rawExit=0,elapsedMs=result['elapsedMs'],profile=profile)
    if passed is not None:
        log=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig');m=re.search(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',log)
        assert m and tuple(map(int,m.groups()))==(passed,0,0)
        runs[name].update(passed=passed,failed=0,ignored=0,artifact=load(ROOT/f'{name}-artifact.json'))
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
        if name=='library01':
            if raw!=(CODE/row['path']).read_bytes():drifts.append(dict(run=name,path=row['path'],reason='later explicit TimedOut mapping and actual elapsed-window test; first exact executed source retained'))
            continue
        assert raw==(CODE/row['path']).read_bytes(),row['path']
        assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n'),row['path']
        if name=='product01':inputs.append(row)
        elif name=='library03' and row['path'].endswith('.rs'):rust.append(row)
assert load(ROOT/'product01/source.json')['head']==head
for row in load(ROOT/'source-scope-verified.json')['changedFiles']:
    raw=(CODE/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
artifacts=[]
for name in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe';raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
write('product01-artifacts.json',artifacts);write('raw-scene-records.json',records);write('directed-traces.json',traces)
write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,releaseRust=rust,finalDataAndReleaseAndProductMatchSource=True,firstRunLaterDifferences=drifts,gitLineEndingsExplicitlyEquivalent=True,preexistingSchemaNotManuallyEditedOrStaged=True))
matrix=load(BASE/'final-status.json')['matrix'];matrix.append(dict(id='ExplicitCompletedHistoryReadWindows',result='window-DATA-and-old-native-receipt-negative-pass/actual-final-history-positive-unexecuted',fact='Private nonserde/nonClone HistoryReadWindow carries exact Status when/end<=5s or copied caller-original ActionWindow with immutable outer/Stop deadlines. CaptureSlot and Journal share same read request throughout source/pins/exit/supervisor/canonical/archive read, no original cutoff mutation. Expired original action maps TimedOut, not generic IO Other, and cannot be reconstructed into now+budget. New native case97 real old handle-close receipt plus fresh Start30 read refuses InvalidData and preserves canonical prefix; actual final Stop guard/history/source positive remains unexecuted. Route pin unchanged/retained; release requires separate explicit native canonical close.'))
requirements=load(BASE/'next-closed-requirements.json')['requirements']
for r in requirements:
    if r['id']=='CanonicalContentAndStatus':r['requirement']='Canonical content/final closure and actual guard factories remain compiled but actual positive unexecuted. Internal completed history now has explicit same-sample Status<=5s and borrowed original ActionWindow outer read windows; expired actions keep TimedOut, no mutation deadline reset. Public Status/typed tags/dispatcher/successor still unjoined; actual final-seal success and closure-specific persistence faults/budgets still require production evidence.'
    if r['id']=='OwnershipClosure':r['requirement']='Actual Stop histories/internal final seal implemented but real ordinary-desktop production remains unexecuted. Current route H/config and ancestor pin stays retained; do not Drop/take it as explicit native close evidence. Integrate same canonical native route-pin close intent/result and exact ownership states into final closure before unblocking independent successor config. Then complete typed/public adapter and successor history.'
write('next-closed-requirements.json',dict(requirements=requirements,startOpen=False,typedClosedQualified=False,actualProductionExecuted=False))
cleanup=dict(scenes=len(scenes),removed=0,failedAndPendingRetained=len(scenes),crossVolumePendingRootsRetained=sum(s['source'].startswith('D:') for s in scenes),deletionAttempted=False,historicalRejectedDeletionRetried=False,overallCleanupComplete=False,zipIsNTFSMetadataClone=False,reason='All plan/native/canonical scenes pending actual complete production Closed; preserve after private archive verification. No task account/Profile/PATH/policy mutation; route pin release not executed.')
write('cleanup.json',cleanup)
status=dict(time=datetime.now().astimezone().isoformat(),contract='TEAM16/IMP142/r6',sourceHead=head,sourceTree=tree,inheritedCode='a653a51acea51dbba5b4e6b4a2488f8b53f09ce4',inheritedDocs='ecb3b27c8a91d188db0832d7185a8b0062f95c98',fetchedMain='fac8573e55441fe514919c63c7f1d65181ff3577',fetchedGovernanceMain='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,artifacts=artifacts,matrix=matrix,limits=requirements,
 firstError=dict(currentProductOrTestError=None,toolError='Source recorder second create-only source-scope-final.json refused FileExistsError; original snapshot preserved, final source written to explicit source-scope-verified.json. Static ActionWindow expiry mapping Other corrected to TimedOut before final new elapsed-expiry test; first50pass source remains distinct.',inheritedLatest=load(BASE/'final-status.json')['firstError'],expectedNativeNegativesPreserved=True),cleanup=cleanup,
 scopeAdjustment=load(ROOT/'scope-adjustment.json'),allAccepted=False,startOpen=False,typedClosedQualified=False,actualProductionExecuted=False,actualFinalStopSealExecuted=False,productionDispatcherUsesV3=False,productionStatusSampleJoined=False,routePinReleased=False,W2=False,dualSID=False,published=False)
write('final-status.json',status)
report=['# r6同次Status与原动作完成历史读取','',f'源码{head}/tree{tree}；承接a653a51ac/ecb3b27c8，TEAM16/IMP142/r6与fetch共同源不变。','', '| 运行 | profile | raw | 过/败/忽略 | ms |','| --- | --- | --- | --- | --- |']
for n,r in runs.items():report.append(f"| {n} | {r['profile']} | {r['rawExit']} | {r.get('passed','—')}/{r.get('failed','—')}/{r.get('ignored','—')} | {r['elapsedMs']} |")
report+=['','本轮无非预期产品/测试失败。首轮50过后静态纠正ActionWindow截止映射Other为TimedOut，补真实已到期窗口不能续用；最终51过。原first source/EXE/75scene保留不归末轮。记录器重复create-only旧scope文件被FileExists拒绝，原快照留存，精确新文件名source-scope-verified记录最终源；未覆盖/放宽验证。',
 '', 'HistoryReadWindow为private nonserde/nonClone，只是read locator，无native许可：Status准确when/end<=5s，Action直接借原ActionWindow.deadline，复制原不可变窗口后前后remaining核验；不生成now+budget，不改变原Stop mutation cutoff。CaptureSlot/Journal source/audit pins/actual completed exit-supervisor/canonical/archive全读继承同一request。实际passed elapsed action窗口read与再次构造均TimedOut，Start/Stop/Restart原parent收窄、Restart30/Stop子10保持。',
 '', '新增native97在真实原件完整关闭/原10秒到期后传fresh Start30 read，仍InvalidData拒绝旧handle-close补签finalStop，phase/current/原immutable事件不变。旧正常/Unknown/partial/late/history5秒用例保持；不是actual ordinary fixedofficial final-history正例。',
 '', '批次开头核对发现H route config/祖先pin不能单纯take/drop声称完整释放；scope-adjustment在源测试前保留说明。本轮RoutePin/原actual mutation/close/finalseal等21处源/函数逐字不变，routes仍保管。下一步同一canonical显式route-pin close intent/result/actual owned handle状态，随后typed/public Status/dispatcher/后继。原包副作用/初始化/ownerreceipt/原缺席Codex安全退根/最大batch容量/普通MSI/CAP-trust/历史清理保持未完，Start拒绝。',
 '',f'{len(scenes)}scene私有ZIP/hash/membership完整核验，全部plan/native/canonical现场保留，{len(records)}份原canonical/快照/history另收存；本轮无删除/历史拒绝重试，ZIP非NTFS元数据克隆。最终data/release/product全部核工作/Git明确CRLF等价，产品{len(inputs)}输入/最终release{len(rust)}Rust。预存schema未手改/暂存；无官方/日常配置/PATH/账户/策略/业务/模型修改，不旧raw5/空GUI/W2/双SID/发布。',
 '', '| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
with (ROOT/'report.md').open('x',encoding='utf-8') as f:f.write('\n'.join(report)+'\n')
print(json.dumps(dict(head=head,passed={k:r.get('passed') for k,r in runs.items()},scenes=len(scenes),rawRecords=len(records),productInputs=len(inputs),releaseRust=len(rust),routePinReleased=False)))
