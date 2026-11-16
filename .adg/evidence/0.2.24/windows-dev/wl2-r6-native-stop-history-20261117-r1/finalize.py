from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=ROOT.parent/'wl2-r6-close-row-grammar-20261116-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,v):
    with (ROOT/name).open('x',encoding='utf-8') as f:json.dump(v,f,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
assert load(ROOT/'source-commit.json')['head']==head
runs={};scenes=[];records=[];traces=[];rust=[];inputs=[];drifts=[]
for name,profile in [('library01','debug'),('library02','debug'),('library03','release'),('product01','debug')]:
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
        if name=='library01':
            if raw!=(CODE/row['path']).read_bytes():drifts.append(dict(run=name,path=row['path'],kind='later explicit factory requires actual CompletedStopGuard and added native case96; earlier exact executed source preserved'))
            continue
        assert raw==(CODE/row['path']).read_bytes(),row['path']
        assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n'),row['path']
        if name=='product01':inputs.append(row)
        elif name=='library03' and row['path'].endswith('.rs'):rust.append(row)
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
write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,releaseRust=rust,finalDataAndReleaseAndProductMatchSource=True,firstRunLaterDifferences=drifts,gitLineEndingsExplicitlyEquivalent=True,preexistingSchemaNotEditedOrStaged=True))
matrix=load(BASE/'final-status.json')['matrix'];matrix.extend([
 dict(id='ActualOriginalStopHistories',result='actual-native-factories-compiled/internal-registered-not-executed',fact='CaptureSlot internal restore now consumes original actual NormalExit/Absent only after actual qualified restore/native original handle closure. Each successful into_closed_history installed before next fallible step; partial evidence retained on poison. CompletedStopGuard binds retained actual exit/supervisor histories to original Source/Owner/StopAccepted/route/native audit pins/content scope/process generations and exact old action end. No fake native test constructor/serde/Clone/current boolean permits.'),
 dict(id='FinalCanonicalStopClosure',result='DATA-order-and-native-old-receipt-negative-pass/actual-positive-unexecuted',fact='Same canonical StopClosureIntent -> StopClosedVerified binds exact last handle-close seq/tip, original close current digest/native receipt state, content/history/inventory/count/Ready/actions. Pending blocks all other events; final chain immutable. Original native close receipt remains parent, final CompletedStop factory explicitly requires actual CompletedStopGuard and captures new exact current inside original cutoff. No reuse of old close state after append, no new deadline. Final actual guard/seal and closure-specific immutable/current fault production paths compiled but unexecuted.'),
 dict(id='CompletedStopSameSampleHistoryRead',result='compiled-internal/not-joined-public-Status',fact='CaptureSlot/Journal completed_stop_at uses original caller when/end<=5s, fresh original source/audit pins, retained actual exit/supervisor histories, original final-completion parent lineage/canonical bytes/native archive/retired objects. Does not call old action.check as a new mutation grant, extend original deadline or accept caller Closed text. Public Status/typed adapter/successor remain unjoined.'),
 dict(id='FinalCurrentCapacityAndOldCloseSeparation',result='directed-pass',fact='Original four-stage+recovery/history/handles/content/finalclosure current3787<=4096 bytes. Final DATA rejects skipped intent/substituted old close/Ready/content/history/handles/unknown completion flags/duplicates atomically and terminal reopen. Existing native close-completion10s and read5s regression intact; new case96 actual original native close receipt alone cannot sign final Stop after expiry, exact existing phase/current prefix unchanged.'),
])
requirements=load(BASE/'next-closed-requirements.json')['requirements']
for r in requirements:
    if r['id']=='OwnershipClosure':r['requirement']='Internal actual Stop restoration/complete expected native inventory/native handle close now consumes actual NormalExit/Absent to retained timely completion histories and joins same canonical final closure/CompletedStopGuard seal. Actual ordinary-desktop production factories/qualification/restoration/closure/seal remain unexecuted; cannot promote component DATA/native old-close negatives to typed Closed or public success. Original route pin still retained; release/closed successor routing must be designed with adapter ownership.'
    if r['id']=='CanonicalContentAndStatus':r['requirement']='Canonical content and final closure DATA/strict final parent lineage/current plus internal actual completion guard are implemented/compiled. Internal completed_stop_at same-sample <=5s read is joined to source/exit/supervisor/archive history; public Status adapter/typed tags/dispatcher and successor remain unjoined. Actual final-seal success and every closure-specific append/current fault window/native budget still require production evidence; public Start stays refused.'
write('next-closed-requirements.json',dict(requirements=requirements,startOpen=False,typedClosedQualified=False,actualProductionExecuted=False))
cleanup=dict(scenes=len(scenes),removed=0,failedAndPendingRetained=len(scenes),crossVolumePendingRootsRetained=sum(s['source'].startswith('D:') for s in scenes),deletionAttempted=False,historicalRejectedDeletionRetried=False,overallCleanupComplete=False,zipIsNTFSMetadataClone=False,reason='All source-plan/native tree/canonical scenes retained pending qualified complete production Closed; original failure/unknown prefixes unchanged. ZIP content archives do not clone native metadata.')
write('cleanup.json',cleanup)
status=dict(time=datetime.now().astimezone().isoformat(),contract='TEAM16/IMP142/r6',sourceHead=head,sourceTree=tree,inheritedCode='18a079fa4cf1c628bab1ad9c3e67513e0a227402',inheritedDocs='f568a540cb8a314754f51364b785815e3ae7bf70',fetchedMain='fac8573e55441fe514919c63c7f1d65181ff3577',fetchedGovernanceMain='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,artifacts=artifacts,matrix=matrix,limits=requirements,
 firstError=dict(currentProductOrTestError=None,toolError='source verifier initial implicit gbk read rejected UTF8 byte0xad/position2422; recorder only corrected to explicit utf-8-sig; original error retained source-verifier-first-error.txt',inheritedLatest=load(BASE/'final-status.json')['firstError'],expectedNativeNegativesPreserved=True),cleanup=cleanup,
 allAccepted=False,startOpen=False,typedClosedQualified=False,actualProductionExecuted=False,actualFinalStopSealExecuted=False,productionDispatcherUsesV3=False,productionStatusSampleJoined=False,W2=False,dualSID=False,published=False)
write('final-status.json',status)
report=['# r6原Stop完成历史与最终同链回执','',f'源码{head}/tree{tree}；承接18a079fa4/f568a540c，TEAM16/IMP142/r6及最新fetch既定共同源不变。','', '| 运行 | profile | raw | 过/败/忽略 | ms |','| --- | --- | --- | --- | --- |']
for n,r in runs.items():report.append(f"| {n} | {r['profile']} | {r['rawExit']} | {r.get('passed','—')}/{r.get('failed','—')}/{r.get('ignored','—')} | {r['elapsedMs']} |")
report+=['','本轮无非预期产品/测试失败。source verifier首用默认GBK读取UTF8失败，原错误留存，仅改记录器encoding，后17处原函数/文件逐字核对通过。library01原源码/EXE/75scene保留；随后强化最终工厂须actual CompletedStopGuard并加native96反例，最终data/release/product对准确提交核源，不把首轮source当末轮。',
 '', '实际内部Stop lane在qualified恢复/native原件关闭后消费原NormalExit/Absent，每项into_closed_history成功先装Store，再下一失败点。实际CompletedStopGuard同时借原Source/Owner/StopAccepted/routes、audit源pin、原内容scope/进程代次与原action end，不能从caller DATA/flags/Clone/serde构造。',
 '', '新增canonical StopClosureIntent/StopClosedVerified绑定准确原handle-close seq/tip/current/native receipt、content/history/inventory/count/Ready；pending挡其它事件，final封链。原关闭回执保持为前驱；最终CompletedStop工厂显式actual guard参数，在原截止前核精确final current并签内部receipt，不重用旧state/重开预算。完整current3787<=4096。DATA跳过/替换前驱/缺qualification/退出文本flag/错Ready/history/重复字段等原子拒绝。',
 '', '内部completed_stop_at同原when/end<=5s，只读fresh Source/audit pins/retained实际exit-supervisor历史/准确canonical/native archives及retired objects；不renew原mutation end。旧完成history/Unknown/partial/late回归保持；新native96真实原件关闭回执在原截止后不能单独补签finalStop，准确phase/current不变。实际new final-seal正例与其append/current专属失败窗口仍未执行，typedClosed/publicStatus/dispatcher/后继未接；路由pin释放仍需adapter所有权闭环。',
 '', 'ordinary fixedofficial生产/官方config-auth-Grok/动态目录副作用、初始化/ownerreceipt、原缺席Codex安全退根、最大完整batch容量、普通MSI/CAP-trust/历史清理继续未完。Start拒绝，不旧raw5/空GUI/W2/双SID/发布。',
 '',f'{len(scenes)}scene私有ZIP完整hash/membership核验，全部原plan/native/canonical scene保留，{len(records)}份原canonical/快照/history另收存；本轮无删除/历史拒绝重试，ZIP非NTFS身份安全克隆。最终data/release/product源对工作/Git明确CRLF等价，产品{len(inputs)}输入/最终release{len(rust)}Rust；预存schema未手改/暂存。',
 '', '| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
with (ROOT/'report.md').open('x',encoding='utf-8') as f:f.write('\n'.join(report)+'\n')
print(json.dumps(dict(head=head,passed={k:r.get('passed') for k,r in runs.items()},scenes=len(scenes),rawRecords=len(records),productInputs=len(inputs),releaseRust=len(rust),typedClosed=False)))
