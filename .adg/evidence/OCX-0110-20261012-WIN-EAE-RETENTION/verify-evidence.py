"""Frozen final-desktop retention subset; neither elapsed D30 nor full acceptance."""
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
def ids(records):
    return {record["backup"]["backupId"] for record in records}
manual_prepare = native("prepare-manual.json")
assert manual_prepare["policy"] == {"schema_version": 1, "mode": "manual", "keep_recent": 10, "keep_days": 30}
assert manual_prepare["recentManualCount"] == 20
assert manual_prepare["preview"]["candidateIds"] == []
assert not manual_prepare["mouseKeyboardUiAccepted"] and not manual_prepare["syntheticDatesYetApplied"]
original = manual_prepare["before"]
assert len(original) == 28
assert any(item["id"] == "bk_20261012044452_a921f503" and item["pinned"] for item in original)
assert any(item["protected"] for item in original)
manual_ids = ids(manual_prepare["created"])
assert len(manual_ids) == 3
manual = native("manual-pin-cleanup.json")
assert "error" not in manual
assert set(manual["ownedIds"]) == set(manual["preview"]["candidateIds"]) == manual_ids
assert set(manual["pinnedPreview"]["candidateIds"]) == manual_ids - {manual["ownedIds"][0]}
assert manual["stalePreviewAccepted"] is False
assert "backup inventory changed" in manual["staleError"]["message"]
assert manual["afterRejected"] == manual["pinnedInventory"]
assert set(manual["freshPreview"]["candidateIds"]) == set(manual["removed"]) == manual_ids
assert manual["after"] == original and manual["preexistingInventoryExactlyPreserved"]
assert manual["finalPreview"]["candidateIds"] == []
auto_prepare = native("prepare-automatic.json")
assert auto_prepare["before"] == original and auto_prepare["preview"]["candidateIds"] == []
auto_ids = ids(auto_prepare["created"])
assert len(auto_ids) == 3 and not auto_ids.intersection(manual_ids)
automatic = native("automatic-cleanup.json")
assert "error" not in automatic
assert set(automatic["ownedIds"]) == set(automatic["preview"]["candidateIds"]) == auto_ids
assert automatic["automaticPolicy"]["mode"] == "automatic"
assert automatic["before"] == automatic["afterPolicySave"] and automatic["policySaveDidNotDelete"]
assert set(automatic["created"]["removedIds"]) == auto_ids and automatic["created"]["cleanupError"] is None
new_id = automatic["created"]["backup"]["backupId"]
assert new_id not in manual_ids | auto_ids
assert [item for item in automatic["after"] if item["id"] != new_id] == original
assert automatic["newlySuccessfulBackupRetained"] and automatic["preexistingInventoryExactlyPreserved"]
assert automatic["originalPolicyRestored"] and automatic["finalPolicy"] == manual_prepare["policy"]
assert automatic["cyclePreferences"]["appUpdateChannel"] == "stable"
assert automatic["cyclePreferences"]["appUpdateAutoCheck"] is True
assert automatic["cyclePreferences"]["theme"] == "system"
assert automatic["finalPreview"]["candidateIds"] == []
before = read("filesystem-before.json")
after = read("filesystem-after.json")
for value in [before, after]:
    assert value["sourceCommit"] == source and value["applicationPid"] == 37276
    assert value["executableSha256"] == image and value["cooperativeWindowsLockHeld"]
assert before["stateHashes"] == after["stateHashes"]
assert before["stateHashes"]["update-schedule.json"] == "6e51fe41559c287c6036ef495456f1d1d5200710f5f925b3e9e7084621c3df20"
assert before["stateHashes"]["preferences.json"] == "674ad619b46fc27a273eb37da775660e3b44263fe3485e4ade98287f5fbda8fa"
assert after["preexistingBackupFilesAllBytesUnchanged"] and after["preferencesBindingSchedulePolicyAllBytesUnchanged"]
after_by_path = {entry["path"]: entry for entry in after["backupFiles"]}
assert all(after_by_path[entry["path"]] == entry for entry in before["backupFiles"])
assert len(after["backupFiles"]) == len(before["backupFiles"]) + 2
for phase, phase_ids in [("manual", manual_ids), ("automatic", auto_ids)]:
    journal = read(phase + "-fixture-journal.json")
    assert journal["sourceCommit"] == source and journal["applicationPid"] == 37276
    assert journal["syntheticDatesOnly"] and journal["onlyNewOwnedIds"] and journal["cooperativeWindowsLockHeld"]
    assert not journal["systemClockChanged"]
    assert {entry["id"] for entry in journal["entries"]} == phase_ids
    for entry in journal["entries"]:
        original_manifest = json.loads(bytes.fromhex(entry["originalHex"]))
        fixture_manifest = json.loads(bytes.fromhex(entry["fixtureHex"]))
        fixture_manifest["created_at"] = original_manifest["created_at"]
        assert fixture_manifest == original_manifest
        assert entry["payloadSha256"] == before["stateHashes"]["preferences.json"]
        assert original_manifest["bytes"] == entry["payloadBytes"]
    restored = read(phase + "-fixture-restored.json")
    assert restored["survivingOriginalManifestsRestored"] == []
    assert set(restored["ownedIdsRemovedByNativeCleanup"]) == phase_ids
first_life = read("scene-lifecycle.json")
assert first_life["sourceCommit"] == source and first_life["applicationPid"] == 37276
assert all(entry["liveHandleVerified"] and not entry["hasExited"] for entry in first_life["processReadbacks"])
assert (root / "scene-lifecycle.stderr.txt").stat().st_size == 0
notes = read("execution-notes.json")
assert notes["firstArtifactAllThreeRealHandlesPassed"] and not notes["backupSceneRerunForWrapperError"]
assert not notes["applicationRestarted"] and not notes["clockOrScheduleChanged"]
life = read("scene-lifecycle-recheck.json")
assert life["sourceCommit"] == source and life["applicationPid"] == 37276
assert life["desktopApplicationCount"] == 1 and life["executableSha256"] == image
assert life["dailyExecutableSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert {entry["processId"] for entry in life["processReadbacks"]} == {37276, 24300, 3920}
assert all(entry["liveHandleVerified"] and not entry["hasExited"] for entry in life["processReadbacks"])
assert life["applicationToken"]["elevated"] is False and life["applicationToken"]["elevationType"] == 3
assert not any(life[key] for key in [
    "clocksOrTimersChanged", "manualUpdateCheckOrInstallInvoked", "physicalDpiUiMatrixAccepted",
    "fullRetentionSafetyMatrixAccepted", "pairedPerformanceAccepted", "realCyclesAccepted",
    "fullNativeGateAccepted", "releaseAccepted"])
files = sorted(p for p in root.rglob("*") if p.is_file() and p.name not in {"manifest.json", "archive-verification.json"})
assert all(p.suffix.lower() not in {".exe", ".dll", ".zip", ".msi"} for p in files)
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
    "scope": "real final ordinary desktop native backup D30 recent retention, old fixture pin exemption, stale-preview rejection and opt-in after-success rotation",
    "onlyNewOwnedOldFixturesRemoved": True, "preexistingBackupsAllBytesPreserved": True,
    "recentN10SlotCountingMatrixAccepted": False, "elapsedD30Accepted": False,
    "retentionUiAccepted": False, "fullSafetyMatrixAccepted": False,
    "pairedPerformanceAccepted": False, "realCyclesAccepted": False,
    "fullNativeGateAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries), "manualOwnedRemoved": 3,
                  "automaticOwnedRemoved": 3, "preexistingBackupsUnchanged": True}))
