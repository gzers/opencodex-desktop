"""Read-only active test payload witness under the product backup transaction lock."""
import ctypes
import ctypes.wintypes as wt
import hashlib
import json
import msvcrt
from pathlib import Path

evidence = Path(__file__).resolve().parents[1]
root = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\windows0110\data")
class Overlapped(ctypes.Structure):
    _fields_ = [("Internal", ctypes.c_size_t), ("InternalHigh", ctypes.c_size_t),
                ("Offset", wt.DWORD), ("OffsetHigh", wt.DWORD), ("hEvent", wt.HANDLE)]
kernel = ctypes.WinDLL("kernel32", use_last_error=True)
kernel.LockFileEx.argtypes = [wt.HANDLE, wt.DWORD, wt.DWORD, wt.DWORD, wt.DWORD, ctypes.POINTER(Overlapped)]
kernel.UnlockFileEx.argtypes = [wt.HANDLE, wt.DWORD, wt.DWORD, wt.DWORD, ctypes.POINTER(Overlapped)]
path = root / "manager-state/preferences.json"
for part in [path, *path.parents]:
    assert not part.is_symlink() and not (part.stat().st_file_attributes & 0x400)
    if part == root:
        break
with (root / ".backup-w2.lock").open("r+b") as guard:
    handle = wt.HANDLE(msvcrt.get_osfhandle(guard.fileno()))
    overlap = Overlapped()
    if not kernel.LockFileEx(handle, 3, 0, 0xffffffff, 0xffffffff, ctypes.byref(overlap)):
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        payload = path.read_bytes()
        before = json.loads((evidence / "filesystem-before.json").read_text(encoding="utf-8-sig"))
        assert hashlib.sha256(payload).hexdigest() == before["stateHashes"]["preferences.json"]
        (evidence / "active-preferences-payload.json").write_bytes(payload)
    finally:
        if not kernel.UnlockFileEx(handle, 0, 0xffffffff, 0xffffffff, ctypes.byref(overlap)):
            raise ctypes.WinError(ctypes.get_last_error())
print(json.dumps({"payloadBytes": len(payload), "exactOriginalPreferenceHash": True, "cooperativeWindowsLockHeld": True}))
