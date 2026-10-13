"""Morning target recovery and incomplete UI disposition; no full UI/cycle pass."""
import hashlib
import json
import sys
from datetime import datetime
from pathlib import Path

root = Path(__file__).resolve().parent
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
image = "f5e4a2eef0ffc364a2a57b47b1463015faf3433a15ff021d544d6e161004f00c"
def read(name):
    return json.loads((root / name).read_text(encoding="utf-8-sig"))
def native(name):
    value = read(name)
    assert value["sourceCommit"] == source and value["applicationPid"] == 37276
    assert value["executableSha256"] == image and value["screenshot"] is None
    return value["result"]
before = read("scene-before.json")
after = read("scene-after.json")
assert before["sourceCommit"] == after["sourceCommit"] == source
assert before["applicationPid"] == after["applicationPid"] == 37276
assert before["hashes"] == after["hashes"]
assert after["allOriginalBytesPreserved"]
assert before["hashes"]["preferences.json"] == "674ad619b46fc27a273eb37da775660e3b44263fe3485e4ade98287f5fbda8fa"
assert before["hashes"]["update-schedule.json"] == "6e51fe41559c287c6036ef495456f1d1d5200710f5f925b3e9e7084621c3df20"
assert not before["installerOrPackageScriptsAuthorized"]
assert not after["foregroundInputSinceUserPreference"] and not after["updateDetailsUiAccepted"] and not after["cycleRestarted"]
initial, background = native("00-before-input.json"), native("01-background-readback.json")
assert initial["route"] == background["route"] == "#settings?section=backup"
assert initial["foreground"] is True and background["foreground"] is False
assert initial["dpr"] == background["dpr"] == 2
assert initial["manager"] == background["manager"] and initial["panelCache"] == background["panelCache"]
assert initial["manager"]["lastCheckedAt"] == "2026-10-12T14:55:39.240251800+00:00"
assert initial["uiPreferences"] == background["uiPreferences"] == {
    "channel": "stable", "automaticCheck": True, "theme": "system", "error": False}
assert initial["busy"] == background["busy"] == {"manager": False, "panel": False}
for dialog in initial["dialogs"] + background["dialogs"]:
    assert "title" not in dialog and dialog["rectangle"]["width"] == dialog["rectangle"]["height"] == 0
    assert not dialog["activeFocusInside"]
    assert dialog["header"] is None and dialog["body"] is None and dialog["footer"] is None
    assert dialog["notes"] == dialog["progress"] == dialog["versions"] == dialog["results"] == []
metrics = read("00-native-window.json")
assert metrics["sourceCommit"] == source and metrics["applicationPid"] == 37276
assert metrics["hwnd"] == 1051698 and metrics["dpi"] == 192 and metrics["foreground"] and metrics["backdropAttribute"] == 2
assert metrics["visible"] and not metrics["minimized"] and not metrics["maximized"]
metadata = read("00-original-window.metadata.json")
assert metadata["window"]["id"] == 1051698
assert metadata["window"]["app"].endswith("candidateeae\\opencodex-desktop.exe")
assert metadata["width"] == 1182 and metadata["height"] == 790 and metadata["file"] == "00-original-window.jpg"
assert (root / metadata["file"]).read_bytes().startswith(b"\xff\xd8\xff")
ax = (root / "00-original-window.ax.txt").read_text(encoding="utf-8")
assert "#settings?section=backup" in ax and "备份文件与目录" in ax
assert "版本升级" in ax and "标题栏" in ax
disposition = read("morning-disposition.json")
assert disposition["sourceCommit"] == source and disposition["applicationPid"] == 37276 and disposition["windowId"] == 1051698
assert len(disposition["actualUiSequenceBeforeForegroundPreference"]) == 8
assert any("outside window bounds" in step for step in disposition["actualUiSequenceBeforeForegroundPreference"])
assert not any(disposition[key] for key in [
    "versionTabClickAccepted", "cachedUpdateDetailsOpened", "updateChecksInstallOrRestartInvoked",
    "nonTargetFrameArchived", "outOfViewportAxClickIsProductFailure",
    "currentEntireBackupInventoryAcceptedFromOldUi", "afterUserForegroundPreferenceFurtherInput",
    "fullNativeGateAccepted", "treeMatrixAccepted", "updateDetailsAccepted",
    "pairedPerformanceAccepted", "realCycleAccepted", "formalTaskReceipt", "releaseAccepted"])
assert disposition["uiBackupInventoryNotRefreshed"] and disposition["backgroundReadbackRequiresNoForeground"]
plan = read("foreground-cooperation-plan.json")
assert plan["timezone"] == "Asia/Shanghai" and plan["currentDate"] == "2026-10-13"
assert plan["foregroundOnlyAfterHumanStartSignal"]
assert not plan["clockOrAutomaticGoalContinuationIsStartSignal"] and not plan["automaticFocusTaking"]
assert plan["originalDueBeijing"] == ["2026-10-13T22:55:39+08:00", "2026-10-13T22:55:40+08:00"]
assert plan["unchangedMonitorMaximumHours"] == 26
assert plan["originalMonitorLatestEndBeijing"] == "2026-10-14T00:55:52.6406386+08:00"
assert not any(plan[key] for key in ["monitorExtendedOrRestarted", "scheduledAutomationCreated",
                                    "scriptsAuthorizedBySchedule", "formalTaskRecovered", "releaseAccepted"])
assert plan["originalBudgetsAndFullGateScopePreserved"]
assert len(plan["windows"]) == 4
assert plan["windows"][0]["start"] == "2026-10-13T12:30:00+08:00"
assert plan["windows"][1]["start"] == "2026-10-13T22:50:00+08:00"
for window in plan["windows"]:
    assert datetime.fromisoformat(window["start"]) < datetime.fromisoformat(window["end"])
prefix = (root / "real-cycle-observations-prefix.ndjson").read_bytes()
assert prefix.endswith(b"\n")
values = [json.loads(line) for line in prefix.splitlines() if line.strip()]
summary = read("cycle-prefix-summary.json")
assert summary["sourceCommit"] == source and summary["applicationPid"] == 37276
assert summary["monitorPid"] == 3920 and summary["monitorExecSession"] == 46484
assert summary["completeObservations"] == len(values) >= 114
assert summary["rawPrefixSha256"] == hashlib.sha256(prefix).hexdigest()
baseline = read("schedule-before.json")
assert all(value["schedule"] == baseline and value["originalLifecycleLive"] for value in values)
for value in values:
    assert value["native"]["sourceCommit"] == source and value["native"]["applicationPid"] == 37276
    assert value["native"]["executableSha256"] == image
    assert value["native"]["result"]["preferences"] == {"channel": "stable", "automaticCheck": True}
assert all(b["elapsedMonitorMs"] > a["elapsedMonitorMs"] for a, b in zip(values, values[1:]))
assert all(b["native"]["result"]["atMs"] > a["native"]["result"]["atMs"] for a, b in zip(values, values[1:]))
assert summary["observedMonotonicSpanMs"] == values[-1]["elapsedMonitorMs"] - values[0]["elapsedMonitorMs"]
assert summary["appWebviewSpanMs"] == values[-1]["native"]["result"]["atMs"] - values[0]["native"]["result"]["atMs"]
assert summary["observedMonotonicSpanMs"] > 9 * 3_600_000
assert summary["foregroundObservations"] == sum(value["native"]["result"]["foreground"] is True for value in values)
assert not summary["rawNetworkOrNativeQueryCountsInstrumented"] and not summary["realCycleAccepted"]
monitor = read("monitor-state-at-prefix.json")
assert monitor["sourceCommit"] == source and monitor["applicationPid"] == 37276 and monitor["monitorPid"] == 3920
assert monitor["managerDeadlineEpoch"] == 1791903339 and monitor["panelDeadlineEpoch"] == 1791903340
assert monitor["maximumHours"] == 26 and not monitor["fullCycleAccepted"]
assert not monitor["managerNextSuccessfulQueryObserved"] and not monitor["panelNextSuccessfulQueryObserved"]
life = read("retained-lifecycle.json")
assert life["sourceCommit"] == source and life["applicationPid"] == 37276 and life["desktopApplicationCount"] == 1
assert life["executableSha256"] == image
assert life["dailyExecutableSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert {item["processId"] for item in life["processReadbacks"]} == {37276, 24300, 3920}
assert all(item["liveHandleVerified"] and not item["hasExited"] for item in life["processReadbacks"])
assert life["applicationToken"]["elevated"] is False and life["applicationToken"]["elevationType"] == 3
assert not any(life[key] for key in ["clocksOrTimersChanged", "manualUpdateCheckOrInstallInvoked",
                                    "pairedPerformanceAccepted", "realCyclesAccepted", "fullNativeGateAccepted", "releaseAccepted"])
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in {"manifest.json", "archive-verification.json"})
assert all(path.suffix.lower() not in {".exe", ".dll", ".msi", ".zip", ".pending"} for path in files)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
def write(name, value):
    (root / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "sourceCommit": source, "filesVerified": len(entries),
    "scope": "morning original target-window recovery, native DPI192/Mica, preserved failed out-of-viewport UI action, background readback and completed real-cycle prefix; not update-details acceptance",
    "completedCyclePrefixObservations": len(values), "prefixElapsedHours": summary["observedMonotonicSpanMs"] / 3_600_000,
    "afterPreferenceForegroundInputs": False, "cachedUpdateDetailsAccepted": False,
    "treeOrDpiMatrixAccepted": False, "rawNetworkOrQueryCountsAccepted": False,
    "pairedPerformanceAccepted": False, "realCyclesAccepted": False,
    "fullNativeGateAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries), "cyclePrefixObservations": len(values),
                  "updateDetailsAccepted": False, "afterPreferenceForegroundInputs": False}))
