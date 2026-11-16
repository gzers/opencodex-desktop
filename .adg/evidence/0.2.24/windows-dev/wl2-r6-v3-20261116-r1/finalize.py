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
runs={};scenes=[];final=[];rust=[]
for name,expected in [("library01",(0,8,0)),("library02",(101,8,3)),("library03",(0,11,0)),("product01",(0,None,None))]:
    result=load(ROOT/name/"result.json");assert result["exitCode"]==expected[0]
    for row in load(ROOT/name/"source-manifest.json"):
        b=(ROOT/name/"executed-source"/row["path"]).read_bytes();assert len(b)==row["bytes"] and sha(b)==row["sha256"]
        if name=="product01" or name=="library03" and row["path"].endswith(".rs"):
            assert b==(CODE/row["path"]).read_bytes()
            assert b.replace(b"\r\n",b"\n")==git("show",f"{head}:{row['path']}").replace(b"\r\n",b"\n")
            (final if name=="product01" else rust).append(row)
    if expected[1] is not None:
        raw=(ROOT/name/"stdout.log").read_text(encoding="utf-8-sig");m=re.search(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored",raw)
        assert m and tuple(map(int,m.groups()))==(expected[1],expected[2],0)
        for s in load(ROOT/f"{name}-private-scenes.json")["scenes"]:
            assert sha(Path(s["archive"]).read_bytes())==s["archiveSha256"]
            scenes.append(s)
    runs[name]=dict(rawExit=expected[0],passed=expected[1],failed=expected[2],ignored=0 if expected[1] is not None else None,elapsedMs=result["elapsedMs"],artifact=load(ROOT/f"{name}-artifact.json") if expected[1] is not None else None)
assert load(ROOT/"product01/source.json")["head"]==head
assert load(ROOT/"library02/source-manifest.json")==load(ROOT/"library03/source-manifest.json")
artifacts=[]
for name in ["opencodex-desktop","ocxd","ocx"]:
    p=CODE/f"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe";b=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(b),sha256=sha(b),sourceHead=head,sourceTree=tree,profile="debug",features=["custom-protocol"]))
write("product01-artifacts.json",artifacts);write("final-source.json",dict(head=head,tree=tree,productInputs=final,finalTestRust=rust,workBytesEqual=True,gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyEditedOrStaged=True))
for row in load(ROOT.parent/"wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json"):
    b=(CODE/"apps/desktop/ui/dist"/row["path"]).read_bytes();assert len(b)==row["bytes"] and sha(b)==row["sha256"]
raw_journals=[]
for i,s in enumerate(load(ROOT/"library03-private-scenes.json")["scenes"]):
    if not s["source"].startswith("D:") and not s["source"].startswith("D/"):continue
    source=Path(s["source"])
    for row in s["files"]:
        b=(source/row["path"]).read_bytes();assert len(b)==row["bytes"] and sha(b)==row["sha256"]
        target=ROOT/"journal-scene-records"/f"scene-{i:02}"/row["path"];target.parent.mkdir(parents=True,exist_ok=True)
        with target.open("xb") as out:out.write(b)
        raw_journals.append(dict(source=str(source/row["path"]),evidence=target.relative_to(ROOT).as_posix(),bytes=len(b),sha256=sha(b)))
write("journal-scene-records.json",raw_journals)
matrix=load(ROOT.parent/"wl2-r6-anchored-20261116-r1/final-status.json")["matrix"]
for row in matrix:
    if row["id"]=="V3CanonicalNodeJournal":row.update(result="strict-data/native-IO-component-pass; native-action-binding-incomplete",fact="Full native-derived two-root plan/strategy/planDigest/security/Binding/current/event chain. All8 ordered file events, anchor snapshot/release and no moved-only completeness. Private immutable event sync/readback and same-ID exclusive current CAS, actual distinct C/D volumes; pending after event/CAS failure or real partial current remains poisoned, no catchup. Journal still separate from actual node mutation driver, phase/source/history/native eligibility; never grants Ready/Closed.")
matrix.append(dict(id="AnchoredStableRelease",result="component-pass",fact="Both exact coordinator bytes use finite rollback header rule, unknown sidecars remain rejected. After explicit NativeLease release, full native directory/file/retired image and exact two stable kernel ranges acquired/released with original IDs/all8/bytes; no SQLite open/recovery/schema in production."))
write("push-facts.json",dict(sourceHead=head,firstAttempt=dict(origin="tool transcript observation",rawExit=1,message="RPC failed; curl 35 schannel: failed to receive handshake, SSL/TLS connection failed; unexpected disconnect"),retry=dict(rawExit=0,remoteVerified=head,force=False,sslVerificationChanged=False)))
cleanup=load(ROOT/"cleanup.json")
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,baselineMain="fac8573e55441fe514919c63c7f1d65181ff3577",baselineDocs="5782307174f0d1b011b5415c6c1e8d60414e6b36",contract="TEAM16/IMP142/r6",runs=runs,artifacts=artifacts,matrix=matrix,
 firstError=dict(run="library02",rawExit=101,rawWin32=2,stage="fixture distinct-volume task target parent absent",fixedBy="recorded creation of exact own three D test parents; same source inputs reused, actual volume IDs still asserted"),
 limits=["v3 data completeness and journal IO are necessary component checks, not native original node mutation authority","Native node intent/result binding and every native/persistence combined interruption point still pending","Production adapter remains v2 with public require_durable_capture unconditional refusal; no old v2 conversion or text Closed adoption","Existing roots not yet combined with official source/route, Ready/PreStop/Stopped archives, lease segmentation/release, native Restore/Closed/successor and H/S/D","OriginalAbsent Codex safe retirement/initialization remains blocked by r6; ordinary MSI/CAP-trust/historical cleanup remain open","Controller ordinary token component runs cannot replace marenben actual desktop acceptance; no daily token/policy/config change","MSVC custom-protocol debug bins are not release performance or frozen W2 candidate"],
 cleanup=dict(archivedScenes=len(scenes),archivedFiles=sum(len(s["files"]) for s in scenes),archivedBytes=sum(f["bytes"] for s in scenes for f in s["files"]),completedOwnScenesRemoved=len(cleanup["removed"]),failureAndPendingScenesRetained=60,crossVolumePendingRootsRetained=5,newTaskParentDirectoriesRetained=3,nativeMetadataCloned=False,newAccounts=0,newProfiles=0,newPublicRoots=0,newGUIProcesses=0,newOfficialServices=0,fixedBunSyntheticChildren=30,historicalRejectedDeletionRetried=False,dailyPathConfigPolicyChanged=False,overallCleanupComplete=False),
 allAccepted=False,startOpen=False,productionExecuted=False,officialOwnerCreated=False,nativeBoundNodeJournal=False,W2=False,dualSID=False,published=False)
write("final-status.json",status)
report=f"""# r6完整节点计划与v3持久进度组件

TEAM16/IMP142/r6，源码{head}/tree{tree}；main fac8573e5/docs578230717已消费，前批7ece8c4b5/bfbbebf4c证据复用。

| 运行 | 原退出码 | 过/败/忽略 | ms |
| --- | --- | --- | --- |
"""+"\n".join(f"| {n} | {r['rawExit']} | {r['passed']}/{r['failed']}/{r['ignored']} | {r['elapsedMs']} |" for n,r in runs.items())+"""

library01新完整两树计划/纯进度和受影响原语8过；library02三个真实日志用例在创建D任务根前raw2失败，其余8过，失败原日志/源/EXE/22现场保留。记录新建本任务D的test/windows024-native/target三层父目录后，同源码library03为11过。C/D实际native卷号不同由断言和原manifest绑定，未凭盘符推定；未改策略/ACL。原始whoami末尾空格保留，前批内联git字节核验的引号错误已通过保存脚本对268份实际Git blob重新核验，具体修正归prior-closeout-correction/prior-git-bytes。

v3固定storageStrategy、完整Codex/Grok目录/准确两锚点/每普通文件计划，original node ID/parent/security all8/完整children/同卷custody目标与8投影策略入planDigest。重复/缺节点、额外Grok锚点、错父/跨卷custody、异常安全范围、逃逸/未知/重复字段拒绝。每普通文件严格8事件，锚点仅snapshot/release；pending不跨节点/skip/reverse/replay，SID/Profile/D/H/I/source/owner Binding与session/plan/security/manifest/claim epoch/key/deadline scope绑定。原件仅Moved不算Restored，数据完整也不签原生资格。

私有v3 manifest与事件CREATE_NEW/sync/readback，原exclusive current对象同ID/8投影读回后CAS。第二writer实际raw32；已落事件后CAS前故障与current真实写入单字节左花括号并sync后故障均保留原phase与后像、poison拒绝后继，无自动catchup。未知空目录、延长deadline也poison。5处D任务日志原字节另复制journal-scene-records（含单字节current），同原私有ZIP及逐文件摘要交叉核验。没有从文本构造旧broker/lease/session。旧v2 pending/history不迁移、不续写。

稳定协调文件继续有限rollback字节规则，原两稳定文件由原生显式release后以原ID/8投影/全树/退休对象及准确512字节范围实际取得/释放核验；生产无SQLite打开/恢复/PRAGMA/schema写。旧raw5根移动/空GUI/未受影响route/deadline组件不重复。

仍是组件：实际逐节点动作尚未与v3 intent/result原生驱动绑定，整个阶段/current、source/route/history/successor及Ready/PreStop/Stopped封存、官方允许副作用、NativeLease分段/ownerreceipt、完整Stop/Closed与真实H/S/D未组合。公共Start仍无条件拒绝，旧v2适配入口保留。原缺席Codex安全退根依r6仍blocked，不用existing-root组件代首次环境，不释放claim仅靠NLK移根。控制器原普通Token不代marenben实际桌面验收；不W2/双SID/发布/官方或日常配置改动。

完整继承状态表/未完归final-status.json。产品MSVC custom-protocol debug bins成功，最终全部产品输入和最终测试Rust与Git源码字节行尾等价，前端32份复用；不是release预算或冻结候选。首次push TLS握手失败已原命令重试并核远端准确SHA，无force或SSL绕过。

| 产品EXE | 字节 | SHA256 |
| --- | --- | --- |
"""+"\n".join(f"| {a['name']} | {a['bytes']} | {a['sha256']} |" for a in artifacts)+f"""

本轮{len(scenes)}场景内容私有ZIP核验，{len(cleanup['removed'])}处本轮已完成自有场景核全部文件和目录集合后清理；60处失败/pending原场景与5处D日志根及3层新测试父目录保留。ZIP不是NTFS身份/安全克隆。旧broker/账户/Profile/Public拒读与所有自动删除拒绝不重试，普通MSI/CAP-trust及overallCleanup仍未完成。
"""
with (ROOT/"report.md").open("x",encoding="utf-8") as out:out.write(report)
with (DOCS/"docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md").open("a",encoding="utf-8",newline="\n") as out:
    out.write(f"\n\n## 2026-11-16 TEAM16/r6 v3计划与持久日志组件\n\n源码 {head}/tree{tree}；原证据 .adg/evidence/0.2.24/windows-dev/{ROOT.name}/。完整两根节点计划/planDigest/绑定及严格8事件进度，真实不可覆盖phase与同ID current CAS/部分写poison、两稳定对象释放读回。library01 8过；library02 8过3败/raw101（D任务父目录缺席raw2，原现场保留）；按记录补自身父目录后同源library03 11过0败，产品debug MSVC bins raw0/{runs['product01']['elapsedMs']}ms。EXE SHA、原单字节current、完整继承表、首错、10处本轮成功清理/60失败pending保留归final-status/report。\n\n组件尚未绑定真实节点变更与阶段/source/history/lease分段/Stop/Closed/后继/原包H-S-D，生产入口仍v2且Start无条件拒绝；原缺席退根blocked单列，不转换旧记录、不W2/双SID/发布。继续范围内实现native intent/结果绑定及组合中断，非整组已验收。\n")
print(json.dumps(dict(head=head,productInputs=len(final),finalRust=len(rust),artifacts=artifacts,scenes=len(scenes),rawJournalFiles=len(raw_journals))))
