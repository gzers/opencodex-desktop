from pathlib import Path
import hashlib,json,os,subprocess,time
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
exe=CODE/'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/build/opencodex-desktop/182e378daf0091d7/out/opencodex_desktop_lib-182e378daf0091d7.exe'
artifact=hashlib.sha256(exe.read_bytes()).hexdigest()
runs=[('native05','wl2_r7_history_primitives_native_foreign_actual_job_'),('regression06','wl2_r7_attachment_storage_native_'),('regression07','wl2_r7_v2_storage_native_'),('regression08','wl2_r7_quiesce_join_native_'),('regression09','wl2_r7_concrete_driver_native_')]
for name,case in runs:
    root=ROOT/name;root.mkdir(exist_ok=False)
    args=[case,'--test-threads=1','--nocapture']
    env=os.environ.copy()
    # All filters are explicit affected native test prefixes, not W2
    start=time.monotonic()
    with (root/'stdout.log').open('xb') as out,(root/'stderr.log').open('xb') as err:
        result=subprocess.run([str(exe),*args],cwd=CODE,env=env,stdout=out,stderr=err)
    record=dict(arguments=args,raw=result.returncode,elapsedMs=round((time.monotonic()-start)*1000),artifactSha256=artifact,currentR2Touched=False,scope='affected native Job history, original V1/V2 storage, sameclaim quiescence and native pipeline; noW2 or official run')
    (root/'result.json').write_text(json.dumps(record,indent=2),encoding='utf-8')
    print(json.dumps(dict(run=name,**record)),flush=True)
    if result.returncode:raise SystemExit(result.returncode)
