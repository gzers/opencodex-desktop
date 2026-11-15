from pathlib import Path
from datetime import datetime
import hashlib
import json

ROOT = Path(__file__).resolve().parent
DOCS = ROOT.parents[4]
load = lambda p: json.loads(p.read_text(encoding="utf-8-sig"))
sha = lambda raw: hashlib.sha256(raw).hexdigest()
desktop = load(ROOT / "desktop-result.json")
assert desktop["actualGUIComponentPassed"] is True
prep = ROOT.parent / "wl2-real-gui-profile-fix-20261115-r1"
previous = load(prep / "final-status.json")
matrix = previous["matrix"]
for row in matrix:
    if row["id"] == "H":
        row.update(result="actual-GUI-empty-broker-component-pass", fact="Real main GUI/native HWND/pageFinished under ordinary marenben session7 passed across normal exit, exact Child-only forced GUI, fresh authentication and explicit empty-host exit; original broker31484/FILETIME134392264041593579/imageID562949954696391 preserved across manager generations1..4 with original creation handle. No official Owner/Job/H-S-D production proof; broker raw exit unrecorded=null.")
status = dict(time=datetime.now().astimezone().isoformat(), sourceHead=previous["sourceHead"], sourceTree=previous["sourceTree"],
    contract=previous["contract"], baselineMain=previous["baselineMain"], baselineDocs=previous["baselineDocs"],
    artifacts=previous["artifacts"], desktop=desktop["result"], modes=desktop["modes"], matrix=matrix,
    sameSidExportComplete=True, exportedFiles=32, firstError=None,
    originalFailedRunPreserved="../wl2-real-gui-run-20261115-r1/", brokerRawExit=None,
    actualGUIComponentPassed=True, officialOwnerCreated=False, productionExecuted=False, startOpen=False, allAccepted=False,
    cleanup=dict(originalTaskBase=desktop["originalTaskBase"], originalTaskSceneRetained=True,
        newAccounts=0, newProfiles=0, dailyPathConfigPolicyChanged=False, nativeMetadataCloned=False,
        deletionAttempted=False, historicalRejectedDeletionRetried=False, overallCleanupComplete=False,
        oldFailedEmptyBrokerExitUnconfirmed=True),
    remaining=previous["remaining"], W2=False, dualSID=False, published=False)
with (ROOT / "final-status.json").open("x", encoding="utf-8") as out:
    json.dump(status, out, ensure_ascii=False, indent=2)
report = """# 修复后真实GUI普通桌面结果

原marenben SID1003/session7/elevated=false，用户从实际桌面运行修复入口。源码6e312f02270f247d2f440ec369385e523a4a8043/treef669be64f862007a8b148bd2bfc99e1f71c5e528，沿用前轮最终MSVC custom-protocol debug准确三输入，未重编译/重跑其它已通过项。observer114108/raw0，1过0败0忽略，5.78秒。32份sameSID日志逐份base64/长度/SHA256核验，原私有ACL不改。

| 模式 | manager PID | generation | 原退出码 | 事实 |
| --- | --- | --- | --- | --- |
| normal | 89060 | 1 | 0 | 实际main原生可见窗口/pageFinished，正常退出后原broker存活 |
| hold | 40180 | 2 | 1 | 仅对原创建Child GUI句柄执行故障终止，原broker保持 |
| after-failure | 46168 | 3 | 0 | 新真实GUI fresh认证、原broker身份一致 |
| finish | 51796 | 4 | 0 | 新GUI认证、明确请求空host退出 |

每轮由独立observer确认HWND属于刚创建的GUI且visible；实际main URL http://tauri.localhost/、完整normal setup、fixtureManager=false。每轮私有pipe核实际server身份，managerGeneration单调，旧连接重放拒绝。broker31484/FILETIME134392264041593579/SID1003/session7/volume247287385/fileID562949954696391全程同原对象，observer保留原创建句柄；bootstrap/挂起broker与转接边界noJob。finish后原句柄Wait确认退出，host-events记明确非force空host退出，但探针没有采集broker原始exitcode，仍为null，不能用manager0或host事件补签0。

这是实际产品GUI加空broker连续性组件通过；officialOwnerCreated/lifecycleAccepted=false，frontendDomAssertion=false。未创建官方Job/Owner，不能代H完整原官方Owner/Job/source/H生产组合，不产生原件Closed资格或开放Start。完整B1/B2/B3/O1/O2/H/S-D/P/MSI表在final-status.json，原公共门拒绝。

首次0c失败raw101和broker40588现场仍保留，既有原始日志不覆盖，旧broker退出状态仍未知。不凭PID收养/强杀；本成功任务根C:/Users/marenben/ocx024-h-1fa5e23f-12c9-4289-84f3-bcc901642b35保留待同SID全目录归档核验后仅清自有成功目录。原export仅包含日志，不声称已归档完整WebView缓存/制品树，也不是NTFS元数据克隆。无账户/Profile/PATH/日常策略变化，未删除或重试任何历史自动审批拒绝，overallCleanupComplete=false。

后续范围外最小请求仍是shared pure route declaration提取及Status统一原5秒上下文，见前轮精确JSON。Windows完整官方副作用/NativeLease/真实S-D与生产组合继续未完，普通MSI与非空CAP/trust/历史清理缺口保持。无官方修改/业务同步/模型请求/W2/双SID/发布。
"""
# Copy the exact observed hold PID, rather than assuming an earlier fixture's.
hold = load(ROOT / "received/out/after-hold.json")
report = report.replace("| hold | 40180 |", f"| hold | {hold['managerPid']} |")
with (ROOT / "report.md").open("x", encoding="utf-8") as out:
    out.write(report)
handoff = DOCS / "docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md"
with handoff.open("a", encoding="utf-8", newline="\n") as out:
    out.write("\n\n## 2026-11-15 r4修复版真实GUI桌面结果\n\n用户在marenben原普通桌面运行修复6e312f022准确输入，observer114108/raw0/1过0败/5.78秒。32份sameSID导出逐字长度/SHA256核验，原始回执/日志/完整矩阵在wl2-real-gui-fixed-run-20261115-r1。四轮实际main原生visible HWND/pageFinished/normal setup分别managerGeneration1..4；normal/after-failure/finish原退出0，hold仅原创建Child GUI故障终止raw1保留。broker31484/FILETIME134392264041593579/原imageID562949954696391始终同原创建对象/句柄，noJob与fresh认证/旧连接拒绝成立；finish明确空host退出经原Wait观察，但broker原exitcode未采集仍null，不补填0。实际GUI空broker组件过，不是official Owner/Job或H-S-D生产验收，frontendDomAssertion=false，Start仍拒绝。\n\n无新编译/不重跑既过项，源码/制品与前轮6e312准确一致。首次0c失败与broker40588保留且退出仍未知；新成功Profile子树和Public制品根待同SID全目录归档后清理，当前只是日志归档，不称全部清理或NTFS克隆。无账户/Profile/PATH/日常策略改、无删除或旧拒绝重试，overallCleanupComplete=false。共享纯route声明/Status统一原5秒最小接口、完整官方副作用/NativeLease/原包S-D生产及普通MSI/CAP-trust环境/历史清理未完，无官方/业务模型请求/W2/双SID/发布。\n")
files = []
for path in sorted(ROOT.rglob("*")):
    if path.is_file() and path.name != "payload-manifest.json":
        raw = path.read_bytes()
        files.append(dict(path=path.relative_to(ROOT).as_posix(), bytes=len(raw), sha256=sha(raw)))
with (ROOT / "payload-manifest.json").open("x", encoding="utf-8") as out:
    json.dump(dict(files=files, productionAcceptance=False), out, indent=2)
print(json.dumps(dict(payloads=len(files), actualGUIComponentPassed=True, productionAcceptance=False)))
