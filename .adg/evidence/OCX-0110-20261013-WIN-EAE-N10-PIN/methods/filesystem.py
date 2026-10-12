"""Date only fourteen newly owned records; preserve genuine pin until native unpin.
Original test records may be temporarily pinned by native commands, never deleted.
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
def files():
    return [{"path": p.relative_to(root).as_posix(), "bytes": p.stat().st_size, "sha256": sha(checked(p))}
            for p in sorted((root / "backups").rglob("*")) if p.is_file()]
def hashes():
    return {name: sha(checked(root / "manager-state" / name))
            for name in ["preferences.json", "data-root.json", "update-schedule.json", "backup-cleanup-policy.json"]}
def atomic(path, data):
    checked(path)
    pending = path.with_name(path.name + ".eae-n10-fixture-pending")
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
assert mode in {"before", "stage", "restore-dates", "after"}
with checked(root / ".backup-w2.lock").open("r+b") as guard:
    handle = wt.HANDLE(msvcrt.get_osfhandle(guard.fileno()))
    overlap = Overlapped()
    if not kernel.LockFileEx(handle, 3, 0, 0xffffffff, 0xffffffff, ctypes.byref(overlap)):
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        if mode == "before":
            assert not (evidence / "filesystem-before.json").exists()
            write("filesystem-before.json", {**stamp, "stateHashes": hashes(), "backupFiles": files(),
                                            "originalManifestBytes": {
                                                path.relative_to(root).as_posix(): checked(path).read_bytes().hex()
                                                for path in sorted((root / "backups").rglob("backup-manifest.json"))
                                            },
                                            "cooperativeWindowsLockHeld": True})
        else:
            before = read(evidence / "filesystem-before.json")
            assert hashes() == before["stateHashes"], "Cycle, preference, binding or policy changed"
            if mode == "stage":
                assert not (evidence / "fixture-journal.json").exists()
                prepared = read(evidence / "prepared.json")
                assert prepared["sourceCommit"] == stamp["sourceCommit"] and prepared["applicationPid"] == 37276
                created = prepared["result"]["created"]
                assert len(created) == 14
                old_paths = {entry["path"] for entry in before["backupFiles"]}
                entries = []
                for index, result in enumerate(created):
                    reported = result["backup"]["directory"]
                    if reported.startswith("\\\\?\\"):
                        reported = reported[4:]
                    directory = checked(Path(reported))
                    path = checked(directory / "backup-manifest.json")
                    assert path.relative_to(root).as_posix() not in old_paths
                    raw = path.read_bytes()
                    manifest = json.loads(raw)
                    assert manifest["backup_id"] == result["backup"]["backupId"]
                    assert manifest["action"] == "manual-preferences" and manifest["management"]["version"] == 1
                    assert manifest["management"]["kind"] == "standalone" and manifest["management"]["pinned"] is False
                    assert sha(checked(directory / "preferences.json")) == manifest["sha256"] == before["stateHashes"]["preferences.json"]
                    assert raw.count(b'"pinned": false') == 1
                    fixture = {**manifest, "created_at": f"2026-09-01T00:00:{index:02d}Z"}
                    fixture_raw = (json.dumps(fixture, ensure_ascii=False, indent=2) + "\n").encode()
                    entries.append({"id": manifest["backup_id"], "directory": str(directory.relative_to(root)),
                       "manifestOriginalHex": raw.hex(), "fixtureHex": fixture_raw.hex(),
                       "payloadSha256": manifest["sha256"], "payloadBytes": manifest["bytes"],
                       "syntheticCreatedAt": fixture["created_at"], "index": index})
                write("fixture-journal.json", {**stamp, "onlyNewOwnedDatesChanged": True, "systemClockChanged": False,
                       "cooperativeWindowsLockHeld": True, "entries": entries})
                for entry in entries:
                    atomic(root / entry["directory"] / "backup-manifest.json", bytes.fromhex(entry["fixtureHex"]))
                write("fixture-staged.json", {**stamp, "newOwnedRecordsDated": 14, "payloadsUnchanged": True})
            elif mode == "restore-dates":
                journal = read(evidence / "fixture-journal.json")
                restored, missing, still_pinned = [], [], []
                for entry in journal["entries"]:
                    directory = root / entry["directory"]
                    if not directory.exists():
                        missing.append(entry["id"])
                        continue
                    checked(directory)
                    path = checked(directory / "backup-manifest.json")
                    assert sha(checked(directory / "preferences.json")) == entry["payloadSha256"]
                    original_raw = bytes.fromhex(entry["manifestOriginalHex"])
                    original = json.loads(original_raw)
                    current = read(path)
                    pin = current["management"]["pinned"]
                    assert isinstance(pin, bool) and current["backup_id"] == entry["id"]
                    comparable = json.loads(json.dumps(current))
                    comparable["created_at"] = original["created_at"]
                    comparable["management"]["pinned"] = original["management"]["pinned"]
                    assert comparable == original, "Unexpected mutation besides native pin and date fixture"
                    restored_raw = original_raw.replace(b'"pinned": false', b'"pinned": true') if pin else original_raw
                    assert json.loads(restored_raw)["management"]["pinned"] == pin
                    atomic(path, restored_raw)
                    assert path.read_bytes() == restored_raw
                    restored.append(entry["id"])
                    if pin:
                        still_pinned.append(entry["id"])
                write("fixture-dates-restored.json", {**stamp, "survivingOwnedDatesRestored": restored,
                      "missingOwnedDirectories": missing, "genuineNativePinStillPreserved": still_pinned,
                      "cooperativeWindowsLockHeld": True})
            elif mode == "after":
                current = files()
                indexed = {entry["path"]: entry for entry in current}
                assert all(indexed.get(entry["path"]) == entry for entry in before["backupFiles"])
                journal = read(evidence / "fixture-journal.json")
                surviving, missing = [], []
                for entry in journal["entries"]:
                    directory = root / entry["directory"]
                    if not directory.exists():
                        missing.append(entry["id"])
                        continue
                    checked(directory)
                    assert (directory / "backup-manifest.json").read_bytes() == bytes.fromhex(entry["manifestOriginalHex"])
                    assert sha(directory / "preferences.json") == entry["payloadSha256"]
                    surviving.append(entry["id"])
                write("filesystem-after.json", {**stamp, "stateHashes": hashes(), "backupFiles": current,
                      "allOriginalBackupFilesExactlyRestored": True, "allSurvivingNewManifestsExactlyRestored": True,
                      "survivingOwnedIds": surviving, "missingOwnedIds": missing,
                      "preferencesBindingSchedulePolicyAllBytesUnchanged": True, "cooperativeWindowsLockHeld": True})
    finally:
        if not kernel.UnlockFileEx(handle, 0, 0xffffffff, 0xffffffff, ctypes.byref(overlap)):
            raise ctypes.WinError(ctypes.get_last_error())
print(json.dumps({"mode": mode, "result": "completed"}))
