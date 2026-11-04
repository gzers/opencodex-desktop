import ctypes,json,hashlib,sys
from pathlib import Path
exe=Path('D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain/post021-materials/candidate/opencodex-desktop.exe')
if len(sys.argv)>1:exe=Path(sys.argv[1])
manifest=Path('D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain/test-manifest/cc6.manifest').read_bytes()
k=ctypes.WinDLL('kernel32',use_last_error=True)
k.BeginUpdateResourceW.argtypes=[ctypes.c_wchar_p,ctypes.c_bool];k.BeginUpdateResourceW.restype=ctypes.c_void_p
k.UpdateResourceW.argtypes=[ctypes.c_void_p,ctypes.c_void_p,ctypes.c_void_p,ctypes.c_ushort,ctypes.c_void_p,ctypes.c_uint];k.UpdateResourceW.restype=ctypes.c_bool
k.EndUpdateResourceW.argtypes=[ctypes.c_void_p,ctypes.c_bool];k.EndUpdateResourceW.restype=ctypes.c_bool
before=hashlib.sha256(exe.read_bytes()).hexdigest()
handle=k.BeginUpdateResourceW(str(exe),False)
if not handle:raise ctypes.WinError(ctypes.get_last_error())
buffer=ctypes.create_string_buffer(manifest)
if not k.UpdateResourceW(handle,24,1,1033,buffer,len(manifest)):
    error=ctypes.WinError(ctypes.get_last_error());k.EndUpdateResourceW(handle,True);raise error
if not k.EndUpdateResourceW(handle,False):raise ctypes.WinError(ctypes.get_last_error())
data={'reason':'Local GNU driver needs the same Common Controls v6 manifest that build.rs links for the production MSVC target. Only PE resource 24/1 is added; product source and frontend assets unchanged.','beforeSha256':before,'afterSha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'manifest':manifest.decode()}
exe.with_name('manifest-adaptation.json').write_text(json.dumps(data,indent=2)+'\n',encoding='utf-8')
print(json.dumps(data))
