"""Only three IDs from actual scene creation can have synthetic manifest dates.
Never edit a preexisting backup, change a clock or directly delete files.
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
def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
def checked(path):
    assert path.exists() and path.resolve().is_relative_to(root.resolve())
    for part in [path, *path.parents]:
        assert not part.is_symlink()
        assert not (part.stat().st_file_attributes & 0x400), "Reparse point rejected"
        if part == root:
            break
    return path
def inventory():
    return [{"path": p.relative_to(root).as_posix(), "bytes": p.stat().st_size, "sha256": digest(checked(p))}
            for p in sorted((root / "backups").rglob("*")) if p.is_file()]
def state_hashes():
    return {name: digest(checked(root / "manager-state" / name))
            for name in ["preferences.json", "data-root.json", "update-schedule.json", "backup-cleanup-policy.json"]}

session = read(base / "control/current-session.json")
assert session["sourceCommit"] == "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
assert session["applicationPid"] == 37276 and session["status"] == "running"
assert session["applicationToken"]["elevated"] is False
stamp = {"collectedAtUtc": datetime.now(timezone.utc).isoformat(), "sourceCommit": session["sourceCommit"],
         "applicationPid": session["applicationPid"], "executableSha256": session["executableSha256"]}

class Overlapped(ctypes.Structure):
    _fields_ = [("Internal", ctypes.c_size_t), ("InternalHigh", ctypes.c_size_t),
                ("Offset", wt.DWORD), ("OffsetHigh", wt.DWORD), ("hEvent", wt.HANDLE)]
kernel = ctypes.WinDLL("kernel32", use_last_error=True)
kernel.LockFileEx.argtypes = [wt.HANDLE, wt.DWORD, wt.DWORD, wt.DWORD, wt.DWORD, ctypes.POINTER(Overlapped)]
kernel.UnlockFileEx.argtypes = [wt.HANDLE, wt.DWORD, wt.DWORD, wt.DWORD, ctypes.POINTER(Overlapped)]
def atomic(path, data):
    checked(path)
    pending = path.with_name(path.name + ".eae-fixture-pending")
    assert not pending.exists()
    with pending.open("xb") as output:
        output.write(data)
        output.flush()
        os.fsync(output.fileno())
    os.replace(pending, path)

mode = sys.argv[1]
assert mode in {"before", "after", "age", "restore"}
with checked(root / ".backup-w2.lock").open("r+b") as guard:
    handle = wt.HANDLE(msvcrt.get_osfhandle(guard.fileno()))
    overlap = Overlapped()
    if not kernel.LockFileEx(handle, 3, 0, 0xffffffff, 0xffffffff, ctypes.byref(overlap)):
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        if mode in {"before", "after"}:
            hashes = state_hashes()
            files = inventory()
            result = {**stamp, "stateHashes": hashes, "backupFiles": files, "cooperativeWindowsLockHeld": True}
            if mode == "after":
                previous = read(evidence / "filesystem-before.json")
                current = {entry["path"]: entry for entry in files}
                result["preexistingBackupFilesAllBytesUnchanged"] = all(
                    current.get(entry["path"]) == entry for entry in previous["backupFiles"])
                result["preferencesBindingSchedulePolicyAllBytesUnchanged"] = hashes == previous["stateHashes"]
                assert result["preexistingBackupFilesAllBytesUnchanged"]
                assert result["preferencesBindingSchedulePolicyAllBytesUnchanged"]
            write("filesystem-" + mode + ".json", result)
        else:
            phase = sys.argv[2]
            assert phase in {"manual", "automatic"}
            journal = evidence / (phase + "-fixture-journal.json")
            if mode == "age":
                assert not journal.exists()
                creation = read(evidence / ("prepare-" + phase + ".json"))
                assert creation["sourceCommit"] == stamp["sourceCommit"] and creation["applicationPid"] == 37276
                records = creation["result"]["created"]
                ids = [entry["backup"]["backupId"] for entry in records]
                assert len(ids) == len(set(ids)) == 3
                before_paths = {entry["path"] for entry in read(evidence / "filesystem-before.json")["backupFiles"]}
                entries = []
                for index, item in enumerate(records):
                    reported = item["backup"]["directory"]
                    if reported.startswith("\\\\?\\"):
                        reported = reported[4:]
                    directory = checked(Path(reported))
                    path = checked(directory / "backup-manifest.json")
                    assert path.relative_to(root).as_posix() not in before_paths
                    original = path.read_bytes()
                    manifest = json.loads(original)
                    assert manifest["backup_id"] == ids[index] and manifest["action"] == "manual-preferences"
                    assert manifest["management"]["kind"] == "standalone" and not manifest["management"]["pinned"]
                    payload = checked(directory / "preferences.json")
                    assert digest(payload) == manifest["sha256"] == digest(root / "manager-state/preferences.json")
                    assert payload.stat().st_size == manifest["bytes"]
                    fixture = {**manifest, "created_at": f"2026-09-01T00:00:{index:02d}Z"}
                    fixture_bytes = (json.dumps(fixture, ensure_ascii=False, indent=2) + "\n").encode()
                    entries.append({"id": ids[index], "path": str(path.relative_to(root)),
                                    "originalHex": original.hex(), "fixtureHex": fixture_bytes.hex(),
                                    "payloadSha256": digest(payload), "payloadBytes": payload.stat().st_size,
                                    "originalCreatedAt": manifest["created_at"], "syntheticCreatedAt": fixture["created_at"]})
                write(journal.name, {**stamp, "syntheticDatesOnly": True, "systemClockChanged": False,
                                    "onlyNewOwnedIds": True, "cooperativeWindowsLockHeld": True, "entries": entries})
                for entry in entries:
                    path = root / entry["path"]
                    atomic(path, bytes.fromhex(entry["fixtureHex"]))
                    assert path.read_bytes() == bytes.fromhex(entry["fixtureHex"])
                write(phase + "-fixture-applied.json", {**stamp, "appliedOwnedIds": ids, "cooperativeWindowsLockHeld": True})
            else:
                value = read(journal)
                restored, removed = [], []
                for entry in value["entries"]:
                    path = root / entry["path"]
                    if path.exists():
                        assert digest(checked(path.parent / "preferences.json")) == entry["payloadSha256"]
                        atomic(path, bytes.fromhex(entry["originalHex"]))
                        assert path.read_bytes() == bytes.fromhex(entry["originalHex"])
                        restored.append(entry["id"])
                    else:
                        assert not path.parent.exists(), "Partial deletion of owned directory"
                        removed.append(entry["id"])
                write(phase + "-fixture-restored.json", {**stamp, "survivingOriginalManifestsRestored": restored,
                       "ownedIdsRemovedByNativeCleanup": removed, "cooperativeWindowsLockHeld": True})
    finally:
        if not kernel.UnlockFileEx(handle, 0, 0xffffffff, 0xffffffff, ctypes.byref(overlap)):
            raise ctypes.WinError(ctypes.get_last_error())
print(json.dumps({"mode": mode, "result": "completed"}))
