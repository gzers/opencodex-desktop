"""Verify this followup without changing the earlier sampling checkpoint."""
import hashlib
import json
import struct
import subprocess
import sys
from datetime import datetime
from pathlib import Path

root = Path(__file__).resolve().parent
prior = root.parent / "OCX-0110-20261015-WIN-FG-493-CONTINUE"
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
instant = lambda value: datetime.fromisoformat(value.replace("Z", "+00:00"))
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
source = "493da80675e6508e46e0fd83d86ee7216abf96fe"
for name in ("read-state.ps1", "foreground-contract-live.ps1", "foreground-authorization.json",
             "administrator-launch.json", "administrator-first-screen.json",
             "administrator-exit-observer-result.json"):
    assert (root / name).read_bytes() == (prior / name).read_bytes(), name
feedback = read(root / "human-feedback.json")
assert feedback["origin"] == "human" and feedback["userText"] == "最大化最小化都没有问题"
assert feedback["minimumStateEvidenceType"] == "human-report"
assert not feedback["userAskedToRepeat"] and not feedback["screenshotProvesProcessIdentity"]
assert not feedback["screenshotCaptureTimeKnown"] and not feedback["fullAdministratorGatePassed"]
image = (root / feedback["attachment"]).read_bytes()
assert image[:8] == b"\x89PNG\r\n\x1a\n"
dimensions = struct.unpack(">II", image[16:24])
assert dimensions == (3437, 1885)
launch = read(root / "administrator-launch.json")
assert launch["source"] == source and launch["pid"] == 78512
assert launch["applicationToken"]["elevated"] and launch["applicationToken"]["elevationType"] == 2
deadline = instant(launch["authorization"]["expiresAtUtc"])
old_exit = read(root / "administrator-exit-observer-result.json")
assert old_exit["pid"] == launch["pid"] and old_exit["creationTimeUtc"] == launch["creationTimeUtc"]
assert old_exit["timeout"] and not old_exit["exitObserved"] and old_exit["exitCode"] is None
for name in ("reader-result.json", "reader-utf8-error.json"):
    failure = read(root / name)
    assert "ArgumentException" == failure["errorType"]
    assert not failure["uiInputInjected"] and not failure["productLaunched"]
    assert not failure["automaticRetryAllowed"]
result = read(root / "reader-pwsh7-result.json")
assert result["status"] == "read-completed" and result["authorizationEncoding"] == "utf8"
assert not result["uiInputInjected"] and not result["productLaunched"] and not result["userManualOperationRepeated"]
assert instant(result["atUtc"]) < deadline
window = read(root / "administrator-manual-window-followup.json")
assert instant(window["atUtc"]) < deadline
assert window["pid"] == launch["pid"] and window["creationTimeUtc"] == launch["creationTimeUtc"]
assert window["hwnd"] == 8786154
assert window["applicationToken"]["elevated"] and window["applicationToken"]["elevationType"] == 2
assert window["visible"] and not window["maximized"] and not window["minimized"]
assert not window["foreground"] and window["backdropRawValue"] == 1 and window["backdropHresult"] == 0
assert not window["inputInjectedByThisReader"] and not window["settingsChangedByThisReader"]
ready = read(root / "followup-exit-observer-ready.json")
exit_result = read(root / "followup-exit-observer-result.json")
for observation in (ready, exit_result):
    assert observation["pid"] == launch["pid"] and observation["creationTimeUtc"] == launch["creationTimeUtc"]
    assert not observation["inputInjected"] and not observation["productLaunched"]
assert ready["handleHeld"] and 0 < ready["timeoutMilliseconds"] <= 120000
assert not exit_result["agentForcedTermination"]
assert exit_result["timeout"] and not exit_result["exitObserved"] and exit_result["exitCode"] is None
post = read(root / "followup-checkpoint.original.json")
assert post["source"] == source and post["codeWorktreeClean"]
assert post["candidateExeSha256"] == launch["sha256"]
assert post["nativeCount"] == 1 and post["nativeProcesses"][0]["ProcessId"] == launch["pid"]
post_exit = post["administratorExitObserverResult"].copy()
original_exit = exit_result.copy()
for key in ("atUtc", "creationTimeUtc"):
    assert instant(post_exit.pop(key)) == instant(original_exit.pop(key))
assert post_exit == original_exit
assert post["dailyExeUnchanged"] and post["protectedSandboxUnchanged"]
assert post["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
for name in ("uiInputInjected", "productLaunched", "agentForcedTermination", "installerExecuted",
             "macNativeHandoffPerformed", "fullVersionGatePassed", "taskClosed", "releaseRequested"):
    assert post[name] is False
log = (root / "app-log-followup.original.txt").read_text(encoding="utf-8-sig")
old_log = (prior / "app-log-at-checkpoint.original.txt").read_text(encoding="utf-8-sig")
assert log.startswith(old_log)
admin_start = log.rfind(f"startup version=0.1.10 commit={source}")
assert admin_start >= 0
admin_log = log[admin_start:]
mica_lines = [line for line in admin_log.splitlines() if "material=mica reason=mica" in line]
inactive_lines = [line for line in admin_log.splitlines() if "material=solid reason=inactive" in line]
assert len(mica_lines) == 5 and len(inactive_lines) == 6
old_sampler = read(prior / "administrator-window-sampler-result.json")
assert all(instant(line.split()[0]) > instant(old_sampler["atUtc"]) for line in mica_lines)
snapshot = read(root / "owned-state-followup/snapshot.json")
previous = read(prior / "owned-state-before-ordinary-exit/snapshot.json")
assert snapshot["pid"] == launch["pid"] and snapshot["creationTimeUtc"] == launch["creationTimeUtc"]
assert len(snapshot["files"]) == len(previous["files"]) == 11
assert not snapshot["sourceDataChanged"] and not snapshot["crossFileAtomicSnapshotClaimed"]
changed = []
for row in snapshot["files"]:
    path = root / "owned-state-followup" / row["path"]
    assert path.stat().st_size == row["bytes"] and sha(path) == row["sha256"]
    old = prior / "owned-state-before-ordinary-exit" / row["path"]
    unchanged = path.read_bytes() == old.read_bytes()
    assert row["unchangedFromBeforeOrdinaryExit"] == unchanged
    if not unchanged:
        before, after = read(old), read(path)
        if row["path"] == "manager-state/notifications.json":
            assert len(before["items"]) == len(after["items"]) == 1
            before["items"][0].pop("created_at")
            after["items"][0].pop("created_at")
        elif row["path"] == "manager-state/runtime.json":
            before.pop("resolved_at")
            after.pop("resolved_at")
        else:
            raise AssertionError("Unexpected state change: " + row["path"])
        assert before == after
        changed.append(row["path"])
assert changed == ["manager-state/notifications.json", "manager-state/runtime.json"]
subprocess.run([sys.executable, "-X", "utf8", str(prior / "verify-checkpoint.py")],
               capture_output=True, text=True, check=True)
manifest_path = root / "manifest.json"
report_path = root / "verification.json"
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size, "sha256": sha(path)}
           for path in sorted(root.rglob("*")) if path.is_file() and path not in {manifest_path, report_path}]
if "--build" in sys.argv:
    assert not manifest_path.exists() and not report_path.exists()
    manifest_path.write_text(json.dumps(entries, indent=2) + "\n", encoding="utf-8")
else:
    assert read(manifest_path) == entries
report = {
    "archiveIntegrity": "passed",
    "source": source,
    "manifestFiles": len(entries),
    "humanMaximizeMinimizeReportAccepted": True,
    "screenshotDimensions": dimensions,
    "screenshotSha256": sha(root / feedback["attachment"]),
    "sameAdministratorProcessAndHwndReadBack": True,
    "windowRestoredAtFollowup": True,
    "administratorMicaSuccessLogLinesAfterOldSampler": len(mica_lines),
    "administratorInactiveLogLines": len(inactive_lines),
    "concurrentForegroundDwmMicaReadbackClaimed": False,
    "oldSamplerCheckpointUnchangedAndVerified": True,
    "persistentFilesVerified": 11,
    "runtimeTimestampOnlyChanges": changed,
    "nativeCountAtFollowup": post["nativeCount"],
    "administratorExitObserved": False,
    "administratorExitCode": None,
    "historicalFailuresCleared": False,
    "administratorFullUiGatePassed": False,
    "fullVersionGatePassed": False,
    "taskClosed": False,
    "releaseRequested": False,
}
serialized = json.dumps(report, indent=2) + "\n"
if "--build" in sys.argv:
    report_path.write_text(serialized, encoding="utf-8")
else:
    assert report_path.read_text(encoding="utf-8") == serialized
print(serialized)
