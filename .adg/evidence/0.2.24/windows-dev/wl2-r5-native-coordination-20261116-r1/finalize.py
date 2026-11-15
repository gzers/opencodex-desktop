from pathlib import Path
from datetime import datetime
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path(r"C:/Users/15119/.codex/worktrees/024-windows-native/code")
DOCS=ROOT.parents[4]
load=lambda p:json.loads(p.read_text(encoding="utf-8-sig"))
sha=lambda b:hashlib.sha256(b).hexdigest()
git=lambda *args:subprocess.check_output(["git","-C",str(CODE),*args])
def write(name,value):
    with (ROOT/name).open("x",encoding="utf-8") as out:json.dump(value,out,ensure_ascii=False,indent=2)

head=git("rev-parse","HEAD").decode().strip();tree=git("rev-parse","HEAD^{tree}").decode().strip()
runs={};final=[];rust=[];scenes=[]
expected={"library01":(101,59,1,0),"library02":(0,60,0,0),"library03":(0,7,0,1),"library04":(0,8,0,1),"product01":(0,None,None,None)}
for name,(rc,passed,failed,ignored) in expected.items():
    result=load(ROOT/name/"result.json");assert result["exitCode"]==rc
    for row in load(ROOT/name/"source-manifest.json"):
        raw=(ROOT/name/"executed-source"/row["path"]).read_bytes()
        assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
        if name=="product01" or name=="library04" and row["path"].endswith(".rs"):
            assert raw==(CODE/row["path"]).read_bytes(),row["path"]
            assert raw.replace(b"\r\n",b"\n")==git("show",f"{head}:{row['path']}").replace(b"\r\n",b"\n"),row["path"]
            (final if name=="product01" else rust).append(row)
    if passed is not None:
        text=(ROOT/name/"stdout.log").read_text(encoding="utf-8-sig")
        m=re.search(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored",text)
        assert m and tuple(map(int,m.groups()))==(passed,failed,ignored)
        for scene in load(ROOT/f"{name}-private-scenes.json")["scenes"]:
            assert sha(Path(scene["archive"]).read_bytes())==scene["archiveSha256"]
            scenes.append(dict(source=scene["source"],archive=scene["archive"],archiveSha256=scene["archiveSha256"],files=len(scene["files"]),bytes=sum(f["bytes"] for f in scene["files"]),removed=False))
    runs[name]=dict(rawExit=rc,elapsedMs=result["elapsedMs"],passed=passed,failed=failed,ignored=ignored,accepted=False)
assert load(ROOT/"product01/source.json")["head"]==head
write("final-source.json",dict(head=head,tree=tree,productInputs=final,library04RustEqualFinal=rust,gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyEditedOrStaged=True))
artifacts=[]
for name in ["opencodex-desktop","ocxd","ocx"]:
    p=CODE/f"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe";raw=p.read_bytes()
    artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile="debug",features=["custom-protocol"]))
write("product01-artifacts.json",artifacts)
for row in load(ROOT.parent/"wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json"):
    raw=(CODE/"apps/desktop/ui/dist"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
bun=load(ROOT/"bun-competitor-input.json");assert sha(Path(bun["path"]).read_bytes())==bun["sha256"]
text=(ROOT/"library04/stdout.log").read_text(encoding="utf-8-sig")
assert text.count('nativeHeld=true raw=Some(73)')==4 and text.count('nativeHeld=false raw=Some(0)')==4
prior=load(ROOT.parent/"wl2-r5-integration-20261115-r1/final-status.json");matrix=prior["matrix"]
matrix.append(dict(id="NativeLease",result="native-lock-component-pass/not-production-permit",fact="Retained private original four Windows files, claim-exclusive/N-L-K-shared+reserved ranges, same original deadline/token/ID/8projections/content; real rusqlite and exact bundle Bun competitors blocked while held and succeed after release, all bytes unchanged. Busy/sidecar/header/WAL/ADS/links/missing/expiry/raw data drift refuse; failed recheck poisons. Official initialization/owner receipt and root relocation handoff still incomplete; no production mutation authorization."))
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract=prior["contract"],baselineMain=prior["baselineMain"],baselineDocs=prior["baselineDocs"],previousCode=prior["sourceHead"],runs=runs,artifacts=artifacts,matrix=matrix,
    firstError=load(ROOT/"hypotheses02-exact-deadline-and-poison.json")["firstError"],firstErrorRawExit=101,
    firstErrorCause="positive fixture passed a newly computed now+10 deadline rather than the original lease deadline; no-extension production check correctly refused",
    bunStandaloneSQLiteChildren=8,bunHeldRawExit=[73]*4,bunReleasedRawExit=[0]*4,officialModulesImported=False,officialServicesStarted=False,
    cleanup=dict(scenes=scenes,archivedSceneCount=len(scenes),archivedFiles=sum(s["files"] for s in scenes),archivedBytes=sum(s["bytes"] for s in scenes),originalsRetained=True,knownSyntheticADSArchivedSeparatelyAsObservation=True,nativeMetadataCloned=False,deletionAttemptedByController=False,historicalRejectedDeletionRetried=False,overallCleanupComplete=False,newAccounts=0,newProfiles=0,newPublicTaskRoots=0,newGUIProcesses=0,dailyPathConfigPolicyChanged=False),
    limits=["Only the private scoped native exclusion component is implemented; not official initialization/owner receipt/schema authority or root-move handoff/reanchor",
        "acquire_existing derives actual native TokenUser DEFAULT_PATH namespace/source but actual retained Owner path is compiled only, not executed as a production permit",
        "SQLite byte locks exclude compliant users, not raw unrelated-offset writes; drift detection and poison are required, no global atomicity guarantee",
        "The expiry fixture shortens its own private deadline only after no-extension poison; separate expired factory refusal is tested, no actual long-lived production expiry proof",
        "Pure route/status cases repeated in first two broad selectors; final native-only selection avoids rerunning unchanged54 cases",
        "Private ignored namespace child is run by its parent, not a skipped production acceptance case",
        "ZIP contains bytes plus a separately named known test ADS observation, not NTFS link/identity/security clones; all originals retained",
        "GUI empty broker/CLI/MSI not rerun; B2 CAP-trust/ordinary MSI/historical cleanup gaps unchanged; debug products not release/frozen candidate"],
    allAccepted=False,productionExecuted=False,officialOwnerCreated=False,startOpen=False,W2=False,dualSID=False,published=False)
write("final-status.json",status)
rows="\n".join(f"| {a['name']} | {a['bytes']} | {a['sha256']} |" for a in artifacts)
report=f"""# Windows NativeLease原始字节锁与双SQLite互操作

源码{head}/tree{tree}，沿TEAM15/IMP141/r5，公共Start拒绝保持。本批在Windows originals内实现private nonClone/nonserde/nonSend的NativeLease原语及原生private file句柄只读核验，不改共享四trait/HMAC/官方文件、Mac或日常配置。

按固定官方9份源及SQLiteWindowsVFS确认，原claim→N→L→K次序：claim对pending/reserved/shared512独占；N/L/K先按WindowsVFS取得shared510，再reserved1排他，全部FAIL_IMMEDIATELY，不busy-wait、不在生产开SQLite或执行PRAGMA/recovery/init/schema写。保留原文件句柄、父pin、原ID/8安全投影/private TokenUser-SYSTEM/noADS/singlelink/原字节，拒绝sidecar/WAL/未知header/缺席，原范围Unknown拒绝。生产路径仅原Token Profile、原KnownFolder DEFAULT_PATH0x400、SID和canonicalCODEX_HOME摘要派生；HOME/USERPROFILE/LOCALAPPDATA伪值在独立子进程不改变namespace，没有创建真实namespace。

每次check核同原deadline/token/原文件及path，错误poison不再准入，内核锁留到作用域释放，释放自有锁不让过期/失败变成功。真实raw writer写未被SQLite范围覆盖的字节会被检测拒绝，冲突保留，明确不承诺锁阻止任意写入。

| 执行 | raw | 结果 | ms |
| --- | --- | --- | --- |
| library01 | 101 | 59过1败，首错正例测试误给新deadline | {runs['library01']['elapsedMs']} |
| library02 | 0 | 60过：修测试原截止并加失败poison | {runs['library02']['elapsedMs']} |
| library03 | 0 | 窄选7过1helper忽略；新增raw漂移反例 | {runs['library03']['elapsedMs']} |
| library04 | 0 | 窄选8过1helper忽略；固定Bun互操作 | {runs['library04']['elapsedMs']} |
| product01 | 0 | 最终custom-protocol MSVC debug bins | {runs['product01']['elapsedMs']} |

首错coordination_tests.rs:71原InvalidData/raw101，原生产no-extension断言正确；只把正例每次end()改为一个原deadline，未放宽时间/原件/竞争断言，原失败源/log/result/EXE/scene保留。首两轮宽selector重复了未变的54路由状态例，后两轮改窄选，不宣称完全无重复。忽略项是由父测试实际执行的私有namespace child，不冒普通生产通过。private expiry测试先被延长期限检查poison，之后短私有deadline重查不单签真实时间流逝；expired factory、no-extension和失败后禁止重入均有直接反例。

真实rusqlite竞争者对4处BEGINIMMEDIATE均被原生锁挡住，N/L/K普通读可共存、claim普通读拒绝，释放后BEGIN成功；SID/目录/文件原ID/8投影/内容不变。固定bundle Bun115281496字节/SHAcb1d8e6564e655e49d30cead359750a1cbbd12f42651caa4b62244d7bd93c244逐字对原bundle成员。8个独立Bun SQLite子进程只运行自有合成库脚本，不导入官方服务模块；持锁4次原raw73且其SQLitebusy事实真，释放后4次原raw0，仍未写库字节。73为测试helper在实际busy分支设置的原退出码，不能把它冒官方查询码或改写0。

NativeLease::acquire_existing的实际Owner/source/原namespace绑定仅编译，未实际工厂/official init/owner journal receipt或root move handoff/reanchor验收。没有把原语插为生产变更许可证；完整官方副作用、初始化、路径交接及原包H-S-D闭环继续未完，原working unchanged拒绝仍在，Start未开放。

最终{len(final)}份产品输入/{len(rust)}份最终测试Rust对最终工作字节/Gitblob行尾等价，原前端32份字节复用不变。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
{rows}

共{len(scenes)}保留场景/{status['cleanup']['archivedFiles']}文件观察/{status['cleanup']['archivedBytes']}字节私有ZIP内容核验。已知合成ADS另存观察项，ZIP不是NTFS身份/硬链/安全克隆；原scene均保留，无控制器删除尝试/旧删除拒绝重试，无账户/Profile/Public根/GUI/日常策略变化。首次GUI失败broker40588/Public20文件8拒读、B2 CAP/trust非空、普通MSI及全部旧清理缺口继续，overallCleanupComplete=false。完整矩阵在final-status.json，allAccepted/productionExecuted=false，不业务模型请求/W2/双SID/发布。原生原语Bun子进程不冒官方服务或产品进程0。
"""
with (ROOT/"report.md").open("x",encoding="utf-8") as out:out.write(report)
with (DOCS/"docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md").open("a",encoding="utf-8",newline="\n") as out:
    out.write(f"\n\n## 2026-11-16 Windows NativeLease原始锁与Bun SQLite组件\n\n源码{head}/tree{tree}，Windows originals原语保留原4文件/父pin/TokenUser/ID/8投影及kernel ranges，claimEXCLUSIVE、N/L/Kshared+reserved依原次序fail-immediate，生产不SQLiteopen/recover/init/schemawrite。实际Owner/原KnownFolder DEFAULT_PATH/SID namespace构造已编译，fake HOME/USERPROFILE/LOCALAPPDATA独立子进程不能换namespace；尚未接official admission/owner receipt/whole-root handoff或副作用生产许可，Start和working unchanged拒绝保持。\n\n证据wl2-r5-native-coordination-20261116-r1含4组源/log/result/原EXE摘要、首错和现场、最终制品/矩阵。首轮59过1败/raw101，coordination_tests:71正例错误重算now+10被原no-extension正确拒绝，改同一原deadline不降断言；后60过/7过1helper忽略/8过1helper忽略均raw0，窄选末两轮不重复54纯例。真rusqlite与固定bundle Bun SQLite竞争证明held拒绝/释放后成功，同4fileID/8投影/bytes不变；Bun8个单独合成库helperheld4次raw73/busy真、released4次raw0，不导入官方服务/不当官方生产测试。未知sidecar/WAL/header/ADS/硬链/占用/缺席/错期/原始字节漂移全部拒绝，失败poison锁持有到scoped release，留原冲突。expiry fixture的实际覆盖局限在report明列。\n\n最终clean custom-protocol MSVC debug bins0/{runs['product01']['elapsedMs']}ms，{len(final)}产品输入/{len(rust)}Rust对最终，三制品SHA见product01-artifacts。{len(scenes)}场景/{status['cleanup']['archivedFiles']}文件观察/{status['cleanup']['archivedBytes']}字节私有内容归档含已知合成ADS分列，原scene留、非NTFS身份安全克隆。无账户/Profile/Public/GUI/日常策略改、无控制器删除/旧拒绝重试，overallCleanupComplete=false；原GUI空组件复用，不W2/双SID/发布，完整副作用/初始化/NativeLease交接/原包H-S-D闭环仍未验。\n")
files=[]
for p in sorted(ROOT.rglob("*")):
    if p.is_file() and p.name!="payload-manifest.json":
        raw=p.read_bytes();files.append(dict(path=p.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
write("payload-manifest.json",dict(files=files,productionAcceptance=False))
print(json.dumps(dict(head=head,tree=tree,runs=runs,artifacts=artifacts,scenes=len(scenes),files=status['cleanup']['archivedFiles'],bytes=status['cleanup']['archivedBytes'],payloads=len(files))))
