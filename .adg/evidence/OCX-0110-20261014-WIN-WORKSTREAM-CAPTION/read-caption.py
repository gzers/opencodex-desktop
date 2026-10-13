"""Observe one retained native caption; no UI automation, input, focus or window mutation."""
import ctypes
import ctypes.wintypes as wt
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
runtime = code / "test/runtime/windows0110"
source = "be7395acfce0ec1e36b83300b576db408f8c35bb"
image = "3a26098e813ddd56070a0092cbcb640473cb73e626aee45198c4b335ec2d8bd1"
application_pid = 10904
hwnd = 6685560
result_path = root / "actual-readonly-caption.json"
assert not result_path.exists(), "Never overwrite an actual native readback"
kernel = ctypes.WinDLL("kernel32", use_last_error=True)
user = ctypes.WinDLL("user32", use_last_error=True)
dwm = ctypes.WinDLL("dwmapi", use_last_error=True)
class FileTime(ctypes.Structure):
    _fields_ = [("low", wt.DWORD), ("high", wt.DWORD)]
    def ticks(self):
        return (self.high << 32) | self.low
class Rect(ctypes.Structure):
    _fields_ = [(field, wt.LONG) for field in ["left", "top", "right", "bottom"]]
    def value(self):
        return {field: int(getattr(self, field)) for field in ["left", "top", "right", "bottom"]}
class TitleInfo(ctypes.Structure):
    _fields_ = [("size", wt.DWORD), ("title", Rect), ("states", wt.DWORD * 6), ("rectangles", Rect * 6)]
kernel.OpenProcess.argtypes = [wt.DWORD, wt.BOOL, wt.DWORD]
kernel.OpenProcess.restype = wt.HANDLE
kernel.CloseHandle.argtypes = [wt.HANDLE]
kernel.GetProcessTimes.argtypes = [wt.HANDLE, *([ctypes.POINTER(FileTime)] * 4)]
kernel.GetProcessTimes.restype = wt.BOOL
kernel.WaitForSingleObject.argtypes = [wt.HANDLE, wt.DWORD]
kernel.WaitForSingleObject.restype = wt.DWORD
kernel.QueryFullProcessImageNameW.argtypes = [wt.HANDLE, wt.DWORD, wt.LPWSTR, ctypes.POINTER(wt.DWORD)]
kernel.QueryFullProcessImageNameW.restype = wt.BOOL
user.GetWindowThreadProcessId.argtypes = [wt.HWND, ctypes.POINTER(wt.DWORD)]
user.GetWindowThreadProcessId.restype = wt.DWORD
user.GetWindowRect.argtypes = [wt.HWND, ctypes.POINTER(Rect)]
user.GetWindowRect.restype = wt.BOOL
user.GetDpiForWindow.argtypes = [wt.HWND]
user.GetDpiForWindow.restype = wt.UINT
user.SetThreadDpiAwarenessContext.argtypes = [wt.HANDLE]
user.SetThreadDpiAwarenessContext.restype = wt.HANDLE
user.GetForegroundWindow.restype = wt.HWND
for name in ["IsWindow", "IsWindowVisible", "IsIconic", "IsZoomed"]:
    function = getattr(user, name)
    function.argtypes = [wt.HWND]
    function.restype = wt.BOOL
user.SendMessageTimeoutW.argtypes = [
    wt.HWND, wt.UINT, ctypes.c_size_t, ctypes.c_ssize_t, wt.UINT, wt.UINT,
    ctypes.POINTER(ctypes.c_size_t),
]
user.SendMessageTimeoutW.restype = ctypes.c_ssize_t
dwm.DwmGetWindowAttribute.argtypes = [wt.HWND, wt.DWORD, ctypes.c_void_p, wt.DWORD]
dwm.DwmGetWindowAttribute.restype = wt.LONG

def send_query(message, parameter):
    response = ctypes.c_size_t()
    if not user.SendMessageTimeoutW(hwnd, message, 0, parameter, 2, 1000, ctypes.byref(response)):
        raise ctypes.WinError(ctypes.get_last_error())
    return int(response.value)

def state():
    rect = Rect()
    if not user.GetWindowRect(hwnd, ctypes.byref(rect)):
        raise ctypes.WinError(ctypes.get_last_error())
    backdrop = wt.LONG()
    dwm_result = dwm.DwmGetWindowAttribute(hwnd, 38, ctypes.byref(backdrop), ctypes.sizeof(backdrop))
    return {
        "windowBounds": rect.value(), "dpi": int(user.GetDpiForWindow(hwnd)),
        "visible": bool(user.IsWindowVisible(hwnd)), "minimized": bool(user.IsIconic(hwnd)),
        "maximized": bool(user.IsZoomed(hwnd)), "foreground": user.GetForegroundWindow() == hwnd,
        "backdropAttribute": int(backdrop.value), "dwmResult": int(dwm_result),
    }
def creation_ticks(value):
    whole, fraction = value.removesuffix("Z").split(".")
    seconds = int(datetime.strptime(whole, "%Y-%m-%dT%H:%M:%S").replace(tzinfo=timezone.utc).timestamp())
    return seconds * 10_000_000 + int(fraction.ljust(7, "0")) + 116444736000000000
def data_hashes():
    return {
        name: hashlib.sha256((runtime / "data/manager-state" / name).read_bytes()).hexdigest()
        for name in ["preferences.json", "data-root.json", "backup-cleanup-policy.json", "update-schedule.json"]
    }
session = json.loads((runtime / "control/current-session.json").read_text(encoding="utf-8-sig"))
assert session["sourceCommit"] == source and session["applicationPid"] == application_pid
assert session["status"] == "running" and session["executableSha256"] == image
assert hashlib.sha256(Path(session["executable"]).read_bytes()).hexdigest() == image
handle = kernel.OpenProcess(0x101000, False, application_pid)
if not handle:
    raise ctypes.WinError(ctypes.get_last_error())
context = None
try:
    started, ended, cpu_kernel, cpu_user = FileTime(), FileTime(), FileTime(), FileTime()
    assert kernel.WaitForSingleObject(handle, 0) == 258
    if not kernel.GetProcessTimes(handle, ctypes.byref(started), ctypes.byref(ended), ctypes.byref(cpu_kernel), ctypes.byref(cpu_user)):
        raise ctypes.WinError(ctypes.get_last_error())
    assert started.ticks() == creation_ticks("2026-10-13T14:57:31.5757733Z")
    buffer = ctypes.create_unicode_buffer(32768)
    size = wt.DWORD(len(buffer))
    if not kernel.QueryFullProcessImageNameW(handle, 0, buffer, ctypes.byref(size)):
        raise ctypes.WinError(ctypes.get_last_error())
    assert Path(buffer.value) == Path(session["executable"])
    owner = wt.DWORD()
    assert user.IsWindow(hwnd) and user.GetWindowThreadProcessId(hwnd, ctypes.byref(owner)) and owner.value == application_pid
    context = user.SetThreadDpiAwarenessContext(ctypes.c_void_p(-4))
    if not context:
        raise ctypes.WinError(ctypes.get_last_error())
    before_hashes = data_hashes()
    before = state()
    info = TitleInfo()
    info.size = ctypes.sizeof(info)
    assert info.size == 140
    send_query(0x033F, ctypes.addressof(info))
    entries = []
    for index, label, expected_hit in [(2, "minimize", 8), (3, "maximize-or-restore", 9), (5, "close", 20)]:
        rect = info.rectangles[index]
        bounds = rect.value()
        available = not (info.states[index] & (0x8000 | 0x10000 | 1))
        nonempty = rect.right > rect.left and rect.bottom > rect.top
        result = None
        center = None
        if available and nonempty and before["visible"] and not before["minimized"]:
            x, y = (rect.left + rect.right) // 2, (rect.top + rect.bottom) // 2
            if not (-32768 <= x <= 32767 and -32768 <= y <= 32767):
                raise ValueError("Hit-test message signed coordinate bounds exceeded")
            parameter = (x & 0xFFFF) | ((y & 0xFFFF) << 16)
            result = ctypes.c_int32(send_query(0x0084, parameter)).value
            center = {"x": x, "y": y}
        entries.append({
            "index": index, "control": label, "screenBounds": bounds, "accessibilityState": int(info.states[index]),
            "available": available, "nonempty": nonempty, "queriedScreenCenter": center,
            "hitResult": result, "expectedHit": expected_hit, "expectedHitMatched": result == expected_hit,
            "actualClickIssued": False,
        })
    after = state()
    after_hashes = data_hashes()
    assert before_hashes == after_hashes
    assert kernel.WaitForSingleObject(handle, 0) == 258
    record = {
        "atUtc": datetime.now(timezone.utc).isoformat(), "sourceCommit": source, "executableSha256": image,
        "applicationPid": application_pid, "creationTimeUtc": "2026-10-13T14:57:31.5757733Z", "hwnd": hwnd,
        "actualLiveHandleVerified": True, "actualImagePathVerified": True, "windowOwnerVerified": True,
        "queryApi": ["WM_GETTITLEBARINFOEX", "WM_NCHITTEST"],
        "dpiAwarenessScope": "helper thread only, restored after readback; no OS setting mutation",
        "titlebarStructBytes": info.size, "titlebarScreenBounds": info.title.value(),
        "captionControls": entries, "windowBefore": before, "windowAfter": after,
        "windowGeometryAndStateUnchangedDuringReadback": before == after,
        "protectedDataHashesBefore": before_hashes, "protectedDataHashesAfter": after_hashes,
        "scope": "current retained window caption geometry/hit-test only; no past input coordinates or delivery inferred",
        "inputOrFocusIssued": False, "windowOrProcessMutated": False, "productCodeChanged": False,
        "captionRestoreClickAccepted": False, "pastClickFailureCauseIsolated": False,
        "fullNativeGateAccepted": False, "pairedPerformanceAccepted": False, "releaseAccepted": False,
    }
    result_path.write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({
        "readonlyCaptionObserved": True, "sourceCommit": source,
        "controls": [{"control": item["control"], "hit": item["hitResult"], "expectedHitMatched": item["expectedHitMatched"]} for item in entries],
        "windowMaximized": before["maximized"], "inputIssued": False, "pastClickFailureCauseIsolated": False,
    }))
finally:
    if context:
        user.SetThreadDpiAwarenessContext(context)
    kernel.CloseHandle(handle)
