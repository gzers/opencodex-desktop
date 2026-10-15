"""Verify the later process-absence receipt without inventing an unobserved exit code."""
import hashlib
import json
import sys
from datetime import datetime
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
names = [
    "administrator-exit-observer-result.json",
    "administrator-exit-postcheck.json",
    "administrator-exit-sky-window-check.json",
    "verify-administrator-exit.py",
]
manifest_path = root / "administrator-exit-manifest.json"
report_path = root / "administrator-exit-verification.json"
if "--build" in sys.argv:
    assert not manifest_path.exists() and not report_path.exists()
    manifest = [
        {"path": name, "bytes": (root / name).stat().st_size,
         "sha256": hashlib.sha256((root / name).read_bytes()).hexdigest()}
        for name in names
    ]
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
else:
    manifest = read(manifest_path.name)
    assert {row["path"] for row in manifest} == set(names)
for row in manifest:
    path = root / row["path"]
    assert path.stat().st_size == row["bytes"]
    assert hashlib.sha256(path.read_bytes()).hexdigest() == row["sha256"]
launch = read("administrator-launch.json")
observer = read("administrator-exit-observer-result.json")
post = read("administrator-exit-postcheck.json")
windows = read("administrator-exit-sky-window-check.json")
assert launch["pid"] == observer["pid"] == post["boundProductPid"] == 73048
assert launch["creationTimeUtc"] == observer["creationTimeUtc"] == post["boundCreationTimeUtc"]
assert launch["applicationToken"]["elevated"] is True and launch["applicationToken"]["elevationType"] == 2
assert post["source"] == launch["source"] == "e6a4b32f8b60e362911cc02bc67fc0a2f82e7963"
assert observer["timeout"] and not observer["exitObserved"] and observer["exitCode"] is None
assert post["exitObserverTimedOut"] and post["exitObserverDeadlineUtc"] == observer["atUtc"]
assert datetime.fromisoformat(observer["atUtc"]) < datetime.fromisoformat(post["atUtc"])
assert not observer["forcedTerminationByObserver"] and not post["agentForcedTermination"]
assert post["userReported"] == windows["userText"] == "已退出" and post["userAssistedCloseReported"]
assert windows["applicationWindows"] == [] and not windows["uiInputByAgent"]
assert not windows["newApplicationStarted"] and not post["newApplicationStarted"]
assert post["nativeProcesses"] == [] and post["nativeCount"] == 0
assert post["processAbsenceVerified"] and not post["boundPidPresent"]
assert not post["actualExitTimeCaptured"] and not post["exitCodeCaptured"] and post["actualExitCode"] is None
assert all(not helper["pidPresent"] and not helper["originalHelperStillPresent"]
           for helper in post["observerHelpers"])
assert post["dailyExeUnchanged"] and post["protectedSandboxUnchanged"]
assert post["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert not post["fullNativeGatePassed"] and not post["versionCompletionClaimed"]
assert not post["platformHandoffCompleted"]
report = {
    "archiveIntegrity": "passed",
    "manifestFiles": len(manifest),
    "administratorUserReportedExit": True,
    "nativeInstancesAfter": 0,
    "administratorWindowAbsent": True,
    "readOnlyObserversAbsent": True,
    "agentForcedTermination": False,
    "exitObserverTimedOutBeforePostcheck": True,
    "exitCodeCaptured": False,
    "actualExitCode": None,
    "dailyApplicationAndProtectedSandboxUnchanged": True,
    "fullNativeGatePassed": False,
    "taskOrVersionCompletionClaimed": False,
    "platformHandoffCompleted": False,
}
if "--build" in sys.argv:
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert report == read(report_path.name)
print(json.dumps(report))
