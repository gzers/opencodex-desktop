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
expected={"library01":(101,None,None),"library02":(0,6,0),"library03":(101,24,1),"library04":(0,1,0),"library05":(101,7,1),"library06":(0,8,0),"library07":(0,2,0),"product01":(0,None,None)}
runs={};scenes=[];final=[];rust=[];journal_files=[];native_frames=[];archives=[];traces=[]
for name,(rc,passed,failed) in expected.items():
    result=load(ROOT/name/"result.json");assert result["exitCode"]==rc
    for row in load(ROOT/name/"source-manifest.json"):
        raw=(ROOT/name/"executed-source"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
        if name=="product01" or name=="library07" and row["path"].endswith(".rs"):
            assert raw==(CODE/row["path"]).read_bytes()
            assert raw.replace(b"\r\n",b"\n")==git("show",f"{head}:{row['path']}").replace(b"\r\n",b"\n")
            (final if name=="product01" else rust).append(row)
    if name=="library01":assert not load(ROOT/"library01-artifact.json")["executed"]
    if passed is not None:
        text=(ROOT/name/"stdout.log").read_text(encoding="utf-8-sig");m=re.search(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored",text)
        assert m and tuple(map(int,m.groups()))==(passed,failed,0)
        for line in text.splitlines():
            for marker,destination in [("r6-work-observed-native=",native_frames),("r6-work-native-archive-reference=",archives)]:
                if marker in line:destination.append(dict(run=name,observation=json.loads(line.split(marker,1)[1])))
            if "r6-work-" in line and len(line)<2500:traces.append(dict(run=name,trace=line[line.index("r6-work-"):]))
        for i,s in enumerate(load(ROOT/f"{name}-private-scenes.json")["scenes"]):
            assert sha(Path(s["archive"]).read_bytes())==s["archiveSha256"];scenes.append(s)
            for row in s["files"]:
                # D canonical phase/current plus full same-volume immutable
                # work observation chunks/metadata. Raw synthetic content only.
                if s["source"].startswith("D:") or "/.ocxd024-observed-" in "/"+row["path"]:
                    path=Path(row.get("source",str(Path(s["source"])/row["path"])));raw=path.read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
                    target=ROOT/"raw-scene-records"/name/f"scene-{i:03}"/row["path"];target.parent.mkdir(parents=True,exist_ok=True)
                    with target.open("xb") as out:out.write(raw)
                    journal_files.append(dict(source=str(path),evidence=target.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
    runs[name]=dict(rawExit=rc,passed=passed,failed=failed,ignored=0 if passed is not None else None,elapsedMs=result["elapsedMs"],artifact=load(ROOT/f"{name}-artifact.json") if name.startswith("library") else None)
assert load(ROOT/"product01/source.json")["head"]==head
write("native-frame-observations.json",native_frames);write("native-archive-reference-observations.json",archives);write("native-work-traces.json",traces);write("raw-scene-records.json",journal_files)
write("final-source.json",dict(head=head,tree=tree,productInputs=final,finalTestRust=rust,workBytesEqual=True,gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyEditedOrStaged=True))
artifacts=[]
for name in ["opencodex-desktop","ocxd","ocx"]:
    p=CODE/f"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe";raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile="debug",features=["custom-protocol"]))
write("product01-artifacts.json",artifacts)
for row in load(ROOT.parent/"wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json"):
    raw=(CODE/"apps/desktop/ui/dist"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
matrix=load(ROOT.parent/"wl2-r6-source-actions-20261116-r1/final-status.json")["matrix"]
matrix.extend([
 dict(id="CanonicalWorkWritableInterval",result="native-component-pass; official-sideeffects-not-admitted",fact="Same node/lease/current chain adds work-open intent/verified and seal intent/verified. Actual work creation handle uses ReOpenFile exact metadata observer while original custody/anchors/stable share remains strict. Active claim guards all transitions. Synthetic writes succeed only after full Open, frozen observation denies later writes. Native old work object retained across atomic replace; old unlink0or1 lookup limited to already-known work observation ID, never original/current path authority."),
 dict(id="WholeAnchoredWorkObservation",result="component-pass",fact="Current bounded full tree frozen with actual file handles/IDs/all8/children. All non-artifacts exact generation/content/security; unknown empty dir/node, ADS/hardlink/allowed-name directory refuse. Exact3 Codex artifact leaves may differ only as data, no baseline adoption or mutation permit. Full unchanged frame equality can re-enable precise retirement+restore; changed digest cannot launder older baseline."),
 dict(id="SameVolumeWorkObservationArchive",result="component-pass; phase-reference-link-incomplete",fact="Native frame full bytes/metadata sealed into private same-volume immutable chunks<=1MiB, actual archive/part/metadata IDs/all8 and hashes rechecked.1280KiB reconstructs two exact chunks; partial first NONEMPTY chunk retained and poison; unknown archive empty child refuses. Snapshot context binds actual live Journal session/plan/binding/manifest/deadline and both observed digests, but archive ref not yet in canonical official Ready/PreStop/Stopped phase or source sideeffect chain."),
 dict(id="WorkHandleAndPersistenceInterruptions",result="component-pass",fact="Durable work intent before handle transition, exact old/new handles held before result, result publication/current partial failure preserve Opening or pending native image and poison; no release when incomplete Opening/Sealing. Current retains separate baseline and observed frame digests; textual Ready/Closed/native authority unknown fields refuse."),
])
cleanup=load(ROOT/"cleanup.json")
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract="TEAM16/IMP142/r6",baselineMain="fac8573e55441fe514919c63c7f1d65181ff3577",baselineDocs="5782307174f0d1b011b5415c6c1e8d60414e6b36",runs=runs,artifacts=artifacts,matrix=matrix,
 firstError=dict(run="library01",rawExit=101,rustError="E0277",stage="ArtifactFrame entry-copy iterator returned Entry despite fallible snapshot encoding",newExecutableProduced=False,fixedBy="closure returns Result<Entry> and collects bounded fallible data"),
 firstNativeError=dict(run="library03",rawExit=101,passed=24,failed=1,stage="old work observer after external atomic replacement has0 links; generic id expected1",rawWin32=None,fixedBy="exact expected registered-work-ID observation only accepts0or1 links; all current/original path single-link checks unchanged",fixedTarget="library04",finalWorkRegression="library06"),
 secondNativeError=dict(run="library05",rawExit=101,passed=7,failed=1,stage="new historical work check incorrectly placed at custody-only pre-acquire caused original hardlink assertion to fail earlier",fixedBy="move historical work identity check after actual complete frame capture; current hardlink/ADS frame checks and original test assertions unchanged",finalWorkRegression="library06"),
 coverage=dict(finalWorkCases=8,finalWorkPassed=8,finalArchivePoisonIncrementPassed=2,previousAffectedRegression=25,previousAffectedPassed=24,previousAffectedFailed=1,unchangedRemainderReused=True),
 limits=["Working opens currently component-only; new source factory/production adapter and formal phase not invoked","Archive reference not yet included in canonical Ready/PreStop/Stopped, content policy assessment or typed Stop/native Closed","Changed artifact data is frozen for decision only; no current-generation baseline replacement or official write authorization","Different ActionWindow/production context handoff/source route/history/successor/H-S-D remain incomplete","OriginalAbsent Codex initialization/safe retirement remains blocked; old ordinary MSI/CAP-trust/historical cleanup remain"],
 allAccepted=False,startOpen=False,productionExecuted=False,productionAdapterUsesV3=False,officialOwnerCreated=False,W2=False,dualSID=False,published=False,
 cleanup=dict(archivedScenes=len(scenes),archivedFiles=sum(len(s["files"]) for s in scenes),archivedBytes=sum(f["bytes"] for s in scenes for f in s["files"]),completedOwnScenesRemoved=len(cleanup["removed"]),failureAndPendingScenesRetained=cleanup["failedAndPendingScenesRetained"],crossVolumePendingRootsRetained=cleanup["crossVolumePendingRootsRetained"],knownSyntheticADSArchivedAsObservationOnly=True,nativeMetadataCloned=False,historicalRejectedDeletionRetried=False,newAccounts=0,newProfiles=0,newPublicRoots=0,newGUIProcesses=0,newOfficialServices=0,dailyPathConfigPolicyChanged=False,overallCleanupComplete=False))
write("final-status.json",status)
report=f"""# r6工作代次可写窗口与完整后像

TEAM16/IMP142/r6，源码{head}/tree{tree}。承接6dfdd33ad/64616427c。只Windows original/native实现与tests，原预算/共享trait/副作用policy/官方来源/日常策略保持。

| 运行 | 原退出码 | 过/败/忽略 | ms |
| --- | --- | --- | --- |
"""+"\n".join(f"| {n} | {r['rawExit']} | {r['passed']}/{r['failed']}/{r['ignored']} | {r['elapsedMs']} |" for n,r in runs.items())+"""

首编译library01 E0277：ArtifactFrame数据entry-copy闭包返回Entry却使用?与Result收集；raw101源/stderr保留，无本轮新EXE。改Result<Entry>后library02六过。扩展受影响node/lease/v3回归library03为24过1败：外部原子替换的新工作代次已冻结，测试再用通用单链接id查询旧工作观察句柄，旧对象链接数为0而InvalidData；不是原件移动或sideeffect内容通过。修为受限retained_work_identity，仅对已登记准确旧WORK ID/磁盘非reparse非目录句柄接受links0或1，拒绝多链；不能验证当前路径、original custody、rename或恢复。原通用单链接检查不动。target04通过；library05七过1败，新历史WORK检查误放custody-only前检，使硬链用例在取锁前拒绝而非原要求完整frame扫描。保留原断言，把历史WORK检查移到完整frame capture之后，当前ADS/硬链仍由原scanner拒绝。final06八过，原两份失败/EXE/现场保留，不改原子替换/双代次/硬链断言。

canonical work-open/seal意图先写同一v3不可覆盖phase/current，才实际原生句柄交接与后像读回。工作CREATE_NEW原句柄通过ReOpenFile开同对象只metadata观察句柄，保留原work generation而允许外部写入；分享三项只用于本代WORK观察，原件custody和锚点/稳定锁的严格原句柄不动。微软primary-reference原始网页摘要与使用边界单列，不凭文档推定本机成功。活claim/NLK+原deadline保留，完整Open/release后合成写可行，再取得原lease后冻结完整后像，后来写者拒绝。原先工作句柄和原子替换后当前代次独立保留。

完整native frame包含root/nested/empty/每file/两稳定对象的实际ID/8投影/字节/children/原parent。非产物节点原代次与完整内容/安全必须不变，根children只对准确3种Codex产物提供观察数据例外；Grok无例外。未知file/空目录、ADS/硬链、准确产物名却目录均拒绝且保留完整或部分frame。current同时持baseline和observed digest，不能把观测变化重新标成旧baseline；Open/Observed禁止普通节点退休/恢复。只有native全树后像与开放前完整相同，才持claim重新接准确工作对象并登记WorkUnchangedVerified后继续原件恢复。changed artifact仍仅data，不能代允许副作用或Stop恢复许可。

同卷私有快照分块<=1MiB/原始metadata，逐记录CREATE_NEW/sync/readback，保留实际目录/part/metadata ID/all8/摘要与完整名称集合。context来自活原Journal，同session/plan/binding/manifest/deadline+两个observed digests；不能从路径或摘要文本构造。1280KiB/两块实际重建相同，原子新work generation与旧observer分开；故意无效cache字节只验证数据保存，不签原policy格式。首NONEMPTY分块刷盘后故障保留真实chunk/frame并poison；archive目录外加空child拒绝。补归档读回失败永久complete=false的边界，library07只定向两项受影响archive用例通过，其余原8项复用。该引用目前还没有正式Ready/PreStop/Stopped phase linkage，未采用作恢复资格。

四处intent/native/result/current部分写故障保持准确Opening/pending前缀与句柄，树/journal同时poison，Opening/Sealing未完成不能释放租约，不能在unheld状态逃逸回调。受影响节点驱动/租约/v3纯反例回归保留；未重复旧raw5/空GUI/原1314或无变化shared route/deadline测试。

未完：完整v3 Source/Owner Store生产接入、正式阶段及snapshot reference、跨原动作Stop权限桥、源绑定Ready/PreStop/Stopped全影像+原共享内容policy、动态产物新增/退休与代次恢复、完整normal exit/supervisor/source/endpoint/typed Closed/history/successor及原包H-S-D。新生产factory仍编译未实跑；public Start无条件拒绝。原缺席Codex安全退根blocked单列，不能留目录/两锁冒Absent。不W2/双SID/真实业务模型请求/发布。

产品custom-protocol MSVC debug bins、最终输入/testRust与源码字节核验，前端32份复用。不是release预算或冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
"""+"\n".join(f"| {a['name']} | {a['bytes']} | {a['sha256']} |" for a in artifacts)+f"""

{len(scenes)}场景私有ZIP逐内容核验；canonical phase/current与本卷work snapshot块/metadata原字节另归raw-scene-records，原生frame与archive引用归native-*。已知合成ADS额外单独复制为观察，不当NTFS克隆；{len(cleanup['removed'])}处已完成纯数据夹具核所有文件和目录后清理，{cleanup['failedAndPendingScenesRetained']}处失败/正式Closed未闭合pending原场景与{cleanup['crossVolumePendingRootsRetained']} D日志根保留。全部历史拒绝删除/账户/Profile/Public和broker保持，不重试；overallCleanup=false。
"""
with (ROOT/"report.md").open("x",encoding="utf-8") as out:out.write(report)
with (DOCS/"docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md").open("a",encoding="utf-8",newline="\n") as out:
    out.write(f"\n\n## 2026-11-16 TEAM16/r6 工作可写与后像组件\n\n源码 {head}/tree{tree}；证据 .adg/evidence/0.2.24/windows-dev/{ROOT.name}/。同v3 node/lease/current work-open/seal事件，活claim下准确work ReOpenFile观察，完整frame冻结+同卷chunk/metadata；未变化全树允许继续退休还原，变化产物仅data。首library01 E0277/raw101无新EXE；library02六过；library03受影响回归24过1败（原子替换后的旧work observer links0与通用单链接helper冲突），受限已登记旧WORK ID观察修正，target04一过；library05七过1败（新增历史观察位置过早破坏原硬链测试阶段），保持原断言移回完整frame边界，final06八过。原失败/双代次/ADS观察/部分chunk/current源与EXE保留，完整表归final-status/report，产品debug MSVC bins raw0/{runs['product01']['elapsedMs']}ms。\n\n正式phase与snapshot reference/source内容policy、生产v3 Store/跨动作权限、动态工作node/恢复/Stop/Closed/history/后继/原Owner-H-S-D仍待；production factory未实跑、Start无条件拒绝。原缺席安全退根blocked单列。不旧raw5/空GUI/W2/双SID/发布。\n")
print(json.dumps(dict(head=head,tree=tree,productInputs=len(final),finalRust=len(rust),scenes=len(scenes),rawFiles=len(journal_files),artifacts=artifacts)))
