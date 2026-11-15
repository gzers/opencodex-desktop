from pathlib import Path
from datetime import datetime
import hashlib
import json

ROOT = Path(__file__).resolve().parent
DOCS = ROOT.parents[4]
load = lambda p: json.loads(p.read_text(encoding="utf-8-sig"))
sha = lambda raw: hashlib.sha256(raw).hexdigest()
received = load(ROOT / "receive-result.json")
receipt = load(ROOT / "received/archive-cleanup-result.json")
observed = load(ROOT / "path-observations02.json")
public = load(ROOT / "public-current-readback.json")
previous = load(ROOT.parent / "wl2-real-gui-success-cleanup-20261115-r1/final-status.json")
assert received["successfulTaskSameSidCleanupPassed"] is True
assert observed[0]["category"] == "ObjectNotFound"
assert observed[1]["category"] == "PermissionDenied"
assert public["expectedFiles"] == len(public["files"]) == 20
assert sum(r["readable"] for r in public["files"]) == 12
assert sum(not r["readable"] for r in public["files"]) == 8
assert not public["allCurrentBytesVerified"] and not public["deletionAttempted"]
status = dict(time=datetime.now().astimezone().isoformat(), sourceHead=received["sourceHead"],
    productSourceHead=received["productSourceHead"], artifacts=previous["artifacts"],
    contract=previous["contract"], baselineMain=previous["baselineMain"], baselineDocs=previous["baselineDocs"],
    matrix=previous["matrix"], newProductBuild=False, testsRerun=False,
    sameSidCleanupComplete=True, sameSidCleanupProcessRawExit=None,
    cleanup=received, controllerPublicVerification=dict(readable=12, unreadable=8,
        complete=False, deleted=False, aclChanged=False),
    controllerObservations=observed, invalidObservationExcluded="path-observations.json; see path-observation01-invalid.json",
    firstError=dict(stage="controller-public-readback", kind="PermissionDenied", files=8,
        effect="Public source retained, no cleanup attempt or ACL change"),
    originalCleanupError=None, actualGUIComponentPassed=True, officialOwnerCreated=False,
    productionExecuted=False, startOpen=False, allAccepted=False,
    remaining=previous["remaining"], W2=False, dualSID=False, published=False)
with (ROOT / "final-status.json").open("x", encoding="utf-8") as out:
    json.dump(status, out, ensure_ascii=False, indent=2)
report = """# 成功GUI任务归档清理回执

2026-11-15 22:48:32至22:48:47，marenben原普通SID1003/session7执行已准备脚本。源码0549a660ee3f77ea243e798ec55b01f52545e492；产品仍6e312f02270f247d2f440ec369385e523a4a8043，未重编译/重跑GUI或其它已过定向项。执行脚本/plan逐字等于已提交准备件，intent单次且回执complete=true/error缺席。入口进程raw exit未由外部控制器记录，保持null，不由complete推成0。

| 对象 | 原文件/目录 | 原字节 | 结果 |
| --- | --- | --- | --- |
| 成功Profile子树1fa5e23f | 685 / 291 | 119851895 | 同SID完整ZIP/逐文件SHA/目录集合核验后删除；控制器准确绝对路径Get-Item返回ObjectNotFound |
| 成功Public输入日志 | 20 / 2 | 178669957 | 同SID完整ZIP核验，源保留；控制器12项可读且匹配，8项PermissionDenied，未删除 |

同SID私有归档根C:/Users/marenben/ocx024-gui-archive-36ed4617-000c-4efa-9975-49f9e1b29654。successful-profile-tree.zip SHA256=5cd6ec4362e8d057b3bf01f0ae0e48956e303f15bff2705a45ab35dd68633200；successful-public-inputs-logs.zip SHA256=5da2e4d3838b922d1ad601eed039f469abac93f293f41b61f0a165e0230383a7；private manifest SHA256=54b0e65d9a0427924add27d0d2fa88310ad69ce43805947a1ce39678d2bf9276。ZIP及manifest在原SID完成内容核验，控制器读取该私有目录PermissionDenied，未独立复读ZIP、未改ACL；明确不是NTFS身份或安全元数据克隆。

控制器首个路径观察调用因JavaScript反斜杠转义损坏为C:Users...，原错误输出保留且明确无效，不用它证明缺席；随后用完整正斜杠绝对路径重新只读观察，成功子树ObjectNotFound、私有归档PermissionDenied、三Public任务根返回Directory。未执行额外删除或修权限。

本轮清理本身未报错；控制器Public复验的首个缺口为8份私有JSON无读取权限。Public源仍保留，不能将事先归档摘要或不可读当作当前可删证明。当前无新自动审批拒绝，也未重试任何历史拒绝。账户/Profile根/日常PATH配置策略未改，第一次0c失败树和旧broker40588未处理、退出状态仍未知；整体清理未完成。已删除仅准确成功Profile子树，未删除整个marenben Profile。

H真实GUI空broker组件已过，但不签official Owner/Job或真实S-D生产。完整B1/B2/B3/O1/O2/H/S-D/P/MSI矩阵及制品SHA沿final-status.json。共享纯route声明解析器及Status同采样原5秒接口仍需集成；完整官方副作用/NativeLease/原包H-S-D生产、普通MSI、非空CAP/trust和历史清理仍未完。公共Start拒绝，未知/raw/原预算不变，无官方修改/业务模型请求/W2/双SID/发布。
"""
with (ROOT / "report.md").open("x", encoding="utf-8") as out:
    out.write(report)
handoff = DOCS / "docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md"
with handoff.open("a", encoding="utf-8", newline="\n") as out:
    out.write("\n\n## 2026-11-15 成功GUI任务同SID归档清理接收\n\n执行源码0549a660ee3f77ea243e798ec55b01f52545e492，产品源仍6e312f022，未重编译/重复测试。marenben SID1003/session7回执complete=true，脚本和plan逐字等于准备源；成功Profile子树1fa5e23f的685文件/291目录/119851895字节已在同SID私有ZIP核验后删除，控制器准确路径ObjectNotFound。Public成功输入日志20文件/2目录/178669957字节也已同SIDZIP核验但源保留；控制器12可读匹配/8PermissionDenied，未进一步删除/修ACL。原始回执、执行输入、路径事实和完整矩阵归wl2-real-gui-cleanup-receive-20261115-r1。\n\n私有归档根ocx024-gui-archive-36ed4617-000c-4efa-9975-49f9e1b29654，Profile ZIP SHA5cd6ec4362e8d057b3bf01f0ae0e48956e303f15bff2705a45ab35dd68633200，Public ZIP SHA5da2e4d3838b922d1ad601eed039f469abac93f293f41b61f0a165e0230383a7，manifest SHA54b0e65d9a0427924add27d0d2fa88310ad69ce43805947a1ce39678d2bf9276；控制器不能读该私有归档，核验依据为原SID脚本回执，不冒独立复读或NTFS克隆。控制器首次路径转义损坏观察已明确排除并保留，修正绝对路径只读结果单列。cleanup入口rawexit未采集仍null，不补0。\n\n清理本身无error；本轮控制器首个实质缺口为Public8文件读拒绝，整体清理false。旧失败broker40588/失败树、账户/Profile根、历史自动审批拒绝均保留未重试。共享route纯解析器/Status同采样原5秒接口最小请求及完整副作用/NativeLease/H-S-D生产、普通MSI/CAP-trust环境仍待完成。公共Start拒绝，无官方/日常策略修改、业务模型请求/W2/双SID/发布。\n")
files=[]
for path in sorted(ROOT.rglob("*")):
    if path.is_file() and path.name != "payload-manifest.json":
        raw=path.read_bytes()
        files.append(dict(path=path.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
with (ROOT / "payload-manifest.json").open("x",encoding="utf-8") as out:
    json.dump(dict(files=files,productionAcceptance=False),out,indent=2)
print(json.dumps(dict(payloads=len(files),sameSidCleanupPassed=True,publicRetained=True,overallCleanupComplete=False)))
