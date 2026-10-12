"""Accept only the observed native timer subset after real elapsed time and cleanup."""

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent
SOURCE = "afff490a430a81f8c233d44928aa3681a23e91e0"


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def write(name, value):
    (ROOT / name).write_bytes(
        (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    )


monitor = read("monitor-state.json")
assert monitor["sourceCommit"] == SOURCE
assert monitor["status"] == "passed-recorded-real-timer-floor", monitor["status"]
assert monitor["actualFirstFireAfterAnchorMs"] >= 1_800_000
assert monitor["recorderVerifiedInstalled"] and monitor["cleanupSucceeded"]
assert monitor["actualAutoCheckAfterCleanup"] is False
assert monitor["scheduleRestored"]
assert not any(monitor[key] for key in [
    "timerDelaysChanged", "systemClockChanged", "nativeBridgeChanged",
    "responseInjected", "scriptsExecuted",
])
assert monitor["originalScheduleSha256"] == monitor["finalScheduleSha256"]
install = read("recorder-installation.json")["result"]
assert install["recorderInstalled"] and install["nativeBridgeUnchanged"]
assert install["clockUnchanged"] and install["originalAutoCheck"] is False
attribution = read("prior-floor-source-attribution.json")["result"]
assert attribution["timerBridgeStillInstalled"] and attribution["nativeInvokeUnchanged"]
assert len(attribution["matches"]) == 1
assert attribution["matches"][0]["errorWithin3500Characters"] >= 0
observations = [read(path.name) for path in sorted(ROOT.glob("observation-*.json"))]
assert observations
assert all(value["sourceCommit"] == SOURCE and value["applicationPid"] == monitor["applicationPid"]
           for value in observations)
associated = [value["result"] for value in observations if value["result"]["anchor"]]
assert associated
anchor = associated[0]["anchor"]["scheduledAtMs"]
assert associated[0]["anchor"]["delayMs"] == 1_800_000
for value in associated:
    assert value["activeTimerCount"] <= 1
    for record in value["fired"]:
        assert record["firedAtMs"] - anchor >= 1_800_000
wakes = read("route-online-wakes.json")["result"]
assert wakes["sourceEvents"] == 200 and wakes["firedCount"] == 0
assert wakes["activeTimerCount"] <= 1
hidden = read("actual-hidden-observation.json")["result"]
hidden_native = read("actual-hidden-native.json")
assert hidden_native["applicationPid"] == monitor["applicationPid"]
assert hidden_native["hwnd"] == 852848 and hidden_native["minimized"]
assert hidden["visibility"] == "hidden" and hidden["activeTimerCount"] == 0
assert hidden["fired"] == []
restored = read("actual-restored-observation.json")["result"]
assert restored["visibility"] == "visible" and restored["foreground"]
assert restored["activeTimerCount"] == 1 and restored["fired"] == []
assert restored["anchor"]["scheduledAtMs"] == anchor
cleanup = read("native-cleanup.json")["result"]
assert cleanup["cleaned"] and cleanup["originalTimersRestored"]
assert cleanup["nativeBridgeUnchanged"] and cleanup["actualAutoCheck"] is False
post_cleanup = read("post-cleanup-native-verification.json")
assert post_cleanup["sourceCommit"] == SOURCE
assert post_cleanup["applicationPid"] == monitor["applicationPid"]
assert post_cleanup["result"]["traceStatus"] == "finished"
assert all(post_cleanup["result"][key] for key in [
    "standardSetTimeoutRestored", "standardClearTimeoutRestored", "nativeInvokeUnchanged",
])
assert post_cleanup["result"]["autoCheck"] is False
assert post_cleanup["result"]["channel"] == "stable"
terminal = read("terminal-session-verification.json")
assert terminal["sessionFinalExitCode"] == 0 and terminal["candidateCount"] == 1
assert terminal["sessionFinalStatus"] == monitor["status"]
assert terminal["applicationPid"] == monitor["applicationPid"]
records = [record for record in cleanup["records"]
           if record["callbackText"] == cleanup["anchor"]["callbackText"]
           and record["scheduledAtMs"] >= anchor]
fires = [record for record in records if record["fired"]]
assert fires and min(record["firedAtMs"] for record in fires) - anchor >= 1_800_000
assert min(record["firedAtMs"] for record in fires) - anchor == monitor["actualFirstFireAfterAnchorMs"]
assert monitor["sourceWitness"]["sha256"] == attribution["source"]["sha256"]
write("independent-recalculation.json", {
    "sourceCommit": SOURCE, "applicationPid": monitor["applicationPid"],
    "requiredFloorMs": 1_800_000, "anchorAtMs": anchor,
    "anchorAtUtc": cleanup["anchor"]["scheduledAtUtc"],
    "firstRealCallbackFireAtMs": min(record["firedAtMs"] for record in fires),
    "actualFirstFireAfterAnchorMs": min(record["firedAtMs"] for record in fires) - anchor,
    "associatedArmCount": len(records), "realCallbackFireCount": len(fires),
    "periodicObservations": len(observations), "associatedObservations": len(associated),
    "generatedRouteOnlineEvents": 200, "realMinimizeRestoreObserved": True,
    "recordedNoEarlierCallback": True, "maximumObservedActiveTimers": max(
        value["activeTimerCount"] for value in associated),
    "originalScheduleBytesRestoredAndRetained": True,
    "scope": "real production scheduler timer error floor; no claim of complete network/process tracing, clock-adjustment matrix, performance or 24h/6h cycles",
})

if sys.argv[1:] == ["--build"]:
    entries = []
    for path in sorted(ROOT.rglob("*")):
        if not path.is_file() or path.name in {"manifest.json", "archive-verification.json"}:
            continue
        assert path.suffix.lower() not in {".exe", ".msi", ".cab", ".zip", ".dll"}
        data = path.read_bytes()
        entries.append({"path": path.relative_to(ROOT).as_posix(), "bytes": len(data),
                        "sha256": hashlib.sha256(data).hexdigest()})
    write("manifest.json", {"schemaVersion": 1, "hashMode": "raw", "entries": entries})
elif sys.argv[1:]:
    raise SystemExit("Usage: verify-completed-run.py [--build]")
entries = read("manifest.json")["entries"]
assert len({entry["path"] for entry in entries}) == len(entries)
assert {entry["path"] for entry in entries} == {
    path.relative_to(ROOT).as_posix() for path in ROOT.rglob("*")
    if path.is_file() and path.name not in {"manifest.json", "archive-verification.json"}
}
for entry in entries:
    path = ROOT / entry["path"]
    assert path.resolve().is_relative_to(ROOT.resolve())
    data = path.read_bytes()
    assert len(data) == entry["bytes"]
    assert hashlib.sha256(data).hexdigest() == entry["sha256"], entry["path"]
write("archive-verification.json", {
    "result": "passed", "scope": "real native thirty-minute timer subset and archive integrity",
    "filesVerified": len(read("manifest.json")["entries"]), "sourceCommit": SOURCE,
    "realNativeTimerErrorFloor": "passed for recorded scenario",
    "nativeReadFileError": "inferred from source-attributed catch branch and controlled invalid JSON",
    "rawNetworkOrNativeCommandInvocationCount": "not instrumented; not inferred from timer count",
    "systemClockChanges": "not performed", "performanceAccepted": False,
    "real24h6hCyclesAccepted": False, "fullNativeGateAccepted": False,
    "formalTaskReceipt": False, "existingPerformanceFailureCleared": False,
    "scriptsExecuted": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed",
                  "actualFirstFireAfterAnchorMs": monitor["actualFirstFireAfterAnchorMs"]}))
