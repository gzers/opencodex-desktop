import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

BASE = Path(__file__).resolve().parent
CONFIG = json.loads((BASE / 'candidates.json').read_bytes())
HELPERS = ('com.apple.WebKit.GPU', 'com.apple.WebKit.WebContent', 'com.apple.WebKit.Networking')
DAILY = '/Applications/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop'

def run(argv):
    return subprocess.run(argv, capture_output=True, text=True, timeout=10)

def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()

def all_apps():
    r = run(['ps', '-axo', 'pid=,lstart=,comm='])
    assert r.returncode == 0
    rows = []
    for line in r.stdout.splitlines():
        fields = line.strip().split(None, 6)
        if len(fields) == 7 and fields[6].endswith('/opencodex-desktop'):
            rows.append(dict(pid=int(fields[0]), started=' '.join(fields[1:6]), exe=fields[6]))
    return rows

def files(root):
    result = []
    for relative in ('manager-state/preferences.json', 'manager-state/update-schedule.json',
                     'cache/updates/manager-stable.json', 'cache/updates/panel-latest.json'):
        p = Path(root) / relative
        if p.is_file():
            raw = p.read_bytes()
            result.append(dict(path=relative, size=len(raw), sha256=hashlib.sha256(raw).hexdigest(), mtime_ns=p.stat().st_mtime_ns))
    return result

def cpu(value):
    days = 0
    if '-' in value:
        days, value = value.split('-', 1)
        days = int(days)
    seconds = 0.0
    for part in value.split(':'):
        seconds = seconds * 60 + float(part)
    return days * 86400 + seconds

def launch(key, phase):
    c = CONFIG[key]
    active = [r for r in all_apps() if r['exe'] != DAILY]
    assert not active, ('Other test instance still running', active)
    out = BASE / key / phase
    out.mkdir()
    argv = ['open', '-n', '-a', c['app'], '--stdout', str(out / 'stdout.log'), '--stderr', str(out / 'stderr.log')]
    for k, v in dict(OPENCODEX_SANDBOX='1', OPENCODEX_SANDBOX_ROOT=c['root'], OPENCODEX_SANDBOX_BOUNDARY=c['boundary']).items():
        argv += ['--env', k + '=' + v]
    before = all_apps()
    started = time.monotonic()
    r = run(argv)
    record = dict(started_at=utc(), monotonic_start=started, argv=argv, returncode=r.returncode, stdout=r.stdout, stderr=r.stderr, apps_before=before)
    (out / 'launch.json').write_text(json.dumps(record, indent=2) + '\n')
    assert r.returncode == 0
    for _ in range(100):
        found = [row for row in all_apps() if row['exe'] == c['exe']]
        if len(found) == 1:
            record.update(pid=found[0]['pid'], process_observed_seconds=time.monotonic()-started, apps_after=all_apps())
            (out / 'launch.json').write_text(json.dumps(record, indent=2) + '\n')
            print(json.dumps(record), flush=True)
            return
        time.sleep(.1)
    raise RuntimeError('Launched process missing')

def observe(key, phase, duration):
    c = CONFIG[key]
    out = BASE / key / phase
    launch_record = json.loads((out / 'launch.json').read_bytes())
    pid = launch_record['pid']
    known = {}
    attribution = []
    def admitted(row, force=False):
        identity = (row['pid'], row['started'])
        if identity in known and not force:
            return known[identity]
        r = run(['launchctl', 'print', 'pid/' + str(row['pid'])])
        m = re.search(r'(?m)^\s*resource coalition = \{\n(.*?)\n\s*\}', r.stdout, re.S)
        data = {}
        if m:
            for line in m.group(1).splitlines():
                if ' = ' in line:
                    k, v = line.strip().split(' = ', 1)
                    if k in ('ID', 'name', 'bundle ID', 'type', 'state', 'active count'):
                        data[k] = v
        ok = r.returncode == 0 and data.get('bundle ID') == c['bundle']
        attribution.append(dict(captured_at=utc(), identity=identity, exe=row['exe'], returncode=r.returncode, resource_coalition=data, admitted=ok))
        known[identity] = ok
        return ok
    def processes(force=False):
        r = run(['ps', '-axo', 'pid=,ppid=,lstart=,time=,rss=,comm='])
        assert r.returncode == 0
        rows = []
        for line in r.stdout.splitlines():
            f = line.strip().split(None, 9)
            if len(f) != 10 or (f[9] != c['exe'] and not f[9].endswith(HELPERS)):
                continue
            row = dict(pid=int(f[0]), ppid=int(f[1]), started=' '.join(f[2:7]), cpu_seconds=cpu(f[7]), rss_kib=int(f[8]), exe=f[9])
            if admitted(row, force):
                rows.append(row)
        app = [r for r in rows if r['exe'] == c['exe']]
        assert len(app) == 1 and app[0]['pid'] == pid, ('Application changed', app)
        active = [r for r in all_apps() if r['exe'] != DAILY]
        assert len(active) == 1 and active[0]['pid'] == pid, ('Other test instance', active)
        return rows
    assert (time.monotonic() - launch_record['monotonic_start']) >= 90, 'Minimum 90-second warmup required'
    context = dict(candidate=c, started_at=utc(), duration_seconds=duration, interval_seconds=10,
                   actual_warmup_seconds=time.monotonic()-launch_record['monotonic_start'],
                   host=dict(os=run(['sw_vers']).stdout, arch=run(['uname','-m']).stdout.strip(), logical_cpus=os.cpu_count(),
                             power=run(['pmset','-g','batt']).stdout, thermal=run(['pmset','-g','therm']).stdout),
                   files_before=files(c['root']), apps_before=all_apps(),
                   method='ps cumulative CPU/RSS for exact application and launchctl resource-coalition WebKit helpers; no DYLD probe',
                   limits=['10-second polling can miss transient helpers', 'RSS is resident sum, not unique physical footprint',
                           'Cua AX and screenshots are separate UI evidence; collection does not itself prove global foreground',
                           'Sequential single pairing; not randomized repeated runs', 'Preparation state only, not ready-state overview',
                           'No main-thread, network/syscall or startup interactive instrumentation'])
    (out / 'context.json').write_text(json.dumps(context,indent=2)+'\n')
    samples=[]
    begin=time.monotonic()
    overhead=time.process_time()
    with (out / 'samples.jsonl').open('x') as stream:
        for i in range(duration//10+1):
            due=begin+i*10
            if due>time.monotonic(): time.sleep(due-time.monotonic())
            wall=time.monotonic()
            rows=processes(force=i==duration//10)
            value=dict(index=i,captured_at=utc(),elapsed_seconds=time.monotonic()-begin,processes=rows,
                       total_rss_kib=sum(r['rss_kib'] for r in rows),collection_wall_seconds=time.monotonic()-wall)
            samples.append(value);stream.write(json.dumps(value)+'\n');stream.flush()
            if i%6==0: print(json.dumps(dict(index=i,elapsed_seconds=value['elapsed_seconds'],rss_mib=value['total_rss_kib']/1024)),flush=True)
    (out / 'attribution.json').write_text(json.dumps(attribution,indent=2)+'\n')
    identity=lambda r:(r['pid'],r['started'])
    first={identity(r):r for r in samples[0]['processes']};last={identity(r):r for r in samples[-1]['processes']}
    stable=all({identity(r) for r in s['processes']}==set(first) for s in samples)
    elapsed=samples[-1]['elapsed_seconds']-samples[0]['elapsed_seconds']
    delta=sum(last[k]['cpu_seconds']-first[k]['cpu_seconds'] for k in first if k in last)
    breakdown=[dict(exe=first[k]['exe'],pid=k[0],cpu_seconds_delta=last[k]['cpu_seconds']-first[k]['cpu_seconds']) for k in first if k in last]
    summary=dict(source=c['source_sha'],started_at=samples[0]['captured_at'],ended_at=samples[-1]['captured_at'],
                 elapsed_seconds=elapsed,samples=len(samples),process_identity_stable=stable,cpu_seconds_delta=delta,
                 cpu_one_core_percent=delta/elapsed*100 if stable else None,breakdown=breakdown,
                 rss_mib=dict(start=samples[0]['total_rss_kib']/1024,end=samples[-1]['total_rss_kib']/1024,
                              min=min(s['total_rss_kib'] for s in samples)/1024,max=max(s['total_rss_kib'] for s in samples)/1024,
                              mean=sum(s['total_rss_kib'] for s in samples)/len(samples)/1024),
                 collector_cpu_seconds=time.process_time()-overhead,collection_wall_seconds=sum(s['collection_wall_seconds'] for s in samples),
                 files_after=files(c['root']),apps_after=all_apps(),thermal_after=run(['pmset','-g','therm']).stdout,
                 gate='not_evaluated_until_matching_ui_and_comparison_review')
    (out / 'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    print(json.dumps(summary),flush=True)

if __name__=='__main__':
    action,key,phase=sys.argv[1:4]
    if action=='launch': launch(key,phase)
    elif action=='observe': observe(key,phase,int(sys.argv[4]))
    else: raise ValueError(action)
