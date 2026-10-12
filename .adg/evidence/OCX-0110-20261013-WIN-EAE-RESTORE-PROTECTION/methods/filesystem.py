"""Cooperative, atomic mutation of only two newly-owned backup records.
Active preferences, root binding, schedule and original records never mutate.
"""
import ctypes
import ctypes.wintypes as wt
import hashlib
import json
import msvcrt
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

evidence = Path(__file__).resolve().parents[1]
base = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\windows0110")
root = base / "data"
def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))
def write(name, value):
    (evidence / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
def checked(path):
    assert path.exists() and path.resolve().is_relative_to(root.resolve())
    for part in [path, *path.parents]:
        assert not part.is_symlink() and not (part.stat().st_file_attributes & 0x400)
        if part == root:
            break
    return path
def inventory():
    return [{"path": p.relative_to(root).as_posix(), "bytes": p.stat().st_size, "sha256": sha(checked(p))}
            for p in sorted((root / "backups").rglob("*")) if p.is_file()]
def hashes():
    return {name: sha(checked(root / "manager-state" / name))
            for name in ["preferences.json", "data-root.json", "update-schedule.json", "backup-cleanup-policy.json"]}
def atomic(path, data):
    checked(path)
    pending = path.with_name(path.name + ".eae-protection-fixture-pending")
    assert not pending.exists()
    with pending.open("xb") as output:
        output.write(data)
        output.flush()
        os.fsync(output.fileno())
    os.replace(pending, path)
class Overlapped(ctypes.Structure):
    _fields_ = [("Internal", ctypes.c_size_t), ("InternalHigh", ctypes.c_size_t),
                ("Offset", wt.DWORD), ("OffsetHigh", wt.DWORD), ("hEvent", wt.HANDLE)]
kernel = ctypes.WinDLL("kernel32", use_last_error=True)
kernel.LockFileEx.argtypes = [wt.HANDLE, wt.DWORD, wt.DWORD, wt.DWORD, wt.DWORD, ctypes.POINTER(Overlapped)]
kernel.UnlockFileEx.argtypes = [wt.HANDLE, wt.DWORD, wt.DWORD, wt.DWORD, ctypes.POINTER(Overlapped)]
session = read(base / "control/current-session.json")
assert session["sourceCommit"] == "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
assert session["applicationPid"] == 37276 and session["status"] == "running"
stamp = {"atUtc": datetime.now(timezone.utc).isoformat(), "sourceCommit": session["sourceCommit"],
         "applicationPid": 37276, "executableSha256": session["executableSha256"]}
mode = sys.argv[1]
assert mode in {"before", "stage", "repair", "restore", "checkpoint", "after"}
with checked(root / ".backup-w2.lock").open("r+b") as guard:
    handle = wt.HANDLE(msvcrt.get_osfhandle(guard.fileno()))
    overlap = Overlapped()
    if not kernel.LockFileEx(handle, 3, 0, 0xffffffff, 0xffffffff, ctypes.byref(overlap)):
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        if mode == "before":
            assert not (evidence / "filesystem-before.json").exists()
            write("filesystem-before.json", {**stamp, "stateHashes": hashes(), "backupFiles": inventory(),
                                            "cooperativeWindowsLockHeld": True})
        else:
            original = read(evidence / "filesystem-before.json")
            assert hashes() == original["stateHashes"], "Live cycle or preference bytes changed"
            if mode == "stage":
                assert not (evidence / "fixture-journal.json").exists()
                prepared = read(evidence / "prepared.json")
                assert prepared["sourceCommit"] == stamp["sourceCommit"] and prepared["applicationPid"] == 37276
                records = prepared["result"]["created"]
                assert len(records) == 2
                previous = {entry["path"] for entry in original["backupFiles"]}
                entries = []
                for index, item in enumerate(records):
                    reported = item["backup"]["directory"]
                    if reported.startswith("\\\\?\\"):
                        reported = reported[4:]
                    directory = checked(Path(reported))
                    manifest_path = checked(directory / "backup-manifest.json")
                    payload_path = checked(directory / "preferences.json")
                    assert manifest_path.relative_to(root).as_posix() not in previous
                    raw = manifest_path.read_bytes()
                    payload = payload_path.read_bytes()
                    manifest = json.loads(raw)
                    assert manifest["backup_id"] == item["backup"]["backupId"]
                    assert manifest["action"] == "manual-preferences" and not manifest["management"]["pinned"]
                    assert sha(payload_path) == manifest["sha256"] == original["stateHashes"]["preferences.json"]
                    dated = {**manifest, "created_at": f"2026-09-01T00:00:{index:02d}Z"}
                    dated_raw = (json.dumps(dated, ensure_ascii=False, indent=2) + "\n").encode()
                    changed = bytearray(payload)
                    if index == 0:
                        changed[0] ^= 1
                    entries.append({"id": manifest["backup_id"], "directory": str(directory.relative_to(root)),
                                    "manifestOriginalHex": raw.hex(), "datedManifestHex": dated_raw.hex(),
                                    "payloadOriginalHex": payload.hex(), "payloadFixtureHex": bytes(changed).hex(),
                                    "fixturePayloadSha256": hashlib.sha256(changed).hexdigest(),
                                    "payloadSha256": manifest["sha256"], "damaged": index == 0})
                write("fixture-journal.json", {**stamp, "onlyNewOwnedIds": True, "syntheticDatesOnly": True,
                       "systemClockChanged": False, "cooperativeWindowsLockHeld": True, "entries": entries})
                for entry in entries:
                    directory = root / entry["directory"]
                    atomic(directory / "backup-manifest.json", bytes.fromhex(entry["datedManifestHex"]))
                    atomic(directory / "preferences.json", bytes.fromhex(entry["payloadFixtureHex"]))
                write("fixture-staged.json", {**stamp, "dateAndSingleByteFixtureApplied": True,
                       "candidateVerificationOrder": [entries[1]["id"], entries[0]["id"]]})
            elif mode in {"repair", "restore"}:
                journal = read(evidence / "fixture-journal.json")
                for entry in journal["entries"]:
                    directory = checked(root / entry["directory"])
                    atomic(directory / "preferences.json", bytes.fromhex(entry["payloadOriginalHex"]))
                    assert sha(directory / "preferences.json") == entry["payloadSha256"]
                    if mode == "restore":
                        atomic(directory / "backup-manifest.json", bytes.fromhex(entry["manifestOriginalHex"]))
                        assert (directory / "backup-manifest.json").read_bytes() == bytes.fromhex(entry["manifestOriginalHex"])
                write("fixture-" + mode + ".json", {**stamp, "ownedIds": [entry["id"] for entry in journal["entries"]],
                      "payloadsExactlyRestored": True, "datesExactlyRestored": mode == "restore",
                      "cooperativeWindowsLockHeld": True})
            elif mode == "checkpoint":
                phase = sys.argv[2]
                assert phase in {"failed", "other", "returned"}
                for name in ["event-scope.json", "notifications.json"]:
                    (evidence / (phase + "-" + name)).write_bytes(checked(root / "manager-state" / name).read_bytes())
                created = read(evidence / "prepared.json")["result"]["created"]
                ids = {item["backup"]["backupId"] for item in created}
                for output_name in ["rejected-and-protected.json", "other-candidate-success.json", "returned-candidate-success.json"]:
                    if (evidence / output_name).exists():
                        result = read(evidence / output_name)["result"]
                        if "failedGuard" in result:
                            ids.add(result["failedGuard"]["id"])
                        if "actualRestore" in result:
                            ids.add(result["actualRestore"]["protectionBackupId"])
                manifests = []
                for path in sorted((root / "backups").rglob("backup-manifest.json")):
                    record = read(checked(path))
                    if record["backup_id"] in ids:
                        manifests.append({"manifest": record, "payloadSha256": sha(path.parent / "preferences.json")})
                write(phase + "-checkpoint.json", {**stamp, "stateHashes": hashes(), "newOwnedRecords": manifests,
                      "cooperativeWindowsLockHeld": True})
            elif mode == "after":
                current = inventory()
                indexed = {entry["path"]: entry for entry in current}
                assert all(indexed.get(entry["path"]) == entry for entry in original["backupFiles"])
                write("filesystem-after.json", {**stamp, "stateHashes": hashes(), "backupFiles": current,
                      "allPreexistingBackupFilesExactlyPreserved": True,
                      "preferencesBindingSchedulePolicyAllBytesUnchanged": True, "cooperativeWindowsLockHeld": True})
    finally:
        if not kernel.UnlockFileEx(handle, 0, 0xffffffff, 0xffffffff, ctypes.byref(overlap)):
            raise ctypes.WinError(ctypes.get_last_error())
print(json.dumps({"mode": mode, "result": "completed"}))
