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
runs={};final=[];rust=[];scenes=[];raw_files=[];traces=[]
for name,rc in [("library01",101),("library02",0),("library03",0),("product01",0)]:
    result=load(ROOT/name/"result.json");assert result["exitCode"]==rc
    for row in load(ROOT/name/"source-manifest.json"):
        raw=(ROOT/name/"executed-source"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
        if name=="product01" or name=="library03" and row["path"].endswith(".rs"):
            assert raw==(CODE/row["path"]).read_bytes()
            assert raw.replace(b"\r\n",b"\n")==git("show",f"{head}:{row['path']}").replace(b"\r\n",b"\n")
            (final if name=="product01" else rust).append(row)
    passed=failed=ignored=None
    if name=="library01":assert not load(ROOT/"library01-artifact.json")["executed"]
    if name in ["library02","library03"]:
        text=(ROOT/name/"stdout.log").read_text(encoding="utf-8-sig");m=re.search(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored",text);assert m
        passed,failed,ignored=map(int,m.groups());assert failed==0 and ignored==0 and passed>0
        if name=="library02":assert passed==3
        traces.extend(dict(run=name,trace=line[line.index("r6 "):]) for line in text.splitlines() if "r6 " in line and len(line)<2000)
        for i,s in enumerate(load(ROOT/f"{name}-private-scenes.json")["scenes"]):
            assert sha(Path(s["archive"]).read_bytes())==s["archiveSha256"];scenes.append(s)
            for row in s["files"]:
                if s["source"].startswith("D:") or "/.ocxd024-observed-" in "/"+row["path"]:
                    p=Path(row.get("source",str(Path(s["source"])/row["path"])));raw=p.read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
                    target=ROOT/"raw-scene-records"/name/f"scene-{i:03}"/row["path"];target.parent.mkdir(parents=True,exist_ok=True)
                    with target.open("xb") as out:out.write(raw)
                    raw_files.append(dict(source=str(p),evidence=target.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
    runs[name]=dict(rawExit=rc,passed=passed,failed=failed,ignored=ignored,elapsedMs=result["elapsedMs"],artifact=load(ROOT/f"{name}-artifact.json") if name.startswith("library") else None)
write("fresh-action-traces.json",traces);write("raw-scene-records.json",raw_files)
assert load(ROOT/"product01/source.json")["head"]==head
write("final-source.json",dict(head=head,tree=tree,productInputs=final,finalTestRust=rust,workBytesEqual=True,gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyEditedOrStaged=True))
artifacts=[]
for name in ["opencodex-desktop","ocxd","ocx"]:
    p=CODE/f"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe";raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile="debug",features=["custom-protocol"]))
write("product01-artifacts.json",artifacts)
for row in load(ROOT.parent/"wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json"):
    raw=(CODE/"apps/desktop/ui/dist"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
matrix=load(ROOT.parent/"wl2-r6-snapshot-links-20261116-r1/final-status.json")["matrix"]
matrix.extend([
 dict(id="TimelyReleaseFreshHistory",result="native-component-pass",fact="Actual original1s lease explicitly unlocked and native receipt completed before deadline; fresh native history reads after actual expiry preserve4 IDs/all8/bytes but cannot use generic reacquire(new deadline). Historical read is DATA only, not reauthorized old lease. Previous released objects remain checked under current action read window."),
 dict(id="CanonicalOriginalStopActionBridge",result="compiled-native-factory/data-counterexamples-pass; actual-Owner-positive-unexecuted",fact="Original Source/Owner/typedReady -> fresh NativeJournalContext and exact Stop/Restart NativeAction required. Intent/current keeps old scope and actual pending native receipt, verified binding advances current to new original scope;13 context drifts/stale Ready/missing release/duplicate scopes/pending lease operations refuse. No generic deadline setter, no serde action factory. New native bound reacquire requires actual Owner and NativeAction, original4 objects/scope; no synthetic production bridge."),
 dict(id="TypedPreStopTeardownStoppedV3",result="compiled-native-factories/data-progression-pass; production-execution-pending",fact="Current retains exact Ready PID/port/record digest; PreStop requires actual Source/current/live typed Ready+original Stop window and both PreStop archives. TeardownPrepared persisted before official callback; Stopped requires exact NormalExit/SupervisorAbsent/StopAccepted/record/listener/source and both Stopped archives. DATA stages refuse generation drift/order/foreign refs, never issue Closed. No actual new factory invocation or v3 adapter switch yet."),
])
cleanup=load(ROOT/"cleanup.json")
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract="TEAM16/IMP142/r6",baselineMain="fac8573e55441fe514919c63c7f1d65181ff3577",baselineDocs="5782307174f0d1b011b5415c6c1e8d60414e6b36",runs=runs,artifacts=artifacts,matrix=matrix,
 firstError=dict(run="library01",rawExit=101,rustError="E0603",stage="shared pure test helpers visible only to snapshots parent, inaccessible from sibling action tests",newExecutableProduced=False,fixedBy="test-helper visibility restricted to v3 only; no production type visibility widened"),
 limits=["Fresh Stop native factory and PreStop/Stopped production factories compile but actual original Owner/typed official proof path not exercised","Native new action data tests progress serialized values only; they construct no fake NativeAction/Owner/NormalExit/Absent/StopAccepted","Next actual v3 Store/source/routes adapter, full phase snapshot intervals and original shared content policy/changed artifact restoration/dynamic nodes/Closed/history/successor/H-S-D still incomplete","OriginalAbsent Codex safe initialization/retirement blocked; ordinary MSI/CAP-trust/historical cleanup remain"],
 productionAdapterUsesV3=False,allAccepted=False,startOpen=False,productionExecuted=False,officialOwnerCreated=False,newNativeStopFactoryExecuted=False,W2=False,dualSID=False,published=False,
 cleanup=dict(archivedScenes=len(scenes),archivedFiles=sum(len(s["files"]) for s in scenes),archivedBytes=sum(f["bytes"] for s in scenes for f in s["files"]),completedOwnScenesRemoved=len(cleanup["removed"]),failureAndPendingScenesRetained=cleanup["failedAndPendingScenesRetained"],crossVolumePendingRootsRetained=cleanup["crossVolumePendingRootsRetained"],nativeMetadataCloned=False,historicalRejectedDeletionRetried=False,newAccounts=0,newProfiles=0,newPublicRoots=0,newGUIProcesses=0,newOfficialServices=0,dailyPathConfigPolicyChanged=False,overallCleanupComplete=False))
write("final-status.json",status)
report=f"""# r6原Owner新Stop动作与PreStop/Stopped链

TEAM16/IMP142/r6，源码{head}/tree{tree}；承接d0a31c2de/fec063512。只Windows原生允许集，原trait/预算/官方/日常配置/未知拒绝保持。

| 运行 | 原退出码 | 过/败/忽略 | ms |
| --- | --- | --- | --- |
"""+"\n".join(f"| {n} | {r['rawExit']} | {r['passed']}/{r['failed']}/{r['ignored']} | {r['elapsedMs']} |" for n,r in runs.items())+"""

首错library01 E0603：三个纯数据辅助函数原pub(super)只到snapshots父模块，兄弟action测试不能调用。原源码/stderr/raw101保留，无新EXE；仅test helper可见范围限定v3，不扩大生产证明构造器。library02三项（两纯状态反例＋一实际native历史读回）通过，受影响进度/节点/lease/snapshot/current回归如上；不重复旧raw5/空GUI/1314/无关安装器。

实际native测试：四个独立合成协调文件，原1秒lease在截止前显式unlock成功并核原对象/8投影/字节；真实等待原截止过去，随后新1秒读预算下readback_history可以鲜读原对象，但generic reacquire(None,newEnd)仍拒绝，不能把历史读当新权限。原释放回执保存completed_at<原deadline，不从Drop/JSON生成；所有原句柄/parent pin仍保留。旧check/reacquire的原时间约束不删除，只有新实际Owner+NativeAction的专用factory才可重取全域。

新Stop/Restart bridge：实际Source/current/image/Owner和typed Ready先复核，取得原Owner保存的原动作NativeJournalContext/NativeAction，严格同原session/scope/token/Binding/Ready代次。没有通用set_deadline。先保管新原生pending action用于只读旧journal/历史回执核验，写StopActionIntent/current保持旧scope；结果verified刷盘/CAS并再验实际Owner/Ready后，current换新原scope UUID，保留旧action对象历史。pending期间阻止节点/work/lease/snapshot权限逃逸。原lease仍原deadline不改；实际new lease factory要求新NativeAction/原Owner/原4协调对象ID/all8/字节和scope全部重验，new epoch依原事件链登记，不能把过去持有当现在持有。

上述原生产factory仅编译，未创建测试版fake Owner或可序列化NativeAction；纯数据反例验证13处context、旧Ready、跳序/重复动作/旧scope、unknown deadline字段、pending取锁均拒绝。合规locator状态转换不等于native动作许可。

current同时保留Ready的原PID/port/recordDigest与actionKind/pending。PreStop snapshot工厂必须在实际新Stop/Restart原窗、原Source与typed live Ready、同活claim下；phase只接同purpose双根native引用，代次漂移拒绝。TeardownPrepared在原链持久，释放完整临界段后才允许后续官方请求，不能持claim等官方自取锁。Stopped工厂要求实际NormalExit/原SupervisorAbsent/typed StopAccepted、已记录原Ready代次、端点/record/Source原生后验和同Stop deadline，再接两根Stopped快照；只是Stopped阶段，不发Closed。新生产工厂均未实跑，完整新phase快照区间/工作后像series仍待组合。

剩余产品要求保持：实际v3 Store首次变更前保管/source/route接入口、Ready/PreStop/Stopped全影像+原副作用policy、动态产物/新目录准确退休恢复、完整原NormalExit/监督/监听/record/current/typed Closed/history/后继、普通实际原Owner H-S-D生产。原缺席Codex初始化/安全退根仍blocked，不能以existing-root代首次空环境。生产入口仍v2、公共Start无条件拒绝，未W2/双SID/模型业务请求/发布。

产品custom-protocol MSVC debug bins，最终产品输入/testRust与源码字节核验、前端32份复用；不是release预算或冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
"""+"\n".join(f"| {a['name']} | {a['bytes']} | {a['sha256']} |" for a in artifacts)+f"""

{len(scenes)}场景私有ZIP内容核验，D canonical事件/current与工作快照块原字节另归raw-scene-records；{len(cleanup['removed'])}处已完成native历史/纯数据场景核全部文件和目录后清理，{cleanup['failedAndPendingScenesRetained']}失败/pending原场景和{cleanup['crossVolumePendingRootsRetained']} D日志根保留。ZIP不是NTFS身份/安全克隆，所有旧拒绝删除/账户/Profile/Public/旧broker与普通MSI/CAP-trust缺口不代签，overallCleanupComplete=false。
"""
with (ROOT/"report.md").open("x",encoding="utf-8") as out:out.write(report)
with (DOCS/"docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md").open("a",encoding="utf-8",newline="\n") as out:
    out.write(f"\n\n## 2026-11-16 TEAM16/r6 原Owner Stop动作及阶段接线\n\n源码 {head}/tree{tree}；证据 .adg/evidence/0.2.24/windows-dev/{ROOT.name}/。canonical新StopAction intent/current/result绑定实际Owner/Source/typedReady新原窗；旧release completed_at历史鲜读不延长原lease；PreStop/teardown/Stopped工厂保持实际NormalExit/Absent/StopAccepted/source/记录/端点与双根引用断言。新生产factory编译未实跑，非v3入口切换或native Stop正例。首library01 E0603/raw101无新EXE，修test helper到v3可见，library02三项通过（真实lease原1秒到期后历史读不grant重取＋两状态数据反例），受影响回归见report；产品MSVCdebug bins raw0/{runs['product01']['elapsedMs']}ms。原始源码/EXE/完整表/首错/清理归final-status/report。\n\n实际v3 Store、全phase snapshots/原policy/动态工作节点/Stop恢复/Closed/history/后继/原Owner H-S-D仍待；原缺席退根blocked单列且Start无条件拒绝。不旧raw5/空GUI/W2/双SID/发布。\n")
print(json.dumps(dict(head=head,tree=tree,productInputs=len(final),finalRust=len(rust),scenes=len(scenes),rawFiles=len(raw_files),runs=runs,artifacts=artifacts)))
