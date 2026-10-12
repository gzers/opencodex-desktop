"""Offline verification of interrupted diagnostic; never acceptance."""
import argparse
import datetime as dt
import gzip
import hashlib
import json
import math
from pathlib import Path

p = argparse.ArgumentParser()
p.add_argument('--root', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
root = a.root.resolve()
prefix = Path('candidate0110/diagnostic-mid-mesh-finder-reopen')
inputs = {}
def raw(path):
    f = root / path
    b = f.read_bytes() if f.exists() else gzip.decompress(Path(str(f)+'.gz').read_bytes())
    inputs[str(path)] = hashlib.sha256(b).hexdigest()
    return b
def read(path):
    return json.loads(raw(path))
def identity(x):
    return x['pid'], x['started'], x['exe']
def near(x, y):
    assert math.isclose(x, y, rel_tol=1e-10, abs_tol=1e-8), (x, y)
def utc(wall):
    return dt.datetime.fromtimestamp(wall, dt.timezone.utc).isoformat()
ctx, summary, launch, fg, before, after = [read(prefix/f) for f in (
    'context.json','summary.json','launch.json','foreground.json','settings-start.json','settings-end.json')]
s = [json.loads(x) for x in raw(prefix/'samples.jsonl').splitlines()]
assert len(s) == 19
ids = {identity(x) for x in s[0]['processes']}
assert len(ids) == 4
previous = {identity(x):x for x in s[0]['processes']}
for i, row in enumerate(s):
    current = {identity(x):x for x in row['processes']}
    assert row['index'] == i and set(current) == ids
    assert all(current[k]['cpu_seconds'] >= previous[k]['cpu_seconds'] for k in ids)
    assert sum(x['rss_kib'] for x in row['processes']) == row['total_rss_kib']
    assert all(x['rss_kib'] >= 0 for x in row['processes'])
    if i:
        assert row['elapsed_seconds'] > s[i-1]['elapsed_seconds']
        assert dt.datetime.fromisoformat(row['captured_at']) > dt.datetime.fromisoformat(s[i-1]['captured_at'])
    previous = current
assert (launch['pid'], s[0]['processes'][0]['started'], ctx['candidate']['exe']) in ids
assert ctx['candidate']['source_sha'] == summary['source'] == 'deb740c09857b5f0bef7c9b4090fe5e8587bb4b3'
assert summary['process_identity_stable'] is True
assert ctx['files_before'] == summary['files_after']
assert ctx['actual_warmup_seconds'] >= 90
assert before['settings'] == after['settings'] == dict(glow_render='mesh', interface_scale=100, theme='light', visual_effects='mid')
assert before['preferences_sha256'] == after['preferences_sha256']
assert next(x['sha256'] for x in ctx['files_before'] if x['path']=='manager-state/preferences.json') == before['preferences_sha256']
assert dt.datetime.fromisoformat(before['captured_at']) < dt.datetime.fromisoformat(s[0]['captured_at'])
assert dt.datetime.fromisoformat(after['captured_at']) > dt.datetime.fromisoformat(s[-1]['captured_at'])
assert ctx['candidate']['appearance']['visual_effects'] == 'high'
assert after['ui_capture'] == 'CUA reported Mac locked; end UI unavailable'
def metrics(rows):
    first, last = [{identity(x):x for x in r['processes']} for r in (rows[0],rows[-1])]
    duration = rows[-1]['elapsed_seconds'] - rows[0]['elapsed_seconds']
    cpu = sum(last[k]['cpu_seconds'] - first[k]['cpu_seconds'] for k in ids)
    rss = [r['total_rss_kib']/1024 for r in rows]
    return dict(sample_count=len(rows),start_utc=rows[0]['captured_at'],end_utc=rows[-1]['captured_at'],
        elapsed_seconds=duration,cpu_seconds_delta=cpu,cpu_one_core_percent=cpu/duration*100,
        rss_mib=dict(start=rss[0],end=rss[-1],min=min(rss),max=max(rss),mean=sum(rss)/len(rss)),
        breakdown=[dict(pid=k[0],cpu_seconds_delta=last[k]['cpu_seconds']-first[k]['cpu_seconds'],
            rss_start_mib=first[k]['rss_kib']/1024,rss_end_mib=last[k]['rss_kib']/1024) for k in sorted(ids)])
whole = metrics(s)
for k in ('elapsed_seconds','cpu_seconds_delta','cpu_one_core_percent'):
    near(whole[k],summary[k])
for k,v in whole['rss_mib'].items():
    near(v,summary['rss_mib'][k])
near(sum(x['collection_wall_seconds'] for x in s),summary['collection_wall_seconds'])
recorded = {x['pid']:x for x in summary['breakdown']}
for x in whole['breakdown']:
    near(x['cpu_seconds_delta'],recorded[x['pid']]['cpu_seconds_delta'])
for apps in (ctx['apps_before'],summary['apps_after']):
    daily = [x for x in apps if x['exe']=='/Applications/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop']
    tests = [x for x in apps if x not in daily]
    assert len(daily)==len(tests)==1
    assert daily[0]['pid']==75764 and daily[0]['started']=='Fri Oct 9 22:00:24 2026'
    assert identity(tests[0]) in ids and tests[0]['pid']==31501
f = fg['samples']
valid = lambda x: x['target_frontmost'] and x['target_window_on_screen']
assert len(f)==2427 and fg['pid']==31501
assert len(fg['readiness_samples']) >= 1 and valid(fg['readiness_samples'][-1])
assert all(f[i]['elapsed_s']>f[i-1]['elapsed_s'] and f[i]['wall_s']>f[i-1]['wall_s'] for i in range(1,len(f)))
invalid = [x for x in f if not valid(x)]
assert len(invalid)==367 and fg['all_samples_frontmost_visible'] is False
cut = invalid[0]['wall_s']
assert not invalid[0]['target_frontmost'] and invalid[0]['target_window_on_screen']
measurement_start = dt.datetime.fromisoformat(s[0]['captured_at']).timestamp()
measurement_end = dt.datetime.fromisoformat(s[-1]['captured_at']).timestamp()
assert f[0]['wall_s'] < measurement_start < cut < measurement_end < f[-1]['wall_s']
pre = [x for x in s if dt.datetime.fromisoformat(x['captured_at']).timestamp() < cut]
assert len(pre)==14
sub = metrics(pre)
sub['role'] = 'Partial diagnostic strictly before first invalid foreground sample; not acceptance or causal evidence'
raw_attr = root / prefix / 'attribution.json'
if raw_attr.exists():
    rows = read(prefix/'attribution.json'); admitted = [x for x in rows if x['admitted']]
    rejected_count = len(rows)-len(admitted)
else:
    deriv = read(prefix/'attribution-admitted.json'); admitted = deriv['entries']; rejected_count = deriv['rejected_entry_count']
assert len(admitted)==8
assert all(x['admitted'] and x['returncode']==0 and x['resource_coalition']['bundle ID']==ctx['candidate']['bundle'] for x in admitted)
assert {(x['identity'][0],x['identity'][1],x['exe']) for x in admitted} == ids
preparations = []
for phase in ('diagnostic-unlocked-mid-mesh','diagnostic-mid-mesh-active','diagnostic-mid-mesh-unlocked-final'):
    phase_path=Path('candidate0110')/phase
    for name in ('foreground.json','foreground-preflight.json'):
        if not (root/phase_path/name).exists(): continue
        v=read(phase_path/name); attempts=v['readiness_samples']
        assert v['error']=='foreground visible window not ready within 10s'
        assert v['pid']==31501 and all(not valid(x) for x in attempts) and 'samples' not in v
        assert not (root/phase_path/'samples.jsonl').exists()
        assert not (root/phase_path/'summary.json').exists()
        preparations.append(dict(phase=phase,monitor=name,readiness_attempt_count=len(attempts),
            first_utc=utc(attempts[0]['wall_s']),last_utc=utc(attempts[-1]['wall_s']),
            error=v['error'],resource_measurement_started=False,app_defect_attributed=False))
assert len(preparations)==4
out=dict(schema_version=1,kind='interrupted_midmesh_diagnostic_independent_recompute',
    actual_appearance=before['settings'],manifest_appearance=ctx['candidate']['appearance'],
    appearance_authority='Actual settings captures and stable preferences SHA; context candidate.appearance is a stale static manifest, raw retained unchanged',
    inputs_sha256=inputs,source=summary['source'],process_identities_stable=True,files_unchanged=True,
    whole_run=whole,foreground=dict(sample_count=len(f),valid_count=len(f)-len(invalid),invalid_count=len(invalid),
        first_invalid_utc=utc(cut),first_invalid_elapsed_s=invalid[0]['elapsed_s'],
        max_consecutive_monotonic_sample_gap_s=max(f[i]['elapsed_s']-f[i-1]['elapsed_s'] for i in range(1,len(f))),
        whole_run_valid=False,lock_onset_known=False),pre_interruption_subwindow=sub,
    attribution=dict(admitted_count=len(admitted),rejected_count=rejected_count),failed_preparations=preparations,
    gate='invalid_diagnostic_not_acceptance',limitations=[
        'Last resource interval is not frontmost; end native UI capture unavailable because CUA reported locked',
        'First observed focus loss is not established as lock onset or application defect',
        'Pre-interruption approximately 130 seconds is a diagnostic subwindow, not a valid full run or baseline pair',
        'Foreground samples observe points at about 0.1-second spacing; no claim about every instant between samples',
        'First AX artifact is a delta reporting unchanged tree and overview focus, not a full standalone AX tree',
        'RSS is resident sum, not deduplicated physical footprint or proof of a leak',
        'Reused instance and changed quality are not independent cold runs or a matched high-versus-mid causal study',
        'No network, startup interactive or main-thread instrumentation; file stability alone does not prove zero network',
        'Existing ten-minute CPU and RSS budget failure remains unresolved'])
a.output.write_text(json.dumps(out,ensure_ascii=False,indent=2)+chr(10))
print(json.dumps(dict(recompute='passed',whole_run_valid=False,pre_interruption_seconds=sub['elapsed_seconds'],
    pre_interruption_samples=len(pre),first_invalid_utc=utc(cut),gate=out['gate']),indent=2))
