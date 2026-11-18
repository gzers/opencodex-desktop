from pathlib import Path
import hashlib,json,subprocess,time
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
exe=CODE/'test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/build/opencodex-desktop/182e378daf0091d7/out/opencodex_desktop_lib-182e378daf0091d7.exe';artifact=hashlib.sha256(exe.read_bytes()).hexdigest()
for name,case in [('regression02','wl2_r7_prefix_read_tree_native_'),('regression03','wl2_r7_v2_reference_native_'),('regression04','wl2_r7_partial_capture_native_'),('regression05','wl2_r7_concrete_driver_native_'),('regression06','infrastructure::windows_client_originals024::adapter::tests::wl2_r6_typed_adapter_retained_v3_slot_and_private_observation_types_are_send')]:
    root=ROOT/name;root.mkdir(exist_ok=False);args=[case,'--test-threads=1','--nocapture']
    if name=='regression06':args.insert(1,'--exact')
    start=time.monotonic()
    with (root/'stdout.log').open('xb') as out,(root/'stderr.log').open('xb') as err:result=subprocess.run([str(exe),*args],cwd=CODE,stdout=out,stderr=err)
    record=dict(arguments=args,raw=result.returncode,elapsedMs=round((time.monotonic()-start)*1000),artifactSha256=artifact,scope='affected oldnative treeprefix/V2mapping/partialcapture/canonicalpipeline/Send only; no oldr2/GUI/CLI/MSI/scan/W2',currentR2Touched=False)
    (root/'result.json').write_text(json.dumps(record,indent=2),encoding='utf-8');print(json.dumps(dict(run=name,**record)),flush=True)
    if result.returncode:raise SystemExit(result.returncode)
