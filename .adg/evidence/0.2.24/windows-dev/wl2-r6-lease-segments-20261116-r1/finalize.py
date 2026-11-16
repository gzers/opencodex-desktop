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
expected={"library01":(0,20,0),"library02":(101,21,1),"library03":(101,0,1),"library04":(0,1,0),"product01":(0,None,None)}
runs={};scenes=[];final=[];rust=[];reuse=[];journals=[];leases=[];traces=[]
for name,(raw_exit,passed,failed) in expected.items():
    result=load(ROOT/name/"result.json");assert result["exitCode"]==raw_exit
    for row in load(ROOT/name/"source-manifest.json"):
        raw=(ROOT/name/"executed-source"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
        if name=="product01" or name=="library04" and row["path"].endswith(".rs"):
            assert raw==(CODE/row["path"]).read_bytes()
            assert raw.replace(b"\r\n",b"\n")==git("show",f"{head}:{row['path']}").replace(b"\r\n",b"\n")
            (final if name=="product01" else rust).append(row)
        if name=="library02" and row["path"].endswith(".rs") and not row["path"].endswith("_tests.rs"):
            assert raw==(CODE/row["path"]).read_bytes(),row["path"]
            reuse.append(row)
    if passed is not None:
        text=(ROOT/name/"stdout.log").read_text(encoding="utf-8-sig");m=re.search(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored",text)
        assert m and tuple(map(int,m.groups()))==(passed,failed,0)
        traces.extend(dict(run=name,trace=line[line.index("r6-segment"):]) for line in text.splitlines() if "r6-segment" in line)
        traces.extend(dict(run=name,trace=line[line.index("r6-busy-"):]) for line in text.splitlines() if "r6-busy-" in line)
        for i,s in enumerate(load(ROOT/f"{name}-private-scenes.json")["scenes"]):
            assert sha(Path(s["archive"]).read_bytes())==s["archiveSha256"];scenes.append(s)
            if s["source"].startswith("D:"):
                for row in s["files"]:
                    raw=(Path(s["source"])/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
                    target=ROOT/"journal-scene-records"/name/f"scene-{i:02}"/row["path"];target.parent.mkdir(parents=True,exist_ok=True)
                    with target.open("xb") as out:out.write(raw)
                    journals.append(dict(source=str(Path(s["source"])/row["path"]),evidence=target.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
                    if "/event-" in row["path"]:
                        entry=json.loads(raw)
                        if entry["documentKind"]=="lease-event":leases.append(dict(run=name,scene=s["source"],evidence=target.relative_to(ROOT).as_posix(),event=entry))
    runs[name]=dict(rawExit=raw_exit,passed=passed,failed=failed,ignored=0 if passed is not None else None,elapsedMs=result["elapsedMs"],artifact=load(ROOT/f"{name}-artifact.json") if passed is not None else None)
assert load(ROOT/"product01/source.json")["head"]==head
raw=(ROOT/"library04/stdout.log").read_text(encoding="utf-8-sig")
assert "raw=Some(33)" in raw and "state=AcquirePrepared" in raw
assert "claim.sqlite-journal" not in raw.split("codexChildren=",1)[1].splitlines()[0]
write("segment-traces.json",traces);write("canonical-lease-events.json",leases);write("journal-scene-records.json",journals)
write("final-source.json",dict(head=head,tree=tree,productInputs=final,finalTargetTestRust=rust,reused21PassingTestsProductionRustEqual=reuse,workBytesEqual=True,gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyEditedOrStaged=True))
artifacts=[]
for name in ["opencodex-desktop","ocxd","ocx"]:
    p=CODE/f"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe";raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile="debug",features=["custom-protocol"]))
write("product01-artifacts.json",artifacts)
for row in load(ROOT.parent/"wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json"):
    raw=(CODE/"apps/desktop/ui/dist"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
matrix=load(ROOT.parent/"wl2-r6-node-driver-20261116-r1/final-status.json")["matrix"]
for row in matrix:
    if row["id"]=="V3LeaseSegmentsAndPhases":row.update(result="same-original-window-lease-segments-pass; production-phase/action-bridge-incomplete",fact="Same immutable node+lease event chain/current state; durable ReleasePrepared before actual four-range unlock, retained nonserde original-object receipt, explicit Released result, AcquirePrepared and fresh same-context/objects/deadline lease result before native node resume.36 events restore both trees. New/historical epoch replay, Drop-as-release, half tree/foreign data/empty dir/extended deadline and result/current failures refuse. Later lifecycle ActionWindow/source/phase permissions, official writable interval/archives/Stop/Closed remain pending; public Start gate stays.")
matrix.extend([
 dict(id="CanonicalLeaseState",result="component-pass",fact="Manifest binds initial epoch/claim/deadline; current binds active epoch/lease state/full four-object digest. Node and lease records replay in same immutable chain from initial epoch; previous release original handles/bytes/security continuously rechecked, no Drop-derived or serde receipt."),
 dict(id="LeaseBusyAndSidecar",result="component-pass-with-failures-retained",fact="Empty synthetic SQLite BEGIN EXCLUSIVE created transient unknown -journal, so earlier full-tree InvalidData/rawNone correctly refuses before acquire-prepared; original expected raw33 test failed in library02 and diagnostic03. New PRE-capture initialized rollback fixture has no sidecar; exact original raw33 assertion passes, pending AcquirePrepared, foreign transaction untouched, refuses retry after rollback. No production initialization or sidecar exception."),
])
cleanup=load(ROOT/"cleanup.json")
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract="TEAM16/IMP142/r6",baselineMain="fac8573e55441fe514919c63c7f1d65181ff3577",baselineDocs="5782307174f0d1b011b5415c6c1e8d60414e6b36",runs=runs,artifacts=artifacts,matrix=matrix,
 firstError=dict(run="library02",rawExit=101,rawWin32=None,passed=21,failed=1,stage="busy-claim expected33 assertion hit earlier unknown SQLite journal rejection",canonicalSequence=20,canonicalState="Released",diagnosticRun="library03",diagnosticRemainingMs=8609,fixedBy="fixture-only initialize synthetic rollback claim BEFORE capture; no product safety change",fixedTargetRun="library04"),
 finalCoverage=dict(reusedUnchangedPassing=21,finalAffectedTargetPassed=1,all22CasesCovered=True,finalFull22SuiteRepeated=False,sourceDeltaAfter21Pass="diagnostic and fixture-only setup helper; all production Rust verified byte-identical"),
 retainedControlledFailures=["prepared event/current failure before native unlock","explicit native unlock -> result/partial-current windows","new native lease -> result/partial-current windows","Drop without receipt/partial original tree/foreign objects/deadline extension/historical epoch","actual busy claim raw33 preserves canonical pending and foreign transaction"],
 productionAdapterUsesV3=False,allAccepted=False,startOpen=False,productionExecuted=False,officialOwnerCreated=False,W2=False,dualSID=False,published=False,
 cleanup=dict(archivedScenes=len(scenes),archivedFiles=sum(len(s["files"]) for s in scenes),archivedBytes=sum(f["bytes"] for s in scenes for f in s["files"]),completedOwnScenesRemoved=len(cleanup["removed"]),failureAndPendingScenesRetained=cleanup["failedAndPendingScenesRetained"],crossVolumePendingRootsRetained=cleanup["crossVolumePendingRootsRetained"],reusedTaskParentDirectoriesRetained=3,nativeMetadataCloned=False,historicalRejectedDeletionRetried=False,newAccounts=0,newProfiles=0,newPublicRoots=0,newGUIProcesses=0,newOfficialServices=0,dailyPathConfigPolicyChanged=False,overallCleanupComplete=False))
write("final-status.json",status)
report=f"""# r6同一原动作窗口租约释放与重新取得

TEAM16/IMP142/r6，源码{head}/tree{tree}。承接285be9fad/475f493e6；本轮fetch main/docs仍fac8573e5/578230717，未改正式合同/四trait/官方/日常策略。

| 运行 | 原退出码 | 过/败/忽略 | ms |
| --- | --- | --- | --- |
"""+"\n".join(f"| {n} | {r['rawExit']} | {r['passed']}/{r['failed']}/{r['ignored']} | {r['elapsedMs']} |" for n,r in runs.items())+"""

首轮20过。第二轮新严格数据反例与真实busy用例后21过1败：原断言要求Win32 raw33，但空合成SQLite的BEGIN EXCLUSIVE生成claim.sqlite-journal，整树完整children按合同提前InvalidData/rawNone拒绝。诊断03保留原断言，明确看到新sidecar；原截止还剩8609ms（后8603），current序20 Released，没有21 AcquirePrepared，不是超时或实际锁竞争已经成功。两轮raw101/原源码/EXE/失败现场保留，不把更早拒绝改成raw33通过。

只修夹具：合成rollback SQLite在native capture前初始化独立表，生产不初始化/不删或豁免sidecar。定向library04同原raw33断言通过，真正native acquire失败保留AcquirePrepared/current，foreign BEGIN EXCLUSIVE保持，由测试自行ROLLBACK；竞争消失后仍拒绝自动重试。其余21过复用library02，逐份生产Rust对最终字节相同，仅诊断及夹具setup(false保持原行为/true只新用例)改动；未重复全22套，也未重跑旧raw5/空GUI/共享route/deadline旧前提。

全部节点intent/result和LeaseReleasePrepared/LeaseReleasedVerified/LeaseAcquirePrepared/LeaseAcquiredVerified写同一不可覆盖phase链和准确current。manifest保留初始epoch/claim/deadline供整链重放，current保存当前epoch/leaseState/四域对象digest；不直接改内存绕过链。普通节点事件在Prepared/Released期间拒绝。每次Prepared只在两树完整Working或Restored、每原文件和工作代次完整时允许，不能把中途缺配置的树交官方使用。

释放：在同实际borrowed claim/NLK与原deadline核全树/原件custody，Prepared/current刷盘读回后才消费NativeLease，逐个UnlockFileEx并核范围空、原4文件ID/all8/字节/sidecar/Token/父pin，生成nonserde/nonClone非Send原生回执。回执保留原4文件及parent对象，不能从JSON或Drop造。先安装回执，再做后验与result IO；失败保留明确pending/native事实。重新取得：AcquirePrepared/current后，原回执单用、核原scope与原4对象内容/安全摘要，实际完整新租约重验两树后登记新epoch，再准节点变更。历史epoch不得复用，旧释放对象以后每次journal.check仍核。

正例在同一原10秒组件窗口，工作树完成后释放，固定Bun只导入bun:sqlite的claim-only回调此时raw0可取；原持锁时raw73，重新取全域后拒绝竞争。三原文件原件恢复及最后显式释放共36事件（原30节点+6租约事件），最终目录/两稳定文件/原件ID/all8/内容/完整children和退休工作对象核验，稳定释放范围证明仍独立。这里没有原包模块、业务/模型/网络请求或production Owner。

失败覆盖Prepared写失败（原锁不放）、native释放后/result或真实current部分写、new lease后/result或current部分写、Drop无回执、半树、外改字节/空目录、延长原deadline、claim/manifest/security/source/epoch等数据漂移；实际错误/后像归segment-traces和D journal-scene-records原字节，不能catchup/隐式重试。失效新lease在驱动中先安装，失败清理显式释放自有范围，不能冒作结果成功。

范围限制：本轮只同一原ActionWindow连续分段。后续不同Start/Status/Stop动作预算与source/phase许可不能拿此回执自动延长；仍需实际生产Source/route/阶段/Ready与PreStop/Stopped封存、允许副作用和工作文件对原包可写窗口、完整Stop/typed Closed/history/successor、broker原Owner/H-S-D。生产适配入口仍v2、公共Start无条件拒绝。原缺席Codex初始化/安全退根仍blocked，不能留目录/两锁冒Absent或释放claim只持NLK移根。旧v2/v3 pending不迁移/接管，旧ordinary MSI/CAP-trust/清理未代签。不W2/双SID、不发布。

产品MSVC custom-protocol debug bins与最终输入/测试Rust对准确源码字节核验；前端32份复用。非release预算、非冻结W2候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
"""+"\n".join(f"| {a['name']} | {a['bytes']} | {a['sha256']} |" for a in artifacts)+f"""

{len(scenes)}场景私有ZIP内容核验；{len(cleanup['removed'])}处本轮已完成自有原语/纯数据场景核全部文件和目录后清理，{cleanup['failedAndPendingScenesRetained']}失败或正式phase未闭合的pending原场景、{cleanup['crossVolumePendingRootsRetained']}处D日志根保留。清理选择器曾因通过测试后紧跟suite failures摘要而触发断言，未生成计划或执行删除；仅补原文failures分界，保留原stdout并重新核每处准确归属/内容/目录，原工具失败见cleanup-selector-first-error。成功同窗release也没有正式Closed，不删或冒后继。ZIP不是NTFS身份/安全克隆；旧拒绝删除/账户/Profile/Public与旧broker不重试。overallCleanupComplete=false。
"""
with (ROOT/"report.md").open("x",encoding="utf-8") as out:out.write(report)
with (DOCS/"docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md").open("a",encoding="utf-8",newline="\n") as out:
    out.write(f"\n\n## 2026-11-16 TEAM16/r6 同原动作窗口lease分段\n\n源码 {head}/tree{tree}；证据 .adg/evidence/0.2.24/windows-dev/{ROOT.name}/。同一canonical节点/lease链与current，Prepared刷盘→实际显式解锁原对象回执→Released→新lease scope/4对象/原deadline全验→新epoch事件；36事件完整原件恢复。首轮20过，扩展后21过1败/raw101，诊断03原断言仍败（空SQLite生成-journal，按未知拒绝而非native busy，current20 Released/原窗剩8.6秒）；仅新夹具在capture前初始化合成rollback DB，定向04原raw33断言通过，其余21过生产实源字节一致复用。原失败/真实部分写与release/acquire前缀/EXE摘要/完整表/清理归final-status/report。\n\n本轮仅同原窗口，不签不同ActionWindow/正式Source阶段、工作可写窗口/副作用、封存/Stop/Closed/history/后继或原包H-S-D；现生产入口仍v2，Start无条件拒绝，原缺席安全退根blocked。产品debug MSVC bins raw0/{runs['product01']['elapsedMs']}ms；不重复旧raw5/空GUI，不W2/双SID/发布。\n")
print(json.dumps(dict(head=head,tree=tree,productInputs=len(final),finalRust=len(rust),reusedPassingProductionRust=len(reuse),scenes=len(scenes),rawJournalFiles=len(journals),canonicalLeaseEvents=len(leases),artifacts=artifacts)))
