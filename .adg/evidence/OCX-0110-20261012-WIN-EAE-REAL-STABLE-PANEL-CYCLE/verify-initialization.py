"""Only verify a fresh source-bound real daily-cycle setup, not elapsed acceptance."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
native = read("native-initialization.json")
assert native["sourceCommit"] == source and native["applicationPid"] == 37276
assert native["executableSha256"] == "f5e4a2eef0ffc364a2a57b47b1463015faf3433a15ff021d544d6e161004f00c"
value = native["result"]
assert value["status"] == "observing-real-stable-and-panel-twenty-four-hour-deadlines"
assert value["managerInitialQuery"]["status"] == "up_to_date"
assert value["managerInitialQuery"]["update"]["channel"] == "stable"
assert value["managerInitialQuery"]["update"]["error"] is None
assert value["panelInitialRemote"]["version"] == "2.83.0"
assert not any(value[key] for key in [
    "businessResponsesInjected", "clocksChanged", "timersWrapped", "fullRealCycleAccepted",
])
baseline = read("schedule-baseline.json")
for target, due in [("manager-stable", 1791903339), ("panel-latest", 1791903340)]:
    entry = baseline["entries"][target]
    assert entry["failures"] == 0 and entry["next_due"] == due
    assert entry["next_due"] - entry["last_success"] == 86400
first = read("initial-observation.json")
assert first["originalLifecycleLive"] and first["native"]["sourceCommit"] == source
assert first["native"]["applicationPid"] == 37276 and first["schedule"] == baseline
assert first["native"]["result"]["preferences"] == {"channel": "stable", "automaticCheck": True}
assert not first["native"]["result"]["foreground"]
session = read("application-baseline-session.json")
assert session["sourceCommit"] == source and session["applicationPid"] == 37276
assert session["applicationToken"]["elevated"] is False and session["applicationToken"]["elevationType"] == 3
assert session["sourceCommitMatched"]
live = read("initial-live-handle-verification.json")
assert live["sourceCommit"] == source and live["applicationLiveHandleVerified"] and live["monitorLiveHandleVerified"]
assert live["monitorPid"] == 3920 and live["supervisorPid"] == 24300 and live["execSession"] == 46484
assert not live["fullRealCycleAccepted"] and not live["sourceTransitionOriginalCycleAccepted"]
assert live["state"]["status"] == "observing-real-deadlines"
provenance = read("monitor-provenance.json")
assert provenance["sourceCommit"] == source and provenance["applicationPid"] == 37276
assert provenance["supervisorPid"] == 24300 and provenance["oldRunMinutesAdded"] == 0
assert provenance["observationBudgetHours"] == 26 and not provenance["oldCycleAccepted"]
init = read("initialization-filesystem.json")
assert init["expectedSource"] == source and init["oldSampleTimeCarriedOverMs"] == 0
assert init["originalPreferencesSha256"] == "8498f7cf8984c74a555202027800027fa7fd1366e0e5ce51c455aaf3ba1fdb7a"
assert not init["realCycleAccepted"]
original = root.parent / "OCX-0110-20261012-WIN-REAL-STABLE-PANEL-CYCLE/methods"
for name in ["initialize.js", "observe.js", "cleanup.js"]:
    assert (root / "methods" / name).read_bytes() == (original / name).read_bytes()
original_monitor = (original / "monitor-real-cycle.ps1").read_text(encoding="utf-8-sig")
expected = (original_monitor
    .replace("afff490a430a81f8c233d44928aa3681a23e91e0", source)
    .replace("38924", "37276").replace("44780", "24300")
    .replace("cycles/stable-panel-20261012", "cycles/eae-stable-panel-20261012"))
assert (root / "methods/monitor-real-cycle.ps1").read_text(encoding="utf-8-sig") == expected
excluded = {"manifest.json", "archive-verification.json"}
paths = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in excluded)
assert all(path.suffix.lower() not in {".exe", ".dll", ".zip", ".msi", ".cab"} for path in paths)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in paths]
def write(name, data):
    (root / name).write_bytes((json.dumps(data, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "scope": "exact eae final desktop real stable/panel initial queries and fresh live monitor setup only",
    "sourceCommit": source, "newDailyCycleAccepted": False, "oldDailyCycleAccepted": False,
    "betaSixHourCycleAccepted": False, "pairedPerformanceAccepted": False,
    "fullNativeGateAccepted": False, "formalTaskReceipt": False,
    "releaseAccepted": False, "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "finalSourceCycle": "fresh setup, pending real time",
                  "files": len(entries)}))
