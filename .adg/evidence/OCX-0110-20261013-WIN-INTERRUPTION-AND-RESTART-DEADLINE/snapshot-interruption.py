"""Freeze stopped owner observations and original persistent deadlines, without restarting."""
import hashlib
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
runtime = code / "test/runtime/windows0110"
cycle = runtime / "cycles/eae-stable-panel-20261012"
original = root.parent / "OCX-0110-20261012-WIN-EAE-REAL-STABLE-PANEL-CYCLE"
old_source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
candidate = "be7395acfce0ec1e36b83300b576db408f8c35bb"
output = root / "interruption-snapshot"
assert not output.exists(), "Never overwrite any interruption witness"
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
sha = lambda value: hashlib.sha256(value).hexdigest()
def guarded_bytes(path):
    for ancestor in [path, *path.parents]:
        if ancestor.is_symlink() or ancestor.stat().st_file_attributes & 0x400:
            raise ValueError("Reparse source rejected")
    return path.read_bytes()
files = {
    "old-current-session.json": runtime / "control/current-session.json",
    "old-supervisor-session.json": runtime / "evidence/run-20261012T145310Z/session.json",
    "old-monitor-state.json": cycle / "monitor-state.json",
    "original-initialization.json": original / "native-initialization.json",
    "original-schedule-baseline.json": original / "schedule-baseline.json",
    "actual-schedule.json": runtime / "data/manager-state/update-schedule.json",
}
raw = {name: guarded_bytes(path) for name, path in files.items()}
observation_bytes = guarded_bytes(cycle / "observations.ndjson")
last_lf = observation_bytes.rfind(b"\n")
assert last_lf >= 0
raw["complete-old-observations-prefix.ndjson"] = observation_bytes[:last_lf + 1]
observations = [json.loads(line) for line in raw["complete-old-observations-prefix.ndjson"].splitlines() if line.strip()]
monitor = json.loads(raw["old-monitor-state.json"].decode("utf-8-sig"))
session = json.loads(raw["old-current-session.json"].decode("utf-8-sig"))
schedule = json.loads(raw["actual-schedule.json"].decode("utf-8-sig"))
assert monitor["sourceCommit"] == session["sourceCommit"] == old_source
assert monitor["applicationPid"] == session["applicationPid"] == 37276
assert len(observations) == monitor["observations"] == 201
assert monitor["status"] == "observing-real-deadlines" and session["status"] == "running"
assert not monitor.get("finishedAtUtc") and not monitor["fullCycleAccepted"]
assert raw["actual-schedule.json"] == raw["original-schedule-baseline.json"]
shell = Path(r"C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\native\powershell\pwsh.exe")
command = r"""
$ErrorActionPreference='Stop'
$taskProcesses=foreach($taskId in @(37276,24300,3920)){
    $taskProcess=Get-Process -Id $taskId -ErrorAction SilentlyContinue
    if($taskProcess){
        try {[void]$taskProcess.SafeHandle;[ordered]@{pid=$taskId;present=$true;hasExited=$taskProcess.HasExited;creationUtc=$taskProcess.StartTime.ToUniversalTime().ToString('o')}}
        finally {$taskProcess.Dispose()}
    }else{[ordered]@{pid=$taskId;present=$false}}
}
$taskApps=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'")
$taskEventErrors=@()
$taskEvents=@(Get-WinEvent -FilterHashtable @{LogName='Application';Id=1000,1001,1026;StartTime=[datetime]::Parse('2026-10-13T15:30:00+08:00')} -ErrorAction SilentlyContinue -ErrorVariable taskEventErrors | Where-Object {$_.Message -match 'opencodex-desktop\.exe'} | Select-Object -First 10 TimeCreated,Id,ProviderName,Message)
[ordered]@{collectedAtUtc=[datetime]::UtcNow.ToString('o');originalProcesses=@($taskProcesses);desktopCount=$taskApps.Count;desktopProcesses=@($taskApps | Select-Object ProcessId,CreationDate,ExecutablePath);productCrashEventScope='Application1000/1001/1026 since15:30 Beijing';matchingProductCrashEvents=$taskEvents;eventQueryErrors=@($taskEventErrors | ForEach-Object {$_.Exception.Message});crashCauseProven=$false}|ConvertTo-Json -Depth 8 -Compress
"""
process_raw = subprocess.check_output(
    [str(shell), "-NoProfile", "-NonInteractive", "-Command", command],
    encoding="utf-8", timeout=30, creationflags=subprocess.CREATE_NO_WINDOW,
).encode()
actual = json.loads(process_raw)
assert actual["desktopCount"] == 0 and all(not item["present"] for item in actual["originalProcesses"])
assert not (cycle / "native-cleanup.json").exists()
assert not (cycle / "stop.request").exists() and not (runtime / "control/stop.request").exists()
expected_hashes = {
    "preferences.json": "674ad619b46fc27a273eb37da775660e3b44263fe3485e4ade98287f5fbda8fa",
    "data-root.json": "8ae75370e9f9966182caddd12ef094f16449c88119e66012a8a4a99a0c1f0e25",
    "backup-cleanup-policy.json": "85884ab649b82b6b4ab880fe24492c6c400a266254277e29e78c8080b20d2e11",
    "update-schedule.json": "6e51fe41559c287c6036ef495456f1d1d5200710f5f925b3e9e7084621c3df20",
}
actual_hashes = {name: sha(guarded_bytes(runtime / "data/manager-state" / name)) for name in expected_hashes}
assert actual_hashes == expected_hashes
new_image = code / "test/software/candidate-be7395a-38355803684/msi-readonly/opencodex-desktop.exe"
assert sha(new_image.read_bytes()) == "3a26098e813ddd56070a0092cbcb640473cb73e626aee45198c4b335ec2d8bd1"
daily_hash = sha(Path(r"D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe").read_bytes())
assert daily_hash == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
witnesses = []
for path in [
    "apps/desktop/ui/src/features/updates/scheduler.ts",
    "apps/desktop/tauri/src/commands/update_schedule.rs",
    "apps/desktop/tauri/src/modules/update/schedule.rs",
]:
    def blob(source):
        return subprocess.check_output(["git", "-C", str(code), "rev-parse", source + ":" + path]).decode().strip()
    before, after = blob(old_source), blob(candidate)
    assert before == after
    witnesses.append({"path": path, "oldSourceBlob": before, "candidateBlob": after, "unchanged": True})
original_pairs = {
    target: {
        "lastSuccessEpoch": schedule["entries"][target]["last_success"],
        "nextDueEpoch": schedule["entries"][target]["next_due"],
        "realIntervalSeconds": schedule["entries"][target]["next_due"] - schedule["entries"][target]["last_success"],
    } for target in ["manager-stable", "panel-latest"]
}
assert all(value["realIntervalSeconds"] == 86400 for value in original_pairs.values())
record = {
    "observedAtUtc": datetime.now(timezone.utc).isoformat(),
    "status": "old-owner-lifecycle-unavailable-before-original-deadlines",
    "originalSourceCommit": old_source, "originalApplicationPid": 37276,
    "originalObservationsPreserved": len(observations),
    "lastActualNativeObservationUtc": monitor["lastNativeObservationUtc"],
    "lastRecordedOwnerHandleCheckUtc": monitor["lastRetainedHandleCheckUtc"],
    "exactExitTimeKnown": False, "productCrashCauseProven": False,
    "staleRunningStatusIsLiveProof": False, "oldContinuousLifecycleCompleted": False,
    "ownerCleanupReceiptExists": False, "recordedStopRequestExists": False,
    "originalPersistentDeadlinePairs": original_pairs, "deadlineStateByteUnchanged": True,
    "protectedDataHashes": actual_hashes, "dailyExecutableSha256": daily_hash,
    "candidateSourceCommit": candidate, "candidateExecutable": str(new_image),
    "candidateExecutableSha256": sha(new_image.read_bytes()), "scheduleSourceBlobWitnesses": witnesses,
    "crossStartupRealDeadlineMayBeEvaluatedFromUnchangedPersistentState": True,
    "continuousOldProcessObservationMayBeClaimed": False,
    "candidateNativeBehaviorProvenByOldRecords": False,
    "nextStartupMustPreserveOriginalSuccessAndDeadlineWithoutManualCheck": True,
    "restartEvidenceMustIdentifyNewProcessAndSourceSeparately": True,
    "actualAutomaticTerminalQueryStillRequired": True,
    "rawNoDuplicateNetworkOrQueryCountsProven": False,
    "remainingHumanTriggeredForegroundWindowBeijing": ["2026-10-13T22:50:00+08:00", "2026-10-13T23:10:00+08:00"],
    "currentTurnForegroundInputIssued": False, "candidateStarted": False, "clocksOrTimersChanged": False,
    "originalTaskRecovered": False, "formalTaskReceipt": False, "mainIntegrated": False,
    "realFullCyclesAccepted": False, "pairedPerformanceAccepted": False, "fullNativeGateAccepted": False,
    "releaseAccepted": False,
}
output.mkdir()
for name, value in raw.items():
    (output / name).write_bytes(value)
(output / "actual-processes-and-product-crash-events.json").write_bytes(process_raw + b"\n")
(output / "interruption-and-restart-disposition.json").write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({
    "oldOwnerProcessesAlive": False, "desktopCount": 0, "oldObservationsPreserved": 201,
    "originalDeadlineBytesPreserved": True, "candidateStarted": False, "fullCyclesAccepted": False,
}))
