"""Temporary window/taskbar icon on the one retained sandbox, auto-restored."""
import ctypes
import hashlib
import importlib.util
import json
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
control = code / "test/runtime/windows-icon-preview-20261014"
control.mkdir(exist_ok=False)
out = root / "live-preview"
out.mkdir(exist_ok=True)
session_path = code / "test/runtime/windows0110/control/current-session.json"
session = json.loads(session_path.read_text(encoding="utf-8-sig"))
assert session["status"] == "running" and session["applicationPid"] == 10904
assert session["applicationToken"]["creationTimeUtc"] == "2026-10-13T14:57:31.5757733Z"
expected_sha = "3a26098e813ddd56070a0092cbcb640473cb73e626aee45198c4b335ec2d8bd1"
assert session["executableSha256"] == expected_sha
module = code / "test/runtime/windows-caption-point-5a36cda-20261014/test/smoke/windows_caption_probe.py"
spec = importlib.util.spec_from_file_location("caption_probe", module)
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)
native = probe.NativeWindows()
user = native.user
probe.NativeWindows.api(user, "LoadImageW",
    [ctypes.c_void_p, ctypes.c_wchar_p, ctypes.c_uint32, ctypes.c_int, ctypes.c_int, ctypes.c_uint32], ctypes.c_void_p)
probe.NativeWindows.api(user, "DestroyIcon", [ctypes.c_void_p], ctypes.c_int32)
pid, hwnd = 10904, 6685560
identity = native.identity(pid, hwnd)
assert identity["sha256"] == expected_sha and identity["creationFileTime"] == 134363770515757733
image = root / "app-90.ico"
state = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "identity": identity, "pid": pid, "hwnd": hwnd,
    "sourceCommit": session["sourceCommit"], "image": str(image),
    "imageSha256": hashlib.sha256(image.read_bytes()).hexdigest(),
    "scope": "temporary live window/taskbar visual preview, not a rebuilt EXE or installation",
    "maximumPreviewSeconds": 7200, "nativeInstanceStarted": False,
    "stopRequest": str(control / "restore.request"),
}
icons, previous = {}, {}
try:
    dpi = user.GetDpiForWindow(hwnd)
    for slot, logical_size in [(0, 16), (1, 32)]:
        size = logical_size * dpi // 96
        icon = user.LoadImageW(None, str(image), 1, size, size, 0x10)
        if not icon:
            raise ctypes.WinError(ctypes.get_last_error())
        icons[slot] = icon
        old = native.message(hwnd, 0x80, wp=slot, lp=icon)  # WM_SETICON only, no input
        assert old["ok"], old
        previous[slot] = old["value"]
        actual = native.message(hwnd, 0x7f, wp=slot, lp=dpi)
        assert actual["ok"] and actual["value"] == icon, actual
    state.update(status="preview-active", dpi=dpi, previousHandles=previous, previewHandles=icons)
    (out / "window-applied.json").write_text(json.dumps(state, indent=2) + "\n", encoding="utf-8")
    (control / "status.json").write_text(json.dumps(state, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": state["status"], "pid": pid, "hwnd": hwnd, "dpi": dpi}), flush=True)
    deadline = time.monotonic() + 7200
    while time.monotonic() < deadline and not (control / "restore.request").exists():
        if not user.IsWindow(hwnd):
            state["status"] = "target-exited"
            break
        if native.identity(pid, hwnd) != identity:
            state["status"] = "target-identity-changed"
            break
        time.sleep(2)
    else:
        state["status"] = "restore-requested-or-preview-expired"
finally:
    restored = False
    if user.IsWindow(hwnd) and native.identity(pid, hwnd) == identity:
        for slot, handle in previous.items():
            result = native.message(hwnd, 0x80, wp=slot, lp=handle)
            assert result["ok"], result
        restored = bool(previous)
        # Restore the original tray in the same identified instance, too.
        node = Path(r"C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe")
        if (out / "tray-applied.json").exists() and not (out / "tray-restored.json").exists():
            cleanup = subprocess.run([str(node), str(root / "live-tray-preview.mjs"), "--restore"],
                                     capture_output=True, text=True, timeout=20,
                                     creationflags=subprocess.CREATE_NO_WINDOW)
            state["trayCleanupExitCode"] = cleanup.returncode
    for icon in icons.values():
        user.DestroyIcon(icon)
    native.close()
    state.update(finishedAtUtc=datetime.now(timezone.utc).isoformat(), originalWindowIconsRestored=restored)
    (out / "window-final.json").write_text(json.dumps(state, indent=2) + "\n", encoding="utf-8")
    (control / "status.json").write_text(json.dumps(state, indent=2) + "\n", encoding="utf-8")
