import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
initial = read("native-initialization.json")
assert initial["sourceCommit"] == "afff490a430a81f8c233d44928aa3681a23e91e0"
assert initial["applicationPid"] == 38924
r = initial["result"]
assert r["status"] == "observing-real-stable-and-panel-twenty-four-hour-deadlines"
assert r["managerInitialQuery"]["status"] == "up_to_date"
assert r["managerInitialQuery"]["update"]["channel"] == "stable"
assert r["managerInitialQuery"]["update"]["error"] is None
assert r["panelInitialRemote"]["version"] == "2.83.0"
assert not r["clocksChanged"] and not r["businessResponsesInjected"] and not r["timersWrapped"]
assert not r["fullRealCycleAccepted"] and not r["foregroundAtStart"]
baseline = read("schedule-baseline.json")["entries"]
for target in ["manager-stable", "panel-latest"]:
    assert baseline[target]["failures"] == 0
    assert baseline[target]["next_due"] - baseline[target]["last_success"] == 86400
observation = read("initial-observation.json")
assert observation["originalLifecycleLive"]
assert observation["native"]["applicationPid"] == 38924
assert observation["native"]["result"]["preferences"] == {"channel": "stable", "automaticCheck": True}
assert not observation["native"]["result"]["foreground"]
assert observation["schedule"] == read("schedule-baseline.json")
verification = read("initial-live-handle-verification.json")
assert verification["monitorRetainedHandleConfirmedLive"] and verification["applicationRetainedHandleConfirmedLive"]
assert verification["monitorPid"] == 18944 and verification["execSession"] == 93170
assert not verification["cycleAccepted"] and not verification["state"]["fullCycleAccepted"]
excluded = {"manifest.json", "archive-verification.json"}
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in excluded)
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
    "result": "passed", "scope": "real successful initial queries, future daily deadlines and verified live monitor setup only",
    "runtimeLogLocation": verification["state"]["runtimeRoot"],
    "monitorStatusAtSnapshot": verification["state"]["status"],
    "realTwentyFourHourCycleAccepted": False, "fullNativeGateAccepted": False,
    "betaCycleAccepted": False, "performanceAccepted": False, "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "realCycles": "pending", "files": len(entries)}))
