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
runs={};final=[];rust=[];scenes=[];raw_files=[];links=[];native_refs=[];traces=[]
for name,rc in [("library01",101),("library02",101),("library03",0),("library04",0),("product01",0)]:
    result=load(ROOT/name/"result.json");assert result["exitCode"]==rc
    for row in load(ROOT/name/"source-manifest.json"):
        raw=(ROOT/name/"executed-source"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
        if name=="product01" or name=="library04" and row["path"].endswith(".rs"):
            assert raw==(CODE/row["path"]).read_bytes()
            assert raw.replace(b"\r\n",b"\n")==git("show",f"{head}:{row['path']}").replace(b"\r\n",b"\n")
            (final if name=="product01" else rust).append(row)
    passed=failed=ignored=None
    if name in ["library01","library02"]:assert not load(ROOT/f"{name}-artifact.json")["executed"]
    if name in ["library03","library04"]:
        text=(ROOT/name/"stdout.log").read_text(encoding="utf-8-sig");m=re.search(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored",text);assert m
        passed,failed,ignored=map(int,m.groups());assert failed==0 and ignored==0 and passed>0
        if name=="library03":assert passed==5
        for line in text.splitlines():
            if "r6-linked-native-snapshot-references=" in line:native_refs.append(dict(run=name,references=json.loads(line.split("r6-linked-native-snapshot-references=",1)[1])))
            if "r6-snapshot" in line and len(line)<3000:traces.append(dict(run=name,trace=line[line.index("r6-snapshot"):]))
        for i,s in enumerate(load(ROOT/f"{name}-private-scenes.json")["scenes"]):
            assert sha(Path(s["archive"]).read_bytes())==s["archiveSha256"];scenes.append(s)
            for row in s["files"]:
                if s["source"].startswith("D:") or "/.ocxd024-observed-" in "/"+row["path"]:
                    p=Path(row.get("source",str(Path(s["source"])/row["path"])));raw=p.read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
                    target=ROOT/"raw-scene-records"/name/f"scene-{i:03}"/row["path"];target.parent.mkdir(parents=True,exist_ok=True)
                    with target.open("xb") as out:out.write(raw)
                    raw_files.append(dict(source=str(p),evidence=target.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
                    if "/event-" in row["path"]:
                        event=json.loads(raw)
                        if event["documentKind"]=="snapshot-phase-event":links.append(dict(run=name,scene=s["source"],evidence=target.relative_to(ROOT).as_posix(),event=event))
    runs[name]=dict(rawExit=rc,passed=passed,failed=failed,ignored=ignored,elapsedMs=result["elapsedMs"],artifact=load(ROOT/f"{name}-artifact.json") if name.startswith("library") else None)
write("native-snapshot-reference-observations.json",native_refs);write("canonical-snapshot-events.json",links);write("snapshot-traces.json",traces);write("raw-scene-records.json",raw_files)
assert load(ROOT/"product01/source.json")["head"]==head
write("final-source.json",dict(head=head,tree=tree,productInputs=final,finalTestRust=rust,workBytesEqual=True,gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyEditedOrStaged=True))
artifacts=[]
for name in ["opencodex-desktop","ocxd","ocx"]:
    p=CODE/f"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe";raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile="debug",features=["custom-protocol"]))
write("product01-artifacts.json",artifacts)
for row in load(ROOT.parent/"wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json"):
    raw=(CODE/"apps/desktop/ui/dist"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
matrix=load(ROOT.parent/"wl2-r6-work-observation-20261116-r1/final-status.json")["matrix"]
matrix.extend([
 dict(id="CanonicalBothRootSnapshotReferences",result="native-component-pass",fact="Snapshot intent/current before archive writes, actual borrowed BOTH frozen frames+native archive objects required for link result. Same chain/current carries snapshot pending/purpose/observation sequence/link digests. Native callbacks recheck both frame/source/archive IDs/all8/content/member lists before/after CAS. Missing second/archive failure/link-result/current partial write keep pending and poison, not textual completion."),
 dict(id="StrictSnapshotLocatorContext",result="data-counterexamples-pass; no native authority",fact="Strict deny_unknown_fields includes exact session/plan/binding/manifest/deadline/purpose/observed-sequence/both frame digests, Codex/Grok root order, same-volume private path and metadata IDs/all8/hash. Original/custody/duplicate IDs refused; missing or swapped root, stale frame sequence, context/ref changes, unknown authority/Closed fields refused. Decoding never constructs actual SnapshotWitness/NativeAction/Ready."),
 dict(id="SnapshotHistoryAfterWorkingHandoff",result="native-component-pass",fact="Whole unchanged native work can be handed back to accurate retirement+original restore with archive objects still retained in tree history; subsequent readback checks immutable snapshot bytes/metadata, not a later live client tree as historical baseline.39 canonical events restore original files/anchors and explicit lease release, lifecycle remains Preparing in component test."),
 dict(id="V3TypedReadySnapshotFactory",result="compiled-not-executed",fact="begin_ready_snapshot_link and record_ready_phase separately require actual Source.current/image/Owner scope+original NativeAction and typed ServiceObservation with original live record/listener checks, same live claim and matching native Ready snapshot refs. No production Ready positive or adapter switch executed; pure link success remains Preparing, cannot substitute PID/digest strings."),
])
cleanup=load(ROOT/"cleanup.json")
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract="TEAM16/IMP142/r6",baselineMain="fac8573e55441fe514919c63c7f1d65181ff3577",baselineDocs="5782307174f0d1b011b5415c6c1e8d60414e6b36",runs=runs,artifacts=artifacts,matrix=matrix,
 firstError=dict(run="library01",rawExit=101,rustError="E0365",stage="unneeded outward reexport of private SnapshotPurpose",newExecutableProduced=False,fixedBy="remove outward reexport, purpose selected by internal factory"),
 secondCompileError=dict(run="library02",rawExit=101,rustError="E0425",stage="private internal SnapshotPurpose import omitted after removing outward reexport",newExecutableProduced=False,fixedBy="private use only, no wider visibility"),
 productionAdapterUsesV3=False,allAccepted=False,startOpen=False,productionExecuted=False,officialOwnerCreated=False,nativeReadyFactoryExecuted=False,W2=False,dualSID=False,published=False,
 limits=["Native observation snapshot link and history are exercised, not native official Ready/Stop/Closed","PreStop/Stopped purpose ordering only locator data so far; original Ready/Stop ActionWindow bridge and actual phase factories still needed","Original/Ready/PreStop/Stopped snapshots plus independent declared routes/retained process scope must feed unchanged content policy before changed-node retirement","Dynamic work artifacts/new directories, source-bound exact restore plan, complete NormalExit/supervisor/listener/record/current/Closed/history/successor and actual Owner H-S-D production still incomplete","OriginalAbsent Codex safe initialization/retirement blocked; old ordinary MSI/CAP-trust/historical cleanup unchanged"],
 cleanup=dict(archivedScenes=len(scenes),archivedFiles=sum(len(s["files"]) for s in scenes),archivedBytes=sum(f["bytes"] for s in scenes for f in s["files"]),completedOwnScenesRemoved=len(cleanup["removed"]),failureAndPendingScenesRetained=cleanup["failedAndPendingScenesRetained"],crossVolumePendingRootsRetained=cleanup["crossVolumePendingRootsRetained"],nativeMetadataCloned=False,historicalRejectedDeletionRetried=False,newAccounts=0,newProfiles=0,newPublicRoots=0,newGUIProcesses=0,newOfficialServices=0,dailyPathConfigPolicyChanged=False,overallCleanupComplete=False))
write("final-status.json",status)
report=f"""# r6双根快照引用接入canonical阶段链

TEAM16/IMP142/r6，源码{head}/tree{tree}；承接c61327035/0325d1d92。共享四trait/原预算/8投影/ExcludedUnknown/副作用policy不变，官方和日常配置未改。

| 运行 | 原退出码 | 过/败/忽略 | ms |
| --- | --- | --- | --- |
"""+"\n".join(f"| {n} | {r['rawExit']} | {r['passed']}/{r['failed']}/{r['ignored']} | {r['elapsedMs']} |" for n,r in runs.items())+"""

首错library01 E0365/private SnapshotPurpose重复向父重导出，原源码/原stderr/raw101保留，无新EXE；移除无必要导出后library02 E0425缺少内部本地导入，补私有use而不扩大可见性，两次编译失败无新EXE。内部工厂控制用途。原生和数据用例通过，受影响旧work/current/节点/lease/v3回归如上，未重复旧raw5/空GUI/1314或无变化官方读取。

SnapshotLinkIntent不可覆盖事件+同实际current CAS先完成，才创建本卷完整两根archive；SnapshotLinkedVerified必须借实际BOTH WorkFrame和archive对象的SnapshotWitness，重新核源帧/current路径原ID/all8/字节/children、archive原目录/metadata/chunk ID/all8/字节/集合后才发布引用。文本/JSON/路径/hash不能构造Witness。canonical单链/current绑定用途、观察序号、session/plan/binding/manifest/action及两frame digests，pending快照期间禁止其它节点/work/lease变更。

严格参考数据含准确Codex/Grok次序、同卷Profile私有目录、metadata对象ID/8投影/hash及完整context。目录含session-purpose-root防用途覆盖；拒绝旧观察序号、跨session/plan/binding/deadline/purpose/根、重复或原件/custody身份、未知field/nativeAuthority/Closed、缺第二根等。数据反例可构造合规locator，不构造原生证明；链接成功仍是Preparing，不是Ready资格。

原生正例在同原窗口完成39事件：先完整工作→Open/Observed→快照intent→实际两archive→完成引用，未变化源帧返回准确工作节点时，两个不可变archive移入原树history保留；再准确退休工作/恢复原件/最终显式lease释放，全树/锚点/stable证明重验。history读回只查原snapshot对象，不拿之后活树当旧baseline。失败窗口覆盖intent后未创建、两archive后未result、result发布而CAS前、真实单字节current部分写、第一nonemptychunk中断、第二根缺席、重复purpose和archive未知child，原后像与pending保留、poison、不catchup/补记录/删现场。

生产Ready接线独立：begin_ready_snapshot_link/record_ready_phase要求原Source/current/image/Owner/nativeaction、typed ServiceObservation原live record/listener和同活claim，还要准确Ready用途双根snapshot；Source与typedReady在前后复核。该生产工厂仅编译未实际执行；没有用控制器组件或纯数据PID/digest替代native Ready。当前原生组件只链接Observation，生命周期保持Preparing。正式PreStop/Stopped工厂、新Stop原动作权限桥、独立routes/原processscope和原共享内容policy、动态工作新增物/退休、完整Stop/Closed/history/successor/原Owner-H-S-D仍待，不宣称整组收口。

公共Start仍无条件拒绝，生产适配入口仍v2，旧v2/v3记录不迁移/续写。原缺席Codex安全退根blocked单列，不能放claim仅持NLK移根或保留目录冒Absent。不W2/双SID/模型业务请求/发布。

产品MSVC custom-protocol debug bins、最终产品输入/testRust与准确源码字节核验；前端32份复用；不签release预算或冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
"""+"\n".join(f"| {a['name']} | {a['bytes']} | {a['sha256']} |" for a in artifacts)+f"""

本轮{len(scenes)}场景私有内容ZIP核验，canonical snapshot事件和source snapshot/chunk原字节另归canonical-snapshot-events/raw-scene-records。{len(cleanup['removed'])}处已完成纯数据场景核完整目录/文件后清理，{cleanup['failedAndPendingScenesRetained']}失败/pending原场景与{cleanup['crossVolumePendingRootsRetained']} D日志根保留；原生39事件正例缺正式Closed，也保持pending。ZIP不是NTFS身份/安全克隆，所有历史拒绝删除/账户/Profile/Public/旧broker缺口保持。overallCleanupComplete=false。
"""
with (ROOT/"report.md").open("x",encoding="utf-8") as out:out.write(report)
with (DOCS/"docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md").open("a",encoding="utf-8",newline="\n") as out:
    out.write(f"\n\n## 2026-11-16 TEAM16/r6 双根native快照canonical引用\n\n源码 {head}/tree{tree}；证据 .adg/evidence/0.2.24/windows-dev/{ROOT.name}/。snapshot intent/current先写，真实两WorkFrame/archive借用Witness核ID/all8/字节后完成link；严绑purpose/观察序号/两根/source上下文，不接受文本。39事件原件恢复与显式release正例仍Preparing，不签Ready/Closed；archive history在还原工作句柄后保留。首library01 E0365/raw101无新EXE，移除无必要导出后原生/数据/受影响回归通过如report。产品debug MSVC bins0/{runs['product01']['elapsedMs']}ms；EXE摘要/完整表/失败及清理归final-status/report。\n\n生产typedReady snapshot工厂编译未实跑，PreStop/Stopped/跨动作桥/原policy/动态节点/Stop/Closed/后继/实际H-S-D仍待；v2生产入口/Start拒绝保持，原缺席退根blocked单列。不W2/双SID/发布，不重复旧raw5/空GUI。\n")
print(json.dumps(dict(head=head,tree=tree,productInputs=len(final),finalRust=len(rust),scenes=len(scenes),rawFiles=len(raw_files),snapshotEvents=len(links),runs=runs,artifacts=artifacts)))
