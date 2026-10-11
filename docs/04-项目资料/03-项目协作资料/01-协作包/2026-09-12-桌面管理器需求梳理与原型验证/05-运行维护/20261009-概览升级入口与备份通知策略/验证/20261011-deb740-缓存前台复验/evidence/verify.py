import collections,hashlib,json,subprocess,sys,time
from pathlib import Path
base=Path('/private/tmp/ocx-0110-deb740-foreground')
phase=sys.argv[1]
launch=json.loads((base/f'probe-{phase}-launch.json').read_text())
rows=[json.loads(x) for x in (base/f'probe-{phase}.jsonl').read_text().splitlines()]
assert rows and all(x['event']=='probe_loaded' and x['pid']==launch['pid'] for x in rows),rows
assert len(rows)==1,rows
before=json.loads((base/'cache-before.json').read_text())
files={}
for name,expected in before['files'].items():
 data=(base/'test-data'/name).read_bytes()
 files[name]={'sha256':hashlib.sha256(data).hexdigest(),'bytes':len(data)}
 assert files[name]==expected,(name,files[name],expected)
assert all(x['next_due']>time.time() and x['failures']==0 for x in before['schedule']['entries'].values())
foreground=None
if phase.startswith('restart-'):
 monitor_name = sys.argv[2] if len(sys.argv) > 2 else f'probe-{phase}-foreground.json'
 foreground=json.loads((base/monitor_name).read_text())
 assert foreground['pid']==launch['pid'] and foreground['all_samples_frontmost_visible']
 assert foreground['ended_at']-foreground['started_at']>=46
value={'phase':phase,'verified_at':time.time(),'pid':launch['pid'],'elapsed_since_launch_s':time.time()-launch['started_at'],'boundary_attempts':0,'probe_loaded_records':len(rows),'cache_and_schedule_unchanged':True,'files':files,'foreground_duration_s':None if foreground is None else foreground['ended_at']-foreground['started_at'],'foreground_sample_count':None if foreground is None else len(foreground['samples']),'all_foreground_samples_visible':None if foreground is None else foreground['all_samples_frontmost_visible'],'foreground_file':None if foreground is None else monitor_name,'scope':'instrumented main process DNS/connect/exec/spawn boundary; not system-wide traffic or CPU/startup measurement'}
with (base/f'probe-{phase}-verification.json').open('x') as out:json.dump(value,out,indent=2);out.write(chr(10))
print(json.dumps(value))
