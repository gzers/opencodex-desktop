"""Verify a failed observer and cleanup; never grant the elapsed-time gate."""

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def write(name, value):
    (ROOT / name).write_bytes(
        (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    )


if sys.argv[1:] == ["--build"]:
    entries = []
    for path in sorted(ROOT.rglob("*")):
        if not path.is_file() or path.name in {"manifest.json", "archive-verification.json"}:
            continue
        payload = path.read_bytes()
        entries.append({"path": path.relative_to(ROOT).as_posix(), "bytes": len(payload),
                        "sha256": hashlib.sha256(payload).hexdigest()})
    write("manifest.json", {"schemaVersion": 1, "hashMode": "raw", "entries": entries})
elif sys.argv[1:]:
    raise SystemExit("Usage: verify-failed-preparation.py [--build]")
for entry in read("manifest.json")["entries"]:
    path = ROOT / entry["path"]
    assert path.resolve().is_relative_to(ROOT.resolve())
    payload = path.read_bytes()
    assert len(payload) == entry["bytes"]
    assert hashlib.sha256(payload).hexdigest() == entry["sha256"]
monitor = read("monitor-state.json")
assert monitor["status"] == "failed-or-interrupted"
assert monitor["cleanupSucceeded"] and monitor["actualAutoCheckAfterCleanup"] is False
assert monitor["scheduleBeforeSha256"] == monitor["scheduleAfterSha256"]
assert not monitor["timeAccelerated"] and not monitor["systemClockChanged"]
diagnosis = read("recorder-diagnostic.json")["result"]
assert diagnosis["invokeDescriptor"]["writable"] is False
assert diagnosis["invokeDescriptor"]["configurable"] is False
assert diagnosis["originalInvokeRestored"] and not diagnosis["recorderActuallyInstalled"]
assert diagnosis["retainedProbe"]["status"] == "finished"
assert diagnosis["retainedProbe"]["planCalls"] == []
cleanup = read("native-cleanup.json")["result"]
assert cleanup["cleaned"] and cleanup["actualAutoCheck"] is False
disposition = read("failed-observer-disposition.json")
assert not disposition["securityPropertyOverrideAttempted"]
assert not disposition["processRestartedForObservationFailure"]
assert not disposition["productFailureInferred"]
write("archive-verification.json", {
    "result": "passed", "scope": "failed observer provenance and fixture cleanup integrity only",
    "filesVerified": len(read("manifest.json")["entries"]),
    "nativeSource": monitor["sourceCommit"],
    "elapsedThirtyMinuteGate": "not accepted; observer was not installed",
    "actualSchedulerErrorFloorStart": "unobserved; never inferred from empty recorder",
    "testScheduleBytesRestored": True, "testAutoCheckRestoredFalse": True,
    "productRestartedForObservationFailure": False,
    "fullNativeGateAccepted": False, "performanceAccepted": False,
    "realCyclesAccepted": False, "existingPerformanceFailureCleared": False,
    "formalTaskReceipt": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "actualBackoffGate": "not_accepted"}))
