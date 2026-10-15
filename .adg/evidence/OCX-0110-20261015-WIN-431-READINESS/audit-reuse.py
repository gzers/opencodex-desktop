"""Recompute selected historical facts without turning old-source evidence into new acceptance."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parent
evidence = root.parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
sha = lambda data: hashlib.sha256(data).hexdigest()
inputs = []


def admitted(directory, name):
    anchor = evidence / directory
    manifest_path = anchor / "manifest.json"
    manifest = read(manifest_path)
    matches = [row for row in manifest["entries"] if row["path"] == name]
    assert len(matches) == 1
    path = anchor / name
    data = path.read_bytes()
    assert len(data) == matches[0]["bytes"] and sha(data) == matches[0]["sha256"]
    inputs.append({"path": path.relative_to(evidence).as_posix(), "bytes": len(data),
                   "sha256": sha(data), "originalManifestSha256": sha(manifest_path.read_bytes())})
    return read(path)


restart_directory = "OCX-0110-20261013-WIN-RESTART-DUE-NATIVE"
before = admitted(restart_directory, "before-update-schedule.json")
after = admitted(restart_directory, "after-update-schedule.json")
cleanup = admitted(restart_directory, "post-cleanup-schedule.json")
preflight = admitted(restart_directory, "startup-preflight.json")
startup = admitted(restart_directory, "new-session-startup.json")
native = admitted(restart_directory, "new-process-initial-native.json")
assert after == cleanup
assert sha((evidence / restart_directory / "before-update-schedule.json").read_bytes()) == preflight["scheduleHashBefore"]
assert preflight["globalDesktopCountBefore"] == 0 and preflight["oldOriginalOwnersAbsent"]
assert not preflight["manualQueryIssued"] and not preflight["originalDeadlineChanged"]
assert startup["sourceCommit"] == native["sourceCommit"] == preflight["candidateSource"]
assert startup["applicationPid"] == native["applicationPid"] == 10904
assert startup["applicationToken"]["elevated"] is False
assert native["result"]["preferences"]["automaticCheck"]
assert native["result"]["foreground"] and native["result"]["documentVisibility"] == "visible"
assert not native["result"]["automaticTriggerProven"] and not native["result"]["rawNetworkOrQueryCountsProven"]
results = []
for name in ("manager-stable", "panel-latest"):
    previous, current = before["entries"][name], after["entries"][name]
    assert previous["next_due"] - previous["last_success"] == 86400
    assert current["next_due"] - current["last_success"] == 86400
    assert current["last_success"] > previous["next_due"] and current["failures"] == 0
    results.append({
        "object": name, "originalLastSuccess": previous["last_success"],
        "originalDeadline": previous["next_due"], "observedNextSuccess": current["last_success"],
        "newDeadline": current["next_due"], "oldIntervalSeconds": 86400, "newIntervalSeconds": 86400,
        "elapsedSuccessToSuccessSeconds": current["last_success"] - previous["last_success"],
        "factReusable": "Historical same-root persisted deadline followed by later native success across restart",
        "directAutomaticTriggerOrRawRequestCountProven": False,
    })
assert before["entries"]["manager-beta"] == after["entries"]["manager-beta"]
assert after["entries"]["manager-beta"]["last_success"] is None
backoff = admitted("OCX-0110-20261012-WIN-REAL-TIMER-BACKOFF", "independent-recalculation.json")
assert backoff["actualFirstFireAfterAnchorMs"] >= backoff["requiredFloorMs"] == 1800000
assert backoff["actualFirstFireAfterAnchorMs"] == 1800001.5
assert backoff["recordedNoEarlierCallback"] and backoff["maximumObservedActiveTimers"] == 1
assert backoff["realMinimizeRestoreObserved"] and backoff["originalScheduleBytesRestoredAndRetained"]
beta = admitted("OCX-0110-20261012-WIN-REAL-BETA-CYCLE", "schedule-baseline.json")
assert beta["entries"]["manager-beta"]["last_success"] is None
assert beta["entries"]["manager-beta"]["failures"] == 1
files = (
    "apps/desktop/ui/src/features/updates/scheduler.ts",
    "apps/desktop/tauri/src/commands/update_schedule.rs",
    "apps/desktop/tauri/src/modules/update/schedule.rs",
)
sources = [backoff["sourceCommit"], startup["sourceCommit"], "43142bdbcc7a0b9f1325103febae3f44dce407ab"]
identities = {}
for source in sources:
    identities[source] = {path: subprocess.check_output(
        ["git", "rev-parse", source + ":" + path], cwd=code).decode().strip() for path in files}
old, restarted, current = (identities[source] for source in sources)
assert old == restarted
assert current[files[0]] == old[files[0]]
assert current[files[1]] != old[files[1]] and current[files[2]] != old[files[2]]
report = {
    "scope": "Selected historical evidence reuse audit, no application or timer execution",
    "currentSource": sources[-1], "inputFiles": inputs, "sourceBlobs": identities,
    "stableAndPanelFacts": results,
    "historicalThirtyMinuteErrorFloorMs": backoff["actualFirstFireAfterAnchorMs"],
    "uiSchedulerBlobUnchanged": True, "backendScheduleBlobsChanged": True,
    "historicalNativeFactsPreserved": True,
    "oldSourceExecutionRelabelledAsCurrentSource": False,
    "fullCurrentNativeErrorPathAccepted": False,
    "betaSixHourSuccessfulBaselineExistsInSelectedEvidence": False,
    "betaFailureIsNotSixHourCyclePass": True,
    "continuousSingleProcessTwentyFourHoursRequired": False,
    "restartAutomaticallyInvalidatesPersistedDeadlineEvidence": False,
    "rawRequestsAndQueryProcessCountsStillRequired": True,
    "remainingRealCycleScope": [
        "Current-source backend admission/error path and actual 30-minute lower bound",
        "Direct current-source automatic trigger and raw network/query-process count evidence",
        "Real beta six-hour window after a verified usable source and successful baseline",
        "Sleep/resume and cross-start behavior with original lifecycle gaps retained",
    ],
    "newApplicationStarted": False, "timerOrClockChanged": False, "fullVersionGatePassed": False,
}
target = root / "historical-reuse-analysis.json"
if "--build" in sys.argv:
    assert not target.exists()
    target.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read(target) == report
print(json.dumps({
    "historicalFilesHashVerified": len(inputs), "stablePanelIntervalsPreserved": 2,
    "historicalErrorFloorMs": backoff["actualFirstFireAfterAnchorMs"],
    "uiSchedulerUnchanged": True, "backendScheduleChanged": True,
    "fullCurrentGatePassed": False, "applicationStarted": False,
}))
