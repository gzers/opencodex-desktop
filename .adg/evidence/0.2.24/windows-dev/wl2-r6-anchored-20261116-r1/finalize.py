from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path(r"C:/Users/15119/.codex/worktrees/024-windows-native/code")
DOCS=ROOT.parents[4]
load=lambda p:json.loads(p.read_text(encoding="utf-8-sig"))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *a:subprocess.check_output(["git","-C",str(CODE),*a])
def write(n,v):
    with (ROOT/n).open("x",encoding="utf-8") as o:json.dump(v,o,ensure_ascii=False,indent=2)
head=git("rev-parse","HEAD").decode().strip();tree=git("rev-parse","HEAD^{tree}").decode().strip()
runs={};scenes=[];final=[]
for name,expected in [("library01",(101,2,2)),("library02",(0,5,0))]:
    result=load(ROOT/name/"result.json");assert result["exitCode"]==expected[0]
    raw=(ROOT/name/"stdout.log").read_text(encoding="utf-8-sig")
    m=re.search(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored",raw)
    assert m and tuple(map(int,m.groups()))==(expected[1],expected[2],0)
    for row in load(ROOT/name/"source-manifest.json"):
        b=(ROOT/name/"executed-source"/row["path"]).read_bytes();assert len(b)==row["bytes"] and sha(b)==row["sha256"]
        if name=="library02":
            assert b==(CODE/row["path"]).read_bytes()
            assert b.replace(b"\r\n",b"\n")==git("show",f"{head}:{row['path']}").replace(b"\r\n",b"\n")
            final.append(row)
    for s in load(ROOT/f"{name}-private-scenes.json")["scenes"]:
        assert sha(Path(s["archive"]).read_bytes())==s["archiveSha256"]
        scenes.append(s)
    runs[name]=dict(rawExit=expected[0],passed=expected[1],failed=expected[2],ignored=0,elapsedMs=result["elapsedMs"],artifact=load(ROOT/f"{name}-artifact.json"))
prior=load(ROOT.parent/"wl2-r5-claim-relocation-20261116-r1/final-status.json")
matrix=prior["matrix"]
for r in matrix:
    if r["id"]=="ClaimRootHandoff":
        r.update(result="r6-plan-consumed/root-rename-not-retried",fact="TEAM16/IMP142 chooses anchored existing directories and two exact stable Codex objects; ordinary files move individually with uninterrupted claim/NLK. Historical raw5 retained, not retried; no claim gap or old assertion changed.")
matrix.extend([
 dict(id="AnchoredExistingTree",result="component-pass",fact="Codex root/nested/empty directories and two stable objects retain IDs/all8. Ordinary originals roundtrip with separate work/retired generations and full predicted children. Grok stable-name file remains ordinary; 1280KiB copy passes."),
 dict(id="AnchoredClaim",result="component-pass",fact="Borrowed scope ties exact actual claim path/ID/range and original deadline to live NativeLease. Fixed Bun claim-only BEGIN EXCLUSIVE busy initially and after8 native writes (raw73), explicitly released -> raw0. No production Owner/source authority."),
 dict(id="AnchoredInterruptions",result="native-prefix-component-pass",fact="Nine 0..8 two-file native prefixes retain exact locations. Foreign empty dir/sidecar/wrong claim/absence refuse; occupied original file raw32. Not persistent interruption recovery."),
 dict(id="V3CanonicalNodeJournal",result="incomplete",fact="Next batch: complete strict storagePlan/planDigest and immutable node intent/result/current CAS, phase/native integration. NO persistent intent before mutation in this batch; step only called by cfg(test) callers."),
 dict(id="OriginalAbsentCodexRetirement",result="blocked-plan/not-attempted",fact="r6 leaves safe no-gap first-use root retirement unresolved. Do not leave directories/stable files as original absence or release claim and rely on NLK."),
])
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,baselineMain="fac8573e55441fe514919c63c7f1d65181ff3577",baselineDocs="5782307174f0d1b011b5415c6c1e8d60414e6b36",contract="TEAM16/IMP142/r6",runs=runs,matrix=matrix,
 firstError=dict(run="library01",rawExit=101,stage="WorkCreate supplied 2MiB bound to 1MiB journal PrivateRecord",error="record size bound",fixedBy="private CREATE_NEW seed; exact new ID verified before bounded writer; sync and new ID rechecked after seal"),
 finalSource=dict(inputs=len(final),allFinalInputBytesMatchWorktree=True,gitLineEndingEquivalent=True),
 componentContext="controller original ordinary non-elevated TokenUser, not marenben acceptance; no impersonation",productBuild="not rebuilt in this primitive batch; latest product remains prior3e190 evidence; final r6 product pending",
 cleanup=dict(archivedScenes=len(scenes),archivedFiles=sum(len(s["files"]) for s in scenes),archivedBytes=sum(f["bytes"] for s in scenes for f in s["files"]),archivesNativeMetadataClone=False,initialOriginalsRetained=True,successCleanupRecord="cleanup.json",newAccounts=0,newProfiles=0,newPublicRoots=0,newGUIProcesses=0,newOfficialServices=0,fixedBunSyntheticChildren=10,historicalRejectedDeletionRetried=False,dailyPathConfigPolicyChanged=False,overallCleanupComplete=False),
 allAccepted=False,startOpen=False,productionExecuted=False,officialOwnerCreated=False,W2=False,dualSID=False,published=False)
write("final-source.json",dict(head=head,tree=tree,inputs=final));write("final-status.json",status)
report=f"""# r6原目录留位与逐节点原生组件

TEAM16/IMP142/r6；main fac8573e5/docs578230717已合。源码{head}，tree{tree}。

| 原生运行 | 原退出码 | 通过/失败/忽略 | ms |
| --- | --- | --- | --- |
"""+"\n".join(f"| {n} | {r['rawExit']} | {r['passed']}/{r['failed']}/{r['ignored']} | {r['elapsedMs']} |" for n,r in runs.items())+"""

首错library01工作副本误用PrivateRecord的1MiB上限参数，2过2败/raw101；原日志/EXE/10现场均保留。修复为私有CREATE_NEW空文件，核创建ID再开准确writer，写原2MiB边界内容并sync，封存只读DELETE句柄后复核同新ID/字节/8投影。无截断未知路径，未缩小原文件边界。library02五过0败，新增1280KiB Grok同名文件验证不享协调豁免。

原Codex根/嵌套/空目录用非DELETE元数据句柄留位，准确两稳定协调文件只读shareRW、不移动；原普通文件以准确DELETE原handle同卷逐个移入私有custody，工作copy和退休文件分代。每一步全量核原件/工作/退休原ID、字节、8投影与路径同代，目录children按准确前缀预测，拒绝未知空目录或extra custody。0..8九个双文件中断前缀保留。错误claim/缺席根/未知sidecar拒绝，已占用普通文件在capture原码32拒绝，无ACL或分享方式强行回退。

借用型AnchoredClaim只能借活NativeLease，准确原claim path/ID/实际kernel范围与原deadline，不能释放后逃逸。fixed Bun来源/hash沿前证据，只导入bun:sqlite：初始及8次native写后claim-only BEGIN EXCLUSIVE均raw73/SQLITE_BUSY(errno5)，同原deadline下显式lease release后raw0。该新增组合与旧独立Bun互操作不同；无旧raw5/空GUI/共享route与deadline重复。组件普通Token是控制器SID，不能代marenben生产验收。

本批step只有本模块cfg(test)调用；尚无持久intent/current，因此不对生产暴露变更入口、不创建Ready/Closed。后续需v3完整计划/canonical逐节点进度、native journal中断、source/route/history/phase/lease释放分段和原包闭环。原v2不转换、不删除。原缺席Codex安全退根依r6仍blocked单列。公共Start仍拒绝，官方初始化/ownerreceipt/真实Stop/副作用/GUI原Owner/H-S-D、真Closed采样、普通MSI/CAP-trust/历史清理均未代签。不W2/双SID、不发布。

完整继承状态表见final-status.json matrix；EXE见libraryNN-artifact.json，最终输入原字节与Git blob行尾等价，未把前轮产品EXE冒作r6产品。现场ZIP只是内容档案，不是NTFS身份安全克隆；准确本轮已完成场景清理另见cleanup.json，失败/pending原场景保留。
"""
with (ROOT/"report.md").open("x",encoding="utf-8") as out:out.write(report)
with (DOCS/"docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md").open("a",encoding="utf-8",newline="\n") as out:
    out.write(f"\n\n## 2026-11-16 TEAM16/r6 原目录留位原语批次\n\n源码 {head}/tree{tree}；原证据 .adg/evidence/0.2.24/windows-dev/{ROOT.name}/。首轮raw101/2过2败（工作copy误用record上限），修后5过0败；原目录/两稳定文件ID与8投影留位，双原文件准确handle往返及0..8中断前缀、Grok同名普通文件1280KiB；固定Bun在原lease内9次claim-only均raw73，显式释放后raw0。\n\n仅native组件，无canonical intent/current接入，v3和生产组合继续；原缺席退根blocked，Start保持拒绝。完整表/首错/EXE摘要/清理归本轮final-status/report/cleanup，不改官方/日常配置，不重跑旧raw5/空GUI，不W2/双SID/发布。\n")
print(json.dumps(dict(head=head,inputs=len(final),scenes=len(scenes))))
