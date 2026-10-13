"""Verify actual restarted-candidate persisted due results, preserving disconnected UI limits."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "be7395acfce0ec1e36b83300b576db408f8c35bb"
image = "3a26098e813ddd56070a0092cbcb640473cb73e626aee45198c4b335ec2d8bd1"
original_source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
sha = lambda value: hashlib.sha256(value).hexdigest()
before = read("before-update-schedule.json")
after = read("after-update-schedule.json")
assert sha((root / "before-update-schedule.json").read_bytes()) == \
    "6e51fe41559c287c6036ef495456f1d1d5200710f5f925b3e9e7084621c3df20"
assert before["entries"]["manager-stable"]["next_due"] == 1791903339
assert before["entries"]["panel-latest"]["next_due"] == 1791903340
for target, expected_success in [("manager-stable", 1791903503), ("panel-latest", 1791903506)]:
    original, current = before["entries"][target], after["entries"][target]
    assert current["last_success"] == expected_success
    assert current["last_success"] > original["next_due"]
    assert current["last_success"] - original["last_success"] >= 86400
    assert current["failures"] == 0 and current["next_due"] - current["last_success"] == 86400
assert after["entries"]["manager-beta"] == before["entries"]["manager-beta"]
for name in ["new-process-initial-schedule.json", "cleanup-schedule.json", "post-cleanup-schedule.json"]:
    assert (root / name).read_bytes() == (root / "after-update-schedule.json").read_bytes()
for name in ["preferences.json", "data-root.json", "backup-cleanup-policy.json"]:
    assert (root / ("before-" + name)).read_bytes() == (root / ("after-" + name)).read_bytes()
preflight = read("startup-preflight.json")
assert preflight["globalDesktopCountBefore"] == 0 and preflight["oldOriginalOwnersAbsent"]
assert preflight["candidateExeSha256"] == image and preflight["scheduleHashBefore"] == sha((root / "before-update-schedule.json").read_bytes())
assert not preflight["manualQueryIssued"] and not preflight["originalDeadlineChanged"]
assert preflight["foregroundWindowEndsBeijing"] == "2026-10-13T23:10:00+08:00"
for filename in ["new-session-startup.json", "new-session-after-input-failure.json", "retained-session-after-cleanup.json"]:
    session = read(filename)
    assert session["sourceCommit"] == source and session["executableSha256"] == image
    assert session["applicationPid"] == 10904 and session["supervisorPid"] == 32492
    assert session["status"] == "running" and session["sourceCommitMatched"]
    assert session["applicationToken"]["creationTimeUtc"] == "2026-10-13T14:57:31.5757733Z"
    assert session["applicationToken"]["elevated"] is False and session["applicationToken"]["elevationType"] == 3
    assert session["harnessToken"]["elevated"] is False and session["harnessToken"]["elevationType"] == 3
old = read("old-current-session.json")
assert old["sourceCommit"] == original_source and old["applicationPid"] == 37276
def native(filename):
    result = read(filename)
    assert result["sourceCommit"] == source and result["applicationPid"] == 10904
    assert result["executableSha256"] == image
    return result["result"]
initial = native("new-process-initial-native.json")
assert initial["preferences"] == {"channel": "stable", "automaticCheck": True, "intervalSeconds": 86400}
assert initial["manager"]["lastCheckedAt"] == "2026-10-13T14:58:23.486670400+00:00"
assert initial["manager"]["error"] is None and initial["manager"]["currentVersion"] == "0.1.10"
assert initial["panelCache"]["version"] == "2.84.0" and initial["panelCache"]["tag"] == "latest"
assert initial["foreground"] and initial["documentVisibility"] == "visible"
assert initial["currentProcessElapsedMs"] < 86400000 and not initial["originalInMemoryTracePresent"]
for name in ["after-activation-failure-native.json", "before-cleanup-native.json"]:
    value = native(name)
    assert value["preferences"]["automaticCheck"] is True
    assert value["manager"] == initial["manager"] and value["panelCache"] == initial["panelCache"]
    assert not value["foreground"] and not value["automaticTriggerProven"]
post = native("post-cleanup-native.json")
assert post["preferences"] == {"channel": "stable", "automaticCheck": False, "intervalSeconds": 86400}
assert post["manager"] == initial["manager"] and post["panelCache"] == initial["panelCache"]
cleanup = native("native-update-preferences-cleanup.json")
assert cleanup["restored"] and cleanup["originalPair"] == cleanup["actualPair"] == {"channel": "stable", "automaticCheck": False}
assert not cleanup["explicitUpdateQueryIssued"] and not cleanup["scheduleDeadlineRewritten"]
assert read("cleanup-preferences.json")["updates"]["app_update_auto_check"] is False
modal = native("manager-open-focus.json")
assert modal["dialogOpen"] and modal["title"] == "桌面管理器更新"
assert modal["focus"]["insideDialog"] and modal["focus"]["isDialog"]
assert modal["progressElements"] == 0 and modal["notes"][0]["clientHeight"] == modal["notes"][0]["scrollHeight"]
assert next(button for button in modal["buttons"] if button["text"] == "安装管理器更新")["disabled"]
actions = read("native-ui-actions.json")
assert len(actions["actions"]) == 3
assert [value["action"] for value in actions["actions"]] == [
    "click settings", "click version upgrade tab", "open manager cached detail"]
assert actions["inputError"]["error"] == "failed to activate captured window"
assert "0x80070057" in actions["inputError"]["retryObservationError"]
assert not actions["manualUpdateCheckIssued"] and not actions["focusReturnRegressionAccepted"]
assert "15119" in (root / "query-session-after-input-failure.txt").read_text(encoding="utf-8-sig")
assert "Disc" in (root / "query-session-after-input-failure.txt").read_text(encoding="utf-8-sig")
metrics = read("native-normal-window.json")
assert metrics["sourceCommit"] == source and metrics["applicationPid"] == 10904
assert metrics["hwnd"] == 6685560 and metrics["dpi"] == 192
assert metrics["backdropAttribute"] == 1 and metrics["dwmResult"] == 0
assert metrics["visible"] and not metrics["minimized"] and not metrics["maximized"] and not metrics["foreground"]
life = read("retained-lifecycle-and-scoped-processes.json")
assert life["desktopCount"] == 1 and life["applicationPid"] == 10904
assert {item["pid"] for item in life["actualHandleReadbacks"]} == {10904, 32492}
assert all(item["actualLiveHandle"] for item in life["actualHandleReadbacks"])
assert life["actualApplicationToken"]["elevated"] is False and life["actualApplicationToken"]["elevationType"] == 3
assert life["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert "commit=" + source in (root / "sandbox-startup-app.log").read_text(encoding="utf-8-sig")
disposition = read("restart-native-disposition.json")
assert all(item["unchanged"] for item in disposition["scheduleSourceWitnesses"])
assert disposition["crossStartupPersistedNextSuccessObserved"] and disposition["originalUpdatePreferencePairRestored"]
assert disposition["appForegroundReadbackCanPublishActualActivityAndWakeScheduler"]
assert disposition["requestCauseNotInstrumented"] and disposition["oldContinuousObservationInterrupted"]
for key in ["explicitCheckOrInstallCommandIssued", "earliestAutomaticRequestStartProven",
            "rawNoDuplicateRequestOrProcessCountProven", "fullRealCycleMatrixAccepted",
            "successfulScheduleChangedByCleanup", "focusReturnToAllActualActionReached",
            "focusFixNativeRegressionAccepted", "physicalDpiMatrixAccepted", "nativeMicaAccepted",
            "adminTokenNativeGateAccepted", "installerExecuted", "fullNativeGateAccepted",
            "pairedPerformanceAccepted", "formalTaskReceipt", "mainIntegrated", "releaseAccepted"]:
    assert disposition[key] is False
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in {"manifest.json", "archive-verification.json"})
assert all(path.suffix.lower() not in {".exe", ".dll", ".msi", ".zip", ".pyc"} for path in files)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": sha(path.read_bytes())} for path in files]
def write(name, value):
    (root / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "filesVerified": len(entries), "sourceCommit": source,
    "scope": "actual ordinary new candidate with preserved old due state, observed persisted/cache success24h after prior success, native detail opening and disconnected focus attempt",
    "crossStartupPersistedSuccessAccepted": True, "continuousOldLifecycleAccepted": False,
    "automaticDueTriggerDirectlyInstrumented": False, "earliestRequestStartProven": False,
    "rawNoDuplicateRequestOrProcessCountsAccepted": False,
    "nativeFocusReturnRegressionAccepted": False, "physicalDpiMicaMatrixAccepted": False,
    "originalUpdatePreferencePairRestored": True, "retainedSuccessfulScheduleBytesPreserved": True,
    "betaCycleAccepted": False, "realFullCycleMatrixAccepted": False, "pairedPerformanceAccepted": False,
    "fullNativeGateAccepted": False, "formalTaskReceipt": False, "mainIntegrated": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries),
                  "crossStartupPersistedNextSuccess": True, "focusReturnNativeAccepted": False,
                  "realFullCycleMatrixAccepted": False, "fullNativeAccepted": False}))
