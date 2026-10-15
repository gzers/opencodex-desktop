"""Verify the later zero-process checkpoint without rewriting the earlier timeout/live evidence."""
import hashlib
import json
import sys
from datetime import datetime
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
prior = root.parent / "OCX-0110-20261015-WIN-FG-493-UPDATES-TREE"
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
sha = lambda data: hashlib.sha256(data).hexdigest()
instant = lambda value: datetime.fromisoformat(value.replace("Z", "+00:00"))
post = read(root / "exit-postcheck.original.json")
old = read(prior / "foreground-postcheck.json")
observer = read(prior / "restart-exit-observer-result.json")
restart = read(prior / "ordinary-restart.json")
assert post["source"] == restart["source"] == "493da80675e6508e46e0fd83d86ee7216abf96fe"
assert post["candidateExeSha256"] == restart["sha256"] == "9ca21e30c9c2eee68000e1e8a03c58a42918e46d97387f96a102b7f11afb326a"
assert post["formerPid"] == restart["pid"] == 77760
assert instant(post["formerCreationTimeUtc"]) == instant(restart["creationTimeUtc"])
assert post["userSignal"] == "已退出" and post["userSignalOrigin"] == "human" and post["userReportedExit"]
assert post["nativeCount"] == post["nativeCountAfterProtectionChecks"] == 0 and post["nativeProcesses"] == []
assert post["actualExitTimeUtc"] is None and post["actualExitCode"] is None
assert post["originalObserverTimedOut"] and not post["originalObserverExitObserved"]
assert post["originalObserverExitCode"] is None
assert observer["timeout"] and not observer["exitObserved"] and observer["exitCode"] is None
assert old["nativeCount"] == 1 and old["nativeProcesses"][0]["ProcessId"] == 77760
assert instant(post["priorLiveCheckpointAtUtc"]) == instant(old["atUtc"])
assert not post["priorLiveCheckpointChanged"]
initial = read(root / "initial-zero-observation.transcribed.json")
assert initial["nativeCount"] == 0 and initial["nativeProcesses"] == []
assert initial["actualExitCode"] is None and initial["actualExitTimeUtc"] is None
assert instant(old["atUtc"]) < instant(initial["observedAtUtc"]) < instant(post["nativeAbsentObservedAtUtc"])
assert post["dailyExeUnchanged"] and post["dailyExeSha256"] == old["dailyExeSha256"]
assert post["protectedSandboxUnchanged"] and post["protectedSandboxHashes"] == old["protectedSandboxHashes"]
assert post["originalTaskSha256"] == old["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert post["originalTaskBytes"] == 3878 and post["codeWorktreeClean"]
for key in ("applicationStarted", "agentForcedTermination", "uiInputIssued", "installerExecuted",
            "thirdPartyLifecycleScriptsExecuted", "foregroundWindowExtended",
            "macNativeHandoffPerformed", "fullVersionGatePassed", "taskClosed", "releaseRequested"):
    assert post[key] is False
rows = post["unchangedIsolatedPersistentFiles"]
assert len(rows) == 5 and len({row["path"] for row in rows}) == 5
previous_rows = {row["path"]: row for row in read(prior / "owned-state-after-restart/snapshot.json")["files"]}
for row in rows:
    relative = PurePosixPath(row["path"])
    assert not relative.is_absolute() and ".." not in relative.parts and "\\" not in row["path"]
    data = (root / "reread" / row["path"]).read_bytes()
    assert row["matchesAfterRestartSnapshot"] and len(data) == row["bytes"] and sha(data) == row["sha256"]
    assert {key: row[key] for key in ("path", "bytes", "sha256")} == previous_rows[row["path"]]
    assert data == (prior / "owned-state-after-restart" / row["path"]).read_bytes()
prefs = read(root / "reread/manager-state/preferences.json")
assert prefs["appearance"]["theme"] == "light"
assert prefs["updates"]["app_update_channel"] == "stable" and prefs["updates"]["app_update_auto_check"] is False
assert read(root / "prior-archive-reverification.original.json") == read(prior / "archive-complete-verification.json")
complete = read(prior / "archive-complete-manifest.json")
for row in complete:
    path = prior / row["path"]
    assert path.stat().st_size == row["bytes"] and sha(path.read_bytes()) == row["sha256"]
manifest_path, report_path = root / "manifest.json", root / "archive-verification.json"
paths = sorted((path for path in root.rglob("*") if path.is_file() and path not in {manifest_path, report_path}),
               key=lambda path: path.relative_to(root).as_posix())
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": sha(path.read_bytes())} for path in paths]
if "--build" in sys.argv:
    assert not manifest_path.exists() and not report_path.exists()
    manifest_path.write_text(json.dumps(entries, indent=2) + "\n", encoding="utf-8")
else:
    assert read(manifest_path) == entries
report = {
    "archiveIntegrity": "passed", "scope": "User-reported exit followed by independent zero-process readback",
    "manifestFiles": len(entries), "source": post["source"], "formerPid": 77760,
    "nativeInstancesAtCheckpoint": 0, "actualExitTimeKnown": False, "actualExitCodeKnown": False,
    "oldObserverTimeoutAndLiveCheckpointPreserved": True, "priorArchiveFilesRechecked": len(complete),
    "isolatedPreferenceScheduleCacheBackupFilesUnchanged": 5,
    "dailyExeAndProtectedSandboxAndOriginalTaskHashesUnchanged": True,
    "applicationStartedOrForcedTermination": False, "foregroundWindowExtended": False,
    "macNativeHandoffPerformed": False, "fullVersionGatePassed": False,
    "taskClosed": False, "releaseRequested": False,
}
if "--build" in sys.argv:
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read(report_path) == report
print(json.dumps(report))
