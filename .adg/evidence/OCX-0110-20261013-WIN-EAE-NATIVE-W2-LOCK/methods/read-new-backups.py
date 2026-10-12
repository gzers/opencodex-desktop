"""Read only scene-owned surviving payloads under the real backup lock."""
import ctypes
import ctypes.wintypes as wt
import hashlib
import json
import msvcrt
from pathlib import Path

evidence = Path(__file__).resolve().parents[1]
root = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\windows0110\data")
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
setup = read(evidence / "setup.json")["result"]
created = read(evidence / "create-result.json")["result"]["operation"]["result"]
restored = read(evidence / "restore-result.json")["result"]["operation"]["result"]
ids = {setup["created"]["backup"]["backupId"], created["backup"]["backupId"], restored["protectionBackupId"]}
assert len(ids) == 3
class Overlapped(ctypes.Structure):
    _fields_ = [("Internal", ctypes.c_size_t), ("InternalHigh", ctypes.c_size_t),
                ("Offset", wt.DWORD), ("OffsetHigh", wt.DWORD), ("hEvent", wt.HANDLE)]
kernel = ctypes.WinDLL("kernel32", use_last_error=True)
kernel.LockFileEx.argtypes = [wt.HANDLE, wt.DWORD, wt.DWORD, wt.DWORD, wt.DWORD, ctypes.POINTER(Overlapped)]
kernel.UnlockFileEx.argtypes = [wt.HANDLE, wt.DWORD, wt.DWORD, wt.DWORD, ctypes.POINTER(Overlapped)]
with (root / ".backup-w2.lock").open("r+b") as guard:
    handle = wt.HANDLE(msvcrt.get_osfhandle(guard.fileno()))
    overlap = Overlapped()
    if not kernel.LockFileEx(handle, 3, 0, 0xffffffff, 0xffffffff, ctypes.byref(overlap)):
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        entries = []
        for path in sorted((root / "backups").rglob("backup-manifest.json")):
            assert path.resolve().is_relative_to(root.resolve())
            for part in [path, *path.parents]:
                assert not part.is_symlink() and not (part.stat().st_file_attributes & 0x400)
                if part == root:
                    break
            manifest = read(path)
            if manifest["backup_id"] not in ids:
                continue
            payload_path = path.parent / "preferences.json"
            assert not payload_path.is_symlink() and not (payload_path.stat().st_file_attributes & 0x400)
            raw = payload_path.read_bytes()
            assert hashlib.sha256(raw).hexdigest() == manifest["sha256"] and len(raw) == manifest["bytes"]
            entries.append({"manifest": manifest, "payloadSha256": manifest["sha256"],
                            "payloadBytes": len(raw), "payloadTheme": json.loads(raw)["appearance"]["theme"]})
        assert {entry["manifest"]["backup_id"] for entry in entries} == ids
        (evidence / "owned-new-backups.json").write_bytes((json.dumps({
            "sourceCommit": "eae85322ebca3b755c034db97cbebf2ea5bf71fc", "applicationPid": 37276,
            "cooperativeWindowsLockHeld": True, "entries": entries,
        }, ensure_ascii=False, indent=2) + "\n").encode())
    finally:
        if not kernel.UnlockFileEx(handle, 0, 0xffffffff, 0xffffffff, ctypes.byref(overlap)):
            raise ctypes.WinError(ctypes.get_last_error())
print(json.dumps({"ownedSurvivingBackups": 3, "payloadShaAndBytesVerified": True}))
