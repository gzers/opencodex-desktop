"""Real native linked protection and candidate generation subset."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
image = "f5e4a2eef0ffc364a2a57b47b1463015faf3433a15ff021d544d6e161004f00c"
def read(name):
    return json.loads((root / name).read_text(encoding="utf-8-sig"))
def native(name):
    value = read(name)
    assert value["sourceCommit"] == source and value["applicationPid"] == 37276
    assert value["executableSha256"] == image
    return value["result"]
prepared = native("prepared.json")
assert prepared["policy"] == {"schema_version": 1, "mode": "manual", "keep_recent": 10, "keep_days": 30}
assert prepared["currentPreferences"]["appUpdateChannel"] == "stable"
assert prepared["currentPreferences"]["appUpdateAutoCheck"] is True
damaged_id, healthy_id = [item["backup"]["backupId"] for item in prepared["created"]]
assert damaged_id != healthy_id
rejected = native("rejected-and-protected.json")
assert rejected["damagedId"] == damaged_id and rejected["healthyId"] == healthy_id
assert rejected["beforePreview"]["candidateIds"] == [healthy_id, damaged_id]
assert rejected["cleanupRejected"] and "cleanup candidate SHA-256 mismatch" in rejected["cleanupError"]["message"]
assert rejected["before"] == rejected["afterCleanup"], "No deletion before all candidates verified"
assert rejected["restoreRejected"] and "restore size/SHA-256 mismatch" in rejected["restoreError"]["message"]
failed_guard_id = rejected["failedGuard"]["id"]
assert failed_guard_id not in {damaged_id, healthy_id}
assert rejected["failedGuard"]["protected"] and not rejected["failedGuard"]["pinned"]
source_dto = next(item for item in rejected["afterRestore"] if item["id"] == damaged_id)
assert source_dto["protected"] and source_dto["protectionReason"] == "referenced-by-protection"
assert rejected["afterRestorePreview"]["candidateIds"] == [healthy_id]
assert rejected["activeGuardUnpinRejected"] and "restore guards cannot be released" in rejected["activeGuardUnpinError"]["message"]
assert rejected["stalePreviewRejected"] and "backup inventory changed" in rejected["stalePreviewError"]["message"]
assert rejected["afterRestore"] == rejected["finalInventory"]
assert rejected["preferencesDtoExactlyPreserved"]
other = native("other-candidate-success.json")
returned = native("returned-candidate-success.json")
assert other["candidateId"] == healthy_id and returned["candidateId"] == damaged_id
for value in [other, returned]:
    assert value["actualRestore"]["backupId"] == value["candidateId"]
    assert not value["actualRestore"]["refreshRequired"] and not value["actualRestore"]["protectionReconciliationPending"]
    assert value["preferencesDtoExactlyPreserved"]
    assert value["failedGuard"]["id"] == failed_guard_id and value["failedGuard"]["protected"]
    assert not value["successfulGuard"]["protected"] and not value["successfulGuard"]["pinned"]
assert other["actualRestore"]["protectionBackupId"] != returned["actualRestore"]["protectionBackupId"]
assert returned["storeTheme"] == "system" and returned["storeError"] is False and returned["nativeBridgeUnchanged"]
original_by_id = {item["id"]: item for item in prepared["inventory"]}
final_by_id = {item["id"]: item for item in returned["inventory"]}
assert all(final_by_id[key] == value for key, value in original_by_id.items())
assert len(final_by_id) == len(original_by_id) + 5
old_notes = {item["id"]: item for item in prepared["notifications"]}
new_restore_failures = [item for item in rejected["afterFailureNotifications"]
                        if item["id"] not in old_notes and item["id"].startswith("preferences-restore-failed:")]
assert len(new_restore_failures) == 1
new_failure = new_restore_failures[0]
assert new_failure["resolved"] is False and new_failure["resolvedAt"] is None
for notes in [rejected["afterFailureNotifications"], other["notifications"], returned["notifications"]]:
    indexed = {item["id"]: item for item in notes}
    assert all(indexed[key] == value for key, value in old_notes.items())
    assert indexed[new_failure["id"]] == new_failure
def restore_scope(name):
    value = read(name + "-event-scope.json")
    entries = [entry for entry in value["candidates"] if
               (entry["object"], entry["action"], entry["phase"], entry["channel"]) ==
               ("backup", "restore", "execution", "local")]
    assert len(entries) == 1
    return value, entries[0]
failed_scope, first = restore_scope("failed")
other_scope, second = restore_scope("other")
returned_scope, third = restore_scope("returned")
assert failed_scope["object_id"] == other_scope["object_id"] == returned_scope["object_id"]
assert first["fingerprint"] == third["fingerprint"] == hashlib.sha256(damaged_id.encode()).hexdigest()
assert second["fingerprint"] == hashlib.sha256(healthy_id.encode()).hexdigest()
assert len({first["id"], second["id"], third["id"]}) == 3
assert new_failure["id"].endswith(first["id"])
for phase in ["failed", "other", "returned"]:
    raw = read(phase + "-notifications.json")
    item = next(item for item in raw["items"] if item["notification_id"] == new_failure["id"])
    identity = item["event_identity"]
    assert identity == {"object": "backup", "object_id": failed_scope["object_id"], "action": "restore",
                        "phase": "execution", "channel": "local", "candidate": first["id"]}
    assert not item["resolved"] and item["resolved_at"] is None
# Local scene does not rotate remote manager/panel candidate partitions.
remote = lambda value: [entry for entry in value["candidates"] if entry["object"] in {"manager", "panel"}]
assert remote(failed_scope) == remote(other_scope) == remote(returned_scope)
before = read("filesystem-before.json")
after = read("filesystem-after.json")
for value in [before, after]:
    assert value["sourceCommit"] == source and value["applicationPid"] == 37276
    assert value["executableSha256"] == image and value["cooperativeWindowsLockHeld"]
assert before["stateHashes"] == after["stateHashes"]
assert before["stateHashes"]["preferences.json"] == "674ad619b46fc27a273eb37da775660e3b44263fe3485e4ade98287f5fbda8fa"
assert before["stateHashes"]["update-schedule.json"] == "6e51fe41559c287c6036ef495456f1d1d5200710f5f925b3e9e7084621c3df20"
assert after["allPreexistingBackupFilesExactlyPreserved"] and after["preferencesBindingSchedulePolicyAllBytesUnchanged"]
after_files = {entry["path"]: entry for entry in after["backupFiles"]}
assert all(after_files[entry["path"]] == entry for entry in before["backupFiles"])
assert len(after["backupFiles"]) == len(before["backupFiles"]) + 10
journal = read("fixture-journal.json")
assert journal["onlyNewOwnedIds"] and journal["syntheticDatesOnly"] and journal["cooperativeWindowsLockHeld"]
assert not journal["systemClockChanged"] and len(journal["entries"]) == 2
for entry in journal["entries"]:
    original_manifest = json.loads(bytes.fromhex(entry["manifestOriginalHex"]))
    dated = json.loads(bytes.fromhex(entry["datedManifestHex"]))
    dated["created_at"] = original_manifest["created_at"]
    assert original_manifest == dated
    original_payload = bytes.fromhex(entry["payloadOriginalHex"])
    fixture_payload = bytes.fromhex(entry["payloadFixtureHex"])
    assert original_manifest["sha256"] == hashlib.sha256(original_payload).hexdigest() == entry["payloadSha256"]
    assert len(original_payload) == len(fixture_payload) == original_manifest["bytes"]
    differing = sum(a != b for a, b in zip(original_payload, fixture_payload))
    assert differing == (1 if entry["id"] == damaged_id else 0)
    assert entry["damaged"] == (entry["id"] == damaged_id)
    directory = entry["directory"].replace("\\", "/")
    assert after_files[directory + "/preferences.json"]["sha256"] == entry["payloadSha256"]
    assert after_files[directory + "/backup-manifest.json"]["sha256"] == hashlib.sha256(bytes.fromhex(entry["manifestOriginalHex"])).hexdigest()
assert read("fixture-repair.json")["payloadsExactlyRestored"]
assert read("fixture-restore.json")["payloadsExactlyRestored"] and read("fixture-restore.json")["datesExactlyRestored"]
for phase in ["failed", "other", "returned"]:
    checkpoint = read(phase + "-checkpoint.json")
    assert checkpoint["stateHashes"] == before["stateHashes"]
    manifests = {entry["manifest"]["backup_id"]: entry for entry in checkpoint["newOwnedRecords"]}
    guard = manifests[failed_guard_id]["manifest"]
    assert guard["management"]["transaction_state"] == "active" and not guard["management"]["pinned"]
    assert guard["management"]["related_backup_id"] == damaged_id
    assert manifests[failed_guard_id]["payloadSha256"] == before["stateHashes"]["preferences.json"]
    if phase != "failed":
        result = other if phase == "other" else returned
        success_guard = manifests[result["actualRestore"]["protectionBackupId"]]["manifest"]
        assert success_guard["management"]["transaction_state"] == "committed"
        assert success_guard["management"]["related_backup_id"] == result["candidateId"]
    if phase == "failed":
        assert manifests[damaged_id]["payloadSha256"] != before["stateHashes"]["preferences.json"]
    else:
        assert manifests[damaged_id]["payloadSha256"] == before["stateHashes"]["preferences.json"]
life = read("scene-lifecycle.json")
assert life["sourceCommit"] == source and life["applicationPid"] == 37276
assert life["desktopApplicationCount"] == 1 and life["executableSha256"] == image
assert life["dailyExecutableSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert {entry["processId"] for entry in life["processReadbacks"]} == {37276, 24300, 3920}
assert all(entry["liveHandleVerified"] and not entry["hasExited"] for entry in life["processReadbacks"])
assert life["applicationToken"]["elevated"] is False and life["applicationToken"]["elevationType"] == 3
assert not any(life[key] for key in ["clocksOrTimersChanged", "manualUpdateCheckOrInstallInvoked",
    "fullRetentionSafetyMatrixAccepted", "pairedPerformanceAccepted", "realCyclesAccepted",
    "fullNativeGateAccepted", "releaseAccepted"])
assert (root / "methods/verify-lifecycle.ps1").read_bytes() == (
    root.parent / "OCX-0110-20261012-WIN-EAE-RETENTION/methods/verify-lifecycle.ps1").read_bytes()
files = sorted(p for p in root.rglob("*") if p.is_file() and p.name not in {"manifest.json", "archive-verification.json"})
assert all(p.suffix.lower() not in {".exe", ".dll", ".msi", ".zip"} for p in files)
entries = [{"path": p.relative_to(root).as_posix(), "bytes": p.stat().st_size,
            "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in files]
def write(name, value):
    (root / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "sourceCommit": source, "filesVerified": len(entries),
    "scope": "native all-candidate prevalidation, failed Restore guard/source protection and local candidate UUID rotation/old-generation nonresolution",
    "badSecondCandidateDeletesNoFirstCandidate": True, "failedGuardAndSourceProtected": True,
    "oldFingerprintDoesNotResurrectOldUuid": True, "preexistingBackupAndPreferenceBytesPreserved": True,
    "fullChannelPhaseOrInFlightMatrixAccepted": False, "fullBackupSafetyOrUiAccepted": False,
    "pairedPerformanceAccepted": False, "realCyclesAccepted": False,
    "fullNativeGateAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries), "linkedProtection": "passed",
                  "candidateGenerations": 3, "oldGenerationRemainsUnresolved": True}))
