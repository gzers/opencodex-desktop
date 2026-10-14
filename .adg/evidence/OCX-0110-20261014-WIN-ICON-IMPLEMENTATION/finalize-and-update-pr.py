"""Record final checks and update only the existing draft PR description."""
import hashlib
import json
import os
import subprocess
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
docs = root.parents[2]
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "bb3122d678a9c6fdbd2c8bd78fa9f3f22b955c8a"
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
observation_path = sorted(root.glob("ci-observation-*.json"))[-1]
observation = read(observation_path)
assert observation["source"] == source
runs = {item["run"]["id"]: item for item in observation["runs"]}
for identifier in [38499759600, 38499860360, 38499860139]:
    run = runs[identifier]["run"]
    assert run["head_sha"] == source and run["status"] == "completed" and run["conclusion"] == "success"
assets = read(root / "local-assets-red-green.json")
native = read(root / "native-candidate-result.json")
assert native["currentSession"]["sourceCommit"] == source and native["isolatedNativeCount"] == 1
assert not native["currentSession"]["applicationToken"]["elevated"]
archive = read(root / "candidate-archive-verification.json")
delivery = docs / "docs/04-项目资料/04-品牌与图标素材/03-平台交付/Windows"
manifest = read(delivery / "assets-20261014.json")
for item in manifest["artifacts"]:
    raw = (delivery / item["path"]).read_bytes()
    assert len(raw) == item["bytes"] and hashlib.sha256(raw).hexdigest() == item["sha256"]
    if item.get("codeName"):
        assert raw == (code / "apps/desktop/tauri/icons/windows" / item["codeName"]).read_bytes()
raw_exe = Path(native["currentSession"]["executable"]).read_bytes()
for tone in ["light", "dark"]:
    assert (code / "apps/desktop/tauri/icons/windows" / f"tray-{tone}-32.png").read_bytes() in raw_exe
for path in archive["selectedExtractedPaths"]:
    selected = Path(path)
    if selected.name in ["startup.json", "environment.json"]:
        parent = selected.parent.name
        destination = root / f"runner-{parent}-{selected.name}"
        assert not destination.exists()
        destination.write_bytes(selected.read_bytes())
summary = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": source,
    "branch": "feature/0.1.10-windows-transparent-icons", "base": "d3b83b3822bf10a511556d80dc851c2ccfb17cf5",
    "ci": {str(identifier): {"event": item["run"]["event"], "status": item["run"]["status"],
                            "conclusion": item["run"]["conclusion"],
                            "jobs": {job["name"]: job["conclusion"] for job in item["jobs"]["jobs"]}}
           for identifier, item in runs.items()},
    "icoAssetFramesPassed": 7, "trayAssetFramesPassed": 14,
    "oldBackingPlateFramesRejected": 7, "exeIcoFramesMatched": 7,
    "bothMonochromeTrayPngsEmbedded": True, "nativeApplicationPid": 39680,
    "nativeSupervisorPid": 44136, "nativeExecSession": 91561, "nativeHwnd": 2625640,
    "nativeExeSha256": native["currentSession"]["executableSha256"],
    "nativeTokenElevated": False, "nativeTokenType": 3,
    "sameSandboxProtectedBytesPreserved": True,
    "systemThemeSwitchPhysicallyAccepted": False, "taskbarTrayPixelMatrixAccepted": False,
    "userDailyInstallerExecuted": False, "userDailyInstallationReplaced": False,
    "fullVersionTaskClosed": False, "releaseAccepted": False,
    "assetManifestSha256": hashlib.sha256((delivery / "assets-20261014.json").read_bytes()).hexdigest(),
}
(root / "final-result.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
body = """Windows 应用和托盘此前复用带白色圆角底板的共享图标，主体只有75%。本次按用户确认的90%透明方案导出Windows专属ICO/PNG；16px保留一像素透明安全边距。macOS图标、默认PNG、ICNS和模板托盘保持原样。

Windows托盘使用独立单色亮/暗素材，按系统任务栏主题选择并沿既有系统主题事件刷新；应用内主题不强制改变托盘前景。回归检查主体、安全边距、单色和镂空，拒绝把白色底板计入有效图形。

验证：用户确认旧隔离实例的临时预览后已还原；本机7档ICO/14档托盘素材通过，旧7档ICO被新背景合同拒绝。bb3122d push CI38499759600、PR CI38499860360及候选38499860139均success；隔离runner的MSI/NSIS新装、首屏和卸载通过。两ZIP摘要/CRC、安装包及MSI只读提取EXE身份匹配，EXE七档内嵌图标像素与源码一致，两个单色托盘PNG均嵌入。

旧实例结束确认0后，新EXE在同一隔离根启动，PID39680/普通false/type3，来源日志匹配bb3122d；标题栏去底板实际观察通过，原日常EXE及隔离偏好/根/策略/调度摘要保持。实际任务栏/托盘像素与物理主题切换矩阵仍待完整验收，不以旧预览或runner代替。

本草稿基于更新说明分支，仅展示Windows图标改动。不自动合并或发布；原0.1.10全部门禁、性能failed与预算保持。证据：docs/governance-main 的 `.adg/evidence/OCX-0110-20261014-WIN-ICON-IMPLEMENTATION/`，平台素材：`docs/04-项目资料/04-品牌与图标素材/03-平台交付/Windows/assets-20261014.json`。
"""
(root / "pr-body-final.md").write_text(body, encoding="utf-8")
previous = {key: os.environ.get(key) for key in ["GIT_TERMINAL_PROMPT", "GCM_INTERACTIVE"]}
try:
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    os.environ["GCM_INTERACTIVE"] = "never"
    credential = subprocess.run(["git", "credential", "fill"], input="protocol=https\nhost=github.com\n\n",
                                text=True, capture_output=True, check=True, cwd=code, timeout=20)
    token = dict(line.split("=", 1) for line in credential.stdout.splitlines() if "=" in line)["password"]

    def api(method="GET", body=None):
        config = (f'url = "https://api.github.com/repos/gzers/opencodex-desktop/pulls/3"\n'
                  f'header = "Authorization: Bearer {token}"\n'
                  'header = "Accept: application/vnd.github+json"\n'
                  'header = "Content-Type: application/json"\n')
        args = [r"C:\Windows\System32\curl.exe", "--config", "-", "-4", "--http1.1",
                "--connect-timeout", "10", "--max-time", "30", "--fail", "--silent", "--show-error"]
        if body is not None:
            file = root / "pr-patch-body.json"
            file.write_text(json.dumps(body, ensure_ascii=False), encoding="utf-8")
            args += ["--request", method, "--data-binary", "@" + str(file)]
        response = subprocess.run(args, input=config.encode(), capture_output=True, timeout=35)
        assert response.returncode == 0, "PR request failed; read back before retry"
        return json.loads(response.stdout)

    before = api()
    assert before["draft"] and before["state"] == "open" and not before["merged"]
    assert before["head"]["sha"] == source and before["base"]["ref"] == "feature/0.1.10-release-details"
    (root / "pr-before-final-writeback.json").write_text(json.dumps(before, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    after = api("PATCH", {"body": body})
    assert after["draft"] and after["state"] == "open" and not after["merged"] and not after["auto_merge"]
    assert after["head"]["sha"] == source and after["base"]["ref"] == before["base"]["ref"]
    assert after["body"] == body
    (root / "pr-after-final-writeback.json").write_text(json.dumps(after, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
finally:
    for key, value in previous.items():
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = value
print(json.dumps({"ciAndCandidate": "success", "nativeSourceMatched": True,
                  "embeddedIcoFramesMatched": 7, "draftPrUpdated": 3, "releaseAccepted": False}))
