from pathlib import Path
import ctypes as c
from ctypes import wintypes as w
import hashlib,json,re,subprocess,time,sys
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
sha=lambda b:hashlib.sha256(b).hexdigest()
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
producer='library03';artifact=load(ROOT/f'{producer}-artifact.json');exe=Path(artifact['path'])
assert sha(exe.read_bytes())==artifact['sha256']
for row in load(ROOT/producer/'source-manifest.json'):
    raw=(CODE/row['path']).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
full=len(sys.argv)>1 and sys.argv[1]=='full'
out=ROOT/('profile02' if full else 'profile01');out.mkdir(exist_ok=False)
K=c.WinDLL('kernel32',use_last_error=True)
class Counters(c.Structure):
    _fields_=[(name,c.c_ulonglong) for name in ['readOps','writeOps','otherOps','readBytes','writeBytes','otherBytes']]
K.OpenProcess.argtypes=[w.DWORD,w.BOOL,w.DWORD];K.OpenProcess.restype=w.HANDLE
K.GetProcessIoCounters.argtypes=[w.HANDLE,c.POINTER(Counters)];K.GetProcessIoCounters.restype=w.BOOL
K.GetProcessTimes.argtypes=[w.HANDLE,c.POINTER(w.FILETIME),c.POINTER(w.FILETIME),c.POINTER(w.FILETIME),c.POINTER(w.FILETIME)];K.GetProcessTimes.restype=w.BOOL
K.CloseHandle.argtypes=[w.HANDLE];K.CloseHandle.restype=w.BOOL
K.GetSystemTimes.argtypes=[c.POINTER(w.FILETIME),c.POINTER(w.FILETIME),c.POINTER(w.FILETIME)];K.GetSystemTimes.restype=w.BOOL
args=['infrastructure::windows_client_originals024::tree::anchored::observation::retirement::completed_tests::wl2_r6_completed_history_native_original_expiry_allows_fresh_read_but_never_append','--exact','--test-threads=1','--nocapture']
if full:
    producer_args=load(ROOT/producer/'source.json')['arguments'];args=producer_args[producer_args.index('--')+1:]
start=time.monotonic();samples=[]
with (out/'stdout.log').open('xb') as stdout,(out/'stderr.log').open('xb') as stderr:
    child=subprocess.Popen([str(exe),*args],cwd=CODE,stdin=subprocess.DEVNULL,stdout=stdout,stderr=stderr,creationflags=0x08000000)
    process=K.OpenProcess(0x400,False,child.pid)
    if not process:raise c.WinError(c.get_last_error())
    try:
        while True:
            kernel=w.FILETIME();user=w.FILETIME();created=w.FILETIME();exited=w.FILETIME();io=Counters()
            cpu_ok=K.GetProcessTimes(process,c.byref(created),c.byref(exited),c.byref(kernel),c.byref(user))
            io_ok=K.GetProcessIoCounters(process,c.byref(io))
            sample=dict(elapsedMs=round((time.monotonic()-start)*1000),cpuQueryOk=bool(cpu_ok),ioQueryOk=bool(io_ok))
            if cpu_ok:sample.update(kernelMs=((kernel.dwHighDateTime<<32)|kernel.dwLowDateTime)/10000,userMs=((user.dwHighDateTime<<32)|user.dwLowDateTime)/10000)
            else:sample['cpuRawError']=c.get_last_error()
            if io_ok:sample.update({name:getattr(io,name) for name,_ in io._fields_})
            else:sample['ioRawError']=c.get_last_error()
            idle_system=w.FILETIME();kernel_system=w.FILETIME();user_system=w.FILETIME()
            if K.GetSystemTimes(c.byref(idle_system),c.byref(kernel_system),c.byref(user_system)):
                sample['systemTime100ns']={name:(value.dwHighDateTime<<32)|value.dwLowDateTime for name,value in [('idle',idle_system),('kernelIncludesIdle',kernel_system),('user',user_system)]}
            samples.append(sample)
            status=child.poll()
            if status is not None:break
            time.sleep(.25)
    finally:assert K.CloseHandle(process)
elapsed=round((time.monotonic()-start)*1000)
assert sha(exe.read_bytes())==artifact['sha256']
text=(out/'stdout.log').read_text(encoding='utf-8-sig')
summary=re.search(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored',text)
result=dict(producer=producer,program=str(exe),arguments=args,programSha256=artifact['sha256'],rawExit=status,elapsedMs=elapsed,
    sourceInputsByteIdentical=True,executableBeforeAfterHashEqual=True,passed=int(summary.group(1)) if summary else None,failed=int(summary.group(2)) if summary else None,
    samples=samples,acceptanceRun=False,diagnosticOnly=True,fullOriginalDirectedScope=full,nativeCutoffChanged=False,newBuild=False,forcedTermination=False)
with (out/'result.json').open('x',encoding='utf-8') as target:json.dump(result,target,indent=2)
last=samples[-1];print(json.dumps(dict(rawExit=status,elapsedMs=elapsed,passed=result['passed'],failed=result['failed'],lastCounters=last,diagnosticOnly=True)))
