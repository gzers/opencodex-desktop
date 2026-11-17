from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=ROOT.parent/'wl2-r6-completed-read-windows-20261117-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,v):
    with (ROOT/name).open('x',encoding='utf-8') as f:json.dump(v,f,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip();assert load(ROOT/'source-commit.json')['head']==head
runs={};scenes=[];records=[];traces=[];rust=[];inputs=[];drifts=[]
for name,rc,counts in [('library01',101,(47,1,0)),('library02',0,(48,0,0)),('library03',0,(49,0,0)),('product01',0,None)]:
    result=load(ROOT/name/'result.json');assert result['exitCode']==rc and result['profile']=='debug'
    runs[name]=dict(rawExit=rc,elapsedMs=result['elapsedMs'],profile='debug')
    if counts:
        log=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig');m=re.search(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',log)
        assert m and tuple(map(int,m.groups()))==counts
        runs[name].update(passed=counts[0],failed=counts[1],ignored=counts[2],artifact=load(ROOT/f'{name}-artifact.json'))
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
        if name in ['library01','library02']:
            if raw!=(CODE/row['path']).read_bytes():drifts.append(dict(run=name,path=row['path'],reason='actual primary-last ordering and later ADS veto/test; original exact executed source retained'))
            continue
        assert raw==(CODE/row['path']).read_bytes(),row['path']
        assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n'),row['path']
        if name=='product01':inputs.append(row)
        elif row['path'].endswith('.rs'):rust.append(row)
assert load(ROOT/'product01/source.json')['head']==head
for row in load(ROOT/'source-scope-verified.json')['changedFiles']:
    raw=(CODE/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
failed=load(ROOT/'library01-failed-executable.json');assert sha(Path(failed['retained']).read_bytes())==failed['sha256']
observations=load(ROOT/'actual-observations.json')
for row in observations['alternateStreamExports']:
    raw=(ROOT/row['evidence']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
    assert raw==Path(row['source']).read_bytes()
artifacts=[]
for name in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe';raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
write('product01-artifacts.json',artifacts);write('raw-scene-records.json',records);write('directed-traces.json',traces)
write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,testRust=rust,finalTestAndProductMatchSource=True,earlierRunDifferences=drifts,gitLineEndingsExplicitlyEquivalent=True,preexistingSchemaNotManuallyEditedOrStaged=True))
matrix=load(BASE/'final-status.json')['matrix'];matrix.append(dict(id='OwnedRoutePinClosePrimitive',result='actual-native-own-read-handle-component-pass/not-production-integrated',fact='RetainedRouteClose first retains original RoutePin, then observes exact config/security/2ancestor File sets into independent full expected inventory. Actual bounded8/main bytes/hash/streams/token and original end rechecked before native closes. READ_CONTROL-only security handle did NOT prevent write after primary closed: first47pass1fail/raw101 preserved; corrected order keeps primary config LAST, partial/protected failures keep actual deny-write primary. Successful explicit all-handle close frees subsequent independent H/config; historical facts never adopt new bytes. All-closed-before-private-completion failure still unqualified. Added ADS actually created after transfer and vetoed before any close; stream retained/exported. Candidate never signs actual Stop/canonical permission; production current RoutePin unchanged.'))
requirements=load(BASE/'next-closed-requirements.json')['requirements']
for r in requirements:
    if r['id']=='OwnershipClosure':r['requirement']='Original route pin own-read-handle primitive now captures ALL primary/security/2ancestor sets and explicit native close states with full expected inventory; primary must close LAST per actual counterexample. Native partial/protected/postclose/data/deadline/ADS cases pass. Candidate alone not production Stop permit: integrate same canonical route-close intent/result, actual source/Owner/action/normal/supervisor/content guard and final Stop parent lineage BEFORE releasing current session RoutePin/successor. Current production pin remains retained.'
write('next-closed-requirements.json',dict(requirements=requirements,startOpen=False,typedClosedQualified=False,canonicalRouteCloseIntegrated=False,actualProductionExecuted=False))
cleanup=dict(scenes=len(scenes),removed=0,failedAndPendingRetained=len(scenes),crossVolumePendingRootsRetained=sum(s['source'].startswith('D:') for s in scenes),deletionAttempted=False,historicalRejectedDeletionRetried=False,overallCleanupComplete=False,zipIsNTFSMetadataClone=False,alternateStreamExports=len(observations['alternateStreamExports']),reason='Preserve first failed scene, protected UNKNOWN raw handle scenes, native/canonical pending states and ADS original after private archive/hash verification. ZIP regular stream archives are not native metadata or ADS clones; ADS original bytes exported independently.')
write('cleanup.json',cleanup)
first=dict(run='library01',rawExit=101,passed=47,failed=1,kind='actual fault81 primary read handle closed first; READ_CONTROL-only observer did not deny later config write; raw32 assertion failed on Ok',correction='Keep original denial assertion, close primary config LAST in independent expected and actual batch. Added attempt trace before assertions and ADS recheck before close. library02 48pass, final49pass. Original source/log/EXE retained; do NOT rerun old premise.',firstFailureDetailedTraceUnavailable=True,toolError='Source function comparison selected ExpectedHandleInventory.new instead of NativeHandleBatch.new; exact impl anchor corrected, actual old native CloseHandle body unchanged.')
status=dict(time=datetime.now().astimezone().isoformat(),contract='TEAM16/IMP142/r6',sourceHead=head,sourceTree=tree,inheritedCode='fcc7a688da39c6e289dd6478b919bbabfed2da45',inheritedDocs='e3daa9956aed26f5b547aec9805cb25c1fe7f982',fetchedMain='fac8573e55441fe514919c63c7f1d65181ff3577',fetchedGovernanceMain='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,artifacts=artifacts,matrix=matrix,limits=requirements,firstError=first,cleanup=cleanup,
 allAccepted=False,startOpen=False,typedClosedQualified=False,actualProductionExecuted=False,actualFinalStopSealExecuted=False,productionDispatcherUsesV3=False,productionStatusSampleJoined=False,productionRoutePinReleased=False,canonicalRouteCloseIntegrated=False,W2=False,dualSID=False,published=False)
write('final-status.json',status)
report=['# r6原路由pin准确所有权与显式关闭原语','',f'源码{head}/tree{tree}；承接fcc7a688d/e3daa9956，TEAM16/IMP142/r6及共同源不变。','', '| 运行 | raw | 过/败/忽略 | ms |','| --- | --- | --- | --- |']
for n,r in runs.items():report.append(f"| {n} | {r['rawExit']} | {r.get('passed','—')}/{r.get('failed','—')}/{r.get('ignored','—')} | {r['elapsedMs']} |")
report+=['','首轮raw101/47过1败为真实原生反例：fault81先关闭primary config后，余下READ_CONTROL-only观察句柄不能继续denywrite，原raw32断言遇Ok。保留原断言及失败source/EXE/60scene，修独立expected/actual关闭顺序primary最后。首失败用例在probe panic前未输出细trace，此缺口明确保留，不编造/重跑旧前提。末轮加attempt-before-assert trace及ADS veto，49过0败。source verifier早选错new函数，修准确NativeHandleBatch impl anchor；原20函数/文件逐字保持。',
 '', 'RetainedRouteClose先无IO移原RoutePin到retained候选，再核原Token/H/config ID/all8/主流字节及streams，独立fullinventory包含主config、BoundedAccess hidden security文件和两条完整祖先链。准确实际File移入原NativeHandleBatch，所有权错误不drop；close前fresh全部ID/主流/8/ADS/原end。成功所有ConfirmedClosed及原窗completion后允许后继独立config写，旧facts只核immutable路由摘要与自己的完成/原token，不采用新文件字节。',
 '', '实际partial1、protectedUnknown1、allclosed-before-private-completion、wrongdeadline/data/inventory和ADS场景均保留准确Held/ConfirmedClosed/Unknown前缀，未知不重试/重建File/清保护。ADS确实在移交后创建，close在任何native关闭前InvalidData拒绝，所有Held，原流另导出原字节/hash并原位保留。readonly observation实际不等同主句柄sharing保证。',
 '', '本轮只是own-read-handle原语，不接production current RoutePin，不产actual Stop/同链许可/typedClosed/后继放行；candidate data始终nativeAuthority=false。下一步同一canonical route-close intent/result＋原actual Source/Owner/action/content/normal/supervisor＋final parent lineage，不能Drop作完整release。原包生产副作用/初始化/ownerreceipt/原缺席Codex安全退根/最大batch容量/普通MSI/CAP-trust/历史清理保持未完，Start拒绝。',
 '',f'{len(scenes)}scene私有ZIP/hash/membership核验后全部保留（含首失败/protectedUnknown/nativepending/ADS），{len(records)}份canonical/快照/history另收存，本轮无删除/历史拒绝重试；ZIP非NTFS身份安全或ADS克隆，ADS原字节另导出。最终源对工作/Git明确CRLF等价，产品{len(inputs)}输入/最终test{len(rust)}Rust，预存schema未手改/暂存。不改官方/日常/PATH/政策/账户，不旧raw5/空GUI/W2/双SID/发布。',
 '', '| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
with (ROOT/'report.md').open('x',encoding='utf-8') as f:f.write('\n'.join(report)+'\n')
print(json.dumps(dict(head=head,finalPass=49,scenes=len(scenes),rawRecords=len(records),productInputs=len(inputs),testRust=len(rust),productionRouteCloseIntegrated=False)))
