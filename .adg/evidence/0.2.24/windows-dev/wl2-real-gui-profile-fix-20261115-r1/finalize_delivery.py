"""Seal a terminal build/desktop snapshot; pending is never treated as pass."""
from pathlib import Path
from datetime import datetime
import hashlib
import json

ROOT = Path(__file__).resolve().parent
DOCS = ROOT.parents[4]
load = lambda p: json.loads(p.read_text(encoding="utf-8-sig"))
sha = lambda raw: hashlib.sha256(raw).hexdigest()
source = load(ROOT / "final-source.json")
artifacts = load(ROOT / "product01-artifacts.json")
prepared = load(ROOT / "prepared-location.json")
desktop = load(ROOT / "desktop-result.json") if (ROOT / "desktop-result.json").exists() else None
passed = desktop is not None and desktop["actualGUIComponentPassed"] is True
previous = load(ROOT.parent / "wl2-stop-restart-results-20261115-r1/final-status.json")
matrix = previous["matrix"]
for row in matrix:
    if row["id"] == "H":
        if passed:
            row.update(result="actual-GUI-empty-broker-component-pass", fact="New ordinary marenben desktop executes actual product main frontend/visible native HWND/page Finished; normal GUI exit, exact original Child-only forced GUI, fresh GUI authentication and four generations preserve the exact original empty broker creation handle. No official Owner/Job/H-S-D production proof; broker raw exit unavailable and stays null.")
        elif desktop is not None:
            row.update(result="actual-GUI-followup-failed/retained", fact="New fixed-input desktop observer failed; original same-SID export and every raw result retained. See desktop-result.json; no H acceptance.")
        else:
            row.update(result="profile-fix-built/awaiting-desktop", fact="Windows probe now uses original Token Profile while application home remains sandbox/home. Native child counterexample passes with spoofed USERPROFILE. New fixed exact artifacts/launcher prepared; no real desktop result yet. Original0c failure retained.")
status = dict(time=datetime.now().astimezone().isoformat(), sourceHead=source["head"], sourceTree=source["tree"],
    contract="TEAM14/IMP140/windows-security-coverage-r4", baselineMain=previous["baselineMain"], baselineDocs=previous["baselineDocs"],
    codeCommits=[previous["sourceHead"], source["head"]], artifacts=artifacts,
    runs=dict(library01=load(ROOT / "library01/result.json"), product01=load(ROOT / "product01/result.json")),
    testSummary=dict(passed=7, failed=0, ignored=4, oneIgnoredHelperExecutedByParent=True),
    firstError=load(ROOT / "prepare01-short-sha-rejected.json"), originalGuiFailure="../wl2-real-gui-run-20261115-r1/",
    matrix=matrix, preparedEntry=prepared["launchEntry"], desktopExecuted=True if desktop is not None else None,
    desktopResultReceived=desktop is not None,
    actualGUIObserved=passed, officialOwnerCreated=False, productionExecuted=False, startOpen=False, allAccepted=False,
    cleanup=dict(newAccounts=0, newProfiles=0, newPublicTaskRoots=1, oldFailureRootChanged=False,
        failedOriginalEmptyBrokerExitUnconfirmed=True, forensicOriginalsRetained=True, nativeMetadataCloned=False,
        deletionAttempted=False, historicalRejectedDeletionRetried=False, forcedBrokerCleanup=False,
        dailyPathConfigPolicyChanged=False, overallCleanupComplete=False),
    remaining=["Shared pure route declaration parser extraction: ../wl2-working-snapshots-20261115-r1/shared-minimum-request.json",
        "Shared single native Status sample deadline: ../wl2-stop-restart-results-20261115-r1/shared-status-deadline-request-r2.json",
        "Full official side effects/live native coordination/actual Closed and H-S-D production combinations",
        "Ordinary MSI acceptance, populated CAP/trust environment and all historical cleanup gaps"],
    W2=False, dualSID=False, published=False)
with (ROOT / "final-status.json").open("x", encoding="utf-8") as out:
    json.dump(status, out, ensure_ascii=False, indent=2)
artifact_rows = "\n".join(f"| {a['name']} | {a['bytes']} | {a['sha256']} |" for a in artifacts)
desktop_text = ("实际桌面组件通过，四个mode原生窗口/原broker/manager代次与原始退出结果见desktop-result.json；不是official Owner或生产组合验收。"
    if passed else "桌面运行失败并保留全部sameSID原始结果，见desktop-result.json，不计通过。"
    if desktop is not None else "新入口尚待marenben原普通桌面执行，未取得结果，不计通过。")
report = f"""# Windows真实GUI令牌Profile修复

源码{source['head']}，tree{source['tree']}，已推codex/024-windows-native。只改Windows gui_probe；原Stop/Restart/native原件不受此改动影响，复用前轮57项及所有既有不受影响证据，不跑W2/双SID/发布。

原0c真实桌面首错raw101/gui-normal.json缺席和GUI admission refused/rawNone保持在相邻wl2-real-gui-run-20261115-r1。根因是sandbox启用后platform::home_dir返回sandbox/home，探针却用它验证原Token Profile父目录。现消费既有Windows current_user_profile原Token/GetUserProfileDirectoryW接口，继续保留exact SID/session/ordinary token/任务名/sandbox边界/未安装runtime/原源/私管道/窗口/原5秒断言。逐项失败谓词写stderr，不跟随未准入路径写文件，不因诊断授许可。

定向MSVC custom-protocol库7过/0败/4忽略/raw0/220268ms；其中私有helper由父测试在独立子进程中以原sandbox环境、不同USERPROFILE执行，验证app home与原Token Profile独立且未创建路径。它不是桌面组件验收。最终clean MSVC custom-protocol debug全部bins raw0/101991ms/lib69 warnings；99份产品输入和55份测试Rust对最终工作源/Gitblob行尾等价，32份前端1162800字节不变，非release预算或冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
{artifact_rows}

本轮prepare01因调用者给strict full SHA检查传短SHA而raw1，前置第14行拒绝、尚未创建任务根，原事实保留；prepare02传完整SHA后raw0，未削断言。新Public任务根只在自有对象上设controller/SYSTEM full、marenben read及out Modify，三输入摘要/脚本解析通过，prepared脚本仅将原exact任务根替换为新的exact根，其余commands/guards/budgets逐字保持；原失败根和其pin图像未改。

{desktop_text}

完整B1/B2/B3/O1/O2/H/S-D/P/MSI表归final-status.json。公共Start仍拒绝；真实官方副作用/NativeLease依赖独立路由声明解析器最小提取，共享StatusSource还缺同一次采样fetch/fetch_light/recovery/debounce的原5秒上下文，两个具体最小请求沿前轮路径，不擅改共享trait/HMAC/其它平台。普通MSI、非空CAP/trust环境缺口不重跑旧前提失败。

本轮创建1处新Public任务根，无账户/Profile/日常PATH配置策略改。旧失败空broker40588仍未取得原始退出码，不填0、不凭PID强杀/收养；所有原失败现场和旧自动审批拒绝保持，不重试删除。无新删除尝试，overallCleanupComplete=false；原档/ZIP不是NTFS身份安全克隆。无官方进程/业务同步/模型请求/W2/双SID/发布，allAccepted/productionExecuted/startOpen=false。
"""
with (ROOT / "report.md").open("x", encoding="utf-8", newline="\n") as out:
    out.write(report)
handoff = DOCS / "docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md"
assert handoff.exists(), str(handoff)
with handoff.open("a", encoding="utf-8", newline="\n") as out:
    out.write(f"\n\n## 2026-11-15 r4真实GUI令牌Profile修复\n\n源码{source['head']}已推，tree{source['tree']}。只改Windows gui_probe，既有shared platform不改；使用原Token Profile验证任务根、sandbox home继续隔离，保留所有原来源/令牌/窗口/5秒断言并逐项输出准入失败。子进程反例开启sandbox且伪造USERPROFILE，实际原Profile仍独立；定向7过4忽略/raw0（helper由父测试执行），220268ms；最终clean custom-protocol MSVC debug全部bins0/101991ms/lib69warnings，99产品输入/55Rust对最终、前端32文件不变。原0c失败证据已于121f1739a推送；新原始source/log/result/制品/准备脚本/完整矩阵归wl2-real-gui-profile-fix-20261115-r1。\n\n{desktop_text} 新入口{prepared['launchEntry']}。\n\nDesktop77505024/c780330b94511602772e3740d82659412a1c3f5557ccf005458f4256ca3c1927；ocxd5398528/b80dd57317c91700aee4c77de1023f53b6da0f2c38e6760006db4b6abf8d3a67；ocx306176/430e193f62cc68f96e945e381758d9c761379831cbd2fe5c5990226b770712d8。prepare01短SHA调用被原严格检查先拒绝，prepare02完整SHA通过，未创建旧场景替换或放宽守门。新Public根1、无账户/Profile/日常策略变更、无删除/旧拒绝重试，旧失败broker40588退出仍未知，overallCleanupComplete=false。共享纯route声明提取和Status同采样原5秒接口最小请求未解决；完整官方副作用/NativeLease/H-S-D生产、普通MSI及CAP/trust环境/历史清理未完。Start拒绝，不官方/日常配置修改、无业务模型请求/W2/双SID/发布。\n")
inventory = []
for path in sorted(ROOT.rglob("*")):
    if path.is_file() and path.name != "payload-manifest.json":
        raw = path.read_bytes()
        inventory.append(dict(path=path.relative_to(ROOT).as_posix(), bytes=len(raw), sha256=sha(raw)))
with (ROOT / "payload-manifest.json").open("x", encoding="utf-8") as out:
    json.dump(dict(files=inventory, productionAcceptance=False), out, indent=2)
print(json.dumps(dict(head=source["head"], payloads=len(inventory), desktopExecuted=desktop is not None,
    actualGUIComponentPassed=passed, productionAcceptance=False)))
