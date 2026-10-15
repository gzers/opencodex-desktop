"""Verify the later absent-process fact while preserving timeout/live history."""
from datetime import datetime
import hashlib
import json
from pathlib import Path, PurePosixPath
import subprocess
import sys

root = Path(__file__).resolve().parent
evidence = root.parent
prior = evidence / "OCX-0110-20261015-WIN-FG-493-CONTINUE"
followup = evidence / "OCX-0110-20261015-WIN-FG-493-MANUAL-WINDOW"
browser = evidence / "OCX-0110-20261015-WIN-TREE-BROWSER"
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
instant = lambda value: datetime.fromisoformat(value.replace("Z", "+00:00"))
digest = lambda data: hashlib.sha256(data).hexdigest()
post = read(root / "exit-postcheck.original.json")
launch = read(prior / "administrator-launch.json")
last = read(browser / "checkpoint.original.json")
assert post["source"] == launch["source"] == "493da80675e6508e46e0fd83d86ee7216abf96fe"
assert post["formerPid"] == launch["pid"] == 78512
assert instant(post["formerCreationTimeUtc"]) == instant(launch["creationTimeUtc"])
assert post["userSignalOrigin"] == "human" and post["userSignal"] == "我已退出" and post["userReportedExit"]
assert post["nativeCount"] == post["nativeCountAfterProtectionChecks"] == 0
assert post["nativeProcesses"] == []
assert post["formerPidPresent"] is post["controllerPidPresent"] is post["followupReaderPidPresent"] is False
assert post["controllerPid"] == 74792 and post["followupReaderPid"] == 79860
assert post["actualExitTimeUtc"] is post["actualExitCode"] is None
assert last["nativeCount"] == 1 and last["nativeProcesses"][0]["ProcessId"] == 78512
assert post["priorLiveCheckpointUnchanged"]
assert instant(post["priorLiveCheckpointAtUtc"]) == instant(last["atUtc"])
assert instant(last["atUtc"]) < instant(post["nativeAbsentObservedAtUtc"]) <= instant(post["atUtc"])
assert len(post["historicalObservers"]) == 2
for row, anchor in zip(post["historicalObservers"], (prior, followup)):
    copied = root / row["file"]
    original = anchor / row["file"]
    assert copied.read_bytes() == original.read_bytes()
    assert digest(copied.read_bytes()) == row["sha256"]
    observer = read(copied)
    assert observer["pid"] == 78512
    assert instant(observer["creationTimeUtc"]) == instant(launch["creationTimeUtc"])
    assert instant(observer["atUtc"]) == instant(row["atUtc"]) < instant(post["nativeAbsentObservedAtUtc"])
    assert observer["timeout"] and not observer["exitObserved"] and observer["exitCode"] is None
assert post["codeWorktreeClean"] and post["dailyExeUnchanged"] and post["protectedSandboxUnchanged"]
assert post["candidateExeSha256"] == launch["sha256"]
assert post["dailyExeSha256"] == last["dailyExeSha256"]
assert post["protectedSandboxHashes"] == last["protectedSandboxHashes"]
assert post["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert post["originalTaskBytes"] == 3878 and post["originalTaskRevision"] == 2
assert post["originalTaskState"] == "in_progress" and post["originalTaskResult"] == "pending"
assert len(post["originalTaskRequiredChecks"]) == 8
for name in ("productLaunched", "uiInputInjected", "foregroundWindowExtended", "agentForcedTermination",
             "installerExecuted", "accountOrProfileCreated", "macNativeHandoffPerformed",
             "fullVersionGatePassed", "taskClosed", "releaseRequested"):
    assert post[name] is False
state = read(root / "state-reread.json")
previous_rows = {row["path"]: row for row in read(followup / "owned-state-followup/snapshot.json")["files"]}
assert state["source"] == post["source"] and state["formerPid"] == 78512
assert len(state["files"]) == len(previous_rows) == 11 and state["allElevenFilesUnchanged"]
assert not state["sourceDataChanged"] and not state["webviewOrDailyDataCopied"]
for row in state["files"]:
    relative = PurePosixPath(row["path"])
    assert not relative.is_absolute() and ".." not in relative.parts and "\\" not in row["path"]
    data = (root / "reread" / relative).read_bytes()
    assert row["unchangedFromAdministratorFollowup"]
    assert len(data) == row["bytes"] and digest(data) == row["sha256"]
    assert all(row[key] == previous_rows[row["path"]][key] for key in ("path", "bytes", "sha256"))
    assert data == (followup / "owned-state-followup" / relative).read_bytes()
old_log = (followup / "app-log-followup.original.txt").read_bytes()
assert (root / "app-log-after-exit.original.txt").read_bytes().startswith(old_log)
historical = []
for anchor, script in ((prior, "verify-checkpoint.py"), (followup, "verify-manual-window.py"),
                       (browser, "verify-browser-evidence.py")):
    result = subprocess.run([sys.executable, "-X", "utf8", str(anchor / script)],
                            capture_output=True, text=True, check=True)
    historical.append({"directory": anchor.name, "verification": json.loads(result.stdout)})
report_path, manifest_path = root / "verification.json", root / "manifest.json"
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": digest(path.read_bytes())}
           for path in sorted(root.rglob("*")) if path.is_file() and path not in {report_path, manifest_path}]
if "--build" in sys.argv:
    assert not report_path.exists() and not manifest_path.exists()
    manifest_path.write_text(json.dumps(entries, indent=2) + "\n", encoding="utf-8")
else:
    assert read(manifest_path) == entries
report = {
    "archiveIntegrity": "passed", "manifestFiles": len(entries), "source": post["source"],
    "formerPid": 78512, "nativeCountBeforeAndAfterChecks": [0, 0],
    "actualExitTimeKnown": False, "actualExitCodeKnown": False,
    "previousTimeoutAndLiveEvidencePreserved": True,
    "historicalArchivesVerified": historical,
    "isolatedJsonFilesUnchanged": 11, "dailyAndProtectedSandboxAndTaskHashesUnchanged": True,
    "productStartedOrForcedTermination": False, "foregroundWindowExtended": False,
    "macNativeHandoffPerformed": False, "historicalFailuresCleared": False,
    "fullVersionGatePassed": False, "taskClosed": False, "releaseRequested": False,
}
if "--build" in sys.argv:
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read(report_path) == report
print(json.dumps({
    "archiveIntegrity": report["archiveIntegrity"], "manifestFiles": len(entries),
    "nativeCountBeforeAndAfterChecks": [0, 0], "isolatedJsonFilesUnchanged": 11,
    "historicalArchivesVerified": len(historical),
    "actualExitTimeKnown": False, "actualExitCodeKnown": False,
    "foregroundWindowExtended": False, "fullVersionGatePassed": False,
}))
