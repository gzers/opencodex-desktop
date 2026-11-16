from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess,zipfile
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=ROOT.parent/'wl2-r6-exact-close-inventory-20261116-r1'
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a])
def write(name,v):
    with (ROOT/name).open('x',encoding='utf-8') as f:json.dump(v,f,ensure_ascii=False,indent=2)
head=git('rev-parse','HEAD').decode().strip();tree=git('rev-parse','HEAD^{tree}').decode().strip()
assert load(ROOT/'source-commit.json')['head']==head
runs={};scenes=[];records=[];traces=[];rust=[];inputs=[]
for name,profile in [('library01','debug'),('library02','release'),('product01','debug')]:
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
        assert raw==(CODE/row['path']).read_bytes(),row['path']
        assert raw.replace(b'\r\n',b'\n')==git('show',f"{head}:{row['path']}").replace(b'\r\n',b'\n'),row['path']
        if name=='product01':inputs.append(row)
        elif name=='library02' and row['path'].endswith('.rs'):rust.append(row)
assert load(ROOT/'product01/source.json')['head']==head
for row in load(ROOT/'source-scope.json')['changedFiles']:
    raw=(CODE/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
artifacts=[]
for name in ['opencodex-desktop','ocxd','ocx']:
    p=CODE/f'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe';raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile='debug',features=['custom-protocol']))
for row in load(ROOT.parent/'wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json'):
    raw=(CODE/'apps/desktop/ui/dist'/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
write('product01-artifacts.json',artifacts);write('raw-scene-records.json',records);write('directed-traces.json',traces)
write('final-source.json',dict(head=head,tree=tree,productInputs=inputs,releaseRust=rust,allThreeRunInputsMatchSource=True,gitLineEndingsExplicitlyEquivalent=True,preexistingSchemaNotEditedOrStaged=True))
matrix=load(BASE/'final-status.json')['matrix'];matrix.append(dict(id='StrictCanonicalHandleRowGrammar',result='DATA-and-affected-native-handoff-pass/production-not-executed',fact='Canonical close intent rejects unknown tree/lease roles, roots outside0/1, noncanonical bounded ordinals, pendingwork, wrong coordinator ancestor/parent roles, unknown bound fields and directory/history flag mismatches. Original/history-source exact plan objects, initial-work exact original work generation and stopped-anchor exact nonretired stopped snapshot object required.27 bad self-rehashed row/count negatives atomic; known actual producer shapes pass. Independent full native inventory and native close/Stop budget remain unchanged; not parsed authority.'))
requirements=load(BASE/'next-closed-requirements.json')['requirements']
for r in requirements:
    if r['id']=='PlanCompleteness':r['requirement']='Independent complete expected native inventory is checked around actual close and canonical DATA role/key/root/ordinal/kind/history grammar now strict; native affected inventories pass. Canonical old-work/lease locator rows are DATA only, cannot prove native objects/completeness; independent actual expected inventory remains mandatory. Full maximal1MiB/512node batch capacity/subgroup support and total handle/history capacity remain open; original bounds and complete original set must not be weakened.'
write('next-closed-requirements.json',dict(requirements=requirements,startOpen=False,typedClosedQualified=False,actualProductionExecuted=False))
cleanup=dict(scenes=len(scenes),removed=0,failedAndPendingRetained=len(scenes),crossVolumePendingRootsRetained=sum(s['source'].startswith('D:') for s in scenes),deletionAttempted=False,historicalRejectedDeletionRetried=False,overallCleanupComplete=False,zipIsNTFSMetadataClone=False,reason='All source-plan/native tree/canonical scenes retained pending actual qualified Closed; original fault/unknown states preserved. Content archives are not NTFS clones.')
write('cleanup.json',cleanup)
status=dict(time=datetime.now().astimezone().isoformat(),contract='TEAM16/IMP142/r6',sourceHead=head,sourceTree=tree,inheritedCode='4e26406402ae20dd3ad8b2c6102c541c2da2d183',inheritedDocs='6b3770202d4c3e5aef8d089a11d8c1ec2ea576d9',fetchedMain='fac8573e55441fe514919c63c7f1d65181ff3577',fetchedGovernanceMain='5782307174f0d1b011b5415c6c1e8d60414e6b36',runs=runs,artifacts=artifacts,matrix=matrix,limits=requirements,
 firstError=dict(currentRound=None,inheritedLatest=load(BASE/'final-status.json')['firstError'],expectedNativeNegativesPreserved=True),cleanup=cleanup,
 allAccepted=False,startOpen=False,typedClosedQualified=False,actualProductionExecuted=False,productionDispatcherUsesV3=False,productionStatusSampleJoined=False,W2=False,dualSID=False,published=False)
write('final-status.json',status)
report=['# r6关闭清单严格角色与节点序号','',f'源码{head}/tree{tree}；承接4e2640640/6b3770202，TEAM16/IMP142/r6及既定共同源不变。轮次于2026-11-16开始、跨11月17日完成，原目录名不变。','', '| 运行 | profile | raw | 过/败/忽略 | ms |','| --- | --- | --- | --- | --- |']
for n,r in runs.items():report.append(f"| {n} | {r['profile']} | {r['rawExit']} | {r.get('passed','—')}/{r.get('failed','—')}/{r.get('ignored','—')} | {r['elapsedMs']} |")
report+=['','本轮无非预期失败，既往失败/Unknown原记录不改。新增strict canonical handle-row DATA按实际producer语法拒绝未知role/root/ordinal/parent/bound/kind/history；原件/history-source对完整原计划，initialwork对实际原工作代次，stoppedanchor对停止快照非退休节点。27个额外坏行均重算hash/count，完整原必需行仍在，拒绝且progress全部字节不变；不是靠旧hash失败。正例覆盖各known producer形状与树nativeID/generation绑定，actual handoff回归核真实生成清单。',
 '', '独立ExpectedHandleInventory/native close/lease transfer/trees/actualtypedguard/内容scope/原窗口等13处实现逐字保持。DATA locator不是native完整性或许可，仍必须按原对象独立全清单比较。未改预算/unknown拒绝/Start门禁，不拿synthetic native原语代ordinary fixedofficial生产。',
 '', '原Stop内process/supervisor完成history、typedClosed/同次Status5/dispatcher/后继、官方config/auth/Grok/动态目录副作用、首次初始化/ownerreceipt/原缺席Codex安全退根、最大完整batch容量及普通MSI/CAP-trust/历史清理继续未完。',
 '',f'{len(scenes)}scene完整私有ZIP/hash/membership核验，全部原plan/native/canonical场景保留，{len(records)}份原canonical/快照/history另收存；本轮无删除/历史拒绝重试，ZIP非NTFS身份安全克隆。三轮source全部核工作/Git明确CRLF等价，产品{len(inputs)}输入/最终release{len(rust)}Rust，预存schema未手改/暂存。不旧raw5/空GUI/W2/双SID/发布，Start/allAccepted/typedClosed/productionExecuted仍false。',
 '', '| 制品 | 字节 | SHA256 |','| --- | --- | --- |']
for a in artifacts:report.append(f"| {a['name']} | {a['bytes']} | {a['sha256']} |")
with (ROOT/'report.md').open('x',encoding='utf-8') as f:f.write('\n'.join(report)+'\n')
print(json.dumps(dict(head=head,passed={k:r.get('passed') for k,r in runs.items()},scenes=len(scenes),rawRecords=len(records),productInputs=len(inputs),releaseRust=len(rust),typedClosed=False)))
