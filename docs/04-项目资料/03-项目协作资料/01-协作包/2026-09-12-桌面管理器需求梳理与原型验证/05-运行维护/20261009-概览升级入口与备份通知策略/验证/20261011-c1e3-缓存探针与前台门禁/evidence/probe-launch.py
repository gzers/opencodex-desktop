import json, os, subprocess, sys, time
from pathlib import Path
base=Path('/private/tmp/ocx-0110-c1e3-candidate-hqjf10k6')
exe=base/'test-install/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop'
phase=sys.argv[1]
assert phase.replace('-', '').isalnum()
rows=subprocess.check_output(['ps','-axo','pid=,command='], text=True).splitlines()
assert not any(row.strip().split(None,1)[1] == str(exe) for row in rows if len(row.strip().split(None,1))==2), 'Test already running'
log=base/f'probe-{phase}.jsonl'
assert not log.exists(), 'Never overwrite evidence'
env=dict(os.environ, OPENCODEX_SANDBOX='1', OPENCODEX_SANDBOX_ROOT=str(base/'test-data'), OPENCODEX_SANDBOX_BOUNDARY=str(base), DYLD_INSERT_LIBRARIES=str(base/'update-probe.dylib'), OPENCODEX_UPDATE_PROBE_LOG=str(log))
with (base/f'probe-{phase}-console.log').open('xb') as out:
    process=subprocess.Popen([str(exe)], env=env, stdout=out, stderr=subprocess.STDOUT, start_new_session=True)
    time.sleep(2)
    assert process.poll() is None, 'Candidate failed to start'
rows=[json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []
assert any(r['event']=='probe_loaded' and r['pid']==process.pid for r in rows), 'Injection not active'
value={'pid':process.pid,'phase':phase,'started_at':time.time(),'source_sha':'c1e3c09d8ed33238543ce5943e439991b8099088','log':str(log),'injection_loaded':True}
(base/f'probe-{phase}-launch.json').write_text(json.dumps(value,indent=2)+chr(10))
print(json.dumps(value))
