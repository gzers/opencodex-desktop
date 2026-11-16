from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
PREVIOUS=ROOT.parent/'wl2-r6-original-history-20261116-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,value):
    with (ROOT/name).open('x',encoding='utf-8') as out:json.dump(value,out,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
assert head!='b30cc913201f972d62b2d2b07a990fd5f5cded44'
runs={};inputs=[];rust=[];records=[];scenes=[];traces=[]
libraries=sorted(p.name for p in ROOT.glob('library[0-9][0-9]') if p.is_dir())
assert libraries
last=libraries[-1]
for name in libraries+['product01']:
    result=load(ROOT/name/'result.json')
    if name in [last,'product01']:assert result['exitCode']==0
    if name.startswith('library'):
        text=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig');m=re.search(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',text)
        if name==last:assert m and int(m.group(2))==0
        runs[name]=dict(rawExit=result['exitCode'],passed=int(m.group(1)) if m else None,failed=int(m.group(2)) if m else None,
            ignored=int(m.group(3)) if m else None,elapsedMs=result['elapsedMs'],profile=result['profile'],artifact=load(ROOT/f'{name}-artifact.json'))
        for line in text.splitlines():
            if 'r6 ' in line and len(line)<2200:traces.append(dict(run=name,trace=line[line.index('r6 '):]))
        for i,scene in enumerate(load(ROOT/f'{name}-private-scenes.json')['scenes']):
            assert sha(Path(scene['archive']).read_bytes())==scene['archiveSha256'];scenes.append(scene)
            for row in scene['files']:
                if scene['source'].startswith('D:') or any(marker in '/'+row['path'] for marker in ['/.ocxd024-observed-','/.ocxd024-history-']):
                    p=Path(scene['source'])/row['path'];raw=p.read_bytes();assert sha(raw)==row['sha256'] and len(raw)==row['bytes']
                    target=ROOT/'raw-scene-records'/name/f'scene-{i:03}'/row['path'];target.parent.mkdir(parents=True,exist_ok=True)
                    with target.open('xb') as out:out.write(raw)
                    records.append(dict(source=str(p),evidence=target.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
    else:runs[name]=dict(rawExit=result['exitCode'],profile=result['profile'],elapsedMs=result['elapsedMs'])
    for row in load(ROOT/name/'source-manifest.json'):
        raw=(ROOT/name/'executed-source'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
        if name=='product01' or name==last and row['path'].endswith('.rs'):
            assert raw==(CODE/row['path']).read_bytes(),row['path']
            assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n'),row['path']
            (inputs if name=='product01' else rust).append(row)
assert load(ROOT/'product01/source.json')['head']==head
native=load(ROOT/f'native-traces-{last}.json')['observations'];assert len(native)==8
assert {r['observation']['fault'] for r in native}=={0,*range(80,87)}
for row in load(ROOT/'source-scope.json')['changedFiles']:
    raw=(CODE/row['path']).read_bytes();assert sha(raw)==row['sha256'] and len(raw)==row['bytes']
write('handle-handoff-traces.json',traces);write('raw-scene-records.json',records)
write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,finalTestRust=rust,finalNativeRun=last,gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyModifiedOrStaged=True))
artifacts=[]
for name in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe';raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
write('product01-artifacts.json',artifacts)
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
matrix=load(PREVIOUS/'final-status.json')['matrix']
matrix.extend([
 dict(id='NativeOriginalHandleOwnershipHandoff',result='native-component-pass/private-production-qualification-pending',fact='Owns complete exact original trees before fallible validation; native original/history-source/stopped-anchor/initial/historical work Files and all4 original released-lease file/ancestor/bound ownership sets transfer into a retained exact close batch. Archive copies and retained retired-current Files remain separate. Same-chain immutable intent/current binds inventory and linked original-history/source/manifest/claim/action/native scope before any native closes.'),
 dict(id='ExplicitNativeOwnHandleClose',result='native-component-pass',fact='CloseHandle only consumes actual File.into_raw_handle ownership, each once. Success records ConfirmedClosed, error retains Unknown and raw error without recreating File/clearing protection/retry/reopening numeric values. Post-close original positions/ID/all8/bytes/children and private archives/retired collection read back before immutable result/current. Native scope has no production Source/Owner/typedClosed permit.'),
 dict(id='OriginalHandleCloseInterruptedPrefixes',result='native-component-pass/6-fault-cases',fact='Before close, first confirmed close, all closes before result, immutable result/current gap, one-byte current and actual own protected-handle close failure retain exact Held/ConfirmedClosed/Unknown inventory and canonical prefix. Original history/native objects and physical scenes preserved; poison cannot replay/catch-up. Residual Held files may close at test Drop and Unknown protection survives until test process exit; neither is canonical ordinary closure or cleanup success.'),
 dict(id='HistoryAfterOriginalHandleClose',result='native-component-pass/within-original-window',fact='After successful explicit actual client handle closure, synthetic successor config write succeeds. Independent retained old archives/retired objects and timely lease completion histories read back without comparing later live config to old originals. Foreign archive membership refuses and poisons while past current/events stay unchanged; foreign directory retained. Fresh expired-window receipt/actual process/supervisor/typedClosed/status bridge still pending.'),
])
cleanup=load(ROOT/'cleanup.json');counts=load(ROOT/'cleanup-counts.json');assert cleanup['removed']==counts['ownCompletedScenes'] and cleanup['failed']==0
failures=[dict(run=n,**r) for n,r in runs.items() if r['rawExit']!=0]
first=failures[0] if failures else dict(run=None,unexpectedError=None,fact='No unexpected test/compiler failure; deliberate protected native close and injected faults retain exact error outcomes without claiming ordinary success.')
if failures:
    first['fact']='library02 raw101/3pass2fail before close points at original10s; library03 raw101/4pass2fail at upstream original10s while close80/81/82 and history86 now passed. Both sets retained without catch-up. Consolidated duplicate orchestration while all raw comparison implementations/10s/assertions stayed unchanged; final library04 optimized10pass includes unchanged native recovery prefix tests. No timer/profile/assertion relaxation.'
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract='TEAM16/IMP142/r6',baselineMain='fac8573e55441fe514919c63c7f1d65181ff3577',baselineDocs='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,artifacts=artifacts,matrix=matrix,firstError=first,executionBoundary=load(ROOT/'execution-boundary.json'),allAccepted=False,startOpen=False,productionDispatcherUsesV3=False,actualSourceOrRecoveryFactoryExecuted=False,productionExecuted=False,officialOwnerCreated=False,componentNativeOriginalHandlesCloseVerified=True,productionOriginalHandleReleaseQualified=False,typedClosedQualified=False,W2=False,dualSID=False,published=False,cleanup=dict(**counts,removed=cleanup['removed'],historicalRejectedDeletionRetried=False,overallCleanupComplete=False),limits=load(ROOT/'next-production-requirements.json')['requirements'])
report=['# r6原件句柄显式关闭与历史所有权交接','',f'源码{head}/tree{tree}；承接b30cc9132/52b5dbc78，TEAM16/IMP142/r6。既定Windows写集，原目录/两稳定对象留位、all8/原预算/未知拒绝保持。','', '| 运行 | profile | raw退出 | 过/败/忽略 | ms |','| --- | --- | --- | --- | --- |']
for name,r in runs.items():report.append(f"| {name} | {r['profile']} | {r['rawExit']} | {r.get('passed','—')}/{r.get('failed','—')}/{r.get('ignored','—')} | {r['elapsedMs']} |")
report+=['', '完整原生关闭前，旧树、原件history源副本、Stopped anchor视图、初始/历史work observer和全部4原lease释放receipt的准确File/ancestor/bound对象按Rust所有权移入保留batch。归档副本和retired current对象继续单独保管，数据计划不构造File。原native original/完整history已验证后，同一canonical链/current先记含每准确ID/role/key的完整close inventory，再逐个实际CloseHandle。原路径/ID/all8/bytes/children与retired/archive后验成立后才result/current；生产scope尚未接，不签typedClosed。', '', 'native close仅消耗实际File.into_raw_handle，每个一次；成功ConfirmClosed，失败Unknown保留原raw error和未关闭前缀，既不从数值重建File也不清测试protect标志/重试/绕过。原生完整1、before-close/first-close/all-close/result-current/partial-current/protected-close6、历史外改拒绝1共8实际场景，准确inventory闭合数/held/unknown与事件/current前缀独立核验。自有protected句柄真实关闭失败不推断成功；测试scope结束对Held的Drop/进程退出关闭Unknown均不算canonical关闭，物理场景保留。', '', '完整关闭后准确旧client不再被捕获/work/lease clone保持，合成后继配置可写；保管的旧原件副本/原Stopped快照/retired集合与原期限内lease完成历史独立读回，不拿新live字节重写旧历史。外来归档目录拒绝并poison，过去canonical事实不补写或涂改。后续需严格全部inventory角色/数量约束、实际native history引用与canonical逐字接合、生产typedSource/Owner/NormalExit/Supervisor/content资格和原期限完成receipt，再typedClosed/Status5/管理器交接/后继。', '', 'Start保持无条件拒绝，原缺席Codex安全初始化/退根独立blocked；first-use/ownerreceipt、普通MSI/CAP-trust/历史清理未关闭。无旧raw5/空GUI/相同1314/W2/双SID/发布或官方/日常配置/机器策略更改。完整继承状态与首错归final-status.json；历史失败不改通过。', '', f'本轮{len(scenes)}场景私有ZIP完整核验；{cleanup["removed"]}个完成纯数据scene核fresh准确路径/全部文件hash/目录membership清理，{counts["failedAndPendingRetained"]}失败/pending及{counts["crossVolumePendingRootsRetained"]}个D根保留。{len(records)}份canonical/快照/history原字节另归raw-scene-records。ZIP非NTFS身份/安全克隆，历史删除拒绝不重试，overallCleanupComplete=false。', '', '| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
report+=['','首实际非预期失败为library02优化轮raw101/3过2败，后继library03 raw101/4过2败：两轮均在原10秒的上游或交接准备窗口超时，未到相应关闭点，原EXE/source/事件/current/物理场景均保留。新交接及恢复编排中相邻位置重复整树扫描收敛，关闭实际native历史引用与canonical精确链接；所有原生比较实现、双树actual move前后、原语affected-tree前后、原journal/claim/预算与旧fault断言保持，验证归barrier-ownership/recovery-barrier文件。最终library04原优化参数10过（含受影响旧恢复前缀），未延长/重开计时或删除断言。编辑、取证组内多scene和PowerShell JSON大小写诊断失败各独列，不覆盖原始负例。']
status['handoffBarrierOwnership']=load(ROOT/'barrier-ownership-verification.json')
status['recoveryBarrierOwnership']=load(ROOT/'recovery-barrier-verification.json')
status['failuresPreserved']=[r['run'] for r in failures]
write('final-status.json',status)
with (ROOT/'report.md').open('x',encoding='utf-8') as out:out.write('\n'.join(report)+'\n')
print(json.dumps(dict(head=head,runs={k:v['rawExit'] for k,v in runs.items()},productInputs=len(inputs),testRust=len(rust),rawRecords=len(records),scenes=len(scenes),removed=cleanup['removed'],production=False)))
