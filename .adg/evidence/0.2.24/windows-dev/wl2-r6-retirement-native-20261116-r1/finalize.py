from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
DOCS=ROOT.parents[4]
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,value):
    with (ROOT/name).open('x',encoding='utf-8') as out:json.dump(value,out,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
expected={'library01':(101,None),'library02':(0,3),'library03':(0,5),'library04':(0,1),'library05':(101,21),'library06':(0,1),'product01':(0,None)}
runs={};scenes=[];inputs=[];rust=[];records=[];traces=[];plans=[]
for name,(rc,passed) in expected.items():
    result=load(ROOT/name/'result.json');assert result['exitCode']==rc
    for row in load(ROOT/name/'source-manifest.json'):
        raw=(ROOT/name/'executed-source'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
        if name=='product01' or name=='library06' and row['path'].endswith('.rs'):
            assert raw==(CODE/row['path']).read_bytes()
            assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n')
            (inputs if name=='product01' else rust).append(row)
    if passed is not None:
        text=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig');m=re.search(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',text);assert m and tuple(map(int,m.groups()))==(passed,1 if name=='library05' else 0,0)
        for line in text.splitlines():
            for marker in ['r6-native-retirement-plans=','r6-native-retirement-existing-plan=']:
                if marker in line:plans.append(dict(run=name,nativePlan=json.loads(line.split(marker,1)[1])))
            if 'r6' in line and len(line)<2200:traces.append(dict(run=name,trace=line[line.index('r6'):]))
    for i,scene in enumerate(load(ROOT/f'{name}-private-scenes.json')['scenes'] if name.startswith('library') else []):
        assert sha(Path(scene['archive']).read_bytes())==scene['archiveSha256'];scenes.append(scene)
        for row in scene['files']:
            if scene['source'].startswith('D:') or '/.ocxd024-observed-' in '/'+row['path']:
                p=Path(scene['source'])/row['path'];raw=p.read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
                target=ROOT/'raw-scene-records'/name/f'scene-{i:03}'/row['path'];target.parent.mkdir(parents=True,exist_ok=True)
                with target.open('xb') as out:out.write(raw)
                records.append(dict(source=str(p),evidence=target.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
    if name=='library01':assert not load(ROOT/'library01-artifact.json')['executed']
    runs[name]=dict(rawExit=rc,profile=result['profile'],passed=passed,failed=(1 if name=='library05' else 0) if passed is not None else None,ignored=0 if passed is not None else None,elapsedMs=result['elapsedMs'],artifact=load(ROOT/f'{name}-artifact.json') if name.startswith('library') else None)
assert load(ROOT/'library04-artifact.json')['sha256']==load(ROOT/'library05-artifact.json')['sha256']
assert load(ROOT/'product01/source.json')['head']==head
write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,finalTestRust=rust,gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyModifiedOrStaged=True))
write('native-retirement-plan-observations.json',plans);write('native-retirement-traces.json',traces);write('raw-scene-records.json',records)
artifacts=[]
for name in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe';raw=p.read_bytes();artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
write('product01-artifacts.json',artifacts)
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
matrix=load(ROOT.parent/'wl2-r6-capture-stop-20261116-r1/final-status.json')['matrix']
matrix.extend([
 dict(id='CompleteNativeRetirementFrozenPlan',result='native-component-pass/production-preview-only',fact='Full original anchored trees/custody and complete BOTH observed current frames retained separately; rows name exact current ID/volume/security/hash/length/parent/retire target and corresponding original ID/absence. Source-bound typed Stopped preview factory requires original Source/Owner/NormalExit/Absent/StopAccepted/current action/four-domain lease. Preview returns DATA and is compiled unexecuted; no parsed/native mutation factory.'),
 dict(id='CurrentWorkRetirementOriginalRestore',result='native-component-pass/private-raw-tests-only',fact='Six current ordinary work files retire by actual current handles within original volume/private custody, then three original files restore by exact original handles to absent paths. Exact directories/stable coordinators remain original; all IDs/all8/bytes/children restored, original3 absent artifacts absent again. Existing captured artifacts with modified/atomic-replaced current work restore actual original IDs separately from both initial/current working generations. Post-explicit-release readback independently proves exact stable released lock ranges.'),
 dict(id='RetirementNativeInterruptedPrefixes',result='native-component-pass',fact='All0..9 actual retire/restore prefixes retain original/current/retired paths/IDs/all8/bytes and precise child sets; unknown target/empty directory, early restore, foreign original destination and repeated step poison without overwrite/deletion/rollforward. Raw private steps NOT canonical retirement events; no production Stop/content/recovery/Closed signed.'),
])
cleanup=load(ROOT/'cleanup.json')
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract='TEAM16/IMP142/r6',baselineMain='fac8573e55441fe514919c63c7f1d65181ff3577',baselineDocs='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,artifacts=artifacts,matrix=matrix,
 firstError=dict(run='library01',rawExit=101,rustError='E0282/E0283',stage='custody name Vec element .into() type ambiguity',newExecutableProduced=False,fixedBy='explicit Vec<String>; assertions/native deadlines unchanged'),
 debugTimeoutFacts=load(ROOT/'debug-timeout-facts.json'),finalCoverage='library05 debug21 passed1 timeout retained; same Rust original deadline failing-case release target library06 passed; unrelated prior successes reused, no full-suite retries',
 executionBoundary=dict(newRetirementMethodsPrivate=True,rawMutationCallers='retirement_tests.rs only',newProductionMutationInvoked=False,newCanonicalRetirementNodeProgressIntegrated=False,nativeTestCurrentJournalRemains='Observation component prefix; no text history/Stop/Closed adoption',existingNativeFileIdentityChecksUnchanged=True),
 limits=['Raw native retirement tests use synthetic objects/known invalid cache+audit DATA, no content-policy or actual Source/Stop authority; tests prove mechanism only','New actual typed Stopped method captures a read-only complete retirement preview, compiles but not invoked; core execute paths remain private','Still must persist complete source/policy-qualified retirement plan and every native per-node intent/result/current under actual original Stop authority, then wire same CaptureSlot stopped transaction','Dynamic new directories and source-declared config/auth/Grok effects remain rejected/unimplemented; original absence safety not reduced','NormalExit/SCM/Task/listener/record/source plus full original history/typedClosed/next session/realOwner Status5/GUI-H-S-D production not complete','OriginalAbsent Codex initialization/safe root retirement blocked; ordinary MSI/CAP-trust/historical cleanup gaps unchanged'],allAccepted=False,startOpen=False,productionDispatcherUsesV3=False,actualSourceOrRecoveryFactoryExecuted=False,productionExecuted=False,officialOwnerCreated=False,W2=False,dualSID=False,published=False,
 cleanup=dict(archivedScenes=len(scenes),archivedFiles=sum(len(s['files']) for s in scenes),archivedBytes=sum(f['bytes'] for s in scenes for f in s['files']),completedOwnScenesRemoved=len(cleanup['removed']),failureAndPendingScenesRetained=cleanup['failedAndPendingScenesRetained'],crossVolumePendingRootsRetained=cleanup['crossVolumePendingRootsRetained'],nativeMetadataCloned=False,historicalRejectedDeletionRetried=False,newAccounts=0,newProfiles=0,newPublicRoots=0,newGUIProcesses=0,newOfficialServices=0,dailyPathConfigPolicyChanged=False,overallCleanupComplete=False))
write('final-status.json',status)
report=f'''# r6当前工作代次准确退休与原件返回原语

TEAM16/IMP142/r6，源码{head}/tree{tree}；承接13c348580/58cc22b5d。原目录/两稳定协调对象、原8投影/ExcludedUnknown、期限/未知拒绝、固定官方/日常配置不变。

| 运行 | profile | 原退出码 | 过/败/忽略 | ms |
| --- | --- | --- | --- | --- |
'''+ '\n'.join(f"| {name} | {r['profile']} | {r['rawExit']} | {r['passed']}/{r['failed']}/{r['ignored']} | {r['elapsedMs']} |" for name,r in runs.items())+'''

首library01 E0282/E0283：custody_names的.into()元素类型推断不明，raw101/源/stderr保留，无新EXE。仅明确Vec<String>，不改断言/原native截止。library02三原生定向通过，0..9全前缀均实际生成与核验；library03补释放后读回/稳定范围证明与early/foreign/reentry反例，五过。library04独立补原已存在三artifact当前工作修改/原子替换，原件/initial work/current work三身份分别保管，一过。library05准确重用library04同debug EXE（所有执行输入字节与前后hash一致），受影响退休/Capture/currentpin/action回归21过1败：刚通过的既有artifact用例本轮在原10秒窗bounded-original-deadline超时，原失败/当前原件前缀保留，不改通过。library06仅该失败用例以仓库原release参数、完全相同Rust/assertions/绝对10秒截止定向通过，其余21项成功复用；未加预算/删安全读回/改profile参数。原native源码/真实参数/原码全部保留，无旧raw5/空GUI/1314/MSI/W2或候选发布；前批release41/debug失败证据独立复用。

完整原生plan预览读取准确原树/原custody与两根完整当前frozen frame，当前普通work逐项记录relative、source current volume/fileID/all8/hash/length、source parent、同卷private退休target及parent ID、对应original ID或原始缺席。原目录和两稳定文件只作为锚点，不进入移动集合。计划返回JSON只是数据；保留实际原frame文件描述、原件和初始work observer，不从JSON、路径或hash构造原生资格。

新执行方法严格private，仅retirement_tests raw native机制调用。六份当前普通work（原Codex配置/嵌套auth/Grok设置＋本次原先缺席准确三artifact）逐原current handle同卷rename到CREATE_NEW式Absent target，随后三份原件由准确原custody handle回Absent原路径；ReplaceIfExists=false，不覆盖/删除外来文件，不copy-delete重建身份。每步都核claim/NLK、原父pin、原卷/文件ID/8投影/bytes、current与retired完整位置、custody集合和精确predicted children。最终原根/nested/empty目录ID和两协调对象不变，原3artifact缺席重新读回；Bun claim-only在全过程检查点busy，显式release后raw0可取。

追加既有artifact原件场景：原catalog/cache/audit也已捕获，在work区修改/atomic rename产生新current代次。原始catalog对象、初始工作observer与current replacement ID不混淆；准确current对象退休、准确original对象返回原路径，全量original plan保持相等。工作cache/audit字节为明确合成观察数据，未做内容policy准入；这些测试不能授恢复权限。最终实际readback在显式lease.release后仍复核全部原件/retired work/parent/目录children/all8，并独立取得/释放两稳定协调文件512字节范围作释放证明，无SQLite open/recovery/PRAGMA/schema写。

0..9全部prefix实际验证，不从文本补恢复；未退休全work前提前还原拒绝，foreign original destination/退休target和未知空目录拒绝，第一步已retired后重入仍poison，保留准确原/工作/外来对象及已发生后像。无ACL/security修补，无强关其它句柄、POSIX/root rename或放claim仅靠NLK窗口。原缺席Codex根安全退根未尝试，当前只existing-root和文件级original absence机制。

生产仅接read-only inspect_original_retirement到actual typed Stopped前提：必须保管原PreparedStop/绝对end，实际Source/Owner/NormalExit/SupervisorAbsent/StopAccepted/端点/record/current/four-domain lease鲜读，再按本代canonical Ready/PreStop/Stopped上下文生成双根计划，返回contentPolicyAssessed=false、mutationAuthority=false、ClosedEligibility=false。该新factory已编译未实跑；它不调用任何新native退休步骤。actual per-node退休canonical intent/result/current尚未接，raw测试journal保持原Observation组件事实，不能拿旧current当已记录恢复成功或文本Closed。源码调用边界与未完归final-status。

下一段必须把原Source/route/process/audit上下文的原内容policy和complete typed Stop权限、完整退休计划以及每row intent/result/current组合后，才在actual CaptureSlot调用本机制。动态新增目录、config/auth/Grok源声明变化仍未知拒绝，未缩范围或补豁免。还需最终original history/typedClosed/后继、真sameOwner Status5/GUI-H-S-D。production dispatcher/Start门禁不变。原缺席安全初始化/退根blocked，普通MSI/CAP-trust/历史清理缺口保持。不W2/双SID/真实业务模型请求/发布。

最终MSVCcustom-protocol debug产品输入与最终testRust逐份对准确工作源/Git blob行尾等价，前端32份复用；非release预算/冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
'''+ '\n'.join(f"| {a['name']} | {a['bytes']} | {a['sha256']} |" for a in artifacts)+f'''

本轮{len(scenes)}场景私有内容ZIP核验，原native完整plan、各前缀trace单列native-retirement-*；D/current/不可变work快照块原字节另归raw-scene-records。{len(cleanup['removed'])}完成纯数据场景核全部文件和目录后清理，{cleanup['failedAndPendingScenesRetained']}失败/pending原场景和{cleanup['crossVolumePendingRootsRetained']} D日志根保留。raw原语正例即使原件已返回，也没有canonical逐row记录和正式Closed，按测试证据pending保留。ZIP不是NTFS身份/安全克隆，旧拒绝删除/账号/Profile/Public/旧broker不重试，overallCleanup=false。完整继承表与未完归final-status.json。
'''
with (ROOT/'report.md').open('x',encoding='utf-8') as out:out.write(report)
with (DOCS/'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md').open('a',encoding='utf-8',newline='\n') as out:out.write(f"\n\n## 2026-11-16 TEAM16/r6 当前work准确退休与原件返回原语\n\n源码 {head}/tree{tree}；证据 .adg/evidence/0.2.24/windows-dev/{ROOT.name}/。完整native当前/原件计划，6work逐准确handle同卷退休、3original准确handle返回、3原缺席artifact再缺席，0..9前缀/unknown/early/foreign/reentry拒绝，释放后全树和stable范围读回；既有artifact修改/atomic替换三代身份归位增量通过。首library01 E0282/E0283/raw101无新EXE，明确Vec<String>修复后native3/5/1定向过；准确同debugEXE回归21过1败，既有artifact用例在原10秒bounded安全读回超时保留；仅失败用例采用同Rust/断言/原截止与原release参数定向，结果独立列出，不加预算。actual typedStopped计划预览只编译未调用，mutationAuthority=false；新执行private仅raw合成native tests，canonical逐row与恢复原权限未接，不签生产Stop/Closed。产品MSVCdebug bins0/{runs['product01']['elapsedMs']}ms；源/EXE/完整表/首错/清理归report/final-status。\n\n后续实际Source+policy/Stop+canonical retirement/node/current、dynamic目录与config/auth/Grok、history/Closed/后继/实际Owner-H-S-D待；原缺席安全退根blocked且Start拒绝。不旧raw5/空GUI/W2/双SID/发布。\n")
print(json.dumps(dict(head=head,tree=tree,productInputs=len(inputs),finalRust=len(rust),scenes=len(scenes),rawFiles=len(records),artifacts=artifacts)))
