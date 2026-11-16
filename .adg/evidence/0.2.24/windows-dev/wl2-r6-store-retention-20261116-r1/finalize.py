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
for name in ["library01","library02","library03","product01"]:
    result=load(ROOT/name/"result.json");assert result["exitCode"]==0
    for row in load(ROOT/name/"source-manifest.json"):
        raw=(ROOT/name/"executed-source"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
        if name=="product01" or name=="library03" and row["path"].endswith(".rs"):
            assert raw==(CODE/row["path"]).read_bytes()
            assert raw.replace(b"\r\n",b"\n")==git("show",f"{head}:{row['path']}").replace(b"\r\n",b"\n")
            (final if name=="product01" else rust).append(row)
    passed=failed=ignored=None
    if name.startswith("library"):
        text=(ROOT/name/"stdout.log").read_text(encoding="utf-8-sig");m=re.search(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored",text);assert m
        passed,failed,ignored=map(int,m.groups());assert passed>0 and failed==0 and ignored==0
        if name=="library01":assert passed==5
        if name=="library03":assert passed==2
        traces.extend(dict(run=name,trace=line[line.index("r6 "):]) for line in text.splitlines() if "r6 " in line and len(line)<2000)
        for i,s in enumerate(load(ROOT/f"{name}-private-scenes.json")["scenes"]):
            assert sha(Path(s["archive"]).read_bytes())==s["archiveSha256"];scenes.append(s)
            for row in s["files"]:
                if s["source"].startswith("D:") or "/.ocxd024-observed-" in "/"+row["path"]:
                    p=Path(row.get("source",str(Path(s["source"])/row["path"])));raw=p.read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
                    target=ROOT/"raw-scene-records"/name/f"scene-{i:03}"/row["path"];target.parent.mkdir(parents=True,exist_ok=True)
                    with target.open("xb") as out:out.write(raw)
                    raw_files.append(dict(source=str(p),evidence=target.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
    runs[name]=dict(rawExit=0,passed=passed,failed=failed,ignored=ignored,elapsedMs=result["elapsedMs"],artifact=load(ROOT/f"{name}-artifact.json") if name.startswith("library") else None)
write("native-retention-traces.json",traces);write("raw-scene-records.json",raw_files)
assert load(ROOT/"product01/source.json")["head"]==head
write("final-source.json",dict(head=head,tree=tree,productInputs=final,finalTestRust=rust,workBytesEqual=True,gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyEditedOrStaged=True))
artifacts=[]
for name in ["opencodex-desktop","ocxd","ocx"]:
    p=CODE/f"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe";raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile="debug",features=["custom-protocol"]))
write("product01-artifacts.json",artifacts)
for row in load(ROOT.parent/"wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json"):
    raw=(CODE/"apps/desktop/ui/dist"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
equivalence=load(ROOT/"owned-boundary-equivalence.json");assert len(equivalence["methods"])==5 and equivalence["activeNonSendMarkerRetained"] and not equivalence["unsafeImplSendAdded"]
matrix=load(ROOT.parent/"wl2-r6-stop-actions-20261116-r1/final-status.json")["matrix"]
matrix.extend([
 dict(id="ReleasedOriginalObjectsVsLiveLease",result="native-component-pass/compiler-Send-proof",fact="Only after explicit range-empty/readback native release, exact original LockFile+parent pins move by ownership into ReleasedObjects with no NativeLease/OVERLAPPED/active range. Same facts before/after; actual receipt crosses worker+returns then fresh single-use local acquisition. NativeLease remains stack-only nonSend marker; no handle cloning/reopen or unsafe Send."),
 dict(id="V3RetainedCaptureCore",result="native-component-pass",fact="Complete original trees+actual journal stored at sequence0 before any move;18 anchor/node events prepare3 original/work files,20 records explicit release. Exact core/history/full original trees/journal cross a real worker and return; actual failed event/current/native-move prefixes remain in installed core, no replace/catchup. Source is not faked in these component tests."),
 dict(id="V3ProductionCaptureSlot",result="compiled/registered-not-invoked",fact="WindowsOriginals now owns actual v3 CaptureSlot, whose production preparation method requires original Owner/ActionWindow/Source/RoutePin and full NativeLease. Store installed before moving original files, live lease stays local, released receipt persisted. Fresh namespace rejects ANY old entry incl textClosed. Failed/occupied v3 attempt blocks v2 fallback; old v2 records not converted. Production capture dispatcher/remaining phase flow not yet switched or executed."),
 dict(id="ActivePartialRangeCleanup",result="affected-native-regression-pass",fact="NativeLease check/release method bodies token-identical; Held no longer needs independent Drop so range-free original pins move safely. Held acquisition explicitly attempts own partial unlock on errors before enrollment, NativeLease remains sole guard for enrolled ranges. Original raw busy/expiry/drift and unknown refusal retained; no dropping to claim ordinary release."),
 dict(id="V3ProductionJournalNamespace",result="native-component-pass",fact="Static integration review found direct D/session component journal vs production D/official024/windows/UUID mismatch. Strict private canonical native ancestors and UUID leaf now bind immutable manifest/current same session; wrong descendants/nonUUID refuse. Source factory requires canonical namespace; direct D child remains component-only, actual Source factory not invoked."),
])
cleanup=load(ROOT/"cleanup.json")
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract="TEAM16/IMP142/r6",baselineMain="fac8573e55441fe514919c63c7f1d65181ff3577",baselineDocs="5782307174f0d1b011b5415c6c1e8d60414e6b36",runs=runs,artifacts=artifacts,matrix=matrix,
 firstUnexpectedError=None,controlledFailures=["capture anchor event/current write failures retained before native move","capture first actual native move before result retains exact original custody and pending","old namespace/textClosed refusal","fresh local acquisition remains explicit/single-use; no active lease transfer"],
 staticIntegrationFinding="component direct-child journal vs canonical production path rejected before client moves; fixed strict native namespace/UUID session binding, no failed production run fabricated",
 productionV3CaptureSlotRegistered=True,productionV3CaptureInvoked=False,productionAdapterUsesV3=False,allAccepted=False,startOpen=False,productionExecuted=False,officialOwnerCreated=False,W2=False,dualSID=False,published=False,
 limits=["Source/route/actual Owner production v3 capture method compiles and is retained by original adapter, but production dispatcher is not switched and actual full source factory was not invoked","Native core tests use real synthetic files/pins/journal/leases, without constructing fake production Source/Owner/NativeAction; no production qualification from Send or synthetic source digests","Next full phase writable/archive series, source-bound declared side-effects and original shared content policy, dynamic nodes/whole restoration/typed Closed/history/successor/actual Owner H-S-D remain incomplete","OriginalAbsent Codex safe initialization/retirement blocked; ordinary MSI/CAP-trust/historical cleanup remain"],
 cleanup=dict(archivedScenes=len(scenes),archivedFiles=sum(len(s["files"]) for s in scenes),archivedBytes=sum(f["bytes"] for s in scenes for f in s["files"]),completedOwnScenesRemoved=len(cleanup["removed"]),failureAndPendingScenesRetained=cleanup["failedAndPendingScenesRetained"],crossVolumePendingRootsRetained=cleanup["crossVolumePendingRootsRetained"],nativeMetadataCloned=False,historicalRejectedDeletionRetried=False,newAccounts=0,newProfiles=0,newPublicRoots=0,newGUIProcesses=0,newOfficialServices=0,dailyPathConfigPolicyChanged=False,overallCleanupComplete=False))
write("final-status.json",status)
report=f"""# r6释放对象分离与首次变更前v3 Store保管

TEAM16/IMP142/r6，源码{head}/tree{tree}；承接229bcb934/532105f72。Windows独占范围，原trait/预算/Token/8投影/未知拒绝不变，不改官方或日常配置。

| 运行 | 原退出码 | 过/败/忽略 | ms |
| --- | --- | --- | --- |
"""+"\n".join(f"| {n} | {r['rawExit']} | {r['passed']}/{r['failed']}/{r['ignored']} | {r['elapsedMs']} |" for n,r in runs.items())+"""

本轮无非预期原生首错。5项新定向通过；受影响旧kernel ranges/各角色busy/expiry/byte drift、lease分段、v3进度及new capture保管回归27过。静态接线复核发现组件journal只接受D直接子目录，与生产D/official024/windows/UUID不匹配，原代码会在任何客户端移动前拒绝；修严格生产命名空间祖先pin及native UUID同session绑定，再两项定向通过。不捏造生产失败执行；仅实际源码缺陷。旧raw5根移动/空GUI/1314、无变化官方读取/安装器不重跑。

释放后类型分离：NativeLease仍持PhantomData<*mut()>和活OVERLAPPED范围，保持非Send。只有原来全部UnlockFileEx成功、范围空、原4文件/parent/Token/ID/8投影/字节/sidecar读回后，准确原LockFile对象及Bound pins按所有权移入ReleasedObjects，里面没有NativeLease或range/OVERLAPPED，不复制/重开句柄。facts在拆分前后同值；真实线程移交历史receipt+读回+返回后，主线程仍须实际新取得才有活lease，新epoch且单用，不能用迁移历史对象授权重复获取。全程无unsafe impl Send或令牌/ACL修补。

Held原来单独Drop阻止直接移出原pin，现由NativeLease统一保管已登记Held；尚未登记的Held取得失败时先明确尝试其自有partial ranges unlock，再按原错误返回。失败cleanup关闭自有文件不算普通释放成功，收据仍必须显式原生release。NativeLease原check_inner/release方法体token保持基线，原范围/分享/忙立即失败/原deadline/poison语义不变。RoutePin capture/check/declared方法体同样token一致，只把可见范围限定到同Windows原件模块供新Store复用，原共享route解析未改。5份token等价与active marker/无unsafeSend记录归owned-boundary-equivalence。

原生CapturedTrees核心：完整原树+actual journal在序号0先进入Option Store，再发anchor/file意图与native动作；18记录准备3份普通原件与工作副本，20含显式释放后，准确历史文件/pin、全原件树和同journal可以实际跨worker并返回。事件/current失败导致0原件移位，native首次move后失败留下1准确原件custody；所有树/journal被poison，Store仍可定位，不替换/追赶/自动重试。这里是合成native组件，没有fake Source/Owner/NativeAction或production Ready/Closed。

生产v3 CaptureSlot已注册在原WindowsOriginals中。prepare_v3_capture要求原Owner/原ActionWindow/Source/current/image/route/完整NativeLease；拒绝旧v2/native历史混用，D/official024/windows任意旧条目（含文本Closed）均拒绝新会话，不扫描接管/迁移/覆盖。Source与完整双原树/plan/native journal形成CaptureStore后先入原adapter槽，才开始首文件移动；Source/route/未Resume原Owner前后重核，完整工作后像核验后才Prepared→显式释放并留回执。活lease始终在栈上，失败时Store/原句柄/current/前缀保留并显式清理自有活范围。v3已有/失败尝试会阻止旧v2 capture fallback。

新production prepare方法仍未实际调用，原capture dispatcher/Ready及后续v3完整接口尚未切换。生产journal严格D/official024/windows/UUID，准确private祖先/目录pin，manifest/current沿该准确UUID；错目录/非UUID拒绝。直接D/session仅供原组件，actual Source factory不接受。编译器验证CaptureSlot/Source/route/全树/journal/ReleasedObjects自动Send，不能据此签生产通过或强行发活NativeLease。现有公共Start无条件拒绝，不以synthetic Binding/hash代实际Source资格。

剩余：正式v3入口/阶段/全snapshot writable系列、源绑定官方受控副作用/原共享policy、config/auth/Grok非产物声明范围、动态产物/新目录准确退休还原、完整NormalExit/监督/端点/record/source/typed Closed/原history/后继和实际普通Owner-H-S-D。原缺席Codex初始化及安全退根仍blocked，不能留目录/两锁冒Absent。普通MSI/CAP-trust/全部历史清理未代签。不W2/双SID/真实业务模型请求/发布。

产品MSVC custom-protocol debug bins和最终产品输入/testRust对工作/Git源码字节核验，前端32份复用；非release预算/冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
"""+"\n".join(f"| {a['name']} | {a['bytes']} | {a['sha256']} |" for a in artifacts)+f"""

{len(scenes)}场景私有内容ZIP核验，D canonical/source snapshot原字节另归raw-scene-records；{len(cleanup['removed'])}处已完成release历史/纯数据场景核全部文件和目录后清理，{cleanup['failedAndPendingScenesRetained']}失败/pending原场景和{cleanup['crossVolumePendingRootsRetained']} D日志根保留。入Store原生20事件正例仍缺正式Closed，保持pending。ZIP不是NTFS身份/安全克隆，旧删除拒绝/账户/Profile/Public/旧broker均不重试，overallCleanup=false。
"""
with (ROOT/"report.md").open("x",encoding="utf-8") as out:out.write(report)
with (DOCS/"docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md").open("a",encoding="utf-8",newline="\n") as out:
    out.write(f"\n\n## 2026-11-16 TEAM16/r6 原v3 CaptureStore与释放对象保管\n\n源码 {head}/tree{tree}；证据 .adg/evidence/0.2.24/windows-dev/{ROOT.name}/。显式release读回后准确原LockFile/pins移入无活range/NativeLease的历史集合，自动Send；活lease非Send标记/check/release方法体保持，无unsafeSend。实际核心入槽序0先于原件move，18工作准备/20显式释放，历史+原件树/journal真实worker往返通过，0/1原件移位失败前缀保留；新production CaptureSlot已注册、Source/route/原Owner全核并先入槽再动，旧pending/textClosed和v2 fallback拒绝。新生产factory未实跑、原dispatcher未切换，非整组通过。5新定向及受影响回归见report，本轮无非预期首错。产品debug MSVC bins0/{runs['product01']['elapsedMs']}ms，原始EXE/完整表/清理归final-status/report。\n\n后续v3完整阶段/全快照series/原policy与动态节点恢复/Closed/history/后继/实际普通Owner-H-S-D待；原缺席退根blocked且Start无条件拒绝，不W2/双SID/发布，不重复旧raw5/空GUI。\n")
print(json.dumps(dict(head=head,tree=tree,productInputs=len(final),finalRust=len(rust),scenes=len(scenes),rawFiles=len(raw_files),runs=runs,artifacts=artifacts)))
