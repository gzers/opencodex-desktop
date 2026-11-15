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
expected={"library01":(101,0,1,0),"library02":(101,0,1,0),"library03":(0,1,0,0),"library04":(101,1,1,0),"library05":(101,1,1,0),"library06":(101,1,1,0),"library07":(0,1,0,0),"library08":(0,10,0,2),"product01":(0,None,None,None)}
runs={};final=[];rust=[];scenes=[]
for name,(rc,passed,failed,ignored) in expected.items():
    result=load(ROOT/name/"result.json");assert result["exitCode"]==rc
    for row in load(ROOT/name/"source-manifest.json"):
        raw=(ROOT/name/"executed-source"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
        if name=="product01" or name=="library08" and row["path"].endswith(".rs"):
            assert raw==(CODE/row["path"]).read_bytes(),row["path"]
            assert raw.replace(b"\r\n",b"\n")==git("show",f"{head}:{row['path']}").replace(b"\r\n",b"\n"),row["path"]
            (final if name=="product01" else rust).append(row)
    if passed is not None:
        text=(ROOT/name/"stdout.log").read_text(encoding="utf-8-sig");m=re.search(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored",text)
        assert m and tuple(map(int,m.groups()))==(passed,failed,ignored)
        for s in load(ROOT/f"{name}-private-scenes.json")["scenes"]:
            assert sha(Path(s["archive"]).read_bytes())==s["archiveSha256"]
            scenes.append(dict(source=s["source"],archive=s["archive"],archiveSha256=s["archiveSha256"],files=len(s["files"]),bytes=sum(f["bytes"] for f in s["files"]),removed=False))
    runs[name]=dict(rawExit=rc,passed=passed,failed=failed,ignored=ignored,elapsedMs=result["elapsedMs"],accepted=False)
matrix_text=(ROOT/"library07/stdout.log").read_text(encoding="utf-8-sig")
cases=[]
for case,moved,raw in re.findall(r"root-sharing-matrix case=(\d+).*? moved=(true|false) rawWin32=(None|Some\(5\))",matrix_text):
    cases.append(dict(case=int(case),moved=moved=="true",rawWin32=5 if raw=="Some(5)" else None,diagnosticOnly=True))
assert [r["moved"] for r in cases]==[True,True,False,False,False]
assert load(ROOT/"product01/source.json")["head"]==head
write("final-source.json",dict(head=head,tree=tree,productInputs=final,library08RustEqualFinal=rust,gitLineEndingEquivalent=True,preexistingUntrackedSchemaNotManuallyEditedOrStaged=True))
artifacts=[]
for name in ["opencodex-desktop","ocxd","ocx"]:
    p=CODE/f"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/{name}.exe";raw=p.read_bytes();artifacts.append(dict(name=name,path=str(p),bytes=len(raw),sha256=sha(raw),sourceHead=head,sourceTree=tree,profile="debug",features=["custom-protocol"]))
write("product01-artifacts.json",artifacts)
for row in load(ROOT.parent/"wl2-real-gui-preparation-20261115-r1/frontend-dist-manifest.json"):
    raw=(CODE/"apps/desktop/ui/dist"/row["path"]).read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
prior=load(ROOT.parent/"wl2-r5-artifact-observations-20261116-r1/final-status.json");matrix=prior["matrix"]
matrix.append(dict(id="ClaimRootHandoff",result="plan-decision-required/raw5-retained",fact="Original DELETE root coexists with read-control/read-attributes metadata observer and moves with metadata-only. Actual root rename fails raw5 whenever claim file remains open, even read-only/share-delete with no lock. Five-case diagnostic isolates it; no force/alternative rename or automatic release/reopen implemented. A non-authorizing pending handoff or different explicit custody plan needs review."))
status=dict(time=datetime.now().astimezone().isoformat(),sourceHead=head,sourceTree=tree,contract=prior["contract"],baselineMain=prior["baselineMain"],baselineDocs=prior["baselineDocs"],previousCode=prior["sourceHead"],runs=runs,artifacts=artifacts,matrix=matrix,sharingMatrix=cases,
    firstError=dict(run="library01",rawExit=101,rawWin32=32,stage="fixture extra root directory pin blocks original DELETE capture",line=235),
    firstComponentError=dict(run="library02",rawWin32=32,stage="NativeLease generic parent pin conflicts with original DELETE root",fixedBy="metadata-only claim parent observation"),
    blockingError=dict(run="library05/library06",rawWin32=5,stage="actual original root rename with open claim file",originalClaimReleaseImplemented=False),
    planRequest="claim-handoff-decision-request.json",continuousOpenClaimPositivePassed=False,
    ignoredCases=["private namespace helper actually executed by parent","known blocked continuous-open-claim root-move positive; original assertions retained and failed raw5 executions preserved"],
    limits=["Metadata coexistence is proven native component behavior, not complete acquire_existing Owner factory or production source integration",
        "Five-case diagnostic test passes by recording raw outcomes; cases2/3/4 still FAIL TO MOVE, never counted as production successes",
        "Final10 passed/2ignored does not clear the known blocked root claim test. It stays explicitly blocked rather than rewritten to expect success or deleting assertions",
        "Ineffective claim-file share-delete experiment reverted; N/L/K and final claim file keep original strict sharing",
        "Pending claim-unheld interval is conditional only: fixed official reauth source directly uses claim for credential writes, so N/L/K alone are not established equivalent exclusion. Evaluate custody retaining original root/stable locks in place; no new plan implemented",
        "Official initialization/owner receipt, canonical stopped artifact/typed restoration, broker original Owner/H-S-D and actual Closed sampling positives remain incomplete",
        "Debug bins not release budgets/frozen candidate; all old GUI/ordinary MSI/CAP-trust and cleanup limits remain"],
    cleanup=dict(scenes=scenes,archivedScenes=len(scenes),archivedFiles=sum(s["files"] for s in scenes),archivedBytes=sum(s["bytes"] for s in scenes),originalsRetained=True,nativeMetadataCloned=False,knownSyntheticADSArchivedAsObservationOnly=True,newAccounts=0,newProfiles=0,newPublicTaskRoots=0,newGUIProcesses=0,standaloneBunSQLiteChildrenInFinalRegression=8,officialModulesImported=False,newOfficialServices=0,deletionAttemptedByController=False,historicalRejectedDeletionRetried=False,overallCleanupComplete=False,dailyPathConfigPolicyChanged=False),
    allAccepted=False,productionExecuted=False,officialOwnerCreated=False,startOpen=False,W2=False,dualSID=False,published=False)
write("final-status.json",status)
rows="\n".join(f"| {a['name']} | {a['bytes']} | {a['sha256']} |" for a in artifacts)
report=f"""# Windows claim与原件根移动的原生边界

TEAM15/IMP141/r5，源码{head}/tree{tree}。仅Windows原生metadata/private检查及coordination/test变更，原四trait/HMAC/预算/r4排除保持。原claim文件共享恢复strict，保留实际有效的metadata-only父根观察：READ_CONTROL|FILE_READ_ATTRIBUTES0x20080、shareR/W/Delete，原目录FD/ID/8投影/privateDACL及外部祖先pin/当前路径FD一致。N/L/K原pin/共享方式不改，RootTree始终持有原DELETE句柄。

| 执行 | raw | 结果 | ms |
| --- | --- | --- | --- |
"""+"\n".join(f"| {n} | {r['rawExit']} | {r['passed']}过/{r['failed']}败/{r['ignored']}忽略 | {r['elapsedMs']} |" for n,r in runs.items())+f"""

首错library01夹具额外create_private_directory pin阻止RootTree原DELETE capture/raw32/coordination_tests:235，原夹具多余pin释放后library02真正NativeLease父目录pin仍raw32。只将claim父目录改为元数据观察后library03共存1过；根移动library04在意图记录前因夹具journal父根非private拒绝，改新任务私有journal，不修任何旧ACL。library05真正rename raw5；library06仅claim保留文件试shareDelete也raw5，否定了最初共享flag足以解决的假设，原始错误/源/EXE/现场均保留，未重写成通过。

library07一次采齐5个独立root：

| 条件 | move | 原码 |
| --- | --- | --- |
| 无额外句柄 | 成功 | 无错误 |
| 仅元数据父目录句柄 | 成功 | 无错误 |
| 仅只读claim文件句柄，shareDelete | 失败 | 5 |
| claim文件＋元数据句柄 | 失败 | 5 |
| 实际NativeLease | 失败 | 5 |

矩阵的测试raw0只表示诊断记录和原字节保全断言完成，后三格未移动，不能签生产。primary-reference-review提供微软FileRenameInformation的目录内文件不能open条款，符合本机实测，不授权POSIX/force回退。未调用其它move/delete工具绕过，也未关闭无关句柄。

最终library08为10过2忽略：原namespacehelper由父测试实际执行；另一个是明确已阻塞的“continuous-open-claim root rename”正例，原全部断言保留，原真实失败library05/06仍在。不是已通过；标ignored是冻结后继前提而非把原码当0。其它受影响native/kernel/private/security/真实SQLite/Bun反例重新验证；无旧GUI空宿主重跑。无效shareDelete生产实验已撤销，原严格文件分享保持。

最小后继决定见claim-handoff-decision-request.json。原固定tgz的claim-only-writer-source核main-account.ts:463/394：reauth在exclusive claim中直接atomicWriteFile凭据，该函数和claim helper均未取得N/L/K。仅静态核对，未执行凭据写入；不能以N/L/K自动替代claim。ClaimRelocationPending空窗只能是另有充分排他证明的条件备选。优先请评估保留原root和两稳定锁原位/claim连续排他、其它原节点同卷custody的设计，须明确新的canonical节点意图/进度/原身份恢复规则，不兼容迁移旧记录；若仍选空窗，必须覆盖claim-only writer。当前两方案均未实现，不drop/reopen/force/POSIX/copy-delete，原预算/失败保全不变。

metadata原语完善不代表完整acquire_existing实际Owner构造已证明。原SharedRoute与同5秒采样接口已消费；副作用frame和原生四域/Bun互操作组件已有前轮证据，但canonicalStopped/typed restore、official初始化/ownerreceipt、broker原Owner与原包H-S-D以及actualClosed采样正例仍未完成，公共Start拒绝。此轮在具体方案决策处回交，不以接口存在或diag绿灯代整组成功。

最终{len(final)}产品输入/{len(rust)}最终Rust对工作字节/Git blob行尾等价，前端32份复用，MSVCcustom-protocol debug bins0/{runs['product01']['elapsedMs']}ms。非release预算/签名冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
{rows}

{len(scenes)}场景/{status['cleanup']['archivedFiles']}文件观察/{status['cleanup']['archivedBytes']}字节私有ZIP内容核验，已知合成ADS仅另存观察，原现场保留，不是NTFS身份安全克隆。无账户/Profile/Public/GUI/官方服务/日常PATH策略改，无控制器删除尝试和旧拒绝重试；最终回归的8个Bun子进程只是独立合成SQLite竞争者，不导入官方服务。旧失败broker40588/Public20文件8拒读、CAP/trust非空/普通MSI/全部历史清理不关，overallCleanupComplete=false。不业务模型请求/W2/双SID/发布。
"""
with (ROOT/"report.md").open("x",encoding="utf-8") as o:o.write(report)
with (DOCS/"docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md").open("a",encoding="utf-8",newline="\n") as o:
    o.write(f"\n\n## 2026-11-16 claim与根移动原生阻塞及最小后继决定\n\n源码{head}/tree{tree}，原DLL/route/status/artifact前轮证据持续。metadata-only父根原FD/ID/8投影/private读回已解决与RootTree DELETE句柄共存，真实目录移动也可保留该元数据句柄；N/L/K不改。真实claim文件打开则rootrename raw5，即使readonly/shareDelete且无SQLite锁：library07完整5格baseline/meta成功，file/both/lease失败5，diag raw0非后三格成功。微软原API目录内无openfile条件记录primary-reference-review。失败前缀中library01夹具多余rootpin32、library02实际parentpin32、library04夹具journalprivate、library05/06真正rename5均保留分列，未追溯放行。原shareDelete实验无效已撤回strict。\n\n新不可覆盖证据wl2-r5-claim-relocation-20261116-r1含8轮原输入/log/result/EXE、完整矩阵与scene清单、最后源码制品、claim-handoff-decision-request.json。最终library08为10过2忽略，namespacechild实际由父测试执行；continuous-open-claim根移动正例明确blocked/ignored且原断言未改，实际raw5失败保留，不计接受。最终MSVCcustom-protocol debug bins0/{runs['product01']['elapsedMs']}ms，{len(final)}产品输入/{len(rust)}Rust对源，三制品见product01-artifacts。{len(scenes)}scene/{status['cleanup']['archivedFiles']}文件观察/{status['cleanup']['archivedBytes']}字节ZIP内容核验原现场留，非NTFS元数据克隆，无控制器删/旧拒绝重试，overallCleanupComplete=false。\n\n需要明确Windows claim交接计划。claim-only-writer-source已核原tgz中main-account.ts:463/394凭据写入及claim helper，不能把N/L/K当claim自动替代，未执行凭据/网络请求。优先评估保持原root/两稳定锁原位与claim连续排他、其它节点同卷custody的新canonical节点意图/恢复规则；若选typed Pending空窗必须另证claim-only writer排他，不能仅保留N/L/K便放行。两方案均未实施，不自动drop/reopen/force/POSIX/copy-delete，不开Start。\n\n共享route/同采样5秒已消费，完整NativeLease根交接/初始化/ownerreceipt、canonical stopped/typed restore、broker原Owner与原包H-S-D/Closed采样正例仍待，旧broker/Public8拒读/B2CAP-trust/普通MSI/历史清理保持。无官方日常配置/业务模型请求/W2/双SID/发布；本轮仅在具体方案变化处回交。\n")
files=[]
for p in sorted(ROOT.rglob("*")):
    if p.is_file() and p.name!="payload-manifest.json":
        raw=p.read_bytes();files.append(dict(path=p.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
write("payload-manifest.json",dict(files=files,productionAcceptance=False))
print(json.dumps(dict(head=head,tree=tree,runs=runs,artifacts=artifacts,sharingMatrix=cases,scenes=len(scenes),files=status['cleanup']['archivedFiles'],bytes=status['cleanup']['archivedBytes'],payloads=len(files))))
