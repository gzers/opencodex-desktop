from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
PREVIOUS=ROOT.parent/'wl2-r6-completed-history-20261116-r1'
ACCEPTED=ROOT.parent/'wl2-r6-handle-handoff-20261116-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,value):
    with (ROOT/name).open('x',encoding='utf-8') as out:json.dump(value,out,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip();assert head!='d20103fee084a3e63f728bf7c6664af65405e456'
libraries=sorted(p.name for p in ROOT.glob('library[0-9][0-9]') if p.is_dir());last=libraries[-1]
runs={};inputs=[];rust=[];records=[];scenes=[];traces=[]
for name in libraries+['product01']:
    result=load(ROOT/name/'result.json');rc=result['exitCode']
    if name in [last,'product01']:assert rc==0
    if name.startswith('library'):
        text=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig');m=re.search(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',text)
        if name==last:assert m and int(m.group(2))==0
        runs[name]=dict(rawExit=rc,profile=result['profile'],elapsedMs=result['elapsedMs'],passed=int(m.group(1)) if m else None,
            failed=int(m.group(2)) if m else None,ignored=int(m.group(3)) if m else None,artifact=load(ROOT/f'{name}-artifact.json'))
        for line in text.splitlines():
            if 'r6 ' in line and len(line)<2400:traces.append(dict(run=name,trace=line[line.index('r6 '):]))
        for i,scene in enumerate(load(ROOT/f'{name}-private-scenes.json')['scenes']):
            assert sha(Path(scene['archive']).read_bytes())==scene['archiveSha256'];scenes.append(scene)
            for row in scene['files']:
                if scene['source'].startswith('D:') or any(marker in '/'+row['path'] for marker in ['/.ocxd024-observed-','/.ocxd024-history-']):
                    p=Path(scene['source'])/row['path'];raw=p.read_bytes();assert sha(raw)==row['sha256'] and len(raw)==row['bytes']
                    target=ROOT/'raw-scene-records'/name/f'scene-{i:03}'/row['path'];target.parent.mkdir(parents=True,exist_ok=True)
                    with target.open('xb') as out:out.write(raw)
                    records.append(dict(source=str(p),evidence=target.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
    else:runs[name]=dict(rawExit=rc,profile=result['profile'],elapsedMs=result['elapsedMs'])
    for row in load(ROOT/name/'source-manifest.json'):
        raw=(ROOT/name/'executed-source'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
        if name=='product01' or name==last and row['path'].endswith('.rs'):
            assert raw==(CODE/row['path']).read_bytes(),row['path']
            assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n'),row['path']
            (inputs if name=='product01' else rust).append(row)
assert load(ROOT/'product01/source.json')['head']==head
for row in load(ROOT/'source-scope.json')['changedFiles']:
    raw=(CODE/row['path']).read_bytes();assert sha(raw)==row['sha256'] and len(raw)==row['bytes']
batch_observations=load(ROOT/f'native-traces-{last}.json')['observations']
assert len(batch_observations)==32 and sum(r['kind']=='complete' for r in batch_observations)==1
assert sum(r['kind']=='initial' for r in batch_observations)==14 and sum(r['kind']=='recovery' for r in batch_observations)==17
completed=load(ROOT/f'completed-native-traces-{last}.json')['observations'];assert len(completed)==13
assert {r['observation']['case'] for r in completed}=={0,80,81,83,84,85,87,90,91,92,93,94,95}
write('batch-native-traces.json',traces);write('raw-scene-records.json',records)
write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,finalTestRust=rust,priorCandidateTree='958c0deb01aa56c43dda74c68cbca05c31b96884',
    priorSourceBase='d20103fee084a3e63f728bf7c6664af65405e456',priorCandidateBudgetFailuresRetained=True,gitLineEndingEquivalent=True,
    preexistingSchemaNotManuallyModifiedOrStaged=True))
artifacts=[]
for name in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe';raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
write('product01-artifacts.json',artifacts)
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
matrix=load(ACCEPTED/'final-status.json')['matrix']
matrix.extend([
 dict(id='BoundedBatchCanonicalRowData',result='directed-data/native-component-pass',fact='One envelope=one physical event/current sequence, nonrecursive rows carry explicit strict ordinal and exact existing single-node fields bound phase parent/sequence. Complete ordered node coverage and plan digest; intent/prefix/anchor phase operations strict. Each row reuses existing validators on isolated native-free DATA projection; late-row error is atomic. Pending batch blocks all old node/lease/work/snapshot/action/history/handle paths.1MiB/512 and4KiB limits unchanged.'),
 dict(id='NativeBatchOriginalAndCurrentOwnership',result='native-component-pass/production-capture-compiled',fact='Actual initial-tree/recovery borrowed witnesses keep original exact handles/claim/NLK/native all8/ID/content/children/archive predicates before/after every native operation. Full intent phase/current durable before first mutation. Completion requires retained VerifiedBatchValue from exact native readback, no count/JSON factory. Complete initial original move/work-create and current retirement/original return plus released-anchor observations share same canonical chain. CaptureSlot initial prepare consumes batched lane; Source/typedReady/Stop/content admission gates remain unexecuted.'),
 dict(id='BatchInterruptedNativePrefixes',result='native-component-pass/31-negative-scenes',fact='14 initial move/create scenes and17 current-retire/original-restore scenes cover0..N actual steps, immutable-result-before-current/actual1bytecurrent and partial durable exact result prefix=1. Actual native prefix independently read against originals/live/work/retired paths and IDs; canonical remains pending/poison, no catch-up. Wrong targets, unknown directory, ADS/link/security constraints preserve prior raw implementations/assertions.'),
 dict(id='BatchedWholeTransactionOriginalBudget',result='native-component-pass/original10s',fact='Complete captured roots/three independent phases/current atomic replacement/four-domain lease segments,6work retirement/3original return/final fulltree/anchors explicit release/original history/187client-handle close/timely completion finish within original fixture cutoff. Physical event files reduce77->56 with explicit same sequence semantics; complete final assertion56. Prior single-node31pass6failCPU diagnostic remains unchanged failed evidence, not retroactively accepted.'),
 dict(id='CompletedHistoryAfterOriginalExpiry',result='native-component-pass/private-production-entry-pending',fact='13 actual new history scenes now reached: timely completion then original expiry/fresh same caller5s read success while expired append refuses;6 missing/partial/unknown/late native receipt cases and6 context/foreign/current/window negatives. Exact nonserde receipt/current ID/raw/tip/native history bound; no generic timer setter, no textClosed or mutation/reacquire authority. Actual production Status/Source/Owner/normal-exit/supervisor qualification still pending.'),
])
matrix.extend([
 dict(id='ImmutablePlanDataDigestReuse',result='data-counterexamples-pass/no-native-observation-cached',fact='Two private immutable plan DATA objects retain nonserialized OnceLock digest only. Every new parsed plan starts empty, changed serialized bytes hash differently and injected dataDigest input refuses. Current recovery summary binds whole-plan digest plus every mutable retired/restored/pending/whole-tree/anchor state. All native ID/security/content/children/archive checks remain independent and unchanged; no cached native fact.'),
 dict(id='ImmutableCanonicalTypedRecordReplay',result='data-native-byte-test-pass/no-replay-rule-skipped',fact='Immutable phase creation retains exactly the owned typed event whose encoded bytes were passed to native Record::create. Every observation still reopens/checks exact native ID/all8/raw bytes/ADS/membership. New append still independently replays every progression rule from original manifest; skips only repeated decoding of locally immutable DATA. No mutable/public setter, no old record adoption. Test proves encoded typed bytes=actual immutable raw, other writer refused and replay state/tip identical.'),
])
prior_full=[r['observation']['durable'] for r in load(ACCEPTED/'native-traces-library04.json')['observations'] if r['observation']['fault']==0]
assert len(prior_full)==1 and prior_full[0]['sequence']==88
complete_batch=[r['observation']['batch'] for r in batch_observations if r['kind']=='complete'];assert len(complete_batch)==1
write('physical-phase-comparison.json',dict(priorSource='d20103fee084a3e63f728bf7c6664af65405e456',priorEvidence='7d0d567e19b7b3cbc799dcd85ec3a43ad4fca0f4',
    priorActualFinalSequence=88,newActualPhysicalPhaseFiles=complete_batch[0]['physicalEventFiles'],newAssertedFinalSequence=56,
    samePhysicalSequenceMeaning=True,rowOrdinalsAreNotAdditionalPhysicalSequences=True,notOldFailedRunRelabelledPass=True))
for item in matrix:
    if item['id']=='BatchedWholeTransactionOriginalBudget':item['fact']=item['fact'].replace('reduce77->56','reduce88->56')
cleanup=load(ROOT/'cleanup.json');counts=load(ROOT/'cleanup-counts.json');assert cleanup['removed']==counts['ownCompletedScenes'] and cleanup['failed']==0
failures=[dict(run=name,**r) for name,r in runs.items() if r['rawExit']!=0]
first=failures[0] if failures else None
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract='TEAM16/IMP142/r6',fetchedMain='fac8573e55441fe514919c63c7f1d65181ff3577',
    fetchedGovernanceMain='5782307174f0d1b011b5415c6c1e8d60414e6b36',priorBudgetFailureEvidence='ac622c2d8aae4cdcb5b2e8dfd400a49fec50fdcb',
    runs=runs,artifacts=artifacts,matrix=matrix,firstError=first,invariants=load(ROOT/'invariant-verification.json'),
    allAccepted=False,startOpen=False,productionExecuted=False,typedClosedQualified=False,productionStatusSampleJoined=False,
    actualSourceOrRecoveryFactoryExecuted=False,officialOwnerCreated=False,W2=False,dualSID=False,published=False,
    cleanup=dict(**counts,removed=cleanup['removed'],historicalRejectedDeletionRetried=False,overallCleanupComplete=False),limits=load(ROOT/'next-production-requirements.json')['requirements'])
status['pureDataDigestProof']=load(ROOT/'data-digest-verification.json')
status['typedImmutableRecordProof']=load(ROOT/'typed-record-verification.json')
status['finalIoBoundaryProof']=load(ROOT/'final-io-boundary-verification.json')
status['nativeNegativeCoverageExactRequestedFaultsReached']=True
status['recordCachingNativeComparisonsSkipped']=False
write('final-status.json',status)
report=['# r6单phase有界节点批记录与原预算闭环组件','',f'源码{head}/tree{tree}；实际fetch fac8573e5/578230717、TEAM16/IMP142/r6。承接d20103fee及本地候选958c0deb，旧预算失败/CPU证据ac622c2d8原样保留。本轮仍Windows既定写集，原预算/安全保证/未知拒绝不变。','', '| 运行 | profile | raw退出 | 过/败/忽略 | ms |','| --- | --- | --- | --- | --- |']
for name,r in runs.items():report.append(f"| {name} | {r['profile']} | {r['rawExit']} | {r.get('passed','—')}/{r.get('failed','—')}/{r.get('ignored','—')} | {r['elapsedMs']} |")
report+=['', '批封套非递归、最多512行/1MiB，仍同一个manifest/current/不可覆盖phase链。每批只占一个phase.sequence；内部row用显式连续ordinal，row原event.sequence/parent等于该phase.sequence/parent，保留原eventId/node/root/identity/all8/parent/from-to/expected-state/claim/epoch/原action scope。完整ordered plan正好覆盖操作所有节点；每行经未放宽原单node/recovery validator，在隔离DATA projection全部通过才一次更新Progress，晚行错误不泄漏部分进度。current仅batchId/kind/planDigest/count/prefix，4KiB不变。旧失败不转换或catch-up。', '', '真实native驱动只借准确原两树与活claim。初始original move/work create、Stopped current6work退休/3original准确返回，每组完整意图phase/current先刷盘，再逐个samehandle native操作；每步仍完整BOTH-tree before/after，底层affected-tree比较不变，保留实际native VerifiedBatchValue。批结果仅使用这些真实保管值与原逐节点校验；actual file/security/ID不从JSON/count构造。释放后所有原anchor以独立row实际读回封到单phase，未移动原根/子目录/两稳定对象；原完整Restored/release/历史/close条件不缩。', '', '31负例为初始14与恢复17，覆盖两个原3file操作各0..3 native前缀、current退休0..6及还原0..3、结果不可变记录领先current/真实1字节current，并实际持久化prefix1后中断。canonical完整intent保持pending、准确native对象位置/ID/bytes/all8/children与retired/custody全保留，phase/current失败poison不补结果。新phase plan/ordinal/scope/错根/缺row/duplicate/nesting/重算plan摘要仍错native-parent/新work别名反例拒绝原子性。原raw变更/security及旧测试断言核字节等价，不以批处理豁免ADS/hardlink/reparse/未知节点。', '', '完整新native交易断言56物理event文件（原逐节点77），phase序列仍为物理文件序号；批row不冒物理单事件。原三阶段/四域lease/current替换、恢复/显式稳定range释放/原件不可变history/187准确句柄关闭/及时完成receipt均沿原10秒fixture，不重开/延长期限。原预算失败候选只改后新source实际通过，不追溯改旧失败；编译profile参数未改。', '', '跨原窗history13实际场景也已全部执行：及时完成后真实等原截止过期，新调用者same-sample最大5秒只读通过、actual expired append仍原门拒绝；before/partial/unknown/result-current/1byte/最终完成迟到无receipt，source binding漂移/真实foreign archive/超窗/未来/过期/实际current破坏均拒绝。完整journal原比较体仅加batch摘要比较；非serde完成对象绑定准确current ID/tip/原action/history/raw。此为native组件，不当生产Status/typedClosed/Source/Owner资格。', '', '首library02 E0603/raw101为BatchKind Windows内部可见性，未运行测试/无新EXE，修正精确导出后data回归过。工具级补丁格式与invariant空白匹配失败独列，原断言/输入未被改松。CaptureSlot prepare新批lane编译接入，真实Source/Ready/Stop尚未调用。旧单node测试路径仅cfg(test)保留原序号与故障断言，新端到端用例消费批witness；不会把旧测试都改成新布局以掩盖差异。', '', '实际production接线仍需original routes/PID/audit home/原时间窗/独立四阶段原sharedpolicy在current转移前准入，actual Source/Owner/NormalExit/Supervisor/StopAccepted/current/listener/完整inventory角色及typedClosed/Status5/后继/普通desktop-H-S-D证据。原first-use/ownerreceipt、原缺席安全退根、普通MSI/CAP-trust/历史清理继续未完。Start无条件拒绝，不旧raw5/空GUI/相同1314/W2/双SID/发布/官方或日常配置策略改变。', '', f'本轮{len(scenes)}场景完整私有ZIP核验，{cleanup["removed"]}完成纯数据scene核fresh准确路径/全文件hash/目录membership后清理；{counts["failedAndPendingRetained"]}失败/pending及{counts["crossVolumePendingRootsRetained"]}个D根保留。{len(records)}份canonical/快照/history原字节另归raw-scene-records。ZIP非NTFS元数据克隆，历史删除拒绝不重试，overallCleanupComplete=false。完整继承矩阵/原退出码/source/实际EXE与局限归final-status。', '', '| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
rendered='\n'.join(report)+'\n'
rendered=rendered.replace('（原逐节点77）','（原逐节点88；实际旧trace核对）')
rendered+='\n前轮library04优化39过3败、library06优化41过2败仍未通过完整原预算，原EXE/source/raw/current及失败现场完整保留。仅批记录尚不足以收口；后继复用私有不可变计划DATA摘要及本地不可变typed事件以减少重复编码/hash/untagged解码，全部native记录/原件/归档比较及独立进度规则仍运行。摘要cache不序列化、计划无修改入口，changed DATA/注入cache字段反例均过；新current进度摘要用planDigest绑定原完整计划与全部mutable/pending/anchor状态。typed entry与原create的encoded(event)同值，磁盘原native ID/all8/raw/ADS/membership检查体逐字不变，不省略native检查。最终新源码通过才接受本轮，不追溯改旧失败。验证细节归invariant/data-digest/typed-record三份证明。\n'
rendered+='\nlibrary08为43过1败，离线completed verifier正确指出case84实际未到1字节current，泛化失败断言曾让上游超时满足“无receipt”；原结果保留但不冒覆盖。library09加强全部指定负例的exact native prefix/error/1bytecurrent/late状态断言，同时把新history/close相邻无变更的重复journal读回交给原append完整BEFORE/AFTER边界。关闭前后及结果后三次原路径后验、native archive BOTH-tree边界、原receipt均保留，final-io-boundary证明独列；最终原优化profile44过0败/45个完整native trace才证明本轮。旧原件/安全/字节/children/claim/期限/故障断言不删改，新增断言加强，旧失败不追溯改成功。\n'
with (ROOT/'report.md').open('x',encoding='utf-8') as out:out.write(rendered)
print(json.dumps(dict(head=head,runs={k:v['rawExit'] for k,v in runs.items()},productInputs=len(inputs),testRust=len(rust),batchNativeCases=len(batch_observations),
    completedHistoryCases=len(completed),scenes=len(scenes),rawRecords=len(records),removed=cleanup['removed'],production=False)))
