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
runs={};final=[];rust=[]
for name,rc in [("library01",101),("library02",0),("product01",0)]:
    result=load(ROOT/name/"result.json");assert result["exitCode"]==rc and not result["accepted"]
    for row in load(ROOT/name/"source-manifest.json"):
        raw=(ROOT/name/"executed-source"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
        if name=="product01" or name=="library02" and row["path"].endswith(".rs"):
            assert raw==(CODE/row["path"]).read_bytes(),row["path"]
            assert raw.replace(b"\r\n",b"\n")==git("show",f"{head}:{row['path']}").replace(b"\r\n",b"\n"),row["path"]
            (final if name=="product01" else rust).append(row)
    if name=="library02":assert 'test result: ok. 24 passed; 0 failed; 1 ignored' in (ROOT/name/"stdout.log").read_text(encoding="utf-8-sig")
    runs[name]=dict(rawExit=rc,elapsedMs=result["elapsedMs"],passed=24 if name=="library02" else None,ignored=1 if name=="library02" else None,testsExecuted=0 if rc else None,accepted=False)
assert load(ROOT/"product01/source.json")["head"]==head
write("final-source.json",dict(head=head,tree=tree,productInputs=final,library02RustEqualFinal=rust,gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyEditedOrStaged=True))
artifacts=[]
for name in ["opencodex-desktop","ocxd","ocx"]:
    p=CODE/f"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe";raw=p.read_bytes();artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile="debug",features=["custom-protocol"]))
write("product01-artifacts.json",artifacts)
for row in load(ROOT.parent/"wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json"):
    raw=(CODE/"apps/desktop/ui/dist"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
scenes=[]
for s in load(ROOT/"library02-private-scenes.json")["scenes"]:
    assert sha(Path(s["archive"]).read_bytes())==s["archiveSha256"]
    scenes.append(dict(source=s["source"],archive=s["archive"],archiveSha256=s["archiveSha256"],files=len(s["files"]),bytes=sum(f["bytes"] for f in s["files"]),removed=False))
prior=load(ROOT.parent/"wl2-r5-native-coordination-20261116-r1/final-status.json");matrix=prior["matrix"]
matrix.append(dict(id="ArtifactContent",result="native-snapshot/data-policy-pass",fact="Original same-object custody, actual sealed Ready+PreStop working images and fresh current native frame feed unchanged shared content policy; all8 security key separate from live generation/archive-copy identity. Undeclared nodes/config drift/allowed-name directory/archive contamination/PreStop changes refuse. Restore and unchanged-working veto remain; no native permission or canonical Changed-Ready/Closed production factory proved."))
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract=prior["contract"],baselineMain=prior["baselineMain"],baselineDocs=prior["baselineDocs"],previousCode=prior["sourceHead"],runs=runs,artifacts=artifacts,matrix=matrix,
    firstError=dict(run="library01",rawExit=101,testsExecuted=0,codes=["E0616","E0624"],firstFile="tree/artifacts.rs",firstLine=84,cause="sibling WorkingImage fields/private RootTree data-method visibility",newExecutableProduced=False),
    cleanup=dict(scenes=scenes,archivedScenes=len(scenes),archivedFiles=sum(s["files"] for s in scenes),archivedBytes=sum(s["bytes"] for s in scenes),originalsRetained=True,nativeMetadataCloned=False,newAccounts=0,newProfiles=0,newPublicTaskRoots=0,newGUIProcesses=0,newOfficialProcesses=0,taskSceneDeletionAttempted=False,historicalRejectedDeletionRetried=False,overallCleanupComplete=False,incidentalOwnPlaceholderRemoved=True,incidentalEditRecord="tool-edit-record.json",dailyPathConfigPolicyChanged=False),
    limits=["Four new native synthetic snapshot tests plus20 pure content-policy tests passed; one prior data-only evidence replay ignored, not a native acceptance case",
        "Canonical TreeCaptureV2::assess_artifacts compiles, but true source/phase/exit/NativeLease factory is not production-executed; synthetic archive context strings/pids/time are content-only input",
        "Stopped frame is a fresh native data observation, not yet durable canonical stopped snapshot or restore authority",
        "Data-level eligibility never installs a changed working baseline; exact full working-veto and public Start gate remain",
        "Official initialization/claim relocation/NativeLease source+root handoff, final typed restore/Closed and original-package H-S-D combinations remain unfinished",
        "Full GUI empty-host/native-lock/Bun/route/sample passes reused; no repeated old environment failures or old rejected cleanup",
        "ZIP preserves bytes, not NTFS identity/security; originals remain. Own accidental temporary control file was removed, not a historic rejected target",
        "Debug bins do not prove release budgets or frozen-candidate acceptance"],
    allAccepted=False,startOpen=False,productionExecuted=False,officialOwnerCreated=False,W2=False,dualSID=False,published=False)
write("final-status.json",status)
rows="\n".join(f"| {a['name']} | {a['bytes']} | {a['sha256']} |" for a in artifacts)
report=f"""# 原件、封存工作像与官方副作用内容视图

TEAM15/IMP141/r5，源码{head}/tree{tree}，仅Windows originals及其定向脚本。共享client_artifact_policy不改，不改Mac/HMAC/原30/5/10/Restart30Stop子10/r4有界8投影。数据接线不产生变更或恢复许可证，Start拒绝及原完整working unchanged veto保持。

ArtifactFrame从真实原件custody、完整原生working image及同卷封存WorkingArchive提取固定三文件opencodex-catalog.json、models_cache.json、opencodex-catalog-audit.jsonl。每项2MiB，真实代次来自被观察的工作文件volume/fileID，不能使用归档复制品的ID；安全内容键包含全部8投影、scope/policyDigest/ExcludedUnknown/full=false，文件ID单列不混安全保证。不把审计SACL换成full，也不从请求JSON构造frame。

新增非产物树核验严格对其余所有节点名称/字节/ID/8投影/目录子集；只忽略根下三确切叶名，目录冒同名、额外文件、配置/凭据变化、根/父对象变化均拒绝poison。封存frame前继续重验原private metadata/parts/8投影/目录集合。V2的只读assessment入口取original/Ready/PreStop/current实际frames及独立routes/context调用共享策略，错误poison，不推进phase/改工作baseline/移动或写文件。该V2完整调用工厂尚未实跑；新native测试直接组合真实封存frames，context pids/time/binding为明确合成数据，不签官方writer/exit/source许可。

| 执行 | raw | 结果 | ms |
| --- | --- | --- | --- |
| library01 | 101 | 首错E0616/E0624，编译失败0测试，无新EXE | {runs['library01']['elapsedMs']} |
| library02 | 0 | 4新native snapshot＋20共享pure policy，24过1既有忽略 | {runs['library02']['elapsedMs']} |
| product01 | 0 | 最终custom-protocol MSVC debug全部bins | {runs['product01']['elapsedMs']} |

首错tree/artifacts.rs:84，直接读兄弟WorkingImage私有字段；改由WorkingImage自己提供只读数据方法，RootTree方法仅对windows_client_originals024开放，不暴露字段/serde/新native proof。原编译source/log/result保留，旧EXE不冒本轮新制品。早先patch前提不符与偶发自有临时placeholder创建后精确移除记tool-edit-record，无递归或旧拒绝目标重试。

native正例实际封存Ready与PreStop并验证工作代次相同，current模拟原官方允许的native子集/cache/own audit append，共享策略数据评估为RestoreOriginal。None或错独立routes拒绝；ready/prestop变化、归档新增foreign、非产物config变化/新增文件/产物同名目录均拒绝。即使数据eligible，现有verify_working及restore_original仍拒绝变化，不发Closed或代替NativeLease。Stopped当前frame未成为canonical持久Stopped记录，后继必须完成这条恢复权限/阶段链后才能写。

最终{len(final)}产品输入和{len(rust)}Rust逐字对最终工作源/Git blob行尾等价，前端32份字节复用。制品非release预算/冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
{rows}

6场景53文件132595字节已私有ZIP内容核验，原现场留，非NTFS身份/安全克隆。无账户/Profile/Public根/GUI/官方进程或日常策略写，未删测试scene或重试历史拒绝；偶发自有placeholder精确移除单列。旧broker40588/Public20文件8拒读/普通MSI/B2非空CAP-trust及历史清理保持，overallCleanupComplete=false。route/sample/native锁/Bun/已过真GUI空宿主证据复用不重跑。完整官方初始化/NativeLease根移动交接/typed恢复Closed与原包H-S-D闭环仍未完成，allAccepted/productionExecuted=false，无业务模型请求/W2/双SID/发布。
"""
with (ROOT/"report.md").open("x",encoding="utf-8") as o:o.write(report)
with (DOCS/"docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md").open("a",encoding="utf-8",newline="\n") as o:
    o.write(f"\n\n## 2026-11-16 原生封存快照接共享副作用内容视图\n\n源码{head}/tree{tree}，Windows originals的原custody、同卷Ready/PreStop封存和fresh native current提取固定三产物frame；工作文件原ID单列，8投影/scope/excluded安全键不混归档复制ID。其余全部节点/config/凭据/目录子集严格不变，额外项或目录冒产物叶拒绝poison。V2只读assess入口接原policy，不推进phase/写文件或安装新baseline；完整调用native工厂未实跑，旧working unchanged和Start拒绝仍在，不能把content eligible当NativeLease/Closed。\n\n证据wl2-r5-artifact-observations-20261116-r1包含首编E0616/E0624/raw101/tests0/无新EXE原件；WorkingImage私有字段改为自身只读view，RootTree仅originals内部可见。随后24过1旧数据回放忽略/raw0（4native sealed/current＋20共享pure），None/错routes、PreStop漂移/归档foreign/config未知/产物同名目录等拒绝；合成pids/time/binding不签官方身份。最终clean custom-protocol debug bins0/{runs['product01']['elapsedMs']}ms，{len(final)}产品输入/{len(rust)}Rust对最终，三制品见product01-artifacts。6scene/53文件132595字节私有内容ZIP核验原件留；偶发自有placeholder创建后精确移除在tool-edit-record，不动旧拒绝。无账户/Profile/Public/GUI/官方日常策略改，overallCleanupComplete=false。\n\n尚需官方初始化、原NativeLease/claim根移动交接、canonical停止快照/typed恢复Closed和原包H-S-D生产；Stopped目前只是native数据view。旧broker/Public8拒读、普通MSI/CAP-trust/历史清理保持，已有route/sample/Bun/真GUI空宿主复用不重跑，不业务模型请求/W2/双SID/发布。\n")
files=[]
for p in sorted(ROOT.rglob("*")):
    if p.is_file() and p.name!="payload-manifest.json":
        raw=p.read_bytes();files.append(dict(path=p.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
write("payload-manifest.json",dict(files=files,productionAcceptance=False))
print(json.dumps(dict(head=head,tree=tree,runs=runs,artifacts=artifacts,productInputs=len(final),testRustInputs=len(rust),payloads=len(files))))
