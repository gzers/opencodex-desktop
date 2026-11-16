from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=ROOT.parent/'wl2-r6-production-content-20261116-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,value):
    with (ROOT/name).open('x',encoding='utf-8') as f:json.dump(value,f,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
assert load(ROOT/'source-commit.json')['head']==head
runs={};scenes=[];records=[];traces=[];test_inputs=[];product_inputs=[]
for name,rc,counts in [('library01',101,(50,1,0)),('library02',0,(48,0,0)),('product01',0,None)]:
    result=load(ROOT/name/'result.json');assert result['exitCode']==rc
    runs[name]=dict(rawExit=rc,profile=result['profile'],elapsedMs=result['elapsedMs'])
    if counts:
        log=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig')
        m=re.search(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',log)
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
                        records.append(dict(evidence=p.relative_to(ROOT).as_posix(),original=str(Path(s['source'])/row['path']),bytes=len(raw),sha256=sha(raw)))
    for row in load(ROOT/name/'source-manifest.json'):
        raw=(ROOT/name/'executed-source'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
        if name=='product01' or name=='library02' and row['path'].endswith('.rs'):
            assert raw==(CODE/row['path']).read_bytes(),row['path']
            assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n'),row['path']
            (product_inputs if name=='product01' else test_inputs).append(row)
assert load(ROOT/'product01/source.json')['head']==head
for row in load(ROOT/'source-scope-final.json')['changedFiles']:
    raw=(CODE/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
retained=load(ROOT/'library01-failed-executable.json')
assert sha(Path(retained['retained']).read_bytes())==retained['sha256']
artifacts=[]
for name in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe';raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
write('product01-artifacts.json',artifacts)
write('final-source.json',dict(head=head,tree=tree,productInputs=product_inputs,testRust=test_inputs,gitLineEndingsExplicitlyEquivalent=True,preexistingSchemaNotEditedOrStaged=True))
write('raw-scene-records.json',records);write('directed-traces.json',traces)
matrix=load(BASE/'final-status.json')['matrix']
matrix.extend([
 dict(id='CanonicalContentQualification',result='directed-DATA-and-native-current-pass/production-methods-compiled-not-executed',fact='ContentQualificationIntent -> ContentQualifiedVerified on original v3 immutable event/current chain. Exact private actual native assessment serializes scope/series/3stage-bindings/6frame-digests/3fixed-orderactions. Fresh unchanged full BOTH-tree policy barrier between intent/result, same actual claim and Stop scope; before frozen frame transfer. Native Source/Owner typed permit is never decoded or constructed by tests.'),
 dict(id='QualificationReplayAndPending',result='DATA-pass',fact='Replay retains strict stage sequence+context hash derived only from SnapshotLinkedVerified; typed nested context rejects unknown/duplicate fields. Pending blocks otherwise-valid recovery; result binds same content scope/actions; self-rehashed stage substitution, missing phase, wrong lease/action/tree and replay refuse atomically. Parsed DATA has no native authority.'),
 dict(id='QualificationCurrentCapacity',result='native-journal-fixture-and-DATA-pass',fact='Original four-stage maps + recovery/history/handles/qualification current remains3490<=4096 bytes, strict typed roundtrip. Existing full canonical native current fault/unknown/late-deadline tests pass; no max/budget increased.'),
])
requirements=load(BASE/'next-closed-requirements.json')['requirements']
for item in requirements:
    if item['id']=='CanonicalContentAndStatus':
        item['requirement']='Canonical content qualification DATA/event/current and production guard are now implemented/compiled and directed tested; actual ordinary-desktop production qualification is still unexecuted. Full ownership closure/typed Closed/actual5s same-sample Status/dispatcher and successor remain unjoined. No mutation cutoff reset.'
write('next-closed-requirements.json',dict(requirements=requirements,publicStartOpen=False,typedClosedQualified=False,actualProductionExecuted=False))
cleanup=load(ROOT/'cleanup.json');assert cleanup['removed']==2 and cleanup['failed']==0
counts=load(ROOT/'cleanup-counts.json')
first=dict(run='library01',rawExit=101,passed=50,failed=1,newExecutableProduced=True,kind='test-only recovery snapshot reference hash was encoded from Value instead of original typed SnapshotReference; otherwise-valid recovery positive precondition failed',correction='Construct and encode original typed SnapshotReference in synthetic DATA fixture; production validator/assertions unchanged. Add actual valid old/new action-scope substitution and nested duplicate checks; final48pass0fail. Exact first source/log/EXE/87scenes retained.')
status=dict(time=datetime.now().astimezone().isoformat(),contract='TEAM16/IMP142/r6',sourceHead=head,sourceTree=tree,
 fetchedMain='fac8573e55441fe514919c63c7f1d65181ff3577',fetchedGovernanceMain='5782307174f0d1b011b5415c6c1e8d60414e6b36',inheritedCode='ed89291540e1d02c886c66020ebbbdc49c2cebeb',inheritedDocs='a81796823cb480598bca672d58616186b78a43bb',
 runs=runs,artifacts=artifacts,firstError=first,matrix=matrix,limits=requirements,
 allAccepted=False,startOpen=False,typedClosedQualified=False,actualProductionExecuted=False,actualProductionRestoreExecuted=False,productionDispatcherUsesV3=False,productionStatusSampleJoined=False,W2=False,dualSID=False,published=False,
 cleanup=dict(**counts,removed=2,historicalRejectedDeletionRetried=False,overallCleanupComplete=False,zipIsNTFSMetadataClone=False))
write('final-status.json',status)
report=['# r6原内容资格进入同一canonical链','',f'源码 {head} / tree {tree}。承接ed8929154/a81796823；TEAM16/IMP142/r6及fetch共同源不变，仅Windows允许集。','',
 '| 运行 | raw | 过/败/忽略 | ms |','| --- | --- | --- | --- |']
for name,r in runs.items():report.append(f"| {name} | {r['rawExit']} | {r.get('passed','—')}/{r.get('failed','—')}/{r.get('ignored','—')} | {r['elapsedMs']} |")
report+=['','首轮50过1败/raw101：新增测试前置恢复数据的快照引用以Value顺序编码，原typed reference摘要校验正确拒绝；仅修合成测试构造，不放宽生产断言。原失败EXE另留私有目录并校验，87场景/原源码/日志保留。末轮48过0败；复用首轮未受影响纯Bun/sourcepin结果，未重跑空GUI/旧raw5。',
 '', '新ContentQualificationIntent/ContentQualifiedVerified保存实际private assessment的scope/series、原Ready/PreStop/Stopped上下文/6帧及三固定顺序动作；SnapshotLinkedVerified只在原canonical阶段更新严格阶段序号/上下文hash。结果必须匹配intent完整数据，pending阻断其它事件，完成后不再回工作/阶段/新action。current只增加有界资格摘要，完整四阶段+恢复/history/handles为3490字节，原4096上限不变。',
 '', '生产内部入口在同一actual typed Stop/source/owner/routes/NormalExit/Absent/claim/绝对end下先写intent，重跑原完整BOTH-tree内容屏障，再写verified/current，之后才capture_retirement。恢复每段和history完成再核原native assessment与canonical资格摘要。DATA解析不构造Stop/claim/Owner/assessment或Closed许可；原shared policy/原生内容算法、源pin、预算和公开Start拒绝不变。',
 '', '真实普通桌面原包Owner/资格/恢复仍仅编译未执行。后续完整原件handle inventory/所有权关闭、timely completion与typedClosed、同次5秒Status、dispatcher/后继、官方config/auth/Grok/动态目录副作用、首次初始化/ownerreceipt/原缺席Codex安全退根、普通MSI/CAP-trust/历史清理继续未完；全部继承状态归final-status，不将组件通过当整版验收。',
 '',f'本轮{len(scenes)}场景私有ZIP按原字节/全membership核验；仅2完成纯库/pin场景清理，{counts["failedAndPendingRetained"]}失败/pending场景及{counts["crossVolumePendingRootsRetained"]}个D根保留。{len(records)}份原canonical/快照/history文件另入raw-scene-records。ZIP非NTFS身份/安全克隆，历史拒绝删除未重试，overallCleanupComplete=false。',
 '',f'最终 {len(product_inputs)} 产品输入 / {len(test_inputs)} Rust实源逐字核对工作文件及Git（明确CRLF等价）；产品custom-protocol MSVC debug仅编译，不运行服务/同步/模型。',
 '', '| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
with (ROOT/'report.md').open('x',encoding='utf-8') as f:f.write('\n'.join(report)+'\n')
print(json.dumps(dict(head=head,scenes=len(scenes),rawRecords=len(records),productInputs=len(product_inputs),testRust=len(test_inputs),startOpen=False)))
