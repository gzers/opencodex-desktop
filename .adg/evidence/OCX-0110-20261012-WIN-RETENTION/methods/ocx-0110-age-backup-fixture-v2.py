"""Age only this task's manual test manifests; restore surviving originals."""

import ctypes
import ctypes.wintypes
import hashlib
import json
import msvcrt
import os
import sys
from pathlib import Path


BASE = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\windows0110")
ROOT = BASE / "data"
LABEL = sys.argv[2] if len(sys.argv) > 2 else "manual"
assert LABEL in {"manual", "automatic"}
SAVED = BASE / "control" / f"historical-manifest-fixture-{LABEL}.json"
EVIDENCE = Path(r"D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261012-WIN-RETENTION")


class Overlapped(ctypes.Structure):
    _fields_ = [
        ("Internal", ctypes.c_size_t), ("InternalHigh", ctypes.c_size_t),
        ("Offset", ctypes.wintypes.DWORD), ("OffsetHigh", ctypes.wintypes.DWORD),
        ("hEvent", ctypes.wintypes.HANDLE),
    ]


kernel = ctypes.WinDLL("kernel32", use_last_error=True)
kernel.LockFileEx.argtypes = [
    ctypes.wintypes.HANDLE, ctypes.wintypes.DWORD, ctypes.wintypes.DWORD,
    ctypes.wintypes.DWORD, ctypes.wintypes.DWORD, ctypes.POINTER(Overlapped),
]
kernel.UnlockFileEx.argtypes = [
    ctypes.wintypes.HANDLE, ctypes.wintypes.DWORD, ctypes.wintypes.DWORD,
    ctypes.wintypes.DWORD, ctypes.POINTER(Overlapped),
]


def atomic_write(path, data):
    assert path.resolve().is_relative_to(ROOT.resolve())
    assert path.is_file() and not path.is_symlink()
    pending = path.with_name(path.name + ".fixture-pending")
    with pending.open("xb") as output:
        output.write(data)
        output.flush()
        os.fsync(output.fileno())
    os.replace(pending, path)


mode = sys.argv[1]
assert mode in {"age", "restore"}
with (ROOT / ".backup-w2.lock").open("r+b") as guard:
    handle = ctypes.wintypes.HANDLE(msvcrt.get_osfhandle(guard.fileno()))
    overlap = Overlapped()
    if not kernel.LockFileEx(handle, 3, 0, 0xffffffff, 0xffffffff, ctypes.byref(overlap)):
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        if mode == "age":
            assert not SAVED.exists(), "Historical fixture is already staged"
            entries = []
            paths = sorted((ROOT / "backups").rglob("backup-manifest.json"))
            for path in paths:
                assert not path.is_symlink() and path.resolve().is_relative_to(ROOT.resolve())
                original = path.read_bytes()
                manifest = json.loads(original)
                if manifest["action"] != "manual-preferences":
                    continue
                entries.append({"path": str(path.relative_to(ROOT)), "originalHex": original.hex(),
                                "id": manifest["backup_id"], "oldCreatedAt": manifest["created_at"]})
            assert len(entries) == 14, "Expected the recent-creation population"
            SAVED.write_text(json.dumps(entries), encoding="utf-8")
            observations = []
            for index, entry in enumerate(entries):
                path = ROOT / entry["path"]
                manifest = json.loads(bytes.fromhex(entry["originalHex"]))
                manifest["created_at"] = f"2026-09-01T00:00:{index:02d}Z"
                fixture = json.dumps(manifest, ensure_ascii=False, indent=2).encode("utf-8")
                atomic_write(path, fixture)
                observations.append({
                    "id": entry["id"], "originalCreatedAt": entry["oldCreatedAt"],
                    "syntheticCreatedAt": manifest["created_at"],
                    "originalManifestSha256": hashlib.sha256(bytes.fromhex(entry["originalHex"])).hexdigest(),
                    "fixtureManifestSha256": hashlib.sha256(fixture).hexdigest(),
                    "payloadSha256Unchanged": manifest["sha256"], "pinned": manifest["management"]["pinned"],
                })
            report = {"fixtureOnly": True, "changesSystemClock": False,
                      "cooperativeWindowsLockHeld": True, "entries": observations}
            (EVIDENCE / f"{LABEL}-historical-date-fixture.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
        else:
            entries = json.loads(SAVED.read_text(encoding="utf-8"))
            restored, removed = [], []
            for entry in entries:
                path = ROOT / entry["path"]
                if path.exists():
                    data = bytes.fromhex(entry["originalHex"])
                    atomic_write(path, data)
                    assert path.read_bytes() == data
                    restored.append(entry["id"])
                else:
                    removed.append(entry["id"])
            report = {"survivingOriginalManifestsRestored": restored,
                      "removedByNativeCleanup": removed, "cooperativeWindowsLockHeld": True}
            (EVIDENCE / f"{LABEL}-historical-fixture-restoration.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    finally:
        if not kernel.UnlockFileEx(handle, 0, 0xffffffff, 0xffffffff, ctypes.byref(overlap)):
            raise ctypes.WinError(ctypes.get_last_error())
print(json.dumps({"mode": mode, "result": "completed"}))
