"""Real final ordinary desktop N10 plus newest pin, not full UI/cycle acceptance."""
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
assert len(prepared["before"]) == 34
assert prepared["policy"] == {"schema_version": 1, "mode": "manual", "keep_recent": 10, "keep_days": 30}
temporary = prepared["temporaryOriginalPins"]
assert len(temporary) == len(set(temporary)) == 16
expected_temporary = {item["id"] for item in prepared["before"]
                      if item["action"] == "manual-preferences" and not item["pinned"] and not item["protected"]}
assert set(temporary) == expected_temporary
before_by_id = {item["id"]: item for item in prepared["before"]}
after_pin_by_id = {item["id"]: item for item in prepared["afterOriginalPins"]}
assert set(before_by_id) == set(after_pin_by_id)
for key, item in before_by_id.items():
    after = after_pin_by_id[key]
    if key in temporary:
        assert after["pinned"] and after["protected"] and after["protectionReason"] == "pinned"
        assert all(after[field] == item[field] for field in item if field not in {"pinned", "protected", "protectionReason"})
    else:
        assert after == item
assert not any(item["action"] == "manual-preferences" and not item["pinned"] and not item["protected"]
               for item in prepared["afterOriginalPins"])
owned_ids = [result["backup"]["backupId"] for result in prepared["created"]]
assert len(owned_ids) == len(set(owned_ids)) == 14
assert not set(owned_ids).intersection(before_by_id)
assert all(result["removedIds"] == [] and result["cleanupError"] is None for result in prepared["created"])
assert prepared["preview"]["candidateIds"] == []
count = native("count-and-pin.json")
assert count["ownedIds"] == owned_ids and count["newestId"] == owned_ids[13]
assert count["beforePinPreview"]["candidateIds"] == list(reversed(owned_ids[:4]))
assert count["afterPinPreview"]["candidateIds"] == list(reversed(owned_ids[:3]))
assert count["stalePreviewRejected"] and "backup inventory changed" in count["stalePreviewError"]["message"]
assert count["afterRejected"] == count["afterNewestPin"]
assert count["freshPreview"]["candidateIds"] == count["removed"] == list(reversed(owned_ids[:3]))
assert count["ordinaryOwnedRetained"] == 10 and count["pinnedOwnedRetained"] == 1
assert len(count["retainedOwned"]) == 11
retained = {item["id"]: item for item in count["retainedOwned"]}
assert set(retained) == set(owned_ids[3:])
assert retained[owned_ids[13]]["pinned"] and retained[owned_ids[13]]["protectionReason"] == "pinned"
assert all(not retained[key]["pinned"] and not retained[key]["protected"] for key in owned_ids[3:13])
assert count["finalPreview"]["candidateIds"] == []
cleanup = native("cleanup-pins.json")
assert set(cleanup["restoredOriginalPinIds"]) == set(temporary)
assert cleanup["originalInventoryExactlyRestored"] and cleanup["newestTemporaryPinRemoved"]
assert cleanup["nativeBridgeUnchanged"] and cleanup["policy"] == prepared["policy"]
assert cleanup["currentPreferences"]["appUpdateChannel"] == "stable"
assert cleanup["currentPreferences"]["appUpdateAutoCheck"] is True
assert cleanup["currentPreferences"]["theme"] == "system"
assert cleanup["finalPreview"]["candidateIds"] == []
final_by_id = {item["id"]: item for item in cleanup["after"]}
assert all(final_by_id[key] == item for key, item in before_by_id.items())
assert len(final_by_id) == len(before_by_id) + 11
assert all(not final_by_id[key]["pinned"] for key in owned_ids[3:])
assert final_by_id["bk_20261012044452_a921f503"]["pinned"]
before = read("filesystem-before.json")
after = read("filesystem-after.json")
for value in [before, after]:
    assert value["sourceCommit"] == source and value["applicationPid"] == 37276
    assert value["executableSha256"] == image and value["cooperativeWindowsLockHeld"]
assert before["stateHashes"] == after["stateHashes"]
assert before["stateHashes"]["preferences.json"] == "674ad619b46fc27a273eb37da775660e3b44263fe3485e4ade98287f5fbda8fa"
assert before["stateHashes"]["update-schedule.json"] == "6e51fe41559c287c6036ef495456f1d1d5200710f5f925b3e9e7084621c3df20"
assert after["allOriginalBackupFilesExactlyRestored"] and after["allSurvivingNewManifestsExactlyRestored"]
assert after["preferencesBindingSchedulePolicyAllBytesUnchanged"]
original_files = {entry["path"]: entry for entry in before["backupFiles"]}
final_files = {entry["path"]: entry for entry in after["backupFiles"]}
assert all(final_files[key] == value for key, value in original_files.items())
assert len(final_files) == len(original_files) + 22
for name, raw_hex in before["originalManifestBytes"].items():
    assert hashlib.sha256(bytes.fromhex(raw_hex)).hexdigest() == original_files[name]["sha256"]
journal = read("fixture-journal.json")
assert journal["sourceCommit"] == source and journal["onlyNewOwnedDatesChanged"] and not journal["systemClockChanged"]
assert journal["cooperativeWindowsLockHeld"] and len(journal["entries"]) == 14
for index, entry in enumerate(journal["entries"]):
    assert entry["id"] == owned_ids[index] and entry["index"] == index
    original_manifest = json.loads(bytes.fromhex(entry["manifestOriginalHex"]))
    fixture = json.loads(bytes.fromhex(entry["fixtureHex"]))
    assert fixture["created_at"] == f"2026-09-01T00:00:{index:02d}Z"
    fixture["created_at"] = original_manifest["created_at"]
    assert original_manifest == fixture and original_manifest["management"]["pinned"] is False
    assert entry["payloadSha256"] == before["stateHashes"]["preferences.json"]
    directory = entry["directory"].replace("\\", "/")
    assert directory + "/backup-manifest.json" not in original_files
    if entry["id"] in retained:
        assert final_files[directory + "/backup-manifest.json"]["sha256"] == hashlib.sha256(bytes.fromhex(entry["manifestOriginalHex"])).hexdigest()
        assert final_files[directory + "/preferences.json"]["sha256"] == entry["payloadSha256"]
    else:
        assert not any(key.startswith(directory + "/") for key in final_files)
restored = read("fixture-dates-restored.json")
assert set(restored["survivingOwnedDatesRestored"]) == set(owned_ids[3:])
assert set(restored["missingOwnedDirectories"]) == set(owned_ids[:3])
assert restored["genuineNativePinStillPreserved"] == [owned_ids[13]]
assert set(after["survivingOwnedIds"]) == set(owned_ids[3:]) and set(after["missingOwnedIds"]) == set(owned_ids[:3])
ui = read("ui-admission-limitation.json")
assert ui["sourceCommit"] == source and ui["applicationPid"] == 37276 and ui["windowId"] == 1051698
assert "0x80070005" in ui["activationError"]
assert not any(ui[key] for key in ["afterDisconnectionFurtherUiInputs", "updateEntryClicked",
                                 "updateDetailsAccepted", "checkInstallOrRestartClicked"])
life = read("scene-lifecycle.json")
assert life["sourceCommit"] == source and life["applicationPid"] == 37276 and life["desktopApplicationCount"] == 1
assert life["executableSha256"] == image
assert life["dailyExecutableSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert {entry["processId"] for entry in life["processReadbacks"]} == {37276, 24300, 3920}
assert all(entry["liveHandleVerified"] and not entry["hasExited"] for entry in life["processReadbacks"])
assert life["applicationToken"]["elevated"] is False and life["applicationToken"]["elevationType"] == 3
assert not any(life[key] for key in ["clocksOrTimersChanged", "manualUpdateCheckOrInstallInvoked",
    "fullRetentionSafetyMatrixAccepted", "pairedPerformanceAccepted", "realCyclesAccepted", "fullNativeGateAccepted", "releaseAccepted"])
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
    "scope": "real final ordinary desktop fourteen all-old owned records: newest pin excluded from N10, ten ordinary plus one pin, exact original pin and byte restoration",
    "oldOrdinaryN10PlusNewestPin": True, "onlyOwnedThreeRemoved": True,
    "allOriginalBackupFilesExactlyRestored": True,
    "datesAreSyntheticNotElapsedD30": True, "uiOrUpdateDetailsAccepted": False,
    "fullBackupSafetyMatrixAccepted": False, "pairedPerformanceAccepted": False,
    "realCyclesAccepted": False, "fullNativeGateAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries), "oldOrdinaryRetained": 10,
                  "newestPinnedExtra": 1, "originalTemporaryPinsRestored": 16, "onlyOwnedRemoved": 3}))
