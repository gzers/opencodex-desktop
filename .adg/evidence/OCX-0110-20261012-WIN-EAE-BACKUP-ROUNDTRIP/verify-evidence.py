"""Exact final application's command roundtrip; no native-input or full-gate claim."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
native = read("native-backup-roundtrip.json")
assert native["sourceCommit"] == source and native["applicationPid"] == 37276
assert native["executableSha256"] == "f5e4a2eef0ffc364a2a57b47b1463015faf3433a15ff021d544d6e161004f00c"
r = native["result"]
assert "observationError" not in r and not r.get("fallbackPreferenceSaveUsed", False)
assert r["originalPreferences"] == {"theme": "system", "automaticCheck": True, "channel": "stable"}
assert r["manualPolicy"]["mode"] == "manual"
assert len(r["phases"]) == 3
assert [phase["theme"] for phase in r["phases"]] == ["light", "system", "system"]
assert all(phase["automaticCheck"] is True and phase["channel"] == "stable" for phase in r["phases"])
created = r["createdBackup"]
assert created["cleanupError"] is None and created["removedIds"] == []
backup_id = created["backup"]["backupId"]
assert backup_id == r["restoreResult"]["backupId"]
assert r["dtoMatchesOriginal"] and not r["restoreResult"]["refreshRequired"]
assert not r["restoreResult"]["protectionReconciliationPending"]
assert r["pinBeforeRestore"]["pinned"] and r["pinAfterRestore"]["pinned"]
assert not r["protectionAfterRestore"]["protected"]
assert r["beforeNotifications"] == r["afterNotifications"]
cleanup = r["cleanup"]
assert cleanup["originalDtoRestored"] and cleanup["temporaryPinRemoved"] and cleanup["originalUserPinRetained"]
assert not cleanup["storeError"] and cleanup["storeTheme"] == "system"
assert cleanup["policy"]["mode"] == "manual" and cleanup["invokeBridgeUnchanged"]
fs = read("filesystem-verification.json")
assert fs["sourceCommit"] == source and fs["nativeHelperExitCode"] == 0
assert fs["originalPreferenceBytesRestored"] and fs["preferenceBeforeSha256"] == fs["preferenceAfterSha256"]
assert fs["realCycleDeadlineBytesUnchanged"] and fs["scheduleBeforeSha256"] == fs["scheduleAfterSha256"]
assert not fs["actualMouseKeyboardInput"] and fs["actualNativeCommandExecution"]
assert not fs["packageLifecycleScriptsExecuted"] and not fs["installerExecuted"] and not fs["cycleAccepted"]
for name in ["schedule-before.json", "schedule-after.json"]:
    data = (root / name).read_bytes()
    assert hashlib.sha256(data).hexdigest() == fs["scheduleBeforeSha256"]
    schedule = read(name)
    assert schedule["entries"]["manager-stable"]["next_due"] == 1791903339
    assert schedule["entries"]["panel-latest"]["next_due"] == 1791903340
backup = read("new-backup-manifest.json")
protection = read("new-protection-manifest.json")
assert backup["backup_id"] == backup_id and backup["sha256"] == fs["preferenceBeforeSha256"]
assert backup["management"]["pinned"] is False and backup["management"]["kind"] == "standalone"
assert protection["backup_id"] == r["restoreResult"]["protectionBackupId"]
assert protection["management"]["related_backup_id"] == backup_id
assert protection["management"]["transaction_state"] == "committed"
assert protection["sha256"] != backup["sha256"]
independent = read("independent-payload-and-lifecycle.json")
assert independent["sourceCommit"] == source and independent["applicationPid"] == 37276
assert independent["monitorPid"] == 3920 and independent["monitorExecSession"] == 46484
assert independent["actualAppAndMonitorLiveHandlesVerified"] and independent["desktopCount"] == 1
assert independent["backupPayloadSha256"] == backup["sha256"] and independent["originalPayloadCapturedExactly"]
assert independent["protectionPayloadSha256"] == protection["sha256"]
assert independent["originalPayloadTheme"] == "system" and independent["protectionPayloadTheme"] == "light"
assert independent["bothSnapshotAutomaticCheckTrue"]
assert independent["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert not independent["finalCycleAccepted"] and not independent["uiRoundtripAccepted"]
assert not independent["newRealUserDataMigrated"] and not independent["fullGateAccepted"]
excluded = {"manifest.json", "archive-verification.json"}
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in excluded)
assert all(path.suffix.lower() not in {".exe", ".dll", ".msi", ".cab", ".zip"} for path in files)
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
    "result": "passed", "sourceCommit": source,
    "scope": "real final ordinary application different-theme native-command backup/pin/restore and verified unchanged actual cycle deadlines",
    "currentBackupRoundtrip": "passed for recorded commands",
    "nativeMouseKeyboardUiAccepted": False, "fullBackupGateAccepted": False,
    "pairedPerformanceAccepted": False, "realCyclesAccepted": False,
    "fullNativeGateAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
    "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "finalSourceCommandRestore": "passed", "files": len(entries)}))
