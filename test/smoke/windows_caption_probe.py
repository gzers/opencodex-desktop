"""Read-only caption geometry for an existing, explicitly identified Windows EXE.

Does not activate, click, resize, launch, terminate, or change the target window.
Run before/after a separately authorized native input; it is not an acceptance test.
"""
import argparse
import ctypes
import hashlib
import json
import os
import re
import sys
from datetime import datetime, timezone
from pathlib import Path


HT_NAMES = {0: 'nowhere', 1: 'client', 2: 'caption', 8: 'minimize',
            9: 'maximize-or-restore', 20: 'close'}
BUTTON_SLOTS = {'minimize': 2, 'maximizeOrRestore': 3, 'close': 5}


def pack_screen_point(x, y):
    # WM_NCHITTEST uses two *signed* 16-bit screen coordinates, including on
    # monitors left/above the primary. Refuse overflow instead of hitting elsewhere.
    if any(type(v) is not int or not -32768 <= v <= 32767 for v in (x, y)):
        raise ValueError('screen coordinates do not fit WM_NCHITTEST')
    return (x & 0xffff) | ((y & 0xffff) << 16)


def caption_points(rect, window_rect):
    left, top, right, bottom = rect
    wl, wt, wr, wb = window_rect
    if not (wl <= left < right <= wr and wt <= top < bottom <= wb):
        return []
    # Quarter/center samples avoid borders and show a partial/shifted hit region.
    y = top + (bottom - top - 1) // 2
    points = sorted({(left + (right - left - 1) * n // 4, y) for n in (1, 2, 3)})
    for x, y in points:
        pack_screen_point(x, y)
    return points


def collect_snapshot(native, pid, hwnd, expected_sha256, expected_creation=None):
    before = native.identity(pid, hwnd)
    if before['sha256'].lower() != expected_sha256.lower():
        raise ValueError('target EXE SHA256 mismatch; no caption queries issued')
    if expected_creation is not None and before['creationFileTime'] != expected_creation:
        raise ValueError('target creation time mismatch; no caption queries issued')
    state = native.window_state(hwnd)
    caption = native.caption(hwnd)
    buttons = {}
    for name, slot in BUTTON_SLOTS.items():
        item = caption['slots'][slot] if caption['ok'] else None
        probes = []
        points = caption_points(item['rect'], state['windowRect']) if item else []
        for x, y in points:
            under_point = native.point_target(pid, hwnd, x, y)
            hit = native.hit_test(hwnd, pack_screen_point(x, y))
            probes.append({'physicalScreenPoint': [x, y], 'windowUnderPoint': under_point, **hit,
                           'hitName': HT_NAMES.get(hit.get('value'), 'other-or-unavailable')})
        buttons[name] = {'titleBarInfoEx': item, 'probes': probes}
    after = native.identity(pid, hwnd)
    if before != after:
        raise ValueError('target identity changed during observation; discard snapshot')
    return {'schemaVersion': 1, 'collectedAtUtc': datetime.now(timezone.utc).isoformat(),
            'observerPid': os.getpid(), 'pid': pid, 'hwnd': hwnd, 'identity': before,
            'window': state, 'captionQuery': {k: v for k, v in caption.items() if k != 'slots'},
            'buttons': buttons, 'coordinateSpace': 'physical screen; observer thread PMv2',
            'inputIssued': False, 'applicationLifecycleChanged': False,
            'acceptancePassed': False,
            'scope': 'bounded read-only Win32 queries, not proof of click delivery or native acceptance'}


class RECT(ctypes.Structure):
    _fields_ = [(k, ctypes.c_int32) for k in ('left', 'top', 'right', 'bottom')]

    def values(self):
        return [self.left, self.top, self.right, self.bottom]


class TITLEBARINFOEX(ctypes.Structure):
    _fields_ = [('cbSize', ctypes.c_uint32), ('rcTitleBar', RECT),
                ('rgstate', ctypes.c_uint32 * 6), ('rgrect', RECT * 6)]


class FILETIME(ctypes.Structure):
    _fields_ = [('low', ctypes.c_uint32), ('high', ctypes.c_uint32)]


class POINT(ctypes.Structure):
    _fields_ = [('x', ctypes.c_int32), ('y', ctypes.c_int32)]


class WINDOWPLACEMENT(ctypes.Structure):
    _fields_ = [('length', ctypes.c_uint32), ('flags', ctypes.c_uint32),
                ('showCmd', ctypes.c_uint32), ('minPosition', POINT),
                ('maxPosition', POINT), ('normalPosition', RECT)]


class NativeWindows:
    TIMEOUT_MS = 250

    def __init__(self):
        if sys.platform != 'win32':
            raise OSError('this observer requires Windows; portable unit tests use a fake backend')
        self.user = ctypes.WinDLL('user32', use_last_error=True)
        self.kernel = ctypes.WinDLL('kernel32', use_last_error=True)
        self.dwm = ctypes.WinDLL('dwmapi', use_last_error=True)
        self.api(self.user, 'SetThreadDpiAwarenessContext', [ctypes.c_void_p], ctypes.c_void_p)
        self.previous_dpi = self.user.SetThreadDpiAwarenessContext(ctypes.c_void_p(-4))
        if not self.previous_dpi:
            raise ctypes.WinError(ctypes.get_last_error())
        self.api(self.user, 'GetWindowThreadProcessId', [ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint32)])
        self.api(self.user, 'GetWindowRect', [ctypes.c_void_p, ctypes.POINTER(RECT)], ctypes.c_int32)
        self.api(self.user, 'GetWindowPlacement', [ctypes.c_void_p, ctypes.POINTER(WINDOWPLACEMENT)], ctypes.c_int32)
        self.api(self.user, 'GetCursorPos', [ctypes.POINTER(POINT)], ctypes.c_int32)
        self.api(self.user, 'WindowFromPoint', [POINT], ctypes.c_void_p)
        self.api(self.user, 'GetAncestor', [ctypes.c_void_p, ctypes.c_uint32], ctypes.c_void_p)
        self.api(self.user, 'GetClassNameW', [ctypes.c_void_p, ctypes.c_wchar_p, ctypes.c_int32], ctypes.c_int32)
        for name in ('IsWindow', 'IsWindowVisible', 'IsIconic', 'IsZoomed'):
            self.api(self.user, name, [ctypes.c_void_p], ctypes.c_int32)
        self.api(self.user, 'GetForegroundWindow', [], ctypes.c_void_p)
        self.api(self.user, 'GetDpiForWindow', [ctypes.c_void_p])
        # Windows exposes GetWindowLongW on both bitnesses; window style is 32-bit.
        self.api(self.user, 'GetWindowLongW', [ctypes.c_void_p, ctypes.c_int32], ctypes.c_int32)
        self.api(self.user, 'SendMessageTimeoutW', [ctypes.c_void_p, ctypes.c_uint32,
                 ctypes.c_size_t, ctypes.c_ssize_t, ctypes.c_uint32, ctypes.c_uint32,
                 ctypes.POINTER(ctypes.c_size_t)], ctypes.c_ssize_t)
        self.api(self.kernel, 'OpenProcess', [ctypes.c_uint32, ctypes.c_int32, ctypes.c_uint32], ctypes.c_void_p)
        self.api(self.kernel, 'CloseHandle', [ctypes.c_void_p], ctypes.c_int32)
        self.api(self.kernel, 'QueryFullProcessImageNameW', [ctypes.c_void_p, ctypes.c_uint32,
                 ctypes.c_wchar_p, ctypes.POINTER(ctypes.c_uint32)], ctypes.c_int32)
        self.api(self.kernel, 'GetProcessTimes', [ctypes.c_void_p] + [ctypes.POINTER(FILETIME)] * 4, ctypes.c_int32)
        self.api(self.dwm, 'DwmGetWindowAttribute', [ctypes.c_void_p, ctypes.c_uint32,
                 ctypes.c_void_p, ctypes.c_uint32], ctypes.c_int32)

    @staticmethod
    def api(dll, name, args, result=ctypes.c_uint32):
        fn = getattr(dll, name)
        fn.argtypes, fn.restype = args, result

    def close(self):
        if self.previous_dpi:
            self.user.SetThreadDpiAwarenessContext(self.previous_dpi)
            self.previous_dpi = None

    def identity(self, pid, hwnd):
        owner = ctypes.c_uint32()
        if not self.user.IsWindow(hwnd) or not self.user.GetWindowThreadProcessId(hwnd, ctypes.byref(owner)) or owner.value != pid:
            raise ValueError('HWND is missing or belongs to another PID')
        handle = self.kernel.OpenProcess(0x1000, False, pid)
        if not handle:
            raise ctypes.WinError(ctypes.get_last_error())
        try:
            path = ctypes.create_unicode_buffer(32768)
            size = ctypes.c_uint32(len(path))
            times = [FILETIME() for _ in range(4)]
            if not self.kernel.QueryFullProcessImageNameW(handle, 0, path, ctypes.byref(size)) or not self.kernel.GetProcessTimes(handle, *[ctypes.byref(t) for t in times]):
                raise ctypes.WinError(ctypes.get_last_error())
            with open(path.value, 'rb') as source:
                digest = hashlib.sha256()
                for block in iter(lambda: source.read(1024 * 1024), b''):
                    digest.update(block)
            return {'executable': path.value, 'sha256': digest.hexdigest(),
                    'creationFileTime': (times[0].high << 32) | times[0].low, 'ownerPid': owner.value}
        finally:
            self.kernel.CloseHandle(handle)

    def message(self, hwnd, msg, wp=0, lp=0):
        result = ctypes.c_size_t()
        ctypes.set_last_error(0)
        ok = self.user.SendMessageTimeoutW(hwnd, msg, wp, lp, 0x22, self.TIMEOUT_MS, ctypes.byref(result))
        # SMTO_ABORTIFHUNG | SMTO_ERRORONEXIT; failed queries are never HTNOWHERE.
        return {'ok': bool(ok), 'value': ctypes.c_ssize_t(result.value).value if ok else None,
                'lastError': 0 if ok else ctypes.get_last_error(), 'timeoutMs': self.TIMEOUT_MS}

    def hit_test(self, hwnd, packed):
        return self.message(hwnd, 0x84, lp=packed)  # WM_NCHITTEST, no input

    def point_target(self, pid, hwnd, x, y):
        actual = self.user.WindowFromPoint(POINT(x, y))
        root = self.user.GetAncestor(actual, 2) if actual else None  # GA_ROOT
        owner = ctypes.c_uint32()
        owner_ok = bool(actual and self.user.GetWindowThreadProcessId(actual, ctypes.byref(owner)))
        related = root == hwnd or owner_ok and owner.value == pid
        name = ctypes.create_unicode_buffer(256)
        class_ok = bool(related and self.user.GetClassNameW(actual, name, len(name)))
        return {'windowFound': bool(actual), 'rootMatchesTarget': root == hwnd,
                'belongsToTargetPid': owner_ok and owner.value == pid,
                'targetFamilyHwnd': actual if related else None,
                'targetFamilyClass': name.value if class_ok else None,
                'scope': 'point query only; unrelated window identities omitted; not click delivery'}

    def caption(self, hwnd):
        info = TITLEBARINFOEX()
        info.cbSize = ctypes.sizeof(info)
        query = self.message(hwnd, 0x33f, lp=ctypes.addressof(info))  # WM_GETTITLEBARINFOEX
        return {**query, 'titleBarRect': info.rcTitleBar.values() if query['ok'] else None,
                'slots': [{'rect': r.values(), 'stateFlags': int(info.rgstate[i])}
                          for i, r in enumerate(info.rgrect)] if query['ok'] else []}

    def window_state(self, hwnd):
        rect = RECT()
        placement = WINDOWPLACEMENT()
        placement.length = ctypes.sizeof(placement)
        if not self.user.GetWindowRect(hwnd, ctypes.byref(rect)) or not self.user.GetWindowPlacement(hwnd, ctypes.byref(placement)):
            raise ctypes.WinError(ctypes.get_last_error())
        cursor = POINT()
        cursor_ok = self.user.GetCursorPos(ctypes.byref(cursor))
        dwm = {}
        for name, attr, value in [('backdrop', 38, ctypes.c_int32()),
                                  ('captionButtonBounds', 5, RECT()), ('extendedFrameBounds', 9, RECT())]:
            result = self.dwm.DwmGetWindowAttribute(hwnd, attr, ctypes.byref(value), ctypes.sizeof(value))
            dwm[name] = {'hresult': result, 'value': (value.values() if isinstance(value, RECT) else value.value) if result >= 0 else None}
        ctypes.set_last_error(0)
        raw_style = self.user.GetWindowLongW(hwnd, -16)
        style_error = ctypes.get_last_error() if raw_style == 0 else 0
        style = (raw_style & 0xffffffff) if not style_error else None
        return {'windowRect': rect.values(), 'normalPositionWorkspaceRect': placement.normalPosition.values(),
                'showCmd': placement.showCmd, 'dpi': self.user.GetDpiForWindow(hwnd),
                'visible': bool(self.user.IsWindowVisible(hwnd)), 'minimized': bool(self.user.IsIconic(hwnd)),
                'maximized': bool(self.user.IsZoomed(hwnd)), 'foreground': self.user.GetForegroundWindow() == hwnd,
                'style': style, 'styleLastError': style_error,
                'hasMaximizeBox': bool(style & 0x10000) if style is not None else None,
                'hasSystemMenu': bool(style & 0x80000) if style is not None else None,
                'cursorPhysicalScreenPoint': [cursor.x, cursor.y] if cursor_ok else None, 'dwm': dwm,
                'dwmCaptionButtonBoundsSpace': 'relative to window, not screen'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pid', type=int, required=True)
    parser.add_argument('--hwnd', type=lambda value: int(value, 0), required=True)
    parser.add_argument('--expected-sha256', required=True)
    parser.add_argument('--expected-creation-filetime', type=int)
    parser.add_argument('--output', type=Path, required=True, help='new file; existing evidence is never overwritten')
    args = parser.parse_args()
    if args.pid <= 0 or args.hwnd <= 0 or not re.fullmatch('[0-9a-fA-F]{64}', args.expected_sha256):
        parser.error('positive PID/HWND and a complete SHA256 are required')
    if args.output.exists():
        parser.error('output already exists; choose a fresh evidence filename')
    native = NativeWindows()
    try:
        result = collect_snapshot(native, args.pid, args.hwnd, args.expected_sha256, args.expected_creation_filetime)
    finally:
        native.close()
    with args.output.open('x', encoding='utf-8', newline='\n') as stream:
        json.dump(result, stream, ensure_ascii=False, indent=2)
        stream.write('\n')
    print('Saved read-only caption snapshot; no native input or acceptance verdict.')


if __name__ == '__main__':
    main()
