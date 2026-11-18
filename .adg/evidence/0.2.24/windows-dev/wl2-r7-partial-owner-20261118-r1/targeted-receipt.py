from pathlib import Path
import json, subprocess, time
root=Path(__file__).resolve().parent
exe=json.loads((root/'compile-test-05/artifacts.json').read_bytes())[0]['path']
run=root/'targeted-03'
run.mkdir()
filters=['wl2_r7_pending_attachment_', 'wl2_r7_bound_preparation_', 'wl2_r7_partial_owner_bootstrap_', 'wl2_r7_worker_native_failure_', 'wl2_r7_start_worker_', 'wl2_r7_private_worker_wait_', 'wl2_r7_preparation_image_scan_']
results=[]
for name in filters:
    begin=time.monotonic()
    args=[exe,name,'--nocapture','--test-threads=1']
    with (run/(name+'.stdout.log')).open('xb') as out, (run/(name+'.stderr.log')).open('xb') as err:
        done=subprocess.run(args,stdout=out,stderr=err)
    results.append(dict(filter=name,raw=done.returncode,elapsedMs=round((time.monotonic()-begin)*1000),arguments=args[1:]))
    (run/'result.json').write_text(json.dumps(results,indent=2)+'\n',encoding='utf-8')
    print(results[-1],flush=True)
raise SystemExit(any(r['raw'] for r in results))

