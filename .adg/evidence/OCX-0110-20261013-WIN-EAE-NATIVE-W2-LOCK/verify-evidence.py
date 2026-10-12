"""Actual six-command native Windows cooperative lock subset."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
image = "f5e4a2eef0ffc364a2a57b47b1463015faf3433a15ff021d544d6e161004f00c"
phases = ["create", "list", "pin", "policy", "restore", "cleanup"]
commands = ["create_preferences_backup", "list_preferences_backups", "set_preferences_backup_pinned",
            "save_preferences_backup_policy", "restore_preferences_backup", "cleanup_preferences_backups"]
def read(name):
    return json.loads((root / name).read_text(encoding="utf-8-sig"))
def native(name):
    value = read(name)
    assert value["sourceCommit"] == source and value["applicationPid"] == 37276
    assert value["executableSha256"] == image and value["screenshot"] is None
    return value["result"]
setup = native("setup.json")
assert len(setup["inventory"]) == 46 and setup["phases"] == phases
assert setup["preferences"]["theme"] == "system" and setup["preferences"]["appUpdateChannel"] == "stable"
assert setup["preferences"]["appUpdateAutoCheck"] is True
assert setup["policy"] == {"schema_version": 1, "mode": "manual", "keep_recent": 10, "keep_days": 30}
source_id = setup["created"]["backup"]["backupId"]
assert setup["created"]["removedIds"] == [] and setup["created"]["cleanupError"] is None
controller = read("controller-state.json")
assert controller["sourceCommit"] == source and controller["applicationPid"] == 37276
assert controller["status"] == "six-real-native-lock-phases-completed" and controller["controllerPid"] == 45304
assert [item["phase"] for item in controller["phases"]] == phases
assert not any(controller[key] for key in ["uiInputPerformed", "productRestarted", "secondDesktopStarted",
                                        "fullNativeGateAccepted", "pairedPerformanceAccepted", "realCyclesAccepted", "releaseAccepted"])
results = {}
for index, (phase, command, item) in enumerate(zip(phases, commands, controller["phases"])):
    prepared = native(phase + "-prepared.json")
    assert prepared["phase"] == phase and prepared["sourceBackupId"] == source_id
    assert prepared["originalPreferences"] == setup["preferences"] and prepared["originalPolicy"] == setup["policy"]
    assert prepared["currentPreferences"]["theme"] == ("light" if phase == "restore" else "system")
    assert prepared["currentPreferences"]["appUpdateChannel"] == "stable" and prepared["currentPreferences"]["appUpdateAutoCheck"] is True
    ready = read(phase + "-holder-ready.json")
    releasing = read(phase + "-holder-releasing.json")
    assert ready["sourceCommit"] == source and ready["applicationPid"] == 37276 and ready["executableSha256"] == image
    assert ready["phase"] == phase and ready["realWindowsExclusiveLockHeld"]
    assert ready["helperToken"] == {"elevated": False, "elevationType": 3} == item["holderToken"]
    assert ready["helperPid"] != 37276
    assert ready["helperPid"] == item["holderInitialLiveHandle"]["pid"]
    assert ready["helperCreationFileTime100ns"] == item["holderInitialLiveHandle"]["creationFileTime100ns"]
    assert releasing["releaseRequested"] and releasing["releaseRequestAtMonotonicNs"] > ready["lockAcquiredAtMonotonicNs"]
    first = None
    for number in range(item["pendingObservations"]):
        witness = native(phase + "-pending-" + str(number) + ".json")
        assert witness["phaseIndex"] == index and witness["preparedPhase"] == phase
        if witness["operation"]:
            assert witness["operation"]["phase"] == phase and witness["operation"]["command"] == command
            assert witness["operation"]["status"] == "pending" and witness["operation"]["finishedAtMs"] is None
            if first is None:
                first = witness
    assert first is not None
    second = native(phase + "-pending-stable.json")
    assert second["operation"]["status"] == "pending" and second["operation"]["phase"] == phase
    assert second["operation"]["startedAtMs"] == first["operation"]["startedAtMs"]
    assert second["operation"]["finishedAtMs"] is None
    assert second["atMs"] - first["atMs"] == item["actualWebviewPendingMs"] >= 1000
    during = read(phase + "-while-held-files.json")
    assert during["observerAcquiredNoBackupLock"]
    assert during["backupFiles"] == ready["filesAtLockAcquisition"]["backupFiles"]
    assert during["stateHashes"] == ready["filesAtLockAcquisition"]["stateHashes"]
    assert item["backupPreferenceRootPolicyScheduleUnchangedWhileHeld"]
    for label in ["holder", "nativeCaller"]:
        initial = item[label + "InitialLiveHandle"]
        held = item[label + "BeforeReleaseLiveHandle"]
        terminal = item[label + "TerminalHandle"]
        assert initial["liveHandleVerified"] and held["liveHandleVerified"]
        assert initial["pid"] == held["pid"] == terminal["pid"] != 37276
        assert initial["creationFileTime100ns"] == held["creationFileTime100ns"] == terminal["creationFileTime100ns"]
        assert initial["exitFileTime100ns"] == held["exitFileTime100ns"] == 0
        assert not terminal["liveHandleVerified"] and terminal["exitFileTime100ns"] > terminal["creationFileTime100ns"]
        assert item[label + "ExitCode"] == 0
    result = native(phase + "-result.json")
    assert result["nativeBridgeUnchanged"]
    operation = result["operation"]
    assert operation["phase"] == phase and operation["command"] == command and operation["status"] == "completed"
    assert operation["startedAtMs"] == first["operation"]["startedAtMs"]
    assert operation["finishedAtMs"] > second["atMs"]
    assert operation["finishedAtMs"] - operation["startedAtMs"] == item["actualNativeElapsedMs"]
    assert item["actualNativeElapsedMs"] >= item["actualWebviewPendingMs"]
    assert item["status"] == "actual-native-command-completed-after-independent-release"
    results[phase] = operation
create_id = results["create"]["result"]["backup"]["backupId"]
assert create_id != source_id
assert results["create"]["result"]["removedIds"] == [] and results["create"]["result"]["cleanupError"] is None
assert len(results["list"]["result"]) == 48
assert any(item["id"] == source_id and not item["pinned"] for item in results["list"]["result"])
policy_prepared = native("policy-prepared.json")
assert next(item for item in policy_prepared["inventory"] if item["id"] == source_id)["pinned"]
restored = results["restore"]["result"]
assert restored["backupId"] == source_id
assert not restored["refreshRequired"] and not restored["protectionReconciliationPending"]
protection_id = restored["protectionBackupId"]
assert protection_id not in {source_id, create_id}
cleanup_prepared = native("cleanup-prepared.json")
assert cleanup_prepared["currentPreferences"] == setup["preferences"]
owned_ids = {item["backup"]["backupId"] for item in cleanup_prepared["created"]}
assert len(owned_ids) == 3
assert owned_ids.isdisjoint({source_id, create_id, protection_id})
preview = native("cleanup-preview.json")
assert set(preview["ownedIds"]) == set(preview["preview"]["candidateIds"]) == owned_ids
assert set(results["cleanup"]["result"]) == owned_ids
finished = native("native-finish.json")
assert [item["phase"] for item in finished["actualPhases"]] == phases
assert finished["actualPhases"] == [results[phase] for phase in phases]
assert finished["preferences"] == setup["preferences"] and finished["policy"] == setup["policy"]
assert finished["sourceTemporaryPinRemoved"] and finished["originalInventoryExactlyPreserved"]
assert not finished["fallbackPreferencesSaveUsed"]
assert finished["storeTheme"] == "system" and finished["storeError"] is False and finished["nativeBridgeUnchanged"]
original_inventory = {item["id"]: item for item in setup["inventory"]}
final_inventory = {item["id"]: item for item in finished["inventory"]}
assert all(final_inventory[key] == item for key, item in original_inventory.items())
assert len(final_inventory) == len(original_inventory) + 3
assert set(final_inventory) - set(original_inventory) == {source_id, create_id, protection_id}
assert not final_inventory[source_id]["pinned"] and not final_inventory[protection_id]["protected"]
assert final_inventory["bk_20261012044452_a921f503"]["pinned"]
before, after = read("filesystem-before.json"), read("filesystem-after.json")
for value in [before, after]:
    assert value["sourceCommit"] == source and value["applicationPid"] == 37276
    assert value["executableSha256"] == image and value["cooperativeWindowsLockHeld"]
assert after["originalBackupPreferenceBindingPolicyScheduleBytesExactlyPreserved"]
assert before["stateHashes"] == after["stateHashes"]
original_hash = before["stateHashes"]["preferences.json"]
assert original_hash == "674ad619b46fc27a273eb37da775660e3b44263fe3485e4ade98287f5fbda8fa"
assert before["stateHashes"]["update-schedule.json"] == "6e51fe41559c287c6036ef495456f1d1d5200710f5f925b3e9e7084621c3df20"
after_files = {item["path"]: item for item in after["backupFiles"]}
assert all(after_files[item["path"]] == item for item in before["backupFiles"])
assert len(after["backupFiles"]) == len(before["backupFiles"]) + 6
restore_holder = read("restore-holder-ready.json")
assert restore_holder["filesAtLockAcquisition"]["stateHashes"]["preferences.json"] != original_hash
for phase in phases:
    value = read(phase + "-holder-ready.json")["filesAtLockAcquisition"]["stateHashes"]
    for name in ["data-root.json", "update-schedule.json", "backup-cleanup-policy.json"]:
        assert value[name] == before["stateHashes"][name]
    if phase != "restore":
        assert value["preferences.json"] == original_hash
owned = read("owned-new-backups.json")
assert owned["sourceCommit"] == source and owned["applicationPid"] == 37276 and owned["cooperativeWindowsLockHeld"]
records = {item["manifest"]["backup_id"]: item for item in owned["entries"]}
assert set(records) == {source_id, create_id, protection_id}
for key in [source_id, create_id]:
    assert records[key]["payloadSha256"] == original_hash and records[key]["payloadTheme"] == "system"
    assert not records[key]["manifest"]["management"]["pinned"]
protection = records[protection_id]
assert protection["payloadTheme"] == "light" and protection["payloadSha256"] != original_hash
assert protection["payloadSha256"] == restore_holder["filesAtLockAcquisition"]["stateHashes"]["preferences.json"]
assert protection["manifest"]["management"] == {
    "version": 2, "kind": "restore-protection", "pinned": False,
    "related_backup_id": source_id, "transaction_state": "committed"}
date_journal = read("cleanup-date-journal.json")
assert date_journal["onlyNewOwnedDatesChanged"] and not date_journal["systemClockChanged"]
assert {item["id"] for item in date_journal["entries"]} == owned_ids
for entry in date_journal["entries"]:
    original = json.loads(bytes.fromhex(entry["originalHex"]))
    fixture = json.loads(bytes.fromhex(entry["fixtureHex"]))
    fixture["created_at"] = original["created_at"]
    assert original == fixture and entry["payloadSha256"] == original_hash
date_restored = read("cleanup-dates-restored.json")
assert date_restored["survivingOwnedDatesExactlyRestored"] == []
assert set(date_restored["ownedIdsRemovedByNativeCommand"]) == owned_ids
life = read("scene-lifecycle.json")
assert life["sourceCommit"] == source and life["applicationPid"] == 37276 and life["desktopApplicationCount"] == 1
assert {item["processId"] for item in life["processReadbacks"]} == {37276, 24300, 3920}
assert all(item["liveHandleVerified"] and not item["hasExited"] for item in life["processReadbacks"])
assert life["applicationToken"]["elevated"] is False and life["applicationToken"]["elevationType"] == 3
assert life["executableSha256"] == image
assert life["dailyExecutableSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert not any(life[key] for key in ["clocksOrTimersChanged", "manualUpdateCheckOrInstallInvoked",
    "pairedPerformanceAccepted", "realCyclesAccepted", "fullNativeGateAccepted", "releaseAccepted"])
terminal = read("controller-terminal.json")
assert terminal["controllerPid"] == 45304 and terminal["execSession"] == 74199 and terminal["toolTerminalExitCode"] == 0
assert terminal["finalStdout"] == {"status": "six-real-native-lock-phases-completed", "actualPhases": 6}
assert not terminal["helperKilledToUnblockNativeCommands"] and not terminal["productOrCycleRestarted"]
files = sorted(p for p in root.rglob("*") if p.is_file() and p.name not in {"manifest.json", "archive-verification.json"})
assert all(p.suffix.lower() not in {".exe", ".dll", ".msi", ".zip", ".pending"} for p in files)
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
    "scope": "six real native ordinary Windows W2 commands pending under independently held cooperative lock; files unchanged while held, natural-release success and exact original bytes",
    "actualNativeCommandPhases": phases, "ordinaryHolderActualTokensVerified": True,
    "nonCooperatingWritersAccepted": False, "killedHolderRecoveryAccepted": False,
    "fullBackupSafetyOrUiAccepted": False, "adminLockMatrixAccepted": False,
    "pairedPerformanceAccepted": False, "realCyclesAccepted": False,
    "fullNativeGateAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries), "actualNativeCommands": 6,
                  "ordinaryWindowsCrossProcessCooperativeLock": "passed", "originalBytesPreserved": True}))
