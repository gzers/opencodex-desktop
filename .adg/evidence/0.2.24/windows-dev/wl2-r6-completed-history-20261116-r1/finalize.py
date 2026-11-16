from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=ROOT.parent/'wl2-r6-handle-handoff-20261116-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,value):
    with (ROOT/name).open('x',encoding='utf-8') as out:json.dump(value,out,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
assert head!='d20103fee084a3e63f728bf7c6664af65405e456'
libraries=sorted(p.name for p in ROOT.glob('library[0-9][0-9]') if p.is_dir());last=libraries[-1]
runs={};scenes=[];records=[];inputs=[];rust=[];traces=[]
for name in libraries+['product01']:
    result=load(ROOT/name/'result.json')
    if name in [last,'product01']:assert result['exitCode']==0
    if name.startswith('library'):
        text=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig');m=re.search(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',text)
        if name==last:assert m and int(m.group(2))==0
        runs[name]=dict(rawExit=result['exitCode'],profile=result['profile'],elapsedMs=result['elapsedMs'],passed=int(m.group(1)) if m else None,
            failed=int(m.group(2)) if m else None,ignored=int(m.group(3)) if m else None,artifact=load(ROOT/f'{name}-artifact.json'))
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
native=load(ROOT/f'native-traces-{last}.json')['observations'];assert len(native)==13
assert {r['observation']['case'] for r in native}=={0,80,81,83,84,85,87,90,91,92,93,94,95}
for row in load(ROOT/'source-scope.json')['changedFiles']:
    raw=(CODE/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,finalTestRust=rust,gitLineEndingEquivalent=True,
    preexistingSchemaUntrackedNotManuallyEditedOrStaged=True,finalNativeRun=last))
write('completed-history-traces.json',traces);write('raw-scene-records.json',records)
artifacts=[]
for name in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe';raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
write('product01-artifacts.json',artifacts)
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
matrix=load(BASE/'final-status.json')['matrix']
matrix.extend([
 dict(id='TimelyOriginalTransactionCompletion',result='native-component-pass/private-production-qualification-pending',fact='Private nonserde nonClone receipt captured only after actual exact original-handle closes, immutable result/current and final original/retired/archive/full journal postconditions, inside unchanged original deadline. Bound exact session/manifest/binding/plan/action/claim/epoch/currentID/sequence/tip/current bytes/recovery/history/handle summaries. Native-close timestamp alone, late/partial/unknown/poison never mints receipt.'),
 dict(id='FreshReadAfterOriginalWindowExpiry',result='native-component-pass/private-entry-not-status-wired',fact='Wait until actual original10s expiry after timely transaction completion; new caller-owned same-sample when/end max5s permits exact receipt/journal/token/peer/pins/currentID/all8/raw/membership/archive/retired/native lease-completion readback only. No new deadline stored or old mutation end updated. Later synthetic live config remains separate from old immutable history. Actual expired append still refuses at original gate without publication or catch-up.'),
 dict(id='CompletedHistoryUnknownAndTamper',result='native-component-pass/12-negative-cases',fact='Missing receipt for before/partial/immutable-result-current gap/one-byte current/actual protected close/late final completion. Context digest drift, foreign archive directory, over5s/future/expired read windows, actual current truncation all refuse and preserve canonical/physical prefix; poison never becomes success. Case87 native closure and canonical result succeeded but final timely receipt was deliberately late, so completed history still refuses.'),
 dict(id='OriginalMutationCutoffAndFullRecordChecks',result='byte-equivalence-check-and-native-regression-pass',fact='Shared journal-native comparison body byte-identical after extraction into private common function. Mutation callers retain original pending-action/absolute-end guard; common record comparisons callable only from original guard or private receipt-gated historical entry. Eight native files and all raw all8/identity/content/children/retired/archive/close predicates unchanged. No synthetic receipt from JSON or generic end setter.'),
])
cleanup=load(ROOT/'cleanup.json');counts=load(ROOT/'cleanup-counts.json');assert cleanup['removed']==counts['ownCompletedScenes'] and cleanup['failed']==0
failures=[dict(run=name,**r) for name,r in runs.items() if r['rawExit']!=0]
first=failures[0] if failures else dict(run=None,unexpectedError=None,fact='No unexpected native/compiler error. Deliberate transaction87 late receipt, exact native unknown/partial close and tamper/window refusals are expected negatives, not ordinary success. Earlier failed runs retained in predecessor evidence.')
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract='TEAM16/IMP142/r6',
    fetchedMain='fac8573e55441fe514919c63c7f1d65181ff3577',fetchedGovernanceMain='5782307174f0d1b011b5415c6c1e8d60414e6b36',
    networkRecoveryEvidence='0182e060d72ed08ed3355e238307bba493b714e9',runs=runs,artifacts=artifacts,matrix=matrix,firstError=first,
    guards=load(ROOT/'guard-verification.json'),allAccepted=False,startOpen=False,typedClosedQualified=False,productionExecuted=False,
    productionDispatcherUsesV3=False,productionStatusSampleJoined=False,actualSourceOrRecoveryFactoryExecuted=False,officialOwnerCreated=False,
    W2=False,dualSID=False,published=False,cleanup=dict(**counts,removed=cleanup['removed'],historicalRejectedDeletionRetried=False,overallCleanupComplete=False),
    limits=load(ROOT/'next-production-requirements.json')['requirements'])
write('final-status.json',status)
report=['# r6原窗及时完成receipt与跨动作只读历史','',f'源码{head}/tree{tree}；实际fetch fac8573e5/578230717并消费TEAM16/IMP142/r6，网络恢复证据0182e060d已推。代码基线d20103fee，旧证据7d0d567e1及旧失败保持。仅Windows允许集。','', '| 运行 | profile | raw退出 | 过/败/忽略 | ms |','| --- | --- | --- | --- | --- |']
for name,r in runs.items():report.append(f"| {name} | {r['profile']} | {r['rawExit']} | {r.get('passed','—')}/{r.get('failed','—')}/{r.get('ignored','—')} | {r['elapsedMs']} |")
report+=['', '原生关闭时点不能代表交易最终完成。新CompletedOriginals私有非serde/nonClone，只在实际187句柄关闭、canonical结果/current及最后整树/归档/retired/journal后验全部成功后、原截止前构造。receipt固定原original_end/completed_at/current文件ID和精确session/manifest/binding/storagePlan/action/claim/epoch/sequence/tip/current raw与recovery/history/handle摘要；不存在从JSON/布尔/路径构造或修改旧deadline入口。', '', '跨动作只读入口要求实际保管receipt与同调用者when/end最大5秒；原10秒截止过后，仍可全量核准确token/peer/原pins/current ID/all8/bytes、全部immutable记录/membership、私有归档/retired对象和原及时lease完成历史。比较体由旧check_inner逐字提取，guard-verification核完整旧原生比较体字节相同；旧append/lease/恢复/变更仍先检查原authorized_end，fresh读成功不授过期mutation。只读失败poison并保持原current/事件前缀，不补写或自动catch-up。', '', '13原生场景：及时完成后真实等原窗口过期，新5秒读回成功，随后actual expired append在原门拒绝；缺receipt的6场景（before/first-close/result-current/partial current/protected-close/最终receipt迟到）；新窗6拒绝场景（binding漂移、真实foreign archive、超5秒、明确未来、已过期窗口、实际current截成1字节）。晚receipt场景即使原native closes和canonical结果已发布仍不补签。后来live config变化不当旧原件baseline；外来目录/破坏current现场原样保留。测试等待仅观察已固定原截止，没有开新mutation计时。', '', '当前为组件历史证明，完成receipt并非生产typedClosed或完整内容资格。生产Status还须传共享同次when/end并接actual Source/Owner/current；actual OriginalAction/NormalExit/Supervisor history、contentpolicy在current转移前的routes/PID/audit原窗、准确完整inventory角色、生产恢复/Closed/manager交接和后继都仍待。原first-use/ownerreceipt、原缺席Codex安全退根、普通MSI/CAP-trust/旧Profile/Public/失败broker清理未关闭；Start无条件拒绝。无官方/日常配置/token/策略改变、旧raw5/空GUI/相同1314/W2/双SID/发布。', '', f'本轮{len(scenes)}场景完整私有ZIP核验，{cleanup["removed"]}完成纯数据scene核fresh准确路径/全文件hash/目录membership后清理；{counts["failedAndPendingRetained"]}失败/pending及{counts["crossVolumePendingRootsRetained"]}个D根保留。{len(records)}份canonical/快照/history原字节另归raw-scene-records；ZIP非NTFS身份/安全克隆，历史拒绝删除不重试，overallCleanupComplete=false。完整继承状态/首错和实际制品归final-status。', '', '| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
with (ROOT/'report.md').open('x',encoding='utf-8') as out:out.write('\n'.join(report)+'\n')
print(json.dumps(dict(head=head,runs={k:v['rawExit'] for k,v in runs.items()},productInputs=len(inputs),testRust=len(rust),nativeCases=len(native),scenes=len(scenes),rawRecords=len(records),removed=cleanup['removed'],production=False)))
