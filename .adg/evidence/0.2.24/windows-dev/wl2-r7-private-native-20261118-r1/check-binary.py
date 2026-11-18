from pathlib import Path
import hashlib,json,subprocess,time
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
exe=CODE/'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/ocxd.exe'
sha=hashlib.sha256(exe.read_bytes()).hexdigest()
cases=[[],['begin-canonical-abort'],['begin-canonical-abort','--confirm','extra'],['read-worker-diagnostic','--confirm'],['read-canonical-history','21500'],['explicit-recovery'],['force']]
records=[]
for ordinal,args in enumerate(cases):
    start=time.monotonic();actual=['--internal-before-resume-control024-v2',*args]
    result=subprocess.run([str(exe),*actual],stdin=subprocess.DEVNULL,capture_output=True,timeout=10,cwd=CODE)
    (ROOT/f'cli{ordinal:02}.stdout.log').write_bytes(result.stdout)
    (ROOT/f'cli{ordinal:02}.stderr.log').write_bytes(result.stderr)
    record=dict(arguments=actual,raw=result.returncode,elapsedMs=round((time.monotonic()-start)*1000),stage='exact Windows argv guard only, before bootstrap/peer/endpoint/Owner')
    records.append(record)
    assert result.returncode==2 and not result.stdout and b'Exact V2 operation required' in result.stderr,record
assert hashlib.sha256(exe.read_bytes()).hexdigest()==sha
(ROOT/'binary-argv-check.json').write_text(json.dumps(dict(artifactSha256=sha,artifactBytes=exe.stat().st_size,cases=records,nativeEndpointCreated=False,currentR2Touched=False,officialExecuted=False),indent=2),encoding='utf-8')
print(json.dumps(dict(binarySha256=sha,bytes=exe.stat().st_size,invalidArgumentsRefused=len(records),nativeCalls=False)))
