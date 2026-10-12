"""Verify the intentional old-source interruption and sequential exact-candidate launch."""
import hashlib
import json
import sys
from datetime import datetime
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
old_source = "afff490a430a81f8c233d44928aa3681a23e91e0"
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
exe_hash = "f5e4a2eef0ffc364a2a57b47b1463015faf3433a15ff021d544d6e161004f00c"
original_preferences = "8498f7cf8984c74a555202027800027fa7fd1366e0e5ce51c455aaf3ba1fdb7a"
instant = lambda value: datetime.fromisoformat(value.replace("Z", "+00:00"))
decision = read("transition-decision.json")
assert decision["oldSource"] == old_source and decision["newSource"] == source
assert decision["monitorInitiallyLiveAndVerified"] and not decision["observationTimeoutWasReason"]
assert decision["oldCycleResult"] == "not_completed; no24h acceptance"
assert not decision["fullGateAccepted"] and not decision["scriptApprovalGranted"]
monitor = read("old-terminal-monitor-state.json")
assert monitor["status"] == "explicit-monitor-stop-requested" and monitor["sourceCommit"] == old_source
assert monitor["originalUpdatePreferencesRestored"] and monitor["observations"] == 56
assert monitor["failedObservations"] == 0 and monitor["elapsedMonitorMs"] < 86_400_000
assert not any(monitor[key] for key in [
    "fullCycleAccepted", "betaCycleAccepted", "performanceAccepted",
    "clocksChanged", "timersChanged", "businessResponsesInjected",
])
cleanup = read("old-terminal-native-cleanup.json")
assert cleanup["sourceCommit"] == old_source and cleanup["applicationPid"] == 38924
assert cleanup["result"]["originalUpdatePreferencesRestored"]
assert cleanup["result"]["channel"] == "stable" and cleanup["result"]["automaticCheck"] is False
assert read("old-monitor-terminal-verification.json")["monitorSessionExitCode"] == 0
assert read("old-monitor-terminal-verification.json")["activePreferencesSha256"] == original_preferences
samples = [json.loads(line) for line in
           (root / "old-terminal-observations.ndjson").read_text(encoding="utf-8-sig").splitlines() if line.strip()]
assert len(samples) == monitor["observations"]
old_schedule = read("old-schedule-before-stop.json")
for sample in samples:
    assert sample["native"]["sourceCommit"] == old_source and sample["native"]["applicationPid"] == 38924
    assert sample["schedule"] == old_schedule
    assert sample["originalLifecycleLive"]
old_app = read("old-application-terminal-session.json")
assert old_app["sourceCommit"] == old_source and old_app["finishedAtUtc"]
zero = read("zero-applications-before-new-source.json")
assert zero["allDesktopApplicationsCount"] == 0 and zero["oldApplicationAbsent"] and zero["oldSupervisorAbsent"]
assert zero["newApplicationNotYetStarted"] and not zero["normalExitAccepted"]
ordinary = read("new-current-source-session.json")
admin = read("new-administrator-session.json")
final = read("final-ordinary-retained-session.json")
for session, pid, elevated, elevation_type in [
    (ordinary, 45244, False, 3), (admin, 40132, True, 2), (final, 37276, False, 3),
]:
    assert session["status"] == "running" and session["sourceCommit"] == source
    assert session["executableSha256"] == exe_hash and session["applicationPid"] == pid
    assert session["sourceCommitMatched"] and session["startupLogWrittenDuringThisProcess"]
    assert session["applicationToken"]["processId"] == pid
    assert session["applicationToken"]["elevated"] is elevated
    assert session["applicationToken"]["elevationType"] == elevation_type
    assert not session["installerExecuted"]
assert instant(monitor["finishedAtUtc"]) <= instant(old_app["finishedAtUtc"]) < instant(ordinary["startedAtUtc"])
ordinary_terminal = read("new-ordinary-terminal-before-admin.json")
admin_terminal = read("new-administrator-terminal-before-ordinary.json")
assert ordinary_terminal["applicationPid"] == 45244 and ordinary_terminal["finishedAtUtc"]
assert instant(ordinary_terminal["finishedAtUtc"]) < instant(admin["startedAtUtc"])
assert admin_terminal["applicationPid"] == 40132 and admin_terminal["finishedAtUtc"]
assert read("zero-applications-before-final-ordinary.json")["applicationCount"] == 0
assert instant(admin_terminal["finishedAtUtc"]) < instant(final["startedAtUtc"])
independent = read("new-administrator-independent-token.json")
assert independent["applicationCount"] == 1 and not independent["uiInputPerformed"]
assert independent["applicationToken"]["processId"] == 40132
assert independent["applicationToken"]["elevated"] and independent["applicationToken"]["elevationType"] == 2
assert independent["applicationToken"]["creationTimeUtc"] == admin["applicationToken"]["creationTimeUtc"]
for name, pid, hwnd in [
    ("new-native-window-metrics.json", 45244, 854126),
    ("new-administrator-native-window.json", 40132, 985452),
]:
    window = read(name)
    assert window["sourceCommit"] == source and window["applicationPid"] == pid and window["hwnd"] == hwnd
    assert window["visible"] and window["foreground"] and window["dpi"] == 192
    assert window["backdropAttribute"] == 2 and window["dwmResult"] == 0
for name, pid in [
    ("new-native-preference-readback.json", 45244),
    ("new-administrator-after-activation.json", 40132),
    ("final-returned-ordinary-readback.json", 37276),
]:
    value = read(name)
    assert value["sourceCommit"] == source and value["applicationPid"] == pid
    result = value["result"]
    assert result["preferenceReadbackMatches"] and not result["storeError"]
    assert result["theme"] == "system" and result["themeNeedsImport"] is False
    assert result["automaticCheck"] is False and result["channel"] == "stable"
    assert result["originalPin"]["pinned"] and result["dpr"] == 2
    notifications = {item["id"]: item["resolved"] for item in result["notifications"]}
    assert [status for name, status in notifications.items()
            if name.endswith("5a53223f-141f-4694-81e0-138255efddff")] == [True]
    assert [status for name, status in notifications.items()
            if name.endswith("1695188a-60bd-44ad-829d-ca2ea77499fd")] == [False]
files = read("new-source-identity-and-filesystem.json")
assert files["readbackDidNotWritePreferences"] and files["matchesOriginalPreferences"]
assert files["preferencesBeforeReadbackSha256"] == files["preferencesAfterReadbackSha256"] == original_preferences
current = read("final-transition-current-state.json")
assert current["sourceCommit"] == source and current["finalApplicationPid"] == 37276
assert current["finalSupervisorPid"] == 24300 and current["applicationExecSession"] == 80657
assert current["applicationCount"] == 1 and current["liveNewApplicationAndMonitorHandlesVerified"]
assert current["newCycleMonitorPid"] == 3920 and current["newCycleMonitorExecSession"] == 46484
assert current["newCycleOnlyUpdatePreferenceChange"] == {"channel": "stable", "automaticCheck": True}
assert current["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert not any(current[name] for name in [
    "old24hCyclePassed", "new24hCyclePassed", "newInstallerExecuted",
    "realDataMigrated", "scriptsExecuted", "scriptSecondConfirmationGranted",
    "performanceAccepted", "fullGateAccepted", "releaseAccepted",
])
for name in ["new-native-first-screen.jpg", "new-administrator-native-first-screen.jpg"]:
    assert (root / name).read_bytes().startswith(b"\xff\xd8\xff")
excluded = {"manifest.json", "archive-verification.json"}
paths = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in excluded)
assert all(path.suffix.lower() not in {".exe", ".msi", ".cab", ".zip", ".dll"} for path in paths)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in paths]
def write(name, value):
    (root / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed",
    "scope": "intentional interrupted old-source cycle, no-overlap final desktop source/mode transitions, exact native token and DPI192 foreground-Mica readback",
    "oldRealCycleAccepted": False, "newRealCycleAccepted": False,
    "finalDesktopActualTokens": "ordinary false/type3, administrator true/type2",
    "administratorFullUiMatrixAccepted": False, "otherPhysicalDpiAccepted": False,
    "localInstallationAccepted": False, "pairedPerformanceAccepted": False,
    "fullNativeGateAccepted": False, "formalTaskReceipt": False,
    "releaseAccepted": False, "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "oldCycle": "intentionally interrupted, not accepted",
                  "finalDesktopTokens": "two actual modes", "files": len(entries)}))
