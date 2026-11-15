from pathlib import Path
from datetime import datetime
import hashlib
import json

ROOT = Path(__file__).resolve().parent
DOCS = ROOT.parents[4]
load = lambda p: json.loads(p.read_text(encoding="utf-8-sig"))
sha = lambda raw: hashlib.sha256(raw).hexdigest()
source = load(ROOT / "final-source.json")
prepared = load(ROOT / "prepared-location.json")
primitive = load(ROOT / "archive-primitives-result.json")
assert primitive["rawExit"] == 0 and primitive["positiveZipVerified"] is True
assert primitive["sourceChangeRefused"] is True and primitive["extraDirectoryRefused"] is True
assert load(ROOT / "final-parse-and-function-equivalence.json")["allExecutedArchiveFunctionsEqualPrepared"] is True
assert load(ROOT / "prepare-result.json")["rawExit"] == 0
prior = load(ROOT.parent / "wl2-real-gui-fixed-run-20261115-r1/final-status.json")
status = dict(time=datetime.now().astimezone().isoformat(), sourceHead=source["head"], sourceTree=source["tree"],
    productSourceHead=source["productSourceHead"], artifacts=prior["artifacts"], matrix=prior["matrix"],
    contract=prior["contract"], baselineMain=prior["baselineMain"], baselineDocs=prior["baselineDocs"],
    firstError=load(ROOT / "hypotheses02-hashtable-byte-sum.json"), archivePrimitiveResult=primitive,
    preparation=prepared, cleanupExecution="awaiting-original-ordinary-desktop",
    cleanup=dict(newPublicTaskRoots=1, archiveNotYetVerifiedByController=True, originalTaskSceneRetained=True,
        deletionAttemptedByController=False, historicalRejectedDeletionRetried=False, overallCleanupComplete=False,
        newAccounts=0, newProfiles=0, dailyPathConfigPolicyChanged=False,
        ownPrimitiveScenes=load(ROOT / "archive-primitives-scenes.json")["scenes"], failedOriginalBrokerExitUnconfirmed=True),
    productRebuilt=False, productCodeUnchanged=True, actualGUIComponentPassed=True,
    officialOwnerCreated=False, productionExecuted=False, startOpen=False, allAccepted=False,
    remaining=prior["remaining"], W2=False, dualSID=False, published=False)
with (ROOT / "final-status.json").open("x", encoding="utf-8") as out:
    json.dump(status, out, ensure_ascii=False, indent=2)
report = """# 成功GUI任务同SID完整归档清理准备

代码0549a660ee3f77ea243e798ec55b01f52545e492仅增加test/windows024-native下两个一次性归档入口；实际产品源仍为已通过真实GUI组件的6e312f022，产品Rust未改、不重复编译或GUI测试。代码和证据沿原分支分离。定向消费前置假设，不跑W2/双SID/发布。

原同SID32日志已归档，但成功树的全部制品/WebView缓存尚未全量归档。本入口只允许普通marenben/session7，检查固定成功任务根及其原始完成回执摘要；创建新同SID/SYSTEM私有归档目录，对成功Profile树和Public输入日志完整ZIP复制，逐项验证文件SHA256/长度及包含空目录的准确集合，再重新核源和绝对目标，最后只删除C:/Users/marenben/ocx024-h-1fa5e23f-12c9-4289-84f3-bcc901642b35。ZIP只是字节复制，非NTFS安全/身份克隆。未读/占用/新增/漂移/重解析/超界均拒绝删除；不改旧ACL/日常政策、不终止进程、不重试旧拒绝、一次intent后失败亦不自动重试。

Public成功根只归档，等控制器消费同SID结果后再核准清理；首次失败根、旧空broker40588、所有账户/Profile根、历史被拒路径都不作为目标。broker原退出码仍null，不能用成功observer/manager0或explicit-exit事件填0。四轮GUI/空broker组件通过不等于真实official Owner或完整H-S-D。

原语验证实际执行完整ZIP往返及目录集合：正例通过，内容在初始manifest后改变时archive bytes differ，增加空目录时task directory/file set changed；全程无Remove/GUI/官方进程。首次raw1为PowerShell5无法由Measure-Object -Property访问hashtable bytes键，原source/log/小场景保留；投影Int64再求和修正后raw0，守门和摘要断言不减。最终脚本所有归档函数AST内容摘要与实际已执行版本一致；随后只在receipt初值补deletionAttempted=false。两个自有小型原语场景7文件保留，不调用任何旧拒绝清理。

准备脚本原退出0，新Public入口仅自身ACL设置controller/SYSTEM full及marenben read/out Modify；脚本拷贝摘要和解析通过。入口C:/Users/Public/ocx-h024-gui-archive-20261115-r1/archive-h024-fixed-success.cmd。本证据是准备时快照，尚未消费桌面归档结果，不称原成功目录已删/所有残留已清；后继结果另建不可覆盖轮次。

完整矩阵/产品SHA/源码及清理状态见final-status.json。公共Start仍拒绝，shared路由declaration纯提取、状态单次原5秒接口待集成；完整副作用/NativeLease/原包S-D生产组合、普通MSI、非空CAP-trust和历史清理缺口保持。无官方2.99或日常PATH配置策略修改，无业务同步/模型请求/W2/双SID/发布。
"""
with (ROOT / "report.md").open("x", encoding="utf-8") as out:
    out.write(report)
handoff = DOCS / "docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md"
with handoff.open("a", encoding="utf-8", newline="\n") as out:
    out.write("\n\n## 2026-11-15 真实GUI成功树完整归档清理准备\n\n代码0549a660ee3f77ea243e798ec55b01f52545e492已推，只加test原始同SID一次性归档入口，产品源/三制品仍6e312f022、不重编译/重测GUI。入口C:/Users/Public/ocx-h024-gui-archive-20261115-r1/archive-h024-fixed-success.cmd已备，实际桌面结果待接收另立不可覆盖证据。先同SID私有全ZIP归档成功Profile树及Public输入日志，验证全部字节/目录集合，再仅删除成功Profile子树1fa5e23f；Public留待控制器结果核验后清，失败现场/broker40588/账户/Profile根/旧拒绝目标不动，不强杀或改权限。\n\n证据wl2-real-gui-success-cleanup-20261115-r1含前置假设、首原语raw1(Hashtable bytes/Measure-Object适配错误)原源日志、修后raw0正例完整ZIP往返及内容/额外空目录拒绝、2小场景7文件清单，最终所有执行函数AST摘要一致，准备raw0/解析0/源摘要同Git blob。当前是准备快照，未称已执行归档或删除；overallCleanupComplete=false，旧拒绝不重试。产品H实际GUI空broker组件已于2a53076e1回交，非官方Owner/Job生产验收，Start继续拒绝。共享纯route声明/Status同采样原5秒最小接口仍待集成；完整副作用/NativeLease/H-S-D生产及普通MSI/CAP-trust环境/历史清理未完，无官方日常配置/业务模型请求/W2/双SID/发布。\n")
files = []
for path in sorted(ROOT.rglob("*")):
    if path.is_file() and path.name != "payload-manifest.json":
        raw = path.read_bytes()
        files.append(dict(path=path.relative_to(ROOT).as_posix(), bytes=len(raw), sha256=sha(raw)))
with (ROOT / "payload-manifest.json").open("x", encoding="utf-8") as out:
    json.dump(dict(files=files, productionAcceptance=False), out, indent=2)
print(json.dumps(dict(sourceHead=source["head"], payloads=len(files), cleanupExecution="pending-desktop-receipt", productionAcceptance=False)))
