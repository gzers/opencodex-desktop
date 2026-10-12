"""Corrupt only isolated backup policy, preserve/repair raw validated automatic bytes."""
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
policy_path = root / "manager-state/backup-cleanup-policy.json"
def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))
def write(name, value):
    (evidence / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
def checked(path):
    assert path.exists() and path.resolve().is_relative_to(root.resolve())
    for part in [path, *path.parents]:
        assert not part.is_symlink() and not (part.stat().st_file_attributes & 0x400)
        if part == root:
            break
    return path
def digest(path):
    return hashlib.sha256(checked(path).read_bytes()).hexdigest()
def cycle_hashes():
    return {name: digest(root / "manager-state" / name)
            for name in ["preferences.json", "data-root.json", "update-schedule.json"]}
def atomic(path, data):
    checked(path)
    pending = path.with_name(path.name + ".policy-fixture-pending")
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
assert session["status"] == "running" and session["applicationPid"] == 37276
stamp = {"atUtc": datetime.now(timezone.utc).isoformat(), "sourceCommit": session["sourceCommit"], "applicationPid": 37276}
mode = sys.argv[1]
assert mode in {"stage", "repair", "checkpoint"}
with checked(root / ".backup-w2.lock").open("r+b") as guard:
    handle = wt.HANDLE(msvcrt.get_osfhandle(guard.fileno()))
    overlap = Overlapped()
    if not kernel.LockFileEx(handle, 3, 0, 0xffffffff, 0xffffffff, ctypes.byref(overlap)):
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        before = read(evidence / "filesystem-before.json")
        assert all(cycle_hashes()[name] == before["stateHashes"][name] for name in cycle_hashes())
        journal_path = evidence / "policy-fixture-journal.json"
        if mode == "stage":
            assert not journal_path.exists()
            original = checked(policy_path).read_bytes()
            current = json.loads(original)
            assert current == {"schema_version": 1, "mode": "automatic", "keep_recent": 10, "keep_days": 30}
            fixture = {**current, "schema_version": 99}
            raw = (json.dumps(fixture, ensure_ascii=False, indent=2) + "\n").encode()
            write(journal_path.name, {**stamp, "originalAutomaticHex": original.hex(), "fixtureHex": raw.hex(),
                  "originalAutomaticSha256": hashlib.sha256(original).hexdigest(),
                  "fixtureSha256": hashlib.sha256(raw).hexdigest(), "cycleHashes": cycle_hashes(),
                  "onlyIsolatedBackupPolicySchemaChanged": True, "clocksChanged": False,
                  "cooperativeWindowsLockHeld": True})
            atomic(policy_path, raw)
            assert policy_path.read_bytes() == raw
        elif mode == "repair":
            journal = read(journal_path)
            original = bytes.fromhex(journal["originalAutomaticHex"])
            atomic(policy_path, original)
            assert policy_path.read_bytes() == original
            write("policy-exact-repair.json", {**stamp, "restoredAutomaticSha256": digest(policy_path),
                  "originalAutomaticBytesExactlyRestored": True, "cycleHashes": cycle_hashes(),
                  "cooperativeWindowsLockHeld": True})
        else:
            phase = sys.argv[2]
            assert phase in {"failed", "success"}
            for name in ["event-scope.json", "notifications.json"]:
                (evidence / (phase + "-" + name)).write_bytes(checked(root / "manager-state" / name).read_bytes())
            write(phase + "-policy-and-cycle.json", {**stamp, "policySha256": digest(policy_path),
                  "policyRawHex": checked(policy_path).read_bytes().hex(), "cycleHashes": cycle_hashes(),
                  "cooperativeWindowsLockHeld": True})
    finally:
        if not kernel.UnlockFileEx(handle, 0, 0xffffffff, 0xffffffff, ctypes.byref(overlap)):
            raise ctypes.WinError(ctypes.get_last_error())
print(json.dumps({"mode": mode, "result": "completed"}))
