"""Real final native policy-admission failure, no rotation and same-candidate success."""
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
prepared = native("prepare-manual.json")
assert len(prepared["inventory"]) == 45
assert prepared["policy"] == {"schema_version": 1, "mode": "manual", "keep_recent": 10, "keep_days": 30}
assert prepared["preferences"]["appUpdateChannel"] == "stable" and prepared["preferences"]["appUpdateAutoCheck"] is True
assert prepared["preferences"]["theme"] == "system"
assert prepared["preview"]["candidateIds"] == []
owned_ids = {item["backup"]["backupId"] for item in prepared["created"]}
assert len(owned_ids) == 3
assert all(item["cleanupError"] is None and item["removedIds"] == [] for item in prepared["created"])
optin = native("opt-in.json")
assert optin["automaticPolicy"] == {**prepared["policy"], "mode": "automatic"}
assert optin["policySaveDidNotDelete"] and optin["before"] == optin["after"]
assert set(optin["preview"]["candidateIds"]) == owned_ids
rejected = native("actual-rejection.json")
assert rejected["backupRejected"] and rejected["policyReadRejected"]
assert "unsupported cleanup policy schema or retention" in rejected["backupError"]["message"]
assert "unsupported cleanup policy schema or retention" in rejected["policyReadError"]["message"]
assert rejected["before"] == rejected["after"] == optin["after"]
assert set(rejected["previewAfterFailure"]["candidateIds"]) == owned_ids
assert rejected["preferencesDtoExactlyPreserved"]
success = native("actual-success.json")
assert success["created"]["cleanupError"] is None and set(success["created"]["removedIds"]) == owned_ids
new_id = success["created"]["backup"]["backupId"]
assert new_id not in owned_ids
assert success["preexistingInventoryExactlyPreserved"] and success["preferencesDtoExactlyPreserved"]
assert [item for item in success["inventory"] if item["id"] != new_id] == prepared["inventory"]
assert success["finalPreview"]["candidateIds"] == []
cleanup = native("native-cleanup.json")
assert cleanup["originalPolicyRestored"] and cleanup["policy"] == prepared["policy"]
assert cleanup["preferences"] == prepared["preferences"] and cleanup["nativeBridgeUnchanged"]
assert cleanup["inventory"] == success["inventory"] and cleanup["notifications"] == success["notifications"]
assert cleanup["preview"]["candidateIds"] == []
original_notes = {item["id"]: item for item in prepared["notifications"]}
new_errors = [item for item in rejected["notifications"] if item["id"] not in original_notes
              and item["id"].startswith("preferences-backup-failed:")]
assert len(new_errors) == 1
error = new_errors[0]
assert error["resolved"] is False and error["resolvedAt"] is None
success_notes = {item["id"]: item for item in success["notifications"]}
assert success_notes[error["id"]]["resolved"] is True and success_notes[error["id"]]["resolvedAt"]
for notes in [rejected["notifications"], success["notifications"]]:
    by_id = {item["id"]: item for item in notes}
    assert all(by_id[key] == item for key, item in original_notes.items())
failed_scope = read("failed-event-scope.json")
success_scope = read("success-event-scope.json")
assert failed_scope["object_id"] == success_scope["object_id"]
def partition(scope):
    matches = [item for item in scope["candidates"] if
               (item["object"], item["action"], item["phase"], item["channel"]) ==
               ("backup", "backup", "execution", "local")]
    assert len(matches) == 1
    return matches[0]
candidate = partition(failed_scope)
assert candidate == partition(success_scope) and error["id"].endswith(candidate["id"])
payload = (root / "active-preferences-payload.json").read_bytes()
assert candidate["fingerprint"] == hashlib.sha256(b"manual-preferences\0" + payload).hexdigest()
for phase, resolved in [("failed", False), ("success", True)]:
    raw = read(phase + "-notifications.json")
    item = next(item for item in raw["items"] if item["notification_id"] == error["id"])
    assert item["resolved"] is resolved
    assert item["event_identity"] == {
        "object": "backup", "object_id": failed_scope["object_id"], "action": "backup",
        "phase": "execution", "channel": "local", "candidate": candidate["id"]}
remote_partitions = lambda scope: [item for item in scope["candidates"] if item["object"] in {"manager", "panel"}]
assert remote_partitions(failed_scope) == remote_partitions(success_scope)
before, after = read("filesystem-before.json"), read("filesystem-after.json")
for value in [before, after]:
    assert value["sourceCommit"] == source and value["applicationPid"] == 37276
    assert value["executableSha256"] == image and value["cooperativeWindowsLockHeld"]
assert before["stateHashes"] == after["stateHashes"]
assert before["stateHashes"]["preferences.json"] == hashlib.sha256(payload).hexdigest() == "674ad619b46fc27a273eb37da775660e3b44263fe3485e4ade98287f5fbda8fa"
assert before["stateHashes"]["update-schedule.json"] == "6e51fe41559c287c6036ef495456f1d1d5200710f5f925b3e9e7084621c3df20"
assert after["preexistingBackupFilesAllBytesUnchanged"] and after["preferencesBindingSchedulePolicyAllBytesUnchanged"]
after_files = {item["path"]: item for item in after["backupFiles"]}
assert all(after_files[item["path"]] == item for item in before["backupFiles"])
assert len(after["backupFiles"]) == len(before["backupFiles"]) + 2
date_journal = read("manual-fixture-journal.json")
assert date_journal["onlyNewOwnedIds"] and date_journal["syntheticDatesOnly"] and not date_journal["systemClockChanged"]
assert {item["id"] for item in date_journal["entries"]} == owned_ids
for item in date_journal["entries"]:
    original_manifest = json.loads(bytes.fromhex(item["originalHex"]))
    fixture = json.loads(bytes.fromhex(item["fixtureHex"]))
    fixture["created_at"] = original_manifest["created_at"]
    assert original_manifest == fixture
    assert item["payloadSha256"] == before["stateHashes"]["preferences.json"]
date_restored = read("manual-fixture-restored.json")
assert date_restored["survivingOriginalManifestsRestored"] == []
assert set(date_restored["ownedIdsRemovedByNativeCleanup"]) == owned_ids
policy_journal = read("policy-fixture-journal.json")
assert policy_journal["sourceCommit"] == source and policy_journal["applicationPid"] == 37276
assert policy_journal["onlyIsolatedBackupPolicySchemaChanged"] and not policy_journal["clocksChanged"]
original_policy = json.loads(bytes.fromhex(policy_journal["originalAutomaticHex"]))
fixture_policy = json.loads(bytes.fromhex(policy_journal["fixtureHex"]))
assert original_policy == optin["automaticPolicy"]
assert fixture_policy == {**original_policy, "schema_version": 99}
failed_state, success_state = read("failed-policy-and-cycle.json"), read("success-policy-and-cycle.json")
assert failed_state["policyRawHex"] == policy_journal["fixtureHex"]
assert failed_state["policySha256"] == policy_journal["fixtureSha256"]
assert success_state["policyRawHex"] == policy_journal["originalAutomaticHex"]
assert success_state["policySha256"] == policy_journal["originalAutomaticSha256"]
assert read("policy-exact-repair.json")["originalAutomaticBytesExactlyRestored"]
for checkpoint in [failed_state, success_state, policy_journal, read("policy-exact-repair.json")]:
    assert all(checkpoint["cycleHashes"][name] == before["stateHashes"][name]
               for name in ["preferences.json", "data-root.json", "update-schedule.json"])
life = read("scene-lifecycle.json")
assert life["sourceCommit"] == source and life["applicationPid"] == 37276 and life["desktopApplicationCount"] == 1
assert {item["processId"] for item in life["processReadbacks"]} == {37276, 24300, 3920}
assert all(item["liveHandleVerified"] and not item["hasExited"] for item in life["processReadbacks"])
assert life["applicationToken"]["elevated"] is False and life["applicationToken"]["elevationType"] == 3
assert life["executableSha256"] == image
assert life["dailyExecutableSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert not any(life[key] for key in ["clocksOrTimersChanged", "manualUpdateCheckOrInstallInvoked",
    "pairedPerformanceAccepted", "realCyclesAccepted", "fullNativeGateAccepted", "releaseAccepted"])
assert (root / "methods/filesystem-fixture.py").read_bytes() == (
    root.parent / "OCX-0110-20261012-WIN-EAE-RETENTION/methods/filesystem-fixture.py").read_bytes()
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
    "scope": "real final native invalid-policy Backup failure does not create/rotate; exact repair followed by same-payload success rotates only three new owned fixtures and resolves same local candidate",
    "failedBackupDidNotRotate": True, "sameCandidateUuidAndPayloadVerified": True,
    "preexistingBackupPreferenceBindingPolicyScheduleBytesPreserved": True,
    "fullBackupFailureOrUiMatrixAccepted": False, "allChannelPhaseOrInFlightMatrixAccepted": False,
    "pairedPerformanceAccepted": False, "realCyclesAccepted": False,
    "fullNativeGateAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries), "failedBackupNoRotation": True,
                  "sameCandidateActualResolution": True, "onlyOwnedRotated": 3}))
