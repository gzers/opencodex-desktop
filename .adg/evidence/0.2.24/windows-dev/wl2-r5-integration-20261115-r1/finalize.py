"""Seal final r5 build inputs and truthful native/pure test boundaries once."""
from pathlib import Path
from datetime import datetime
import hashlib
import json
import re
import subprocess

ROOT=Path(__file__).resolve().parent
CODE=Path(r"C:/Users/15119/.codex/worktrees/024-windows-native/code")
DOCS=ROOT.parents[4]
load=lambda p:json.loads(p.read_text(encoding="utf-8-sig"))
sha=lambda raw:hashlib.sha256(raw).hexdigest()
git=lambda *args:subprocess.check_output(["git","-C",str(CODE),*args])
def write(name,value):
    with (ROOT/name).open("x",encoding="utf-8") as out:json.dump(value,out,ensure_ascii=False,indent=2)

head=git("rev-parse","HEAD").decode().strip()
tree=git("rev-parse","HEAD^{tree}").decode().strip()
runs={};final=[];rust=[]
for name in ["product01","library01","library02","product02"]:
    result=load(ROOT/name/"result.json")
    assert result["exitCode"]==0 and result["accepted"] is False
    for row in load(ROOT/name/"source-manifest.json"):
        raw=(ROOT/name/"executed-source"/row["path"]).read_bytes()
        assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
        if name=="product02" or name=="library02" and row["path"].endswith(".rs"):
            assert raw==(CODE/row["path"]).read_bytes(),row["path"]
            assert raw.replace(b"\r\n",b"\n")==git("show",f"{head}:{row['path']}").replace(b"\r\n",b"\n"),row["path"]
            (final if name=="product02" else rust).append(row)
    passed=None
    if name.startswith("library"):
        text=(ROOT/name/"stdout.log").read_text(encoding="utf-8-sig")
        match=re.search(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored",text)
        assert match and tuple(map(int,match.groups()))==(54,0,0)
        passed=54
    runs[name]=dict(rawExit=0,elapsedMs=result["elapsedMs"],passed=passed,accepted=False)
assert load(ROOT/"product02/source.json")["head"]==head
write("final-source.json",dict(head=head,tree=tree,product02Inputs=final,library02RustEqualFinal=rust,
    gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyEditedOrStaged=True))
artifacts=[]
for name in ["opencodex-desktop","ocxd","ocx"]:
    p=CODE/f"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe"
    raw=p.read_bytes();artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile="debug",features=["custom-protocol"]))
write("product02-artifacts.json",artifacts)
frontend=load(ROOT.parent/"wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json")
for row in frontend:
    raw=(CODE/"apps/desktop/ui/dist"/row["path"]).read_bytes()
    assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
write("frontend-reuse.json",dict(files=len(frontend),bytes=sum(r["bytes"] for r in frontend),rebuilt=False,verified=True))
prior=load(ROOT.parent/"wl2-real-gui-cleanup-receive-20261115-r1/final-status.json")
matrix=prior["matrix"]
matrix.append(dict(id="Route",result="native-pin/component-pass",fact="Independent shared parser11 plus native H/config ID/8projections/noADS/singlelink/2MiB/exact bytes/None-vs-empty/wrongH/expired checks; active Store source boundaries use the pin, each successor captures fresh. No generated-catalog inference or content-policy permission."))
matrix.append(dict(id="Sample",result="shared/native-unknown-path-pass",fact="Shared6 deadline cases and true OfficialStatusSource fetch/light/native missing-Owner path trace share one original5s each; private one-shot receipt binds Slot/session/scope/stopped/deadline before original Closed recheck. Actual retained Closed Owner positive/change/concurrent proof factories not yet executed."))
for row in matrix:
    if row["id"]=="P":row.update(result="Start-refused/r5-integrated",fact="r5 shared route and status interfaces integrated; Windows full recovery_state_at preserved byte-equivalent during merge. Actual side effects/NativeLease/original-package production closure incomplete; no official launch in this round.")
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,
    contract="TEAM15/IMP141/windows-shared-routes-status-r5",baselineMain="358e7fc4717787616f049f3bdc1cfe02f6d353f9",
    baselineDocs="676cec18a11af5da416c185e626f9ec63821b125",previousWindows="0549a660ee3f77ea243e798ec55b01f52545e492",
    commits=[git("rev-parse",x).decode().strip() for x in ["76c4f7185","a6ea0c1ce",head]],
    runs=runs,artifacts=artifacts,matrix=matrix,firstError=None,
    limits=["Route and sample54 passes are pure/native negative/component evidence, no actual Closed receipt factory or official Owner positive",
        "library01 stdout incorrectly called successful rejection fixtures retained; library02 corrects wording, Fixture scope cleaned them, no retained scene was printed",
        "Same-deadline binding receipt cannot create native Closed; missing/expired/mismatch/lock/source failures remain Unknown/Required",
        "NativeLease/complete official side effects remain; nine pinned official coordination source files verified against unchanged bundle and tgz for implementation input only",
        "No trueGUI empty host rerun, no CAP/trust repeat or ordinary MSI acceptance",
        "Debug products do not establish release budgets/frozen-candidate acceptance"],
    cleanup=dict(newAccounts=0,newProfiles=0,newPublicTaskRoots=0,officialProcesses=0,newGUIProcesses=0,
        nativeMetadataCloned=False,retainedScenes=0,scopedFixtureCleanup=True,deletionAttemptedByController=False,
        historicalRejectedDeletionRetried=False,overallCleanupComplete=False,priorCleanupEvidence="../wl2-real-gui-cleanup-receive-20261115-r1/",
        public20FilesRetained=True,oldFailedBrokerExitUnconfirmed=True,dailyPathConfigPolicyChanged=False),
    allAccepted=False,productionExecuted=False,startOpen=False,officialOwnerCreated=False,W2=False,dualSID=False,published=False)
write("final-status.json",status)
rows="\n".join(f"| {a['name']} | {a['bytes']} | {a['sha256']} |" for a in artifacts)
report=f"""# r5共享接口合入、原生路由与状态同代采样

TEAM15/IMP141/r5，main358e7fc47和docs676cec18a已精确fetch合入。代码merge76c4f7185后分批a6ea0c1ce、{head}，tree{tree}。唯一冲突windows_official_lifecycle024.rs保留完整Windows原文件，merge-resolution给原blob/工作摘要和CRLF等价，完整recovery_state_at保留；其它共享变更正常合入，未全仓ours/theirs。docs承接40ccb3557，正式TEAM/IMP来自集成线，不由Windows改权威进度。

RoutePin在实际H/config.json保留原文件/父pin/ID/8投影及精确内容，逐项/合并scope仍r4；2MiB、single-link、无ADS/reparse，解析消费共享declaration。None/Err拒绝，Some(empty)区分；活跃Store每次source边界重验原配置，Closed释放，后继新捕获。不从生成catalog倒推，不将routes向量变恢复资格。原工作树不变检查/副作用未知拒绝仍保持。

OfficialStatusSource同一采样fetch/light→native status_sample与recovery_for_sample共享相等原deadline。新增私有不可serde/Clone的一次receipt绑定原Slot Arc/session/scope/stopped/deadline，在同H原Owner互斥下比较，并执行同一完整Closed原生读回；缺失/过期/错上下文/锁忙不清恢复。registry准入改try_lock防无界等待。原recovery_state_at完整函数保持；回执不能制造Closed。真实WindowsSource缺Owner路径trace有fetch/recovery截止5000019us、light/recovery5000177us，均各自同一原5秒，注入调度时间未来1小时不作native起点。不声称已测试实际Closed正例或Owner并发更替成功。

| 执行 | raw | 结果 | ms |
| --- | --- | --- | --- |
| product01 | 0 | 合并基线custom-protocol MSVC debug bins | {runs['product01']['elapsedMs']} |
| library01 | 0 | route11/status33/source4/native6，共54项 | {runs['library01']['elapsedMs']} |
| library02 | 0 | 最终相同54项，时钟断言更精确/日志边界纠正 | {runs['library02']['elapsedMs']} |
| product02 | 0 | 最终冻结源custom-protocol MSVC debug bins | {runs['product02']['elapsedMs']} |

无非预期编译/测试错误，首错null。library01成功反例stdout错误称retained，实际Fixture正常scope清理，未打印保留路径，library02纠正原文且全部原日志保留，未虚造私有ZIP。最终产品输入{len(final)}份、最终测试Rust{len(rust)}份对最终工作字节/Git blob行尾等价；初轮脚本未包含共享6输入，冻结构建期间另补逐字shared输入清单，后继脚本正式纳入。前端32文件1162800字节保持。实际执行库EXE在覆盖前固定摘要。制品非release预算/冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
{rows}

完整B1/B2/B3/O1/O2/H/S-D/P/MSI/Route/Sample表见final-status。已过trueGUI空broker不重跑；Full-v1/partial不转，scope audit excluded-unknown/fullSecurityProven=false，原30/5/10/Restart30Stop子10不变。NativeLease/官方副作用和真正原包H-S-D闭环未完，Start仍拒绝。固定官方bundle/内tgz及9文件逐字核验，无官方运行或修改；本地SQLiteWindowsVFS片段只是后继实现参照，不当Bun互操作证书。

无新账户/Profile/Public根/GUI/官方进程，成功合成反例自行scope清理；历史失败现场/旧broker40588/Public20文件8拒读/普通MSI及所有旧清理拒绝保留、不重试，overallCleanupComplete=false。不日常PATH配置策略修改，无业务同步/模型请求/W2/双SID/发布。
"""
with (ROOT/"report.md").open("x",encoding="utf-8") as out:out.write(report)
with (DOCS/"docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md").open("a",encoding="utf-8",newline="\n") as out:
    out.write(f"\n\n## 2026-11-16 r5共享路由与原5秒同代采样\n\n已fetch合入main358e7fc47/docs676cec18a；代码merge76c4f7185唯一lifecycle冲突按合同保留完整Windows原文件/recovery_state_at。分批a6ea0c1ce/{head}，tree{tree}，实际RoutePin固定H/config原生ID/8投影/内容并消费共享declaration，活跃source边界重验、Closed放pin/后继新声明，None/Err不转空，不借此允许副作用。状态同deadline外再用一次性私有内存回执固定原Slot/session/scope/stopped，recovery在原Owner锁内比较并全量Closed读回，未知/锁忙/过期不清恢复；原完整receiver保留，不重开5秒。\n\n新原证据wl2-r5-integration-20261115-r1含合并原冲突及摘要、4组原源/log/result、库EXE与产品SHA、真实缺Owner WindowsSource trace、官方9源来自固定bundle/tgz和SQLite参考。library01/02均54过0忽略/raw0（route11/status33/source4/native6），首错null；初轮成功反例stdout误称retained后纠正，实际Fixture清理无保留路径，原日志留。最终clean custom-protocol MSVC debug bins0/{runs['product02']['elapsedMs']}ms，最终{len(final)}产品输入及{len(rust)}Rust对最终，前端32文件不变。完整状态/制品见final-status/product02-artifacts，不当release预算或真Closed正例。\n\n完整官方副作用/NativeLease及原包H-S-D生产仍未完，Start拒绝；已过trueGUI空broker/MSI-CLI/原件无影响项复用不重跑。无新账户/Profile/Public/GUI/官方，历史失败broker40588/现场/Public20文件8拒读和普通MSI/CAP-trust/旧拒绝清理未关，overallCleanupComplete=false，不改日常/官方，不业务模型请求/W2/双SID/发布。\n")
files=[]
for p in sorted(ROOT.rglob("*")):
    if p.is_file() and p.name!="payload-manifest.json":
        raw=p.read_bytes();files.append(dict(path=p.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
write("payload-manifest.json",dict(files=files,productionAcceptance=False))
print(json.dumps(dict(head=head,tree=tree,productInputs=len(final),testRustInputs=len(rust),runs=runs,artifacts=artifacts,payloads=len(files))))
