import ctypes,hashlib,json,pathlib,shutil,subprocess,os
build_root=pathlib.Path('C:/Users/15119/.codex/visualizations/2026/11/06/01a1b0c8-944d-75d0-9a8b-c2e46c8963a1/L4-R1-20261106')
root=pathlib.Path('C:\\Users\\15119\\.codex\\visualizations\\2026\\11\\06\\01a1b0c8-944d-75d0-9a8b-c2e46c8963a1\\L4-R2-04511769-20261106')
source=pathlib.Path('C:/Users/15119/.codex/visualizations/2026/11/06/01a1b0c8-944d-75d0-9a8b-c2e46c8963a1/L4-R1-20261106/product-target\debug\deps\ocxd-030ba1b9b66e6cbb.exe')
root=root/'cli-tests'
root.mkdir(exist_ok=False)
out=root/'native-product-tests';out.mkdir(exist_ok=True)
target=out/'product-tests.exe';shutil.copy2(source,target)
loader=build_root/'product-target/debug/WebView2Loader.dll'
shutil.copy2(loader,out/'WebView2Loader.dll')
manifest=b'<?xml version="1.0" encoding="UTF-8" standalone="yes"?><assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0"><dependency><dependentAssembly><assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*"/></dependentAssembly></dependency></assembly>'
kernel=ctypes.WinDLL('kernel32',use_last_error=True)
kernel.BeginUpdateResourceW.argtypes=[ctypes.c_wchar_p,ctypes.c_int];kernel.BeginUpdateResourceW.restype=ctypes.c_void_p
kernel.UpdateResourceW.argtypes=[ctypes.c_void_p,ctypes.c_void_p,ctypes.c_void_p,ctypes.c_ushort,ctypes.c_void_p,ctypes.c_uint];kernel.UpdateResourceW.restype=ctypes.c_int
kernel.EndUpdateResourceW.argtypes=[ctypes.c_void_p,ctypes.c_int];kernel.EndUpdateResourceW.restype=ctypes.c_int
h=kernel.BeginUpdateResourceW(str(target),0)
if not h:raise ctypes.WinError(ctypes.get_last_error())
buf=ctypes.create_string_buffer(manifest)
if not kernel.UpdateResourceW(h,ctypes.c_void_p(24),ctypes.c_void_p(1),0,buf,len(manifest)):
 error=ctypes.get_last_error();kernel.EndUpdateResourceW(h,1);raise ctypes.WinError(error)
if not kernel.EndUpdateResourceW(h,0):raise ctypes.WinError(ctypes.get_last_error())
record={'kind':'GNU native libtest harness only; not MSVC or product/release acceptance','source':str(source),'sourceSha256':hashlib.sha256(source.read_bytes()).hexdigest(),'adapted':str(target),'adaptedSha256':hashlib.sha256(target.read_bytes()).hexdigest(),'change':'RT_MANIFEST 24/1 neutral; Common Controls v6 dependency only','loaderSource':str(loader),'loaderSha256':hashlib.sha256(loader.read_bytes()).hexdigest()}
(root/'gnu-test-manifest.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8');print(json.dumps(record))
r=subprocess.run([str(target),'','--test-threads=1','--nocapture'],capture_output=True)
(root/'product-startup-with-manifest.log').write_bytes(r.stdout+b'\n'+r.stderr);print(r.stdout.decode('utf-8',errors='replace'));print(r.stderr.decode('utf-8',errors='replace'));print('exit_code='+str(r.returncode));raise SystemExit(0 if r.returncode==0 else 1)
