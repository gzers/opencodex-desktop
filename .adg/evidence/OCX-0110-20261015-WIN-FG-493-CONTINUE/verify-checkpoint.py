"""Verify completed UI/sampler evidence and freeze a checkpoint while the exit observer may remain live."""
import hashlib
import json
import subprocess
import sys
from datetime import datetime
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
sha = lambda data: hashlib.sha256(data).hexdigest()
instant = lambda value: datetime.fromisoformat(value.replace("Z", "+00:00"))
source = "493da80675e6508e46e0fd83d86ee7216abf96fe"
analysis = read("session-analysis.json")
verified = subprocess.run([sys.executable, "-X", "utf8", str(root / "analyze-session.py"), "--verify"],
                          capture_output=True, text=True, check=True)
assert json.loads(verified.stdout) == analysis
auth = read("foreground-authorization.json")
assert auth["origin"] == "human" and auth["foregroundStartGranted"] and auth["userText"] == "我批准继续"
ordinary, admin = read("ordinary-launch.json"), read("administrator-launch.json")
start = instant(ordinary["authorization"]["startedAtUtc"])
deadline = instant(ordinary["authorization"]["expiresAtUtc"])
assert start == instant(auth["atUtc"]) and (deadline - start).total_seconds() == 1800
assert start < instant(ordinary["creationTimeUtc"]) < instant(admin["creationTimeUtc"]) < deadline
assert instant(admin["authorization"]["expiresAtUtc"]) == deadline
actions = read("ui-actions.json")
assert actions and len({row["phase"] for row in actions}) == len(actions)
for row in actions:
    assert start <= instant(row["atUtc"]) < deadline
    assert row["window"]["id"] == 4786352 and row["actionReturned"]
    if row["phase"] != "ordinary-normal-close":
        assert "error" not in row
for path in root.glob("*.sky.json"):
    value = read(path.name)
    assert start <= instant(value["atUtc"]) < deadline
    assert value["window"]["id"] in {4786352, 8786154}
    for shot in value["screenshots"]:
        relative = PurePosixPath(shot["file"])
        assert not relative.is_absolute() and ".." not in relative.parts
        assert (root / shot["file"]).is_file()
        if value["window"]["id"] == 8786154:
            assert "标题栏" in value["accessibility"]["tree"]
coord = read("administrator-coordinate-action.json")
assert coord["actionReturned"] and coord["productNavigationSuccessNotInferred"]
assert coord["window"]["id"] == 8786154
assert instant(coord["atUtc"]) < deadline
rows = [json.loads(line) for line in (root / "administrator-window-samples.original.jsonl").read_text(
    encoding="utf-8-sig").splitlines() if line.strip()]
assert len(rows) == 240
assert all(row["foreground"] and not row["maximized"] and not row["minimized"]
           and row["backdropHresult"] == 0 and row["backdropValue"] == 1 for row in rows)
assert analysis["administratorForegroundMicaSamples"] == analysis["administratorMaximizedSamples"] == 0
assert analysis["administratorMinimizedSamples"] == 0
sampler = read("administrator-window-sampler-result.json")
assert not sampler["exitObserved"] and sampler["exitCode"] is None and sampler["observationLimitReached"]
assert sampler["elapsedSeconds"] >= 240
post = read("checkpoint.original.json")
assert post["source"] == source and post["codeWorktreeClean"] and post["nativeCount"] == 1
assert post["nativeProcesses"][0]["ProcessId"] == 78512
assert post["controllerLiveAtCheckpoint"] and post["controllerPid"] == 74792
assert post["controllerProcessesObserved"][0]["ProcessId"] == 74792
assert post["currentOrdinaryCimMayNotReadElevatedExecutable"]
assert not post["administratorObserverResultAvailable"] and not post["administratorExitObserved"]
assert post["administratorExitCode"] is None
assert post["dailyExeUnchanged"] and post["dailyExeSha256"] == ordinary["dailyExeSha256"]
assert post["protectedSandboxUnchanged"] and post["protectedSandboxHashes"] == ordinary["protectedSandboxHashes"]
assert post["originalTaskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
for key in ("agentForcedTermination", "installerExecuted", "cdpUsedForUiInput", "systemDpiChanged",
            "systemSecurityOrGlobalEnvironmentChanged", "thirdPartyLifecycleScriptsExecuted",
            "macNativeHandoffPerformed", "newAccountOrProfileCreated", "fullVersionGatePassed",
            "taskClosed", "releaseRequested"):
    assert post[key] is False
log = (root / "app-log-at-checkpoint.original.txt").read_text(encoding="utf-8-sig")
assert f"startup version=0.1.10 commit={source}" in log
admin_start = log.rfind("2026-10-15T04:47:46.")
assert admin_start >= 0
assert "material=solid reason=inactive" in log[admin_start:]
assert "material=mica" not in log[admin_start:]
# A later observer result is a distinct fact and must never silently change this
# immutable checkpoint. The controller is already running and owns those writes.
future_outputs = {root / "administrator-exit-observer-result.json",
                  root / "administrator-controller-error.json"}
manifest_path, report_path = root / "checkpoint-manifest.json", root / "checkpoint-verification.json"
excluded = {manifest_path, report_path} | future_outputs
paths = sorted((path for path in root.rglob("*") if path.is_file() and path not in excluded),
               key=lambda path: path.relative_to(root).as_posix())
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": sha(path.read_bytes())} for path in paths]
if "--build" in sys.argv:
    assert not manifest_path.exists() and not report_path.exists()
    manifest_path.write_text(json.dumps(entries, indent=2) + "\n", encoding="utf-8")
else:
    assert read(manifest_path.name) == entries
assert any("/config-migrations/" in row["path"] and row["path"].endswith("/manifest.json") for row in entries)
for path in paths:
    assert not path.is_symlink()
report = {
    "archiveIntegrity": "passed", "scope": "Completed ordinary UI and administrator sampler checkpoint only",
    "manifestFiles": len(entries), "source": source, "ordinaryPid": 49492, "administratorPid": 78512,
    "ordinaryNormalExitCode": 0, "sequentialSameImageAndRootDualTokensVerified": True,
    "directoryAndFileOpenWarningsCancelled": True, "actualOwnedDirectoryOpened": True,
    "treeLeftAndRightKeyboardObserved": True, "ordinaryMinimizeRestoreDwm2To1To2Verified": True,
    "administratorSampleCount": 240, "administratorForegroundSolidSamples": 240,
    "administratorMaximizedOrMinimizedObserved": False, "administratorActiveMicaAccepted": False,
    "administratorSettingsNavigationVerified": False, "administratorFullUiGatePassed": False,
    "nativeCountAtCheckpoint": 1, "exitObserverLiveAtCheckpoint": True,
    "administratorExitObservedAtCheckpoint": False, "administratorExitCodeAtCheckpoint": None,
    "laterObserverOutputsExcludedAndMustBeVerifiedSeparately": sorted(path.name for path in future_outputs),
    "physicalDpiVerified": False, "installerRetentionVerified": False,
    "actualUpdateDownloadProgressVerified": False, "realThirtyMinuteBackoffVerified": False,
    "pairedPerformanceVerified": False, "historicalFailuresCleared": False,
    "fullVersionGatePassed": False, "releaseRequested": False,
}
if "--build" in sys.argv:
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read(report_path.name) == report
print(json.dumps(report))
