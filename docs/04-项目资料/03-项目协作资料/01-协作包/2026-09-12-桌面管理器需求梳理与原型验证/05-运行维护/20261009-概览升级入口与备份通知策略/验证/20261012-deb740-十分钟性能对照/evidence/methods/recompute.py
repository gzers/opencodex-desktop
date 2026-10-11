"""Independent offline recomputation; optional isolated-binary verification."""
import argparse
import datetime
import hashlib
import gzip
import json
import math
from pathlib import Path

p = argparse.ArgumentParser()
p.add_argument('--root', type=Path, default=Path(__file__).resolve().parent)
p.add_argument('--verify-binaries', action='store_true')
p.add_argument('--output', type=Path)
a = p.parse_args()
root = a.root.resolve()
def digest(data): return hashlib.sha256(data).hexdigest()
def raw_bytes(path):
    candidate=root/path
    return candidate.read_bytes() if candidate.is_file() else gzip.decompress(Path(str(candidate)+'.gz').read_bytes())
def read(path): return json.loads(raw_bytes(path))
def near(x,y): assert math.isclose(x,y,rel_tol=1e-10,abs_tol=1e-8), (x,y)
def identity(row): return row['pid'],row['started'],row['exe']
config = read('candidates.json')
results = {}
inputs = {}
for key,phase,count in [('baseline019','idle-high',61),('candidate0110','idle-high',61),('candidate0110','diagnostic-settings',10)]:
    prefix = f'{key}/{phase}'
    ctx,summary,launch = (read(prefix+'/'+f) for f in ('context.json','summary.json','launch.json'))
    raw = raw_bytes(prefix+'/samples.jsonl')
    samples = [json.loads(x) for x in raw.splitlines()]
    assert len(samples)==count
    assert ctx['candidate']==config[key]
    assert ctx['actual_warmup_seconds']>=90
    ids = {identity(x) for x in samples[0]['processes']}
    assert len(ids)==4
    first,last = ({identity(x):x for x in s['processes']} for s in (samples[0],samples[-1]))
    prev = first
    for i,s in enumerate(samples):
        assert s['index']==i
        current = {identity(x):x for x in s['processes']}
        assert set(current)==ids
        assert sum(x['rss_kib'] for x in s['processes'])==s['total_rss_kib']
        assert all(x['rss_kib']>=0 for x in s['processes'])
        assert all(current[k]['cpu_seconds']>=prev[k]['cpu_seconds'] for k in ids)
        assert len([x for x in s['processes'] if x['pid']==launch['pid'] and x['exe']==config[key]['exe']])==1
        if i:
            assert s['elapsed_seconds']>samples[i-1]['elapsed_seconds']
            assert datetime.datetime.fromisoformat(s['captured_at'])>datetime.datetime.fromisoformat(samples[i-1]['captured_at'])
        prev = current
    elapsed = samples[-1]['elapsed_seconds']-samples[0]['elapsed_seconds']
    assert abs(elapsed-ctx['duration_seconds'])<1
    cpu_delta = sum(last[k]['cpu_seconds']-first[k]['cpu_seconds'] for k in ids)
    rss_values = [s['total_rss_kib']/1024 for s in samples]
    rss = dict(start=rss_values[0],end=rss_values[-1],min=min(rss_values),max=max(rss_values),mean=sum(rss_values)/count)
    near(elapsed,summary['elapsed_seconds']);near(cpu_delta,summary['cpu_seconds_delta'])
    near(cpu_delta/elapsed*100,summary['cpu_one_core_percent'])
    for k,v in rss.items(): near(v,summary['rss_mib'][k])
    assert summary['process_identity_stable'] is True
    assert ctx['files_before']==summary['files_after']
    assert summary['source']==config[key]['source_sha']
    recorded = {x['pid']:x for x in summary['breakdown']}
    breakdown = []
    for k in sorted(ids):
        delta=last[k]['cpu_seconds']-first[k]['cpu_seconds']
        near(delta,recorded[k[0]]['cpu_seconds_delta'])
        breakdown.append(dict(pid=k[0],started=k[1],exe=k[2],cpu_seconds_delta=delta,rss_start_mib=first[k]['rss_kib']/1024,rss_end_mib=last[k]['rss_kib']/1024))
    near(sum(s['collection_wall_seconds'] for s in samples),summary['collection_wall_seconds'])
    attribution_path=root/prefix/'attribution.json'
    if attribution_path.exists():
        attribution=json.loads(attribution_path.read_bytes())
        admitted=[x for x in attribution if x['admitted']]
        rejected_count=len(attribution)-len(admitted)
    else:
        filtered=read(prefix+'/attribution-admitted.json')
        admitted=filtered['entries'];rejected_count=filtered['rejected_entry_count']
    assert len(admitted)==8
    assert all(x['returncode']==0 and x['resource_coalition']['bundle ID']==config[key]['bundle'] for x in admitted)
    assert {(x['identity'][0],x['identity'][1],x['exe']) for x in admitted}==ids
    results[prefix]=dict(source=summary['source'],start_utc=samples[0]['captured_at'],end_utc=samples[-1]['captured_at'],elapsed_seconds=elapsed,sample_count=count,cpu_seconds_delta=cpu_delta,cpu_one_core_percent=cpu_delta/elapsed*100,rss_mib=rss,rss_growth_mib=rss['end']-rss['start'],breakdown=breakdown,files_unchanged=True,process_identities_stable=True,admitted_entry_count=len(admitted),rejected_entry_count=rejected_count,collector_cpu_seconds=summary['collector_cpu_seconds'],collection_wall_seconds=summary['collection_wall_seconds'])
    for filename in ('samples.jsonl','context.json','summary.json','launch.json'):
        relative=prefix+'/'+filename;inputs[relative]=digest(raw_bytes(relative))
baseline,candidate=(results[k+'/idle-high'] for k in ('baseline019','candidate0110'))
cb,cc=(read(k+'/idle-high/context.json') for k in ('baseline019','candidate0110'))
assert cb['host']==cc['host']
assert config['baseline019']['appearance']==config['candidate0110']['appearance']
assert config['baseline019']['updates']==config['candidate0110']['updates']
assert datetime.datetime.fromisoformat(baseline['end_utc'])<datetime.datetime.fromisoformat(candidate['start_utc'])
for k in config:
    apps=read(k+'/idle-high/summary.json')['apps_after']
    tests=[x for x in apps if x['exe']!= '/Applications/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop']
    assert len(tests)==1 and tests[0]['exe']==config[k]['exe']
    daily=[x for x in apps if x['exe']=='/Applications/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop']
    assert len(daily)==1 and daily[0]['pid']==75764 and daily[0]['started']=='Fri Oct 9 22:00:24 2026'
    if a.verify_binaries:
        raw=Path(config[k]['exe']).read_bytes()
        assert digest(raw)==config[k]['test_exe_sha256']
        assert config[k]['source_sha'].encode() in raw
delta=dict(cpu_one_core_percentage_points=candidate['cpu_one_core_percent']-baseline['cpu_one_core_percent'],rss_mean_mib=candidate['rss_mib']['mean']-baseline['rss_mib']['mean'],rss_end_mib=candidate['rss_mib']['end']-baseline['rss_mib']['end'])
out=dict(schema_version=1,kind='independently_recomputed_sequential_pair',inputs_sha256=inputs,runs=results,matching_host=cb['host'],matching_appearance=config['candidate0110']['appearance'],matching_updates=config['candidate0110']['updates'],delta=delta,budgets=dict(cpu_increment_percentage_points=0.5,rss_increment_mib=10),budget_comparison=dict(cpu_within_budget=delta['cpu_one_core_percentage_points']<=0.5,rss_mean_within_budget=delta['rss_mean_mib']<=10,rss_end_within_budget=delta['rss_end_mib']<=10),gate='not_passed',limitations=cb['limits']+['Settings diagnostic lock onset unknown; no contemporaneous endpoint UI; low CPU cannot be assigned to route unmount.', 'Recorded source-binary digests are provenance; this independent rerun hashes isolated test binaries only when requested.', 'Single sequential pair and lack of continuous foreground monitoring do not establish regression causality.', 'Sampling and vmmap are post-pair investigation leads, not leak/root-cause proof.'])
(a.output or root/'comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(dict(recompute='passed',isolated_binaries_verified=a.verify_binaries,delta=delta,budget_comparison=out['budget_comparison'],gate=out['gate']),indent=2))
