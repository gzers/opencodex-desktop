from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=ROOT.parent/'wl2-r6-bounded-node-batch-20261116-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,value):
    with (ROOT/name).open('x',encoding='utf-8') as out:json.dump(value,out,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
assert head!='c259886cf5723409a1c77d41d1f9b42196bd6b61'
runs={};scenes=[];records=[];inputs=[];rust=[];traces=[]
for name,rc,passed,failed in [('library01',101,None,None),('library02',101,45,1),('library03',0,46,0),('library04',0,47,0),('product01',0,None,None)]:
    result=load(ROOT/name/'result.json');assert result['exitCode']==rc
    if name.startswith('library'):
        text=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig');m=re.search(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',text)
        if passed is not None:assert m and tuple(map(int,m.groups()))==(passed,failed,0)
        runs[name]=dict(rawExit=rc,profile=result['profile'],elapsedMs=result['elapsedMs'],passed=passed,failed=failed,ignored=0 if passed is not None else None,artifact=load(ROOT/f'{name}-artifact.json'))
        for line in text.splitlines():
            if 'r6 ' in line and len(line)<2400:traces.append(dict(run=name,trace=line[line.index('r6 '):]))
        for i,scene in enumerate(load(ROOT/f'{name}-private-scenes.json')['scenes']):
            assert sha(Path(scene['archive']).read_bytes())==scene['archiveSha256'];scenes.append(scene)
            for row in scene['files']:
                if scene['source'].startswith('D:') or any(marker in '/'+row['path'] for marker in ['/.ocxd024-observed-','/.ocxd024-history-']):
                    p=Path(scene['source'])/row['path'];raw=p.read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
                    target=ROOT/'raw-scene-records'/name/f'scene-{i:03}'/row['path'];target.parent.mkdir(parents=True,exist_ok=True)
                    with target.open('xb') as out:out.write(raw)
                    records.append(dict(source=str(p),evidence=target.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
    else:runs[name]=dict(rawExit=rc,profile=result['profile'],elapsedMs=result['elapsedMs'])
    for row in load(ROOT/name/'source-manifest.json'):
        raw=(ROOT/name/'executed-source'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
        if name=='product01' or name=='library04' and row['path'].endswith('.rs'):
            assert raw==(CODE/row['path']).read_bytes(),row['path']
            assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n'),row['path']
            (inputs if name=='product01' else rust).append(row)
assert load(ROOT/'product01/source.json')['head']==head
for row in load(ROOT/'source-scope.json')['changedFiles']:
    raw=(CODE/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
observation=load(ROOT/'actual-observations-library04.json');assert len(observation['fixedBunProjectionRuns'])==1 and len(observation['sourcePinNegatives'])==1
write('production-content-traces.json',traces);write('raw-scene-records.json',records)
write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,finalTestRust=rust,gitLineEndingEquivalent=True,
    actualProductionMethodsCompiled=True,actualProductionOwnerContextExecuted=False,preexistingSchemaNotManuallyEditedOrStaged=True))
artifacts=[]
for name in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe';raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
write('product01-artifacts.json',artifacts)
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
matrix=load(BASE/'final-status.json')['matrix']
matrix.extend([
 dict(id='OriginalNativeContentScopeFactory',result='compiled-not-executed/production-entry-internal',fact='Capture original actual unresumed Owner/Source/StartAction, exact root process generation/native FILETIME/Profile/H and immutable original routes/config pin; retain4fixed official audit-source pins with exact bytes/native identity/all8. Fresh actual typed StopAction and NormalExit original Job family produce unambiguous actual process PID+creation generations and original audit interval. No caller PID/home/time/serde/Clone/public-fields constructor. Ordinary desktop actual Owner not created/executed in this round.'),
 dict(id='FixedOfficialAuditProjection',result='fixed-Bun-pure-library-component-pass/validated-input-domain-only',fact='Fixed bundle and inner official tgz4source bytes exact. Actual original Bun executes only redactUserPath(path).slice(0,256):8supported ASCII/Unicode/backslash/slash/custom paths match exactly;4sensitive/token/truncating inputs refuse rather than guess. Explicit ASCII prefix case fold matches official long-s edge. File source pin denies actual other-writer/atomic replacement and wrong hash on new synthetic files. No service/official data/network/model requests.'),
 dict(id='TypedContentBeforeNativeFrameTransfer',result='compiled-native-factory/ordering-verified-not-production-positive',fact='ProductionStop binds actual Source/Owner/NormalExit/SupervisorAbsent/StopAccepted/routes/action/current/listener and canonical unique Ready/PreStop/Stopped. Unchanged shared policy receives original+3actual native archives under complete BOTH-tree before/after original/custody/historical/archive/current barriers BEFORE capture_retirement transfers current frame; private assessment bound scope+series+deadline retained in core before transfer. Old unassessed preview is cfg(test) only. No synthetic production scope/assessment factory.'),
 dict(id='ActualStoppedRestoreHistoryLane',result='compiled/internal-registered-not-invoked',fact='CaptureSlot new restore_stopped_existing validates exact original ready tuple/StopEnd, gets actual stopped scope and full current lease/three stage observation, then qualified batch retirement/return/fulltree/final explicit release/history under guarded before/after actual typed proofs and original absolute window. Failure poisons retained core and lease cleanup remains independent. Returns only internal restored_history, not ClosedEligibility/ordinary Stop success; handle closure/actual process-supervisor histories/dispatcher/Status still incomplete.'),
])
cleanup=load(ROOT/'cleanup.json');counts=load(ROOT/'cleanup-counts.json');assert cleanup['removed']==3 and cleanup['failed']==0
first=dict(run='library01',rawExit=101,kind='MSVC E0603 module production private',newExecutableAttributed=False,
    correction='Reexport exact ProductionStop type through its parent journal without making fields/constructors serde-accessible. Static literal1[USER] outer-template mistake fixed with explicit capture closures before projection test executed.',
    laterFailure=dict(run='library02',rawExit=101,passed=45,failed=1,kind='test source root contained dot-dot path; original native local-path guard correctly refused',
        correction='Canonicalize actual test bundle root; do not relax source pin/path checks. Add ASCII-prefix Unicode parity counterexamples. library03 raw0/46, finalsource library04 raw0/47.'))
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract='TEAM16/IMP142/r6',
    fetchedMain='fac8573e55441fe514919c63c7f1d65181ff3577',fetchedGovernanceMain='5782307174f0d1b011b5415c6c1e8d60414e6b36',
    runs=runs,artifacts=artifacts,matrix=matrix,firstError=first,productionBoundary=load(ROOT/'production-boundary-verification.json'),
    fixedAuditSources=load(ROOT/'fixed-audit-source-manifest.json'),allAccepted=False,startOpen=False,actualProductionOwnerContextExecuted=False,
    actualSourceOrRecoveryFactoryExecuted=False,productionDispatcherUsesV3=False,actualProductionRestoreExecuted=False,productionStatusSampleJoined=False,
    typedClosedQualified=False,officialOwnerCreated=False,W2=False,dualSID=False,published=False,
    cleanup=dict(**counts,removed=3,historicalRejectedDeletionRetried=False,overallCleanupComplete=False),limits=load(ROOT/'next-closed-requirements.json')['requirements'])
write('final-status.json',status)
report=['# r6原内容上下文与typed Stop恢复入口','',f'源码{head}/tree{tree}，承接c259886cf/1228486fe；fetch实际main fac8573e5、治理578230717，TEAM16/IMP142/r6不变。本轮仅Windows允许集。','', '| 运行 | profile | raw退出 | 过/败/忽略 | ms |','| --- | --- | --- | --- | --- |']
for name,r in runs.items():report.append(f"| {name} | {r['profile']} | {r['rawExit']} | {r.get('passed','—')}/{r.get('failed','—')}/{r.get('ignored','—')} | {r['elapsedMs']} |")
report+=['', '首library01 E0603/raw101为production guard模块可见性，无新EXE/未测试；按父模块重导出精确类型修复。静态发现外层模板把预期捕获替换写成1[USER]，在测试前以caps[1]闭包纠正，原期望保持。library02编译后45过1败：测试bundle root包含../../..，被原native路径检查正确拒绝；仅canonicalize实际测试根，不改生产路径断言。library03修正后46过；最终补source pin真实写/替换/错hash负例及旧unqualified preview测试限定后library04 47过。所有错误/实际执行source/参数/raw/stdout/stderr/EXE摘要保留。', '', '新增实际原上下文factory：原Source/Owner/未resume/Start动作成立时capture，保管准确original根进程代次、native创建FILETIME转UTC毫秒、Profile/H、原routes/config native pin及4固定官方审计源码准确hash/ID/all8；Stop只从原Owner获取原Stop/Restart子窗action，需actual NormalExit重核原Job全家族，PID+creation去歧义，不接受caller PID/home/time数组/JSON/boolean/Clone。notBefore来自原native根创建时间毫秒floor、notAfter仅在实际停止族重核后取得；不是新的mutation deadline，窗口/未知拒绝保持。', '', '固定HOME audit投影只在已验证identity-redactor域准入：实际固定Bun115MB/原sha只调用原官方redactUserPath(path).slice(0,256)纯库，对8支持路径逐字相同，含C:/、反斜线、自定义根、中文用户名、Unicode长s前缀不误折叠；4敏感/token/截断路径明确拒绝。源码4份逐字对固定bundle和内tgz，源码位置/hashes原件单列，未改官方。合成source pin对write/atomic replace原码拒绝且bytes不变、wronghash拒绝，三次检查actual original source pin，不启动服务/模型/网络或写官方/日常配置。', '', 'ProductionStop和NativeContentAssessment均为私有非serde/nonClone原对象边界。先实际Source/Owner/NormalExit/SupervisorAbsent/StopAccepted/current/listener/action/routes重核，再从canonical唯一Ready/PreStop/Stopped选择实际3帧；原before+3帧按原sharedpolicy在完整BOTH-tree before/after屏障内评估（全部original/custody/历史work/native archives/current帧）。scope/series/原deadline绑定assessment先装core，之后才capture_retirement转移current frame。旧未评估preview现仅cfg(test)存在，不能让生产提前转移当前帧。', '', '新CaptureSlot.restore_stopped_existing仅内部恢复/history入口：原Ready tuple和Stop绝对end相等、original NativeLease新取得并observed Stopped后，qualified batch current退休/original返回、整树确认、显式release/两稳定范围证明、原件不可变history按guard前后actual typed条件检查。没有把组计数或Policy DATA当mutation许可。返回内部restored_history标记而非ClosedEligibility；source资格/receipt/Closed完整canonical记录、handle所有权闭环、实际process/supervisor完成history、真实Status5/dispatcher和继承接管尚待。', '', '本轮真实production Source/Owner/Ready/NormalExit/Stopped/qualified restore factories仅编译未执行；不能把47组件回归、Bun纯库投影或旧GUI空broker证据当实际原包生产闭环/原10秒Stop证明。原typed上下文不可伪造测试，不以假Owner/正常码制造正例。实际ordinary desktop固定官方生产H-S-D/后继/Status和guard反复完整来源核验的预算还需实证。公开Start无条件拒绝；官方config/auth/Grok与动态目录非artifact变化仍未知拒绝，first-use/ownerreceipt/原缺席Codex安全退根/普通MSI/CAP-trust/历史清理保持未完。不旧raw5/空GUI/相同1314/W2/双SID/发布。', '', f'本轮{len(scenes)}场景完整私有ZIP核验，3完成纯库/合成pin场景核fresh准确路径/全文件hash/目录membership清理；{counts["failedAndPendingRetained"]}失败/pending及{counts["crossVolumePendingRootsRetained"]}个D根保留。{len(records)}份canonical/快照/history原字节另归raw-scene-records。ZIP非NTFS身份/安全克隆，历史拒绝删除不重试，overallCleanupComplete=false。完整继承状态与首错/制品/source归final-status。', '', '| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
with (ROOT/'report.md').open('x',encoding='utf-8') as out:out.write('\n'.join(report)+'\n')
print(json.dumps(dict(head=head,runs={k:v['rawExit'] for k,v in runs.items()},productInputs=len(inputs),testRust=len(rust),
    scenes=len(scenes),rawRecords=len(records),removed=3,productionExecuted=False)))
