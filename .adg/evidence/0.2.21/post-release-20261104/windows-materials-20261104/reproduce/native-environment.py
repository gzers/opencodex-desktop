import ctypes,json,sys
from pathlib import Path
u=ctypes.WinDLL('user32',use_last_error=True)
u.GetDpiForWindow.argtypes=[ctypes.c_void_p];u.GetDpiForWindow.restype=ctypes.c_uint
u.GetWindowThreadProcessId.argtypes=[ctypes.c_void_p,ctypes.POINTER(ctypes.c_uint)]
u.GetSystemMetrics.argtypes=[ctypes.c_int];u.GetSystemMetrics.restype=ctypes.c_int
pid=ctypes.c_uint()
hwnd=int(sys.argv[1]);u.GetWindowThreadProcessId(hwnd,ctypes.byref(pid))
data={'hwnd':hwnd,'pid':pid.value,'dpi':u.GetDpiForWindow(hwnd),'scalePercent':u.GetDpiForWindow(hwnd)/96*100,'remoteSessionMetric':bool(u.GetSystemMetrics(0x1000)),'remoteControlMetric':bool(u.GetSystemMetrics(0x2001)),'note':'Read-only Win32 environment queries; no system settings or window state are modified. Parsec and Microsoft Remote Display adapters are also enumerated in environment-os.json.'}
Path('D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain/post021-materials/native-environment.json').write_text(json.dumps(data,indent=2)+'\n',encoding='utf-8')
print(json.dumps(data))
