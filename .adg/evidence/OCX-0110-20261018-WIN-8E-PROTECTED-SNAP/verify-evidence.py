"""Verify the stored protected-install/Snap subset, including late exit and failed immediate cleanup."""
import hashlib
import json
from datetime import datetime, timedelta, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
read = lambda name: json.loads((ROOT / name).read_text(encoding="utf-8-sig"))
instant = lambda value: datetime.fromisoformat(value.replace("Z", "+00:00"))
sha = lambda value: hashlib.sha256(value).hexdigest()

manifest = read("manifest.json")
actual = {p.relative_to(ROOT).as_posix() for p in ROOT.rglob("*") if p.is_file() and p.name != "manifest.json"}
assert actual == {row["path"] for row in manifest["files"]}
for row in manifest["files"]:
    path = ROOT / row["path"]
    assert path.resolve().is_relative_to(ROOT.resolve()) and not path.is_symlink()
    data = path.read_bytes()
    assert len(data) == row["bytes"] and sha(data) == row["sha256"], row["path"]

result = read("result.json")
source = "8e69ff6db69ea13b8cb29feb9f7a1abacd202a7b"
assert result["source"] == source
ready = read("original/ready.json")
window = read("original/human-window.json")
assert window["origin"] == "human" and window["userText"] == "前台授权。"
assert instant(window["expiresAtUtc"]) - instant(window["authorizedAtUtc"]) == timedelta(minutes=30)
assert ready["source"] == source
assert ready["installerSha256"] == "d7016d62f5e14e2784b01cb5c5494e69fcdb711843f9c03849637173e9088acc"
installed = read("original/admin-install-result.redacted.json")
assert installed["harnessToken"]["elevated"] and installed["harnessToken"]["elevationType"] == 2
assert installed["exitCode"] == 0 and installed["result"] == "installed-exact-protected-payload"
assert "protectedAcl" not in installed and not installed["redaction"]["isOriginal"]
assert installed["installedExeSha256"] == ready["expectedInstalledExeSha256"] == result["installedExeSha256"]
launch = read("original/ordinary-launch.json")
assert launch["source"] == source and launch["pid"] == 552
assert launch["applicationToken"]["elevated"] is False and launch["applicationToken"]["elevationType"] == 3
assert not launch["sandboxUsed"] and not launch["cdpUsed"] and launch["retainedHandle"]
assert instant(window["authorizedAtUtc"]) < instant(launch["creationTimeUtc"]) < instant(window["expiresAtUtc"])
assert launch["executable"] == ready["protectedInstallTarget"] + "\\opencodex-desktop.exe"
first = read("original/ordinary-first-screen.json")
snap = read("original/ordinary-human-snap-readback.json")
for observation in (first, snap):
    assert observation["pid"] == launch["pid"] and observation["hwnd"] == 21367736
    assert instant(observation["creationTimeUtc"]) == instant(launch["creationTimeUtc"])
    assert observation["visible"] and not observation["maximized"] and not observation["minimized"]
    assert not observation["applicationMenuAttached"]
assert first["foreground"] and first["backdropRawValue"] == 2 and first["backdropHresult"] == 0
assert snap["outerRect"] == {"Left": -6, "Top": 0, "Right": 1605, "Bottom": 1150}
assert snap["clientRect"]["Right"] == 1598 and snap["clientRect"]["Bottom"] == 1114
assert not snap["foreground"] and snap["backdropRawValue"] == 1
assert read("original/protected-write-observation.json")["ordinaryWriteDenied"]
assert read("original/human-snap-feedback.json")["userText"] == "已贴靠"
feedback = read("human-feedback.json")
assert feedback["exitMessage"] == "已经退出" and feedback["trayMenuActionAcceptedAsHumanReport"]
assert not feedback["trayMenuClickIndependentlyRecorded"]
sky = read("original/ordinary-snap-final.sky.json")
assert sky["window"]["id"] == snap["hwnd"]
assert any(s["width"] == 1600 and s["height"] == 1144 for s in sky["screenshots"])
assert all("url" not in item for item in sky["screenshots"])
assert "preferences.json" in sky["accessibility"]["document_text"]
assert read("original/ordinary-tree-leaf-focus.sky.json")["window"]["id"] == snap["hwnd"]
exit_record = read("original/manual-exit-observed.json")
assert exit_record["pid"] == launch["pid"] and exit_record["exitCode"] == 0
assert exit_record["kernelTimes"]["retainedHandle"] and exit_record["kernelTimes"]["exited"]
assert exit_record["kernelTimes"]["creationFileTime"] == "134367254130101491"
exit_ticks = int(exit_record["kernelTimes"]["exitFileTime"])
exit_time = datetime(1601, 1, 1, tzinfo=timezone.utc) + timedelta(microseconds=exit_ticks // 10)
assert exit_time > instant(window["expiresAtUtc"])
assert exit_record["foregroundExpiredBeforeExit"] and not exit_record["forcedTermination"]
assert not result["normalExitWithinForegroundWindow"] and not result["plannedMinute25CloseMet"]

failed = read("original/admin-uninstall-result.json")
complete = read("original/admin-uninstall-final-readback.json")
assert failed["result"] == "failed" and failed["exitCode"] == 0 and not failed["mainExecutableAbsent"]
assert complete["originalAttempt"] == "admin-uninstall-result.json"
assert complete["result"] == "uninstalled-default-retention" and complete["mainExecutableAbsent"]
assert complete["delayedReadbackTargetAbsent"] and complete["hkcuUninstallAbsent"] and complete["hklmUninstallAbsent"]
assert complete["actualFinalUninstallerChildExitCode"] is None
assert not complete["uninstallRepeated"] and not complete["originalFailureEdited"]
before = read("original/pre-uninstall-data.json")
before_rows = {row["path"]: row for row in before["files"]}
assert len(complete["retainedDataFiles"]) == 4
for row in complete["retainedDataFiles"]:
    assert row == before_rows[row["path"]]
    data = ROOT / "pre-uninstall-data" / Path(row["path"])
    assert data.stat().st_size == row["bytes"] and sha(data.read_bytes()) == row["sha256"]
restored = read("original/final-restoration.json")
assert restored["originalProfilesRestored"] and restored["testDataRootPreserved"]
assert restored["nativeCount"] == 0 and not restored["originalRealDataDeleted"]
assert restored["originalPreferencesSha256"] == "07fcccd25b7e7dcc6ee6c752092e22659460929d770eef8c28f84bc4eb00bb0d"
assert restored["originalPointerSha256"] == "f62b7543fbb2b258dcaf15324dae91fea96c83d95c969065bfbf10428ac7012c"
assert restored["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
independent = read("original/independent-restoration-check.json")
baseline = read("original/original-directory-baseline.json")
assert {row["path"]: (row["fileId"], row["creationFileTime"]) for row in baseline} == {
    row["path"]: (row["fileId"], row["creationFileTime"]) for row in independent["originalAliasIdentities"]}
assert independent["nativeCount"] == independent["uninstallerCount"] == 0
assert independent["cleanupObserverAbsent"] and independent["testInstallTargetAbsent"]
log = (ROOT / "post-exit-app.original.log").read_text(encoding="utf-8-sig")
latest_start = log.rfind("startup version=0.1.10 commit=" + source)
assert latest_start >= 0 and "2026-10-17T15:43:33.452Z" in log[max(0, latest_start - 40):latest_start]
assert result["administratorProductStarted"] is False and not result["fullWindowsGatePassed"]
assert not result["taskClosed"] and not result["releaseAccepted"]
print(json.dumps({"status": "verified-protected-install-snap-subset", "files": len(actual),
                  "ordinaryProtectedFirstScreen": True, "humanSnapAndTrayExitWithKernelExit0": True,
                  "exitBeforeForegroundDeadline": False, "configurationRestored": True,
                  "nativeCountAtFinalCheck": 0, "fullWindowsGatePassed": False}))
