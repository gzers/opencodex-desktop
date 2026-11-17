from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
observed={};exports=[]
for run in ['library01','library02','library03']:
    log=(ROOT/run/'stdout.log').read_text(encoding='utf-8-sig');rows={name:[] for name in ['native','attempt','stream']}
    for line in log.splitlines():
        for name,marker in [('native','r6-route-close-native='),('attempt','r6-route-close-attempt='),('stream','r6-route-close-stream=')]:
            if marker in line:rows[name].append(json.loads(line.split(marker,1)[1]))
    assert len(rows['native'])==7 and len(rows['attempt'])==3 and len(rows['stream'])==1
    complete=next(r for r in rows['native'] if r.get('case')=='complete');entries=complete['trace']['handles']['rows']
    assert complete['newConfigWriteSucceeded'] and entries[-1]['row']['key']=='route/config' and all(r['state']=='ConfirmedClosed' for r in entries)
    for r in rows['attempt']:
        entries=r['trace']['handles']['rows'];assert sum(x['state']=='ConfirmedClosed' for x in entries)=={81:1,85:0,82:len(entries)}[r['fault']]
        assert sum(x['state']=='Unknown' for x in entries)==int(r['fault']==85)
        if r['fault'] in [81,85]:assert entries[-1]['row']['key']=='route/config' and entries[-1]['state']=='Held'
    stream=rows['stream'][0]
    if stream['added']:
        assert all(r['state']=='Held' for r in stream['trace']['handles']['rows'])
        body=next(body for name,body in re.findall(r'test ([^\n]+?) \.\.\. (.*?)(?=\ntest |\nfailures:|\ntest result:|\Z)',log,re.S) if name.endswith('wl2_r6_route_close_native_added_ads_after_transfer_refuses_before_any_close'))
        sources=re.findall(r'r4-retained-synthetic-fixture=([^\r\n]+)',body);assert len(sources)==1
        scene=Path(sources[0]).resolve();assert scene.is_relative_to(Path('C:/Users/15119/AppData/Local/Temp'))
        source=scene/'route-home/config.json:synthetic-foreign';raw=source.read_bytes();assert raw==b'synthetic unknown stream'
        dest=ROOT/'raw-route-streams'/run/'synthetic-foreign.ads.bin';dest.parent.mkdir(parents=True,exist_ok=True)
        with dest.open('xb') as f:f.write(raw)
        exports.append(dict(source=str(source),evidence=dest.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest(),originalStreamRetained=True))
    else:assert stream['writeRawError']==32
    observed[run]=rows
with (ROOT/'actual-route-observations.json').open('x',encoding='utf-8') as f:json.dump(dict(runs=observed,alternateStreamExports=exports,canonicalActualFactoryExecuted=False,typedClosed=False),f,indent=2)
print(json.dumps(dict(runs=len(observed),routeCasesPerRun=8,streamExports=len(exports),canonicalActualFactoryExecuted=False)))
