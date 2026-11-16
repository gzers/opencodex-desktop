from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
PREVIOUS=ROOT.parent/'wl2-r6-recovery-native-chain-20261116-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,value):
    with (ROOT/name).open('x',encoding='utf-8') as out:json.dump(value,out,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
assert head!='139de12617ebfdb2f5d78c27d93a47abd177eae6'
runs={};inputs=[];rust=[];records=[];scenes=[];traces=[]
for name,passed in [('library01',21),('library02',5),('product01',None)]:
    result=load(ROOT/name/'result.json');assert result['exitCode']==0
    if passed is not None:
        text=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig')
        m=re.search(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',text);assert m and tuple(map(int,m.groups()))==(passed,0,0)
        runs[name]=dict(rawExit=0,passed=passed,failed=0,ignored=0,elapsedMs=result['elapsedMs'],profile=result['profile'],artifact=load(ROOT/f'{name}-artifact.json'))
        for line in text.splitlines():
            if 'r6 ' in line and len(line)<2200:traces.append(dict(run=name,trace=line[line.index('r6 '):]))
        for i,scene in enumerate(load(ROOT/f'{name}-private-scenes.json')['scenes']):
            assert sha(Path(scene['archive']).read_bytes())==scene['archiveSha256'];scenes.append(scene)
            for row in scene['files']:
                if scene['source'].startswith('D:') or '/.ocxd024-observed-' in '/'+row['path']:
                    p=Path(scene['source'])/row['path'];raw=p.read_bytes();assert sha(raw)==row['sha256'] and len(raw)==row['bytes']
                    target=ROOT/'raw-scene-records'/name/f'scene-{i:03}'/row['path'];target.parent.mkdir(parents=True,exist_ok=True)
                    with target.open('xb') as out:out.write(raw)
                    records.append(dict(source=str(p),evidence=target.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
    else:runs[name]=dict(rawExit=0,profile=result['profile'],elapsedMs=result['elapsedMs'])
    for row in load(ROOT/name/'source-manifest.json'):
        raw=(ROOT/name/'executed-source'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
        if name=='product01' or name=='library02' and row['path'].endswith('.rs'):
            assert raw==(CODE/row['path']).read_bytes(),row['path']
            assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n'),row['path']
            (inputs if name=='product01' else rust).append(row)
assert load(ROOT/'product01/source.json')['head']==head
observations=load(ROOT/'native-traces-library02.json')['observations'];assert len(observations)==13
assert {r['observation']['fault'] for r in observations}=={0,*range(50,62)}
write('final-release-traces.json',traces);write('raw-scene-records.json',records)
write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,finalTestRust=rust,gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyModifiedOrStaged=True))
artifacts=[]
for name in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe';raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
write('product01-artifacts.json',artifacts)
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
matrix=load(PREVIOUS/'final-status.json')['matrix']
matrix.extend([
 dict(id='CanonicalWholeTreeRestoration',result='native-component-pass/private-production-permit-pending',fact='All current retirements and exact original returns precede WholeTreeRestoreIntent, actual BOTH-tree original/retired/archive/anchor/children/all8 readback and WholeTreeRestoredVerified in same canonical chain/current. Missing nodes/pending/skip/drift/old release paths refuse. Node counts alone cannot release; restored-tree DATA is not Closed.'),
 dict(id='CanonicalFinalLeaseAndAnchorRelease',result='native-component-pass/private-production-permit-pending',fact='Durable LeaseReleasePrepared with live original claim/NLK precedes NativeLease.release_with_receipt; exact receipt installed before fallible full-tree/stable512range postconditions. Each exact original directory/stable anchor has its own native-verified ReleaseVerified event, identity/parent/from/to/expected-state binding, current anchor count+progress digest. Only after ALL anchor records can final LeaseReleasedVerified/current mark necessary data completeness. No reacquire or textClosed.'),
 dict(id='FinalReleaseNativeFaultsAndUnknown',result='native-component-pass/12-negative-cases',fact='Four whole-tree intent/result/current windows, five lease-prepared/unlock/postcondition/result/current windows, two per-anchor immutable/current failures, one actual unknown private custody directory after unlock. Actual file position IDs/bytes, retained release receipt and claim-only SQLite competition match exact phase/current prefix. Poison never catches up. Foreign directory and original/native scenes retained; no deletion or security repair.'),
])
cleanup=load(ROOT/'cleanup.json');counts=load(ROOT/'cleanup-counts.json');assert cleanup['removed']==9 and cleanup['failed']==0
first=dict(run=None,unexpectedError=None,fact='MSVC debug data regression raw0/21; optimized targeted new release scope raw0/5. Injected failures and actual synthetic unknown-directory veto are expected negative cases, original raw errors/traces retained. Historical compile/Owner-premise/deadline failures remain unchanged in predecessor evidence. No same-premise debug full-transaction timeout retry; new long native scope uses original release profile with unchanged10s fixture cutoff from outset.')
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract='TEAM16/IMP142/r6',baselineMain='fac8573e55441fe514919c63c7f1d65181ff3577',baselineDocs='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,artifacts=artifacts,matrix=matrix,firstError=first,executionBoundary=load(ROOT/'execution-boundary.json'),allAccepted=False,startOpen=False,productionDispatcherUsesV3=False,actualSourceOrRecoveryFactoryExecuted=False,productionExecuted=False,officialOwnerCreated=False,W2=False,dualSID=False,published=False,cleanup=dict(**counts,removed=9,historicalRejectedDeletionRetried=False,overallCleanupComplete=False),limits=['explicit rootId locator on recovery node/anchor events still pending before production','immutable original history and native original handle-release/Closed eligibility pending','actual Source/routes/retained process/audit/shared content policy before frame transfer and production driver entry incomplete','Status5/handoff/successor/actual Owner H-S-D/official first-use initialization unaccepted','official nonartifact sideeffects/dynamic directories not admitted','original-absent Codex safe no-gap initialization/root retirement blocked','ordinary MSI/CAP-trust/historical Profile/Public/failed broker cleanup remain open'])
write('final-status.json',status)
report=['# r6整树恢复确认与canonical最终lease释放','',f'源码{head}/tree{tree}；承接139de1261/dc4b13884，TEAM16/IMP142/r6。仅Windows允许集，原目录/两稳定对象/claim/NLK/all8/原预算/未知拒绝保持。','', '| 运行 | profile | raw退出 | 过/败/忽略 | ms |','| --- | --- | --- | --- | --- |']
for name,r in runs.items():report.append(f"| {name} | {r['profile']} | {r['rawExit']} | {r.get('passed','—')}/{r.get('failed','—')}/{r.get('ignored','—')} | {r['elapsedMs']} |")
report+=['', '本轮无非预期首错。library01 debug MSVC数据及同链回归21过；追加current逐anchor计数/摘要及真实foreign-custody反例后，library02采用仓库原release参数定向新final-release两数据＋三原生组，共5过。原生完整1＋负例12共13实际场景，各继承原10秒fixture，未新开deadline/延时/移除断言。旧debug整段超时前提不重复；原失败仍归前批。product01仅custom-protocol MSVCdebug bins，不启动GUI或官方，不是发布候选。', '', '全部current工作文件逐准确handle退休、全部original准确handle返回后，WholeTreeRestoreIntent/current→实际双树全原件/退休集合/全部历史archive/原目录/两稳定对象/精确children/ID/all8/bytes后验→WholeTreeRestoredVerified/current。普通row全完仍不够，缺节点/错序/pending/外改/旧release绕路拒绝。claim/NLK保持到LeaseReleasePrepared已持久且再次全树重验之后才显式释放。', '', 'NativeLease.release_with_receipt立即把准确原释放对象装入Journal，任何post-release读回前已保留；随后全部原件/退休集合和两稳定对象512范围acquire/release proof。每anchor有准确ReleaseVerified事件，含原native身份/parent/from-to/expected-state与row摘要，不移动目录/协调文件。current同时绑定逐anchor状态摘要与完成数量；所有anchor原生记录完成后才LeaseReleasedVerified/current。必要DATA完整性成立仍无typedClosed/普通Stop成功，旧generic lease入口拒绝恢复场景，不能只靠Drop或JSON释放。', '', '12负例：整树intent后/原生后验后/不可变结果领先current/真实1字节current；lease准备后仍持锁、实际unlock+receipt后、postcondition后、不可变lease结果领先current/1字节current；首anchor不可变发布/current部分写；真实unlock后在本轮custody加入未知空目录。actual claim-only SQLite竞争持有时busy、实际释放后可取，和准确receipt/phase/current吻合。全部失败poison，拒绝check不补写，原件仍准确原路径/ID/bytes；未知目录保持原样，未删除、覆盖、改安全或重新获取mutation权限。', '', 'native final-release driver仍private且只有cfg(test)入口；合成阶段和artifact内容不造Source/Owner/NormalExit/content-policy/Closed资格。接下来须补explicit rootId事件locator、原件不可变history/原handle生命周期、actual Source/routes/process/audit/原shared policy与typedStop许可，再生产恢复/Closed/Status5/后继/Owner-H-S-D。非artifact与动态目录副作用未知拒绝，原缺席Codex安全初始化/退根独立blocked，Start无条件拒绝。普通MSI/CAP-trust/历史清理未关闭，不旧raw5/空GUI/相同1314/W2/双SID/真实业务请求/发布。', '', f'本轮{len(scenes)}场景完整私有ZIP核验；9完成纯数据场景经fresh准确路径/全文件hash/目录membership核验清理，{counts["failedAndPendingRetained"]}失败或pending及{counts["crossVolumePendingRootsRetained"]}个D根保留。{len(records)}份canonical/快照原字节另归raw-scene-records。ZIP非NTFS身份/安全克隆，历史删除拒绝不重试，overallCleanupComplete=false。', '', '| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
with (ROOT/'report.md').open('x',encoding='utf-8') as out:out.write('\n'.join(report)+'\n')
print(json.dumps(dict(head=head,runs={k:v['rawExit'] for k,v in runs.items()},productInputs=len(inputs),testRust=len(rust),rawRecords=len(records),scenes=len(scenes),removed=9,production=False)))
