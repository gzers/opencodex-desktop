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
expected=load(ROOT/"expected-runs.json");final_run=expected["finalRustRun"];runs={};scenes=[];inputs=[];rust=[];records=[];traces=[];entries=[]
for name,outcome in expected["runs"].items():
    result=load(ROOT/name/"result.json");assert result["exitCode"]==outcome[0]
    for row in load(ROOT/name/"source-manifest.json"):
        raw=(ROOT/name/"executed-source"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
        if name=="product01" or name==final_run and row["path"].endswith(".rs"):
            assert raw==(CODE/row["path"]).read_bytes()
            assert raw.replace(b"\r\n",b"\n")==git("show",f"{head}:{row['path']}").replace(b"\r\n",b"\n")
            (inputs if name=="product01" else rust).append(row)
    text=(ROOT/name/"stdout.log").read_text(encoding="utf-8-sig")
    if outcome[1] is not None:
        m=re.search(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored",text);assert m and tuple(map(int,m.groups()))==tuple(outcome[1:])
        traces.extend(dict(run=name,trace=line[line.index("r6"):]) for line in text.splitlines() if "r6" in line and len(line)<4000)
        for i,s in enumerate(load(ROOT/f"{name}-private-scenes.json")["scenes"]):
            assert sha(Path(s["archive"]).read_bytes())==s["archiveSha256"];scenes.append(s)
            for row in s["files"]:
                if s["source"].startswith("D:") or "/.ocxd024-observed-" in "/"+row["path"]:
                    p=Path(row.get("source",str(Path(s["source"])/row["path"])));raw=p.read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
                    target=ROOT/"raw-scene-records"/name/f"scene-{i:03}"/row["path"];target.parent.mkdir(parents=True,exist_ok=True)
                    with target.open("xb") as out:out.write(raw)
                    records.append(dict(source=str(p),evidence=target.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
                    if "/event-" in row["path"]:
                        e=json.loads(raw)
                        if e["documentKind"] in ["snapshot-phase-event","work-interval-event"]:entries.append(dict(run=name,evidence=target.relative_to(ROOT).as_posix(),event=e))
    runs[name]=dict(rawExit=result["exitCode"],profile=result['profile'],passed=outcome[1],failed=outcome[2],ignored=outcome[3],elapsedMs=result["elapsedMs"],artifact=load(ROOT/f"{name}-artifact.json") if name.startswith("library") else None)
assert load(ROOT/"product01/source.json")["head"]==head
write("native-series-traces.json",traces);write("canonical-series-events.json",entries);write("raw-scene-records.json",records)
write("final-source.json",dict(head=head,tree=tree,productInputs=inputs,finalTestRust=rust,gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyModifiedOrStaged=True))
artifacts=[]
for name in ["opencodex-desktop","ocxd","ocx"]:
    p=CODE/f"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe";raw=p.read_bytes();artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile="debug",features=["custom-protocol"]))
write("product01-artifacts.json",artifacts)
for row in load(ROOT.parent/"wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json"):
    raw=(CODE/"apps/desktop/ui/dist"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
matrix=load(ROOT.parent/"wl2-r6-store-retention-20261116-r1/final-status.json")["matrix"]
matrix.extend([
 dict(id="NativeSnapshotSeriesResume",result="native-component-pass",fact="Canonical WorkResume intent/result requires exact current sealed observation sequence and actual BOTH linked archives; old immutable archive objects and work metadata descriptions retained across each new writable interval. Initial working/original plan never replaced. Each independent observation has stable seal sequence and unique archive path; historical per-stage action scope remains that stage's actual recorded scope, no cross-action resigning."),
 dict(id="ReleaseAfterObservedChange",result="native-component-pass/failure-retained",fact="Initial series first run failed at second release because old release boundary compared against initial work bytes. Fix freezes full temporary current native view matching latest linked actual archive before+after native unlock; only after canonical release result drops temporary deny-write view. Original/initial baseline stays immutable, unknown current drift refuses; failed result retains view/pending."),
 dict(id="V3NativeArchiveSharedPolicy",result="content-data-component-pass; no mutation authority",fact="Distinct native archived stage frames reconstruct source generations/bytes/all8 from actual rechecked immutable chunks and source metadata, not archive-copy IDs. A full BOTH-tree compound native barrier checks originals/custody/all historical work objects+archives/current frames before AND after original pure policy, without repeatedly scanning rows or caching permission. Synthetic independent route/PID/audit inputs explicitly data-only. AlreadyOriginal/RestoreOriginal content branches, actual ChangedSinceReady and foreign Grok refusal prove data scope, no Ready/NormalExit/restore grant or baseline adoption."),
 dict(id="TypedStoppedContentSelector",result="compiled-not-executed",fact="Production selector requires original actual Source/Owner/native StopAction plus NormalExit/SupervisorAbsent/StopAccepted/record/listener postconditions and Stopped lifecycle, then selects exact canonical Ready/PreStop/Stopped link sequences. No real production factory invocation or adapter switch performed."),
])
cleanup=load(ROOT/"cleanup.json")
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract="TEAM16/IMP142/r6",baselineMain="fac8573e55441fe514919c63c7f1d65181ff3577",baselineDocs="5782307174f0d1b011b5415c6c1e8d60414e6b36",runs=runs,artifacts=artifacts,matrix=matrix,firstError=load(ROOT/"first-error.json"),debugTimeoutFacts=load(ROOT/'debug-timeout-facts.json'),releaseSameDeadlineCasesPassed=6,releaseAffectedRegressionPassed=41,reusedExactReleaseExecutableSha256=load(ROOT/'release-executable-reuse-proof.json')['sha256'],canceledOwnDuplicateCompiler='library05; raw101 retained, no tests/new executable, then library06 exact original filters direct run',
 controlledFailures=["unlinked/unknown old archive refuses resume","durable resume intent/native/result/partial-current windows preserve Resuming/native histories","actual between-observation content change refuses original shared policy"],
 limits=["Native series observations are Observation-purpose components; no actual native official Ready/PreStop/Stopped/NormalExit manufactured","Actual native policy inputs route/PID/audit are explicitly synthetic data; production selector compiles but not invoked","Production V3 capture remains registered but dispatcher not switched; config/auth/Grok source-declared changes, dynamic artifacts/directories and precise restore/Closed/history/successor/H-S-D remain incomplete","OriginalAbsent Codex initialization/safe retirement blocked; old ordinary MSI/CAP-trust/historical cleanup remain"],allAccepted=False,startOpen=False,productionAdapterUsesV3=False,productionExecuted=False,nativeReadyOrStoppedFactoryExecuted=False,officialOwnerCreated=False,W2=False,dualSID=False,published=False,
 cleanup=dict(archivedScenes=len(scenes),archivedFiles=sum(len(s["files"]) for s in scenes),archivedBytes=sum(f["bytes"] for s in scenes for f in s["files"]),completedOwnScenesRemoved=len(cleanup["removed"]),failureAndPendingScenesRetained=cleanup["failedAndPendingScenesRetained"],crossVolumePendingRootsRetained=cleanup["crossVolumePendingRootsRetained"],nativeMetadataCloned=False,historicalRejectedDeletionRetried=False,newAccounts=0,newProfiles=0,newPublicRoots=0,newGUIProcesses=0,newOfficialServices=0,dailyPathConfigPolicyChanged=False,overallCleanupComplete=False))
write("final-status.json",status)
report=f"""# r6多次原生观察与原共享内容判断

TEAM16/IMP142/r6，源码{head}/tree{tree}，承接eee0ad1a4/937fe6c94。四trait/原截止/8投影/ExcludedUnknown/原共享client_artifact_policy不变；官方与日常配置未改。

| 运行 | profile | 原退出码 | 过/败/忽略 | ms |
| --- | --- | --- | --- | --- |
"""+"\n".join(f"| {n} | {r['profile']} | {r['rawExit']} | {r['passed']}/{r['failed']}/{r['ignored']} | {r['elapsedMs']} |" for n,r in runs.items())+"""

首轮library01两过两败/raw101，四个Resume故障窗口及未知引用正确拒绝；完整多阶段测试在首次观察/resume后的再次release失败。旧release边界仍要求当前work字节等于最初work副本，遇到已登记变化即拒绝。保留初始work/original全部原断言，新增临时完整native release view：先按最近实际已链接archive精确比较ID/8投影/内容/children，再Prepared、原生unlock及后验/result；只在release结果canonical写完后撤下临时拒写句柄。失败view/pending保留，不把当前内容写回基线。原错误/EXE/日志/场景保留，未延长原10秒组件窗或重写原断言。

library02 debug四过；扩展library03 debug三过三败，首次Resume实际TimedOut，原10秒与断言保留，不重写通过。library04使用仓库原release参数，同原源码断言/10秒，无Cargo profile参数变化，六项全部通过；trace保留各阶段原剩余时间。该构建23分41秒；后续Cargo回归又触发重复优化编译，核全部Rust、rustc、编译flags和已通过EXE hash完全一致，仅驱动脚本增加过滤mode不同，确认compiler是本任务Cargo直接子进程后只停止该重复compiler。library05取消raw101/原stderr/source保留，无执行测试或新EXE。library06直接运行library04准确原EXE和library05原回归filters，执行前后hash相同、原Rust输入与编译flags核验，41过0败。这里没有放宽deadline/assertion/filter或更换测试实现，不是把取消当通过；具体证据归release-executable-reuse-proof/redundant-compiler-cancellation/run-command。release组件仍不代表actualOwner生产预算/冻结候选。共享policy直接调用原文件，source equality确认原算法未改；首个内联工具校验引号SyntaxError只影响证据helper，已保存脚本重核，原工具错误独立保留。

Resume必须当前Observed已由实际双根SnapshotWitness验证并链接该sealed observation sequence，先WorkResumeIntent/current，再ReOpenFile准确current工作对象的metadata observer，保留旧代次和不可变archive到history，再完成结果。非链接/旧序号/错误引用拒绝；同一观察重复链接仍拒绝，多次Observation每次独立序号与目录，最新链接不抹旧事件。current保留全部已链frame摘要和当前sealed序号；旧Ready等阶段按自身原action scope保存，不以新Stop scope改签旧快照。初始work_frame与原件plan永不替换。原子/普通写后每次完整观察仍核所有非产物与unknown children。

实际合成native序列取得三个独立完整frame和同卷archive，真实release/reacquire后进行下一段写入。旧archive/part/source metadata与old work generation持续保管、重核。内容调用用两根统一native前后barrier，逐次核全部original/custody/历史work对象/全部archive和当前完整frame；原纯policy调用仅在两个完整barrier之间重建已验证不可变字节，不再逐阶段重复扫描整集合，不缓存或跨调用复用许可。按原件来源ID/8投影/内容重建ArtifactFrame，source generation和archive-copy ID分开，调用原client_artifact_policy::assess，不复制解析/放宽规则。正例验证AlreadyOriginal和保留已捕获合成metadata的RestoreOriginal内容分支（后者没有实际恢复写）；反例在中间真实修改模型内容，原policy ChangedSinceReady拒绝；另实际Grok新增未知空目录在完整barrier拒绝，即使Codex内容本可通过也不给完整结论。原custody/frame保持。路由/PID/audit是明确合成输入，不是实际生产进程/配置资格。内容结论仍是DATA，不直接退休工作或给恢复/Ready/Closed许可。

新production stopped_content_observations selector仅在实际原Source/Owner/NativeAction、typed NormalExit/Absent/StopAccepted/原record/listener后验以及canonical Stopped下，取唯一Ready/PreStop/Stopped link序号；前后复核同claim/source，工厂只编译未真实调用。当前native测试全部Observation用途、生命周期Preparing，不冒充官方阶段。不造fake生产Owner/typed证明来填正例。

故障覆盖Resume意图后/原生句柄已交接后/结果event-current或真实部分current写，均保留准确Resuming与native历史前缀、poison拒绝追赶；未知历史archive空目录拒绝，不能靠只看最新活树忽略旧档案。受影响work/snapshot/lease/v3数据回归按表；不重复旧raw5/空GUI/首1314或无变化共享source/status算法。

未完保持：完整v3生产dispatcher/actualSource、正式phase全快照组合、config/auth/Grok源声明非产物变更、动态产物/新目录准确退休还原、全部normal exit/监督/端点/record/source及typed Closed/history/后继/实际原Owner H-S-D。公共Start无条件拒绝。原缺席Codex安全初始化/退根blocked单列，不能留下目录/两锁冒Absent。普通MSI/CAP-trust/历史清理不代签。不W2/双SID/模型业务请求/发布。

产品MSVCcustom-protocol debug bins成功，最终输入/testRust与准确工作/Git源码字节核验，前端32份复用；不签release预算/冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
"""+"\n".join(f"| {a['name']} | {a['bytes']} | {a['sha256']} |" for a in artifacts)+f"""

{len(scenes)}场景私有内容ZIP核验；canonical多观察/Resume事件和same-volume source archive块/metadata原字节另归canonical-series-events/raw-scene-records，trace保留实际观察序号和结论。{len(cleanup['removed'])}处完成纯数据/native历史场景核完整文件与目录后清理；{cleanup['failedAndPendingScenesRetained']}失败或正式Closed缺席pending原场景、{cleanup['crossVolumePendingRootsRetained']} D日志根保留。ZIP非NTFS身份/安全克隆，旧自动删除拒绝/账户/Profile/Public/旧broker均不重试，overallCleanup=false。
"""
with (ROOT/"report.md").open("x",encoding="utf-8") as out:out.write(report)
with (DOCS/"docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md").open("a",encoding="utf-8",newline="\n") as out:out.write(f"\n\n## 2026-11-16 TEAM16/r6 多观察快照与原policy\n\n源码 {head}/tree{tree}；证据 .adg/evidence/0.2.24/windows-dev/{ROOT.name}/。精确sealed序号/双根actual link后Resume intent/current→原生observer历史保管→verified，多次Observation独立目录，旧action scope保持、原件/初始work基线不换。三个实际独立native frame/archive接原client_artifact_policy内容DATA，合成routes/PID/audit明确；实际中间编辑ChangedSinceReady拒绝。首library01两过两败/raw101，旧release比较初始work导致已登记变化拒绝，补精确最近archive的临时完整native release view并保持原基线，原失败/前缀保留。受影响回归/EXE/完整表/清理见report/final-status，产品debug MSVCbins0/{runs['product01']['elapsedMs']}ms。\n\n生产Stopped内容selector只编译未实跑，actualSource/正式阶段、源声明非产物/动态nodes/恢复Closed/history/后继/实际Owner H-S-D仍待；原缺席退根blocked且Start无条件拒绝。不旧raw5/空GUI/W2/双SID/发布。\n")
print(json.dumps(dict(head=head,tree=tree,productInputs=len(inputs),finalRust=len(rust),runs=runs,scenes=len(scenes),rawFiles=len(records),artifacts=artifacts)))
