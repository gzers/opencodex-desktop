from pathlib import Path
import hashlib,json,subprocess,time
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
exe=CODE/'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/build/opencodex-desktop/182e378daf0091d7/out/opencodex_desktop_lib-182e378daf0091d7.exe'
sha=hashlib.sha256(exe.read_bytes()).hexdigest()
runs=[('regression05','wl2_r6_failure_control_'),('regression06','wl2_strict_frames_'),('regression07','wl2_qpc_preserves_'),('regression08','wl2_h_bootstrap_rejects_')]
for name,case in runs:
    root=ROOT/name;root.mkdir(exist_ok=False);args=[case,'--test-threads=1','--nocapture'];start=time.monotonic()
    with (root/'stdout.log').open('xb') as out,(root/'stderr.log').open('xb') as err:result=subprocess.run([str(exe),*args],cwd=CODE,stdout=out,stderr=err)
    record=dict(raw=result.returncode,elapsedMs=round((time.monotonic()-start)*1000),arguments=args,artifactSha256=sha,scope='unchanged old v1 frame/clock/failurecontrol/bootstrap parsers only; no nativeHost or currentr2 call')
    (root/'result.json').write_text(json.dumps(record,indent=2),encoding='utf-8');print(json.dumps(dict(run=name,**record)),flush=True)
    if result.returncode:raise SystemExit(result.returncode)
