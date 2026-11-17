from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code');BASE=ROOT.parent/'wl2-r6-route-close-canonical-20261117-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));sha=lambda b:hashlib.sha256(b).hexdigest();git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,v):
    with (ROOT/name).open('x',encoding='utf-8') as f:json.dump(v,f,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip();assert load(ROOT/'source-commit.json')['head']==head
runs={};inputs=[];rust=[]
for name in ['library01','product01']:
    r=load(ROOT/name/'result.json');assert r['exitCode']==0;runs[name]=dict(rawExit=0,profile=r['profile'],elapsedMs=r['elapsedMs'])
    if name=='library01':
        log=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig');m=re.search(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored',log);assert m;runs[name].update(passed=int(m[1]),failed=0,ignored=0,artifact=load(ROOT/'library01-artifact.json'))
    for row in load(ROOT/name/'source-manifest.json'):
        raw=(ROOT/name/'executed-source'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256'];assert raw==(CODE/row['path']).read_bytes()
        assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n')
        if name=='product01':inputs.append(row)
        elif row['path'].endswith('.rs'):rust.append(row)
assert load(ROOT/'product01/source.json')['head']==head
artifacts=[]
for name in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe';raw=p.read_bytes();artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
write('product01-artifacts.json',artifacts);write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,testRust=rust,gitLineEndingsExplicitlyEquivalent=True,preexistingSchemaNotManuallyEditedOrStaged=True))
scenes=load(ROOT/'library01-private-scenes.json')['scenes']
assert not scenes,'this run must not conceal unrecorded native scene requirements'
matrix=load(BASE/'final-status.json')['matrix'];matrix.append(dict(id='OriginalTypedReadyRetention',result='compiled-plumbing/Origin-metadata-directed-pass/actual-Owner-unexecuted',fact='Original Owner.observe keeps the same real nonClone ServiceObservation from original HMAC/readiness/current/native live checks after normal phase validation; no reconstruction from PID/port/digest. take_original_ready_for_stop requires original Stop/Restart action and started/notstopped Owner, rechecks actual typed current/native readiness, restores exact object on failure. New retained-owner read check compares actual Owner UUID/scope/token and exact admitted Origin metadata while preserving expired old mutation check. No new source/native/observation factory from DATA. Dispatcher still v2 and Start refusing; actual retained observation/take/read factories not executed.'))
requirements=load(BASE/'next-closed-requirements.json')['requirements'];requirements.append(dict(id='TypedAdapterAndLiveReadyReads',requirement='Wire retained actual ServiceObservation into v3 preStop and typed phase/Closed adapter. Implement bounded actual live-Ready read after old Start expiry, preserving full original custody/anchors/current nonartifact/claim and original snapshot assertions without minting new mutation rights. Then original completed-history same-sample Status and successor. Actual ordinary desktop currently marenben session7 disconnected; login requested; no administrator/controller substitution. No official service launch in this batch.'))
write('next-closed-requirements.json',dict(requirements=requirements,startOpen=False,actualProductionExecuted=False,typedClosedQualified=False))
cleanup=dict(newNativeScenes=0,removed=0,testAccountOrProfileCreated=False,PATHModified=False,policyModified=False,historicalRejectedDeletionRetried=False,overallCleanupComplete=False,inheritedPendingSceneEvidence=str(BASE/'final-cleanup.json'),reason='Directed Origin/Restart tests produced no task filesystem scenes; existing native failure/ADS/unknown scenes remain unchanged.')
write('cleanup.json',cleanup)
environment=load(ROOT/'hypotheses.json')['environment']
status=dict(time=datetime.now().astimezone().isoformat(),contract='TEAM16/IMP142/r6',sourceHead=head,sourceTree=tree,inheritedCode='df0c8c24fdb1e5394290cecc28a4bf68f815dbc4',inheritedDocs='165f7fb88c281c767355a618922221d2d8f68e9e',fetchedMain='fac8573e55441fe514919c63c7f1d65181ff3577',fetchedGovernanceMain='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,artifacts=artifacts,matrix=matrix,limits=requirements,environment=environment,cleanup=cleanup,
 firstError=dict(currentProductOrTestError=None,inheritedLatest=load(BASE/'final-status.json')['firstError']),allAccepted=False,startOpen=False,typedClosedQualified=False,actualProductionExecuted=False,actualTypedObservationFactoryExecuted=False,productionDispatcherUsesV3=False,productionStatusSampleJoined=False,W2=False,dualSID=False,published=False)
write('final-status.json',status)
report=['# r6原typed Ready观察与保留身份读取','',f'源码{head}/tree{tree}；承接df0c8c24f/165f7fb88，TEAM16/IMP142/r6及共同源不变，仅3Windows允许文件。',
 '',f'library01 MSVCdebug raw0/{runs["library01"]["passed"]}过0败0忽略，{runs["library01"]["elapsedMs"]}ms；产品custom-protocol bins raw0/{runs["product01"]["elapsedMs"]}ms。无本轮非预期失败，继承首错保持。',
 '', 'Owner.observe在原HMAC/health/current/live原检查及原phase成功后保留同一个不可Clone ServiceObservation；stop getter必须原Stop/Restart动作/已开始未停止，重核actual typed current/nativeReady，失败把准确观察对象装回。未拿tuple/PID/port/digest重造typed证明。实际factory仅编译未执行，公共typed adapter仍未切换。',
 '', 'NativeAction只读方法比对原OwnerUUID/scope/token及原Origin id/kind/outer/stop/end/admission；只供调用方有界read，不改旧截止或发写许可。原Mutation check与所有Stop/Status/recovery代码逐字保持（构造器只初始化新typed_ready字段）。真实原Origin到期后元数据可比较，旧check仍Timeout且now+30不同end拒绝，五替换反例拒绝；未构造假NativeAction/Owner/ServiceObservation。12处原文件/函数检查通过。',
 '', '会话事实：marenben7为Disc，已请求恢复普通交互桌面；控制器/Administrator不代普通验收。未启动官方服务。下一步完整liveReady只读原件/anchors/nonartifact/claim核验、v3 typed/preStop/Closed/Status/后继和ordinary原包组合；原副作用/初始化/ownerreceipt/原缺席退根/容量/普通MSI/CAP-trust/历史清理保持未完。',
 '',f'本轮无新native测试目录/账号/Profile/PATH/策略残留，原失败场景不动。{len(inputs)}产品输入/{len(rust)}Rust逐字核工作/Git（CRLF等价明确），预存schema未手改/暂存。Start拒绝，不旧raw5/空GUI/W2/双SID/发布。',
 '', '| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
with (ROOT/'report.md').open('x',encoding='utf-8') as f:f.write('\n'.join(report)+'\n')
print(json.dumps(dict(head=head,passed=runs['library01']['passed'],productInputs=len(inputs),testRust=len(rust),newScenes=0,actualFactoryExecuted=False)))
