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
runs={};final=[];rust=[]
for name,expected in [("library01",101),("library02",0),("product01",0)]:
    result=load(ROOT/name/"result.json");assert result["exitCode"]==expected
    for row in load(ROOT/name/"source-manifest.json"):
        raw=(ROOT/name/"executed-source"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
        if name=="product01" or name=="library02" and row["path"].endswith(".rs"):
            assert raw==(CODE/row["path"]).read_bytes()
            assert raw.replace(b"\r\n",b"\n")==git("show",f"{head}:{row['path']}").replace(b"\r\n",b"\n")
            (final if name=="product01" else rust).append(row)
    if name=="library02":
        text=(ROOT/name/"stdout.log").read_text(encoding="utf-8-sig");m=re.search(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored",text)
        assert m and tuple(map(int,m.groups()))==(7,0,0)
    if name=="library01":
        text=(ROOT/name/"stderr.log").read_text(encoding="utf-8-sig");assert "E0624" in text and "bind_stop_action" in text
        assert not load(ROOT/"library01-artifact.json")["executed"]
    runs[name]=dict(rawExit=expected,passed=7 if name=="library02" else None,failed=0 if name=="library02" else None,ignored=0 if name=="library02" else None,elapsedMs=result["elapsedMs"],artifact=load(ROOT/f"{name}-artifact.json") if name.startswith("library") else None)
assert load(ROOT/"product01/source.json")["head"]==head
artifacts=[]
for name in ["opencodex-desktop","ocxd","ocx"]:
    p=CODE/f"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe";raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile="debug",features=["custom-protocol"]))
write("product01-artifacts.json",artifacts);write("final-source.json",dict(head=head,tree=tree,productInputs=final,finalTestRust=rust,workBytesEqual=True,gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyEditedOrStaged=True))
for row in load(ROOT.parent/"wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json"):
    raw=(CODE/"apps/desktop/ui/dist"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
scenes=load(ROOT/"library02-private-scenes.json")["scenes"];assert scenes==[]
matrix=load(ROOT.parent/"wl2-r6-lease-segments-20261116-r1/final-status.json")["matrix"]
matrix.extend([
 dict(id="SharedWindowsOriginalSource",result="refactor-verified/native-current-pin-regression-pass",fact="Five original CurrentPin and Source method bodies token-identical to e7c18e97c. Existing v2 consumer now imports same originals/source.rs for eventual v3 reuse; fixed official299/current/hash/image/noJob/ordinaryToken/native peer/Owner scope checks retained. Two actual current-pin tests pass; not new full Source/Owner acceptance."),
 dict(id="OriginalOwnerActionWindows",result="7-directed-tests-pass; production-execution-pending",fact="Owner retains exact ActionWindow passed by shared boundary before creating native launcher, and original Stop/Restart window before initiating stop. NativeAction nonserde/nonClone binds actual Owner session/scope/Token/origin UUID and exact original deadline. Start parent29 cap/Stop shorter parent/Restart stop10/late stop/repeated or partially started stop all refuse reset; original restart input receipt tests pass. Native production path compiled below unchanged public Start gate."),
 dict(id="V3NativeSourceContextFactory",result="compiled-not-executed",fact="Source journal_context rechecks actual original Owner/source/current/full Binding and creates nonserde native pins/peer+NativeAction context; v3 create_from_source/check_source_owner consumes that context and current live claim, binding original action UUID/deadline. No production v3 adapter switch or real positive factory invocation yet; no parsed Binding or hashes alone create this context."),
])
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract="TEAM16/IMP142/r6",baselineMain="fac8573e55441fe514919c63c7f1d65181ff3577",baselineDocs="5782307174f0d1b011b5415c6c1e8d60414e6b36",runs=runs,artifacts=artifacts,matrix=matrix,
 firstError=dict(run="library01",rawExit=101,rustError="E0624",stage="parent production caller cannot invoke child-private bind_stop_action",newExecutableProduced=False,fixedBy="pub(super) method only; Start ActionOrigin admitted before native creation, no fallible new origin after native create/before Owner storage"),
 limits=["7 tests are3 ActionWindow/private origin rules+2 moved current-pin native tests+2 original restart input/StopReceipt tests; no fabricated actual Owner positive","Full native Source/context factory create_from_source and NativeAction check against real production Owner compiled but not executed","ActionWindow receipts retain original deadline/subdeadline; no later distinct-action canonical bridge or v3 production adapter in this batch","Working writable interval, original/Ready/PreStop/Stopped archives, source-bound official side-effects, complete native Stop/Closed/history/successor/H-S-D still outstanding","OriginalAbsent Codex initialization/safe retirement remains blocked, ordinary MSI/CAP-trust/historic cleanup open"],
 productionAdapterUsesV3=False,allAccepted=False,startOpen=False,productionExecuted=False,officialOwnerCreated=False,W2=False,dualSID=False,published=False,
 cleanup=dict(newRetainedScenes=0,temporaryCurrentPinScenes="successful Fixture scope cleanup; no new retained failure scene in raw test output",newAccounts=0,newProfiles=0,newPublicRoots=0,newGUIProcesses=0,newOfficialServices=0,historicalRejectedDeletionRetried=False,dailyPathConfigPolicyChanged=False,overallCleanupComplete=False,previousRawFailureAndPendingScenesRetained=True))
write("final-status.json",status)
report=f"""# r6原Source复用与Owner真实动作窗口

TEAM16/IMP142/r6，源码{head}/tree{tree}。承接e7c18e97c/80366fdb8，不改四trait/官方/共享ActionWindow/Status算法/日常策略。

| 运行 | 原退出码 | 过/败/忽略 | ms |
| --- | --- | --- | --- |
"""+"\n".join(f"| {n} | {r['rawExit']} | {r['passed']}/{r['failed']}/{r['ignored']} | {r['elapsedMs']} |" for n,r in runs.items())+"""

首错library01 E0624：Owner::bind_stop_action位于子模块但私有，父生产入口不可调用；原源码/原stderr/退出101完整保留，无新EXE不能冒前轮制品。修为pub(super)仅父模块可见。复核将Start ActionOrigin准入置于create_suspended之前，避免新增可失败操作在已创建native后/Owner入库前丢失保管。library02七过0败，产品MSVC debug bins通过。

原v2 CurrentPin/Source移入Windows originals/source.rs，原v2继续消费同实现，不另造v3官方准入规则。五个原方法（CurrentPin bytes/capture/check、Source capture/check）Rust方法体token与e7c18基线逐项完全一致，证据source-extraction-verification，保留原固定2.99/archive/image/current文件原代次/8投影、普通Token、原broker图像/noJob、scope/peer/SID/Profile/D/H/I/来源复核。两项实际current原生pin回归通过，未知schema/字段/硬链/写/rename/外来selection/过期继续拒绝。原v2 pending/history不迁移。

Owner保留原共享入口传入的ActionWindow；创建新原native launcher前登记Start段原outer/end与stop子界。真实Stop或Restart在原Owner开始stop前只登记一次原传入窗口，部分stop已开始/已有origin拒绝换窗口重试。新增private NativeAction非serde/nonClone，只有实际Owner在source/check_current通过后能发，绑定Owner session、scope、Token、origin UUID及精确原end/outer/stop；不now+预算。纯/边界反例验证Start parent29、Stop parent3、Restart原stop10、late restart尚有outer却stop已过、重复/已部分stop与错end；原有Restart StopReceipt→start输入保持原outer的两测试一起过。Start腿的窗口也可来自同次Restart外层，未重开30秒。

新Source::journal_context重新检查实际Owner和原Source/current与完整Binding，返回只可由该方法构造的原生peer/四目录pin/Binding/NativeAction上下文；v3 create_from_source消费它与活claim，原动作UUID入deadlineScope，check_source_owner鲜读原actual Owner。该工厂已编译、尚未真实执行，不能以纯Origin测试或current pin代native Source生产正例，不把数据decode变权限。v3仍未切生产入口，下一步要将此原权限接正式phase/新动作段及完整树/封存/副作用，而非继续使用任意deadline构造器签运行。

完整继承状态表见final-status。工作区可写区间、源绑定Ready/PreStop/Stopped封存/官方允许副作用、v3原始Owner与完整Stop/Closed/history/后继、H-S-D生产仍未闭合。原缺席Codex初始化与安全退根blocked，公共require_durable_capture仍无条件拒绝；不W2/双SID/发布，未重复旧raw5/空GUI/首1314。

最终产品输入及最终testRust对工作源/Git字节（只允许记录的CRLF等价），前端32份复用。MSVC custom-protocol debug不是release预算/冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
"""+"\n".join(f"| {a['name']} | {a['bytes']} | {a['sha256']} |" for a in artifacts)+"""

本轮无新保留scene，current-pin合成夹具成功scope清理；编译失败未运行native测试或创建测试树。账户/Profile/Public/GUI/官方服务/日常配置无新增，旧失败/pending原scene继续保留，所有旧拒绝删除不重试，overallCleanup仍false。
"""
with (ROOT/"report.md").open("x",encoding="utf-8") as out:out.write(report)
with (DOCS/"docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md").open("a",encoding="utf-8",newline="\n") as out:
    out.write(f"\n\n## 2026-11-16 TEAM16/r6 Source复用与Owner原动作窗\n\n源码 {head}/tree{tree}；证据 .adg/evidence/0.2.24/windows-dev/{ROOT.name}/。原CurrentPin/Source五方法体token与e7c18e97c完全一致，v2改消费公共Windows originals/source。Owner保留原传入Start及Stop/Restart窗口，NativeAction实际Owner/scope/Token/原UUID/end绑定；Source新上下文与v3 source工厂仅编译未实跑，不代生产。首library01 E0624/raw101（子方法对父不可见，无新EXE），修pub(super)并将Origin准入放native创建之前，library02七过0败；产品MSVC debug bins raw0/{runs['product01']['elapsedMs']}ms。EXE/完整表/首错归final-status/report；无新保留scene，旧pending和清理缺口保持。\n\nv3生产phase/后续动作canonical桥、工作可写/封存/副作用/Stop/Closed/history/后继/原Owner-H-S-D仍待，原缺席退根blocked且Start无条件拒绝。不重复旧raw5/空GUI/1314，不W2/双SID/发布。\n")
print(json.dumps(dict(head=head,tree=tree,productInputs=len(final),finalRust=len(rust),tests=7,artifacts=artifacts)))
