from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path(r"C:/Users/15119/.codex/worktrees/024-windows-native/code")
DOCS=ROOT.parents[4]
load=lambda p:json.loads(p.read_text(encoding="utf-8-sig"))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(["git","-C",str(CODE),*a])
def write(name,value):
    with (ROOT/name).open("x",encoding="utf-8") as out:json.dump(value,out,ensure_ascii=False,indent=2)
head=git("rev-parse","HEAD").decode().strip();tree=git("rev-parse","HEAD^{tree}").decode().strip()
runs={};scenes=[];final=[];rust=[];journals=[];prefixes=[];created=[]
for name,passed in [("library01",14),("library02",15),("product01",None)]:
    result=load(ROOT/name/"result.json");assert result["exitCode"]==0
    for row in load(ROOT/name/"source-manifest.json"):
        raw=(ROOT/name/"executed-source"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
        if name=="product01" or name=="library02" and row["path"].endswith(".rs"):
            assert raw==(CODE/row["path"]).read_bytes()
            assert raw.replace(b"\r\n",b"\n")==git("show",f"{head}:{row['path']}").replace(b"\r\n",b"\n")
            (final if name=="product01" else rust).append(row)
    if passed is not None:
        text=(ROOT/name/"stdout.log").read_text(encoding="utf-8-sig");m=re.search(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored",text)
        assert m and tuple(map(int,m.groups()))==(passed,0,0)
        prefixes.extend(dict(run=name,trace=line[line.index("r6-native-journal-"):]) for line in text.splitlines() if "r6-native-journal-" in line)
        for line in text.splitlines():
            if "r6-work-created-pending " in line:created.append(dict(run=name,observation=json.loads(line.split("r6-work-created-pending ",1)[1])))
        for i,s in enumerate(load(ROOT/f"{name}-private-scenes.json")["scenes"]):
            assert sha(Path(s["archive"]).read_bytes())==s["archiveSha256"];scenes.append(s)
            if s["source"].startswith("D:"):
                for row in s["files"]:
                    raw=(Path(s["source"])/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
                    target=ROOT/"journal-scene-records"/name/f"scene-{i:02}"/row["path"];target.parent.mkdir(parents=True,exist_ok=True)
                    if target.exists():assert target.read_bytes()==raw
                    else:
                        with target.open("xb") as out:out.write(raw)
                    journals.append(dict(source=str(Path(s["source"])/row["path"]),evidence=target.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
    runs[name]=dict(rawExit=0,passed=passed,failed=0 if passed is not None else None,ignored=0 if passed is not None else None,elapsedMs=result["elapsedMs"],artifact=load(ROOT/f"{name}-artifact.json") if passed is not None else None)
assert len(created)==3 and [r["observation"]["fault"] for r in created]==[1,2,3]
write("native-prefix-traces.json",prefixes);write("pending-work-native-observations.json",created);write("journal-scene-records.json",journals)
assert load(ROOT/"product01/source.json")["head"]==head
artifacts=[]
for name in ["opencodex-desktop","ocxd","ocx"]:
    p=CODE/f"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe";raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile="debug",features=["custom-protocol"]))
write("product01-artifacts.json",artifacts);write("final-source.json",dict(head=head,tree=tree,productInputs=final,finalTestRust=rust,workBytesEqual=True,gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyEditedOrStaged=True))
for row in load(ROOT.parent/"wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json"):
    raw=(CODE/"apps/desktop/ui/dist"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
matrix=load(ROOT.parent/"wl2-r6-v3-20261116-r1/final-status.json")["matrix"]
for row in matrix:
    if row["id"]=="V3CanonicalNodeJournal":row.update(result="native-intent/result-component-pass; production-phase-adapter-incomplete",fact="Actual callback requires same live borrowed claim/epoch/key/deadline and exact immutable complete native plan; intent phase+current synced/read back BEFORE mutation, result records actual same-generation ID/all8. Three original files in Codex+Grok roundtrip with30 canonical events (6 anchor snapshots+24 file intent/results). Actual post-release tree/stable proof succeeds. New lease epoch cannot inherit permissions. This is not production Source/phase/release/Closed eligibility.")
matrix.extend([
 dict(id="V3NativeCombinedInterruptions",result="component-pass",fact="After durable intent/before mutation, after native move/before result, after result publish/before CAS, and partial result-current preserve exact original/native prefix and poison tree+journal; no overwrite/catchup/retry."),
 dict(id="V3WorkCreateOriginalHandle",result="component-pass",fact="Private TokenUser/SYSTEM protected CREATE_NEW returns exact RW/DELETE creation handle immediately and stores it in pending_work BEFORE any fallible query/write. Empty, real1-byte partial-write/sync, complete-sync/prepublication faults retain that exact handle/ID/all8 and untouched original custody. Another writer raw32. Successful work uses same creation description; no seed/reopen identity gap."),
 dict(id="V3LeaseSegmentsAndPhases",result="incomplete",fact="Native component currently binds one active lease. Explicit release-prepared/result receipts, new lease phase/current binding, Ready/PreStop/Stopped archives, source/route/current/history/successor and typed native Closed still required. Work descriptions deny outside writes; official writable interval and precise allowed side-effects remain to integrate. Old v2 adapter/public Start gate unchanged."),
])
cleanup=load(ROOT/"cleanup.json")
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract="TEAM16/IMP142/r6",baselineMain="fac8573e55441fe514919c63c7f1d65181ff3577",baselineDocs="5782307174f0d1b011b5415c6c1e8d60414e6b36",runs=runs,artifacts=artifacts,matrix=matrix,
 firstUnexpectedError=None,retainedControlledFailures=["4 durable/native prefix windows in each library run","3 actual work create/write/preseal windows in library02","new actual lease epoch refusal","all historical raw101/raw2/raw5/1314 evidence unchanged"],
 nativeBoundNodeJournal=True,productionAdapterUsesV3=False,allAccepted=False,startOpen=False,officialOwnerCreated=False,productionExecuted=False,W2=False,dualSID=False,published=False,
 cleanup=dict(archivedScenes=len(scenes),archivedFiles=sum(len(s["files"]) for s in scenes),archivedBytes=sum(f["bytes"] for s in scenes for f in s["files"]),completedOwnScenesRemoved=len(cleanup["removed"]),failureAndPendingScenesRetained=78,crossVolumePendingRootsRetained=25,reusedTaskParentDirectoriesRetained=3,nativeMetadataCloned=False,historicalRejectedDeletionRetried=False,newAccounts=0,newProfiles=0,newPublicRoots=0,newGUIProcesses=0,newOfficialServices=0,fixedBunSyntheticChildren=26,dailyPathConfigPolicyChanged=False,overallCleanupComplete=False))
write("final-status.json",status)
report=f"""# r6逐节点原生动作与v3意图结果绑定

TEAM16/IMP142/r6，源码{head}/tree{tree}。承接5a7cccaaa/ce8c28c37，完整计划与单journal沿前批，不改变官方/四trait/共享算法/预算/日常策略。

| 运行 | 原退出码 | 过/败/忽略 | ms |
| --- | --- | --- | --- |
"""+"\n".join(f"| {n} | {r['rawExit']} | {r['passed']}/{r['failed']}/{r['ignored']} | {r['elapsedMs']} |" for n,r in runs.items())+"""

本轮无非预期首错；原controlled故障全部保留为原例。完整两树plan和当前原普通Token/peer Binding核验后，NodeMove/WorkCreate/WorkRetire/OriginalRestore均先不可覆盖intent sync/readback及current同ID CAS，才借同actual claim/NLK/epoch/key/原deadline执行native回调；随后同原native对象全树读回，结果登记准确ID和8投影。新工作security沿创建结果校验退休，不从manifest猜工作代次。expected摘要绑定全plan和准确位置/内容/分代规则，实际新ID/8投影不能预测，单独由完成记录保存原生读回。

三份原文件Codex+Grok全树往返，30个canonical事件（6个锚点snapshot+24个文件intent/result）；目录及两稳定对象不动，原ID/内容/all8/完整children恢复，工作对象准确原handle退休。固定Bun claim-only在组合开始/最后native变更后仍raw73，显式释放后raw0；各native节点后另有实际SQLite竞争拒绝。旧r5 raw5根移动与空GUI不重跑。该组件没有生产Owner、正式阶段或Closed资格。

四个组合中断分别位于intent/current已落而native未动、native已移而结果未写、结果phase落而CAS前失败、结果current真实单字节部分写并sync。日志与原树/custody实际位置一致，tree和journal同时poison，后续调用拒绝，不catchup。新实际lease即使同路径/Token/ID也因epoch不同拒绝继承许可，明确新段协议仍需实现。

首轮14过后复核识别工作文件seed→writer→seal重开窗口，补原子私有CREATE_NEW工厂：仅新对象使用已有PrivateDescriptor/准确TokenUser+SYSTEM protected DACL，不修现有ACL；返回准确RW/DELETE创建句柄后立即存pending_work，任何后续失败均保留该对象。新增空创建后/实际1字节写+sync后/完整写+sync读回而发布前故障；原件custody不动，创建ID/all8/已写字节摘要及原pending记录保留，第二writer raw32。成功路径也不再关闭重开工作对象。最终15过。新all8事件不提供旧v2或前批pending的转换/接管；旧记录只作定位/原失败证据。

下一段仍需lease释放Prepared/原生结果、重新取得的phase/current权限，真实Ready/PreStop/Stopped封存、原包允许副作用及工作区可写窗口、source/route/history/successor、完整Stop与typed Closed、broker原Owner/H-S-D。当前生产适配入口仍v2，公共Start保持无条件拒绝；原缺席Codex安全退根blocked单列，不能借现有根通过代首次空环境。不W2/双SID/业务模型/发布。完整继承状态表归final-status.json。

最终产品MSVC custom-protocol debug bins及输入/最终测试Rust对准确源码字节核验；前端32份复用；不签release预算/冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
"""+"\n".join(f"| {a['name']} | {a['bytes']} | {a['sha256']} |" for a in artifacts)+f"""

{len(scenes)}场景私有ZIP内容核验；工作部分写原生观察和D场景phase/current原字节另归pending-work-native-observations/journal-scene-records，包含单字节current。10处本轮已完成自有纯/原语场景先核全部文件和目录后清理；78失败/pending场景原树及25处D日志根保留，三层前批任务父目录沿用。完整native回调正例虽原件恢复，正式release/phase未闭合，测试journal仍保持pending，不删除或冒Closed。ZIP不是NTFS身份/安全克隆。旧失败broker/账户/Profile/Public拒读与自动删除拒绝不重试，普通MSI/CAP-trust/overallCleanup未代签。
"""
with (ROOT/"report.md").open("x",encoding="utf-8") as out:out.write(report)
with (DOCS/"docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md").open("a",encoding="utf-8",newline="\n") as out:
    out.write(f"\n\n## 2026-11-16 TEAM16/r6 节点native与v3记录绑定\n\n源码 {head}/tree{tree}；证据 .adg/evidence/0.2.24/windows-dev/{ROOT.name}/。原生动作前intent/current刷盘，后同原对象ID/8投影结果；Codex+Grok三原文件30事件全树往返，四处组合中断/new epoch拒绝通过。首轮14过，补原子CREATE_NEW立即保管准确工作句柄及空/1字节/封存前3故障，最终15过0败；本轮无非预期首错。产品MSVC debug bins原码0/{runs['product01']['elapsedMs']}ms，SHA/完整状态表归final-status/product01-artifacts/report。10成功场景核归档后清理，78失败/pending原场景和25 D日志根保留，不是原生元数据克隆。\n\n还需释放Prepared/真实结果及新lease段、正式阶段/封存/source/route/history/successor/副作用/原包H-S-D/typed Closed，现生产入口仍v2且Start无条件拒绝；原缺席Codex安全退根blocked单列。旧raw5/空GUI不重跑，不转换旧记录，不W2/双SID/发布。\n")
print(json.dumps(dict(head=head,tree=tree,productInputs=len(final),finalRust=len(rust),scenes=len(scenes),rawJournalFiles=len(journals),artifacts=artifacts)))
