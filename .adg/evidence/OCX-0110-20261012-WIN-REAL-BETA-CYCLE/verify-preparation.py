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
assert r["status"] == "preparation-failed-original-preferences-restored"
assert r["managerInitialQuery"]["status"] == "failed"
assert r["managerInitialQuery"]["update"]["channel"] == "beta"
assert r["managerInitialQuery"]["update"]["error"]
assert not r["clocksChanged"] and not r["businessResponsesInjected"] and not r["timersWrapped"]
assert not r["fullRealCycleAccepted"]
source = read("beta-source-api.json")
assert source["httpStatus"] == 404 and not source["releaseCreatedOrModified"]
assert source["authenticationKeptInMemory"] and not source["credentialsIncluded"]
cleanup = read("failed-preparation-cleanup.json")
assert cleanup["preferencesChannel"] == "stable" and not cleanup["preferencesAutomaticCheck"]
assert cleanup["originalPreferencesBytesRestored"] and not cleanup["betaCycleStarted"]
baseline = read("schedule-baseline.json")["entries"]["manager-beta"]
assert baseline["failures"] == 1 and baseline["last_success"] is None
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
    "result": "passed", "scope": "archive integrity and preservation of failed beta preparation",
    "betaPreparation": "failed", "realSixHourCycleStarted": False, "realSixHourCycleAccepted": False,
    "fullNativeGateAccepted": False, "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "betaCycle": "not started", "files": len(entries)}))
