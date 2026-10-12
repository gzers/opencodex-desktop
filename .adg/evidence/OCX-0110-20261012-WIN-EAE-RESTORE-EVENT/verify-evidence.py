"""Current final application Restore recovery, with other-action isolation."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
def native(name):
    value = read(name)
    assert value["sourceCommit"] == source and value["applicationPid"] == 37276
    assert value["executableSha256"] == "f5e4a2eef0ffc364a2a57b47b1463015faf3433a15ff021d544d6e161004f00c"
    return value["result"]
def indexed(items):
    return {item["id"]: item for item in items}
prepare = native("native-preparation.json")
failure = native("native-failure-and-unrelated-success.json")
success = native("native-matching-success.json")
backup_id = prepare["backupId"]
assert failure["backupId"] == success["backupId"] == backup_id
assert prepare["preferences"] == success["preferences"] == {
    "channel": "stable", "automaticCheck": True, "theme": "system",
}
assert prepare["created"]["removedIds"] == [] and prepare["created"]["cleanupError"] is None
assert failure["rejected"] and failure["failedRestorePreferenceDtoUnchanged"]
unrelated = failure["successfulOtherAction"]
assert unrelated["cleanupError"] is None and unrelated["removedIds"] == []
assert unrelated["backup"]["backupId"] != backup_id
old = indexed(prepare["beforeNotifications"])
failed = indexed(failure["afterFailure"])
other_success = indexed(failure["afterSuccessfulOtherAction"])
recovered = indexed(success["notifications"])
new = failed.keys() - old.keys()
assert len(new) == 1
event_id = new.pop()
assert event_id.startswith("preferences-restore-failed:")
assert failed[event_id]["resolved"] is False and failed[event_id]["resolvedAt"] is None
assert failed == other_success
assert recovered[event_id]["resolved"] is True and recovered[event_id]["resolvedAt"]
for old_id, item in old.items():
    assert failed[old_id] == item and recovered[old_id] == item
assert success["invokeBridgeUnchanged"] and not success["storeError"] and success["storeTheme"] == "system"
assert success["originalUserPin"]["pinned"] and not success["protection"]["protected"]
assert not success["actualRestore"]["refreshRequired"]
assert not success["actualRestore"]["protectionReconciliationPending"]
assert success["actualRestore"]["backupId"] == backup_id
fs = read("filesystem-and-fixture-verification.json")
assert fs["sourceCommit"] == source and fs["applicationPid"] == 37276
assert fs["failureHelperExitCode"] == fs["successHelperExitCode"] == 0
assert fs["originalActiveBytesPreserved"] and fs["originalPayloadBytesRepaired"]
assert fs["activePreferencesBeforeSha256"] == fs["afterFailedRestoreSha256"] == fs["afterActualMatchingRestoreSha256"]
assert fs["originalPayloadSha256"] == fs["payloadSha256AfterRepair"] == fs["activePreferencesBeforeSha256"]
assert fs["newRealCycleDeadlineBytesUnchanged"] and fs["scheduleBeforeSha256"] == fs["scheduleAfterSha256"]
assert not any(fs[name] for name in [
    "manuallyResolvedNotifications", "eventsInjected", "mouseKeyboardUiInput", "fullGateAccepted",
])
for name in ["schedule-before.json", "schedule-after.json"]:
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == fs["scheduleBeforeSha256"]
    value = read(name)["entries"]
    assert value["manager-stable"]["next_due"] == 1791903339
    assert value["panel-latest"]["next_due"] == 1791903340
manifest = read("fixture-original-manifest.json")
assert manifest["backup_id"] == backup_id and manifest["sha256"] == fs["originalPayloadSha256"]
before = read("event-scope-after-failure.json")
after = read("event-scope-after-matching-success.json")
def restore_slot(value):
    matches = [item for item in value["candidates"] if (
        item["object"], item["action"], item["phase"], item["channel"]
    ) == ("backup", "restore", "execution", "local")]
    assert len(matches) == 1
    return matches[0]
assert before["object_id"] == after["object_id"]
assert restore_slot(before) == restore_slot(after)
slot = restore_slot(before)
assert slot["fingerprint"] == hashlib.sha256(backup_id.encode()).hexdigest()
assert event_id.endswith(slot["id"])
calculated = read("independent-candidate-recalculation.json")
assert calculated["restoreSlotIdentical"] and calculated["fingerprintMatchesActualBackupId"]
assert calculated["restoreCandidateId"] == slot["id"]
assert not calculated["wholeScopeFilesEqual"]
assert calculated["changedOtherSlots"] == [{
    "object": "backup", "action": "backup", "phase": "execution", "channel": "local",
}]
independent = read("independent-final-check.json")
assert independent["sourceCommit"] == source and independent["applicationPid"] == 37276
assert independent["actualApplicationToken"]["processId"] == 37276
assert independent["actualApplicationToken"]["elevated"] is False
assert independent["actualApplicationToken"]["elevationType"] == 3
assert independent["monitorPid"] == 3920 and independent["monitorExecSession"] == 46484
assert independent["originalFinalLifecycleLiveHandlesVerified"] and independent["applicationCount"] == 1
assert independent["fixturePayloadMatchesOriginal"]
assert independent["independentFixturePayloadSha256"] == fs["originalPayloadSha256"]
assert independent["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert not any(independent[name] for name in [
    "realUserDataMigrated", "newInstallerExecuted", "scriptPermissionGranted",
    "newCycleAccepted", "wholeEventGateAccepted", "releaseAccepted",
])
excluded = {"manifest.json", "archive-verification.json"}
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in excluded)
assert all(path.suffix.lower() not in {".exe", ".msi", ".zip", ".dll"} for path in files)
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
    "scope": "actual final ordinary application Restore/local/execution failure, other Backup-action success isolation and continuous same-candidate actual recovery",
    "sameCandidateEventRecovery": "passed for recorded scene",
    "fullEventGateAccepted": False, "nativeInputUiAccepted": False,
    "pairedPerformanceAccepted": False, "realCyclesAccepted": False,
    "fullNativeGateAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
    "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "currentRestoreEventRecovery": "passed", "files": len(entries)}))
