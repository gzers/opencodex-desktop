import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import statistics
import subprocess
import time

CONTAINER = Path('/private/tmp/ocx-0110-369a-candidate-tgpmm_zq')
OUT = CONTAINER / 'idle-sample-369a'
OUT.mkdir(exist_ok=True)
APP_PID = 97688
EXE = str(CONTAINER / 'test-install/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop')
BUNDLE = 'com.gzers.opencodex.desktop.sandbox0110369a'
SOURCE = '369ab12da55448fbfdc4d0c9bf03970b56fd6c27'
DURATION = 600
INTERVAL = 10
HELPERS = ('com.apple.WebKit.GPU', 'com.apple.WebKit.WebContent', 'com.apple.WebKit.Networking')
attributions = []
known = {}

def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()

def run(argv):
    return subprocess.run(argv, capture_output=True, text=True, timeout=8)

def cpu_seconds(value):
    days = 0
    if '-' in value:
        day, value = value.split('-', 1)
        days = int(day)
    parts = [float(x) for x in value.split(':')]
    total = 0.0
    for part in parts:
        total = total * 60 + part
    return total + days * 86400

def processes():
    r = run(['ps', '-axo', 'pid=,ppid=,lstart=,time=,rss=,comm='])
    if r.returncode:
        raise RuntimeError('ps failed: ' + str(r.returncode))
    result = []
    for line in r.stdout.splitlines():
        parts = line.strip().split(None, 9)
        if len(parts) != 10:
            continue
        pid, ppid = map(int, parts[:2])
        command = parts[9]
        if command != EXE and not command.endswith(HELPERS):
            continue
        result.append(dict(pid=pid, ppid=ppid, started=' '.join(parts[2:7]),
                           cpu_seconds=cpu_seconds(parts[7]), rss_kib=int(parts[8]), executable=command))
    return result

def attribute(row, force=False):
    key = (row['pid'], row['started'])
    if key in known and not force:
        return known[key]
    r = run(['launchctl', 'print', 'pid/' + str(row['pid'])])
    blocks = {}
    for label in ['resource', 'jetsam']:
        m = re.search(r'(?m)^\s*' + label + r' coalition = \{\n(.*?)\n\s*\}', r.stdout, re.S)
        if m:
            fields = {}
            for line in m.group(1).splitlines():
                if ' = ' in line:
                    k, v = line.strip().split(' = ', 1)
                    if k in ['ID', 'type', 'state', 'name', 'bundle ID', 'active count']:
                        fields[k] = v
            blocks[label] = fields
    resource = blocks.get('resource', {})
    admitted = r.returncode == 0 and resource.get('bundle ID') == BUNDLE
    attributions.append(dict(captured_at=utc(), pid=row['pid'], started=row['started'],
                             executable=row['executable'], exit_code=r.returncode,
                             coalitions=blocks, admitted=admitted))
    known[key] = admitted
    return admitted

def files_summary():
    base = CONTAINER / 'test-data'
    result = []
    for directory in ['cache/updates', 'manager-state']:
        for p in sorted((base / directory).glob('*.json')):
            if directory == 'manager-state' and p.name not in ['update-schedule.json', 'preferences.json']:
                continue
            raw = p.read_bytes()
            result.append(dict(relative_path=str(p.relative_to(base)), bytes=len(raw),
                               sha256=hashlib.sha256(raw).hexdigest(), mtime_ns=p.stat().st_mtime_ns))
    return result

prefs = json.loads((CONTAINER / 'test-data/manager-state/preferences.json').read_text())
context = dict(source_sha=SOURCE, executable=EXE, app_pid=APP_PID, bundle_id=BUNDLE,
               test_exe_sha256=hashlib.sha256(Path(EXE).read_bytes()).hexdigest(),
               appearance=prefs.get('appearance'), updates=prefs.get('updates'),
               host=dict(os=run(['sw_vers']).stdout.strip().splitlines(),
                         arch=run(['uname', '-m']).stdout.strip(), logical_cpus=os.cpu_count()),
               requested_duration_seconds=DURATION, interval_seconds=INTERVAL,
               scene='native overview preparation state; Node/npm found, OpenCodex missing',
               focus_visibility='only initial/final bound-app AX/screenshots; not continuously verified',
               limits=['no controlled 0.1.9 baseline', 'no main-thread instrumentation',
                       '10-second process polling can miss short-lived helpers',
                       'no network/syscall tracing; unchanged files do not prove zero network',
                       'preparation scene is not ready-state overview',
                       'RSS sum is per-process resident sum, not unique physical memory'],
               files_before=files_summary(), started_at=utc())
(OUT / 'context.json').write_text(json.dumps(context, ensure_ascii=False, indent=2) + '\n')
samples = []
started = time.monotonic()
overhead_started = time.process_time()
with (OUT / 'samples.jsonl').open('w') as stream:
    for index in range(DURATION // INTERVAL + 1):
        target = started + index * INTERVAL
        if target > time.monotonic():
            time.sleep(target - time.monotonic())
        sample_start = time.monotonic()
        rows = [r for r in processes() if attribute(r, force=index == DURATION // INTERVAL)]
        apps = [r for r in rows if r['executable'] == EXE]
        if len(apps) != 1 or apps[0]['pid'] != APP_PID:
            raise RuntimeError('candidate changed or missing; abort observation')
        coal_ids = {a['coalitions'].get('resource', {}).get('ID') for a in attributions if a['admitted']}
        if len(coal_ids) != 1:
            raise RuntimeError('multiple candidate resource coalitions; abort observation')
        sample = dict(index=index, captured_at=utc(), elapsed_seconds=time.monotonic()-started,
                      processes=rows, total_rss_kib=sum(r['rss_kib'] for r in rows),
                      collection_wall_seconds=time.monotonic()-sample_start)
        samples.append(sample)
        stream.write(json.dumps(sample, ensure_ascii=False) + '\n')
        stream.flush()
        if index % 6 == 0:
            print(json.dumps(dict(index=index, elapsed=round(sample['elapsed_seconds'], 2),
                                  processes=len(rows), rss_mib=round(sample['total_rss_kib']/1024, 2))), flush=True)

(OUT / 'attributions.json').write_text(json.dumps(attributions, indent=2) + '\n')
first, last = samples[0], samples[-1]
identity = lambda row: (row['pid'], row['started'])
initial = {identity(r): r for r in first['processes']}
final = {identity(r): r for r in last['processes']}
identities = [{identity(r) for r in s['processes']} for s in samples]
stable = all(ids == set(initial) for ids in identities)
elapsed = last['elapsed_seconds'] - first['elapsed_seconds']
cpu_delta = sum(final[k]['cpu_seconds']-initial[k]['cpu_seconds'] for k in initial if k in final)
summary = dict(source_sha=SOURCE, started_at=first['captured_at'], ended_at=last['captured_at'],
               elapsed_seconds=elapsed, sample_count=len(samples), process_identity_stable=stable,
               cpu_seconds_delta=round(cpu_delta, 4),
               average_cpu_one_core_percent=round(cpu_delta/elapsed*100, 5) if stable else None,
               rss_mib=dict(start=first['total_rss_kib']/1024, end=last['total_rss_kib']/1024,
                            min=min(s['total_rss_kib'] for s in samples)/1024,
                            max=max(s['total_rss_kib'] for s in samples)/1024),
               collector_cpu_seconds=time.process_time()-overhead_started,
               collection_wall_seconds=sum(s['collection_wall_seconds'] for s in samples),
               files_after=files_summary(), performance_gate='not_evaluated_no_paired_baseline',
               focus_visibility='not continuously observed; scheduler-active attribution not proven')
(OUT / 'summary.json').write_text(json.dumps(summary, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(summary, ensure_ascii=False), flush=True)
