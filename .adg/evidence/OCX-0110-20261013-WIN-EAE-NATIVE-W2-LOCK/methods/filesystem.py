"""OS cooperative lock holder and exact backup/payload snapshots.
Only this scene's three owned cleanup manifests may receive date fixtures.
"""
import ctypes
import ctypes.wintypes as wt
import hashlib
import json
import msvcrt
import os
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

evidence = Path(__file__).resolve().parents[1]
base = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\windows0110")
root = base / "data"
control = base / "control/native-w2-lock-20261013"
kernel = ctypes.WinDLL("kernel32", use_last_error=True)
advapi = ctypes.WinDLL("advapi32", use_last_error=True)
class Overlapped(ctypes.Structure):
    _fields_ = [("Internal", ctypes.c_size_t), ("InternalHigh", ctypes.c_size_t),
                ("Offset", wt.DWORD), ("OffsetHigh", wt.DWORD), ("hEvent", wt.HANDLE)]
class FileTime(ctypes.Structure):
    _fields_ = [("low", wt.DWORD), ("high", wt.DWORD)]
    def integer(self):
        return (self.high << 32) | self.low
kernel.LockFileEx.argtypes = [wt.HANDLE, wt.DWORD, wt.DWORD, wt.DWORD, wt.DWORD, ctypes.POINTER(Overlapped)]
kernel.UnlockFileEx.argtypes = [wt.HANDLE, wt.DWORD, wt.DWORD, wt.DWORD, ctypes.POINTER(Overlapped)]
kernel.GetCurrentProcess.restype = wt.HANDLE
kernel.GetProcessTimes.argtypes = [wt.HANDLE, ctypes.POINTER(FileTime), ctypes.POINTER(FileTime), ctypes.POINTER(FileTime), ctypes.POINTER(FileTime)]
kernel.CloseHandle.argtypes = [wt.HANDLE]
advapi.OpenProcessToken.argtypes = [wt.HANDLE, wt.DWORD, ctypes.POINTER(wt.HANDLE)]
advapi.GetTokenInformation.argtypes = [wt.HANDLE, wt.DWORD, wt.LPVOID, wt.DWORD, ctypes.POINTER(wt.DWORD)]
def token():
    handle = wt.HANDLE()
    if not advapi.OpenProcessToken(kernel.GetCurrentProcess(), 8, ctypes.byref(handle)):
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        values = {}
        for name, kind in [("elevated", 20), ("elevationType", 18)]:
            value, returned = wt.DWORD(), wt.DWORD()
            if not advapi.GetTokenInformation(handle, kind, ctypes.byref(value), ctypes.sizeof(value), ctypes.byref(returned)):
                raise ctypes.WinError(ctypes.get_last_error())
            values[name] = bool(value.value) if name == "elevated" else value.value
        return values
    finally:
        kernel.CloseHandle(handle)
def creation():
    created, ended, cpu_kernel, cpu_user = FileTime(), FileTime(), FileTime(), FileTime()
    if not kernel.GetProcessTimes(kernel.GetCurrentProcess(), ctypes.byref(created), ctypes.byref(ended), ctypes.byref(cpu_kernel), ctypes.byref(cpu_user)):
        raise ctypes.WinError(ctypes.get_last_error())
    return created.integer()
def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))
def write(path, value):
    pending = path.with_suffix(path.suffix + ".pending")
    pending.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
    os.replace(pending, path)
def checked(path):
    assert path.exists() and path.resolve().is_relative_to(root.resolve())
    for part in [path, *path.parents]:
        assert not part.is_symlink() and not (part.stat().st_file_attributes & 0x400)
        if part == root:
            break
    return path
def digest(path):
    return hashlib.sha256(checked(path).read_bytes()).hexdigest()
def snapshot():
    files = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size, "sha256": digest(path)}
             for path in sorted((root / "backups").rglob("*")) if path.is_file()]
    return {
        "atUtc": datetime.now(timezone.utc).isoformat(),
        "backupFiles": files,
        "stateHashes": {name: digest(root / "manager-state" / name)
                        for name in ["preferences.json", "data-root.json", "update-schedule.json", "backup-cleanup-policy.json"]},
        "scope": "backup payload/manifest plus preferences/root/policy/schedule; event-scope/notification metadata intentionally excluded",
    }
def atomic(path, data):
    checked(path)
    pending = path.with_name(path.name + ".native-lock-fixture-pending")
    assert not pending.exists()
    with pending.open("xb") as output:
        output.write(data)
        output.flush()
        os.fsync(output.fileno())
    os.replace(pending, path)

session = read(base / "control/current-session.json")
assert session["sourceCommit"] == "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
assert session["status"] == "running" and session["applicationPid"] == 37276
stamp = {"sourceCommit": session["sourceCommit"], "applicationPid": 37276, "executableSha256": session["executableSha256"]}
mode = sys.argv[1]
assert mode in {"before", "after", "during", "hold", "age-cleanup", "restore-cleanup"}
if mode == "during":
    phase = sys.argv[2]
    assert phase in {"create", "list", "pin", "policy", "restore", "cleanup"}
    write(evidence / (phase + "-while-held-files.json"), {**stamp, **snapshot(), "observerAcquiredNoBackupLock": True})
else:
    with checked(root / ".backup-w2.lock").open("r+b") as guard:
        handle = wt.HANDLE(msvcrt.get_osfhandle(guard.fileno()))
        overlap = Overlapped()
        if not kernel.LockFileEx(handle, 3, 0, 0xffffffff, 0xffffffff, ctypes.byref(overlap)):
            raise ctypes.WinError(ctypes.get_last_error())
        try:
            if mode in {"before", "after"}:
                value = {**stamp, **snapshot(), "cooperativeWindowsLockHeld": True}
                if mode == "after":
                    previous = read(evidence / "filesystem-before.json")
                    indexed = {item["path"]: item for item in value["backupFiles"]}
                    assert all(indexed.get(item["path"]) == item for item in previous["backupFiles"])
                    assert value["stateHashes"] == previous["stateHashes"]
                    value["originalBackupPreferenceBindingPolicyScheduleBytesExactlyPreserved"] = True
                write(evidence / ("filesystem-" + mode + ".json"), value)
            elif mode == "hold":
                phase = sys.argv[2]
                assert phase in {"create", "list", "pin", "policy", "restore", "cleanup"}
                control.mkdir(exist_ok=True)
                ready_path = control / (phase + "-ready.json")
                release_path = control / (phase + "-release.request")
                assert not ready_path.exists() and not release_path.exists()
                identity = {**stamp, "helperPid": os.getpid(), "helperCreationFileTime100ns": creation(),
                            "helperToken": token(), "phase": phase, "realWindowsExclusiveLockHeld": True,
                            "lockAcquiredAtMonotonicNs": time.perf_counter_ns(), "filesAtLockAcquisition": snapshot()}
                assert identity["helperToken"]["elevated"] is False and identity["helperToken"]["elevationType"] == 3
                write(evidence / (phase + "-holder-ready.json"), identity)
                write(ready_path, identity)
                deadline = time.monotonic() + 30
                while not release_path.exists():
                    if time.monotonic() >= deadline:
                        raise TimeoutError("Owned lock holder deadline; release naturally, do not restart product")
                    time.sleep(0.05)
                write(evidence / (phase + "-holder-releasing.json"), {**identity,
                      "releaseRequestAtMonotonicNs": time.perf_counter_ns(), "releaseRequested": True})
            elif mode == "age-cleanup":
                journal_path = evidence / "cleanup-date-journal.json"
                assert not journal_path.exists()
                prepared = read(evidence / "cleanup-prepared.json")
                assert prepared["sourceCommit"] == stamp["sourceCommit"] and prepared["applicationPid"] == 37276
                created = prepared["result"]["created"]
                assert len(created) == 3
                original_paths = {item["path"] for item in read(evidence / "filesystem-before.json")["backupFiles"]}
                entries = []
                for index, result in enumerate(created):
                    directory = result["backup"]["directory"]
                    if directory.startswith("\\\\?\\"):
                        directory = directory[4:]
                    path = checked(Path(directory) / "backup-manifest.json")
                    assert path.relative_to(root).as_posix() not in original_paths
                    raw = path.read_bytes()
                    manifest = json.loads(raw)
                    assert manifest["backup_id"] == result["backup"]["backupId"]
                    assert manifest["action"] == "manual-preferences" and not manifest["management"]["pinned"]
                    assert digest(path.parent / "preferences.json") == manifest["sha256"]
                    fixture = {**manifest, "created_at": f"2026-09-01T00:00:{index:02d}Z"}
                    entries.append({"id": manifest["backup_id"], "path": str(path.relative_to(root)),
                                    "originalHex": raw.hex(), "fixtureHex": (json.dumps(fixture, indent=2) + "\n").encode().hex(),
                                    "payloadSha256": manifest["sha256"]})
                write(journal_path, {**stamp, "onlyNewOwnedDatesChanged": True, "systemClockChanged": False, "entries": entries})
                for entry in entries:
                    atomic(root / entry["path"], bytes.fromhex(entry["fixtureHex"]))
            else:
                journal = read(evidence / "cleanup-date-journal.json")
                restored, removed = [], []
                for entry in journal["entries"]:
                    path = root / entry["path"]
                    if path.exists():
                        assert digest(path.parent / "preferences.json") == entry["payloadSha256"]
                        atomic(path, bytes.fromhex(entry["originalHex"]))
                        restored.append(entry["id"])
                    else:
                        assert not path.parent.exists()
                        removed.append(entry["id"])
                write(evidence / "cleanup-dates-restored.json", {**stamp,
                      "survivingOwnedDatesExactlyRestored": restored, "ownedIdsRemovedByNativeCommand": removed})
        finally:
            if not kernel.UnlockFileEx(handle, 0, 0xffffffff, 0xffffffff, ctypes.byref(overlap)):
                raise ctypes.WinError(ctypes.get_last_error())
print(json.dumps({"mode": mode, "result": "completed"}), flush=True)
