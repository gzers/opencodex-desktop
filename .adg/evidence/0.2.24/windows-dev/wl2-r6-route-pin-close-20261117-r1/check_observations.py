from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
observations={};exports=[]
for run in ['library01','library02','library03']:
    log=(ROOT/run/'stdout.log').read_text(encoding='utf-8-sig')
    rows={name:[] for name in ['native','attempt','stream']}
    for line in log.splitlines():
        for name,marker in [('native','r6-route-close-native='),('attempt','r6-route-close-attempt='),('stream','r6-route-close-stream=')]:
            if marker in line:rows[name].append(json.loads(line.split(marker,1)[1]))
    complete=[r for r in rows['native'] if r.get('case')=='complete'];assert len(complete)==1
    trace=complete[0]['trace'];native_rows=trace['handles']['rows'];facts=complete[0]['data']['facts']
    assert len(native_rows)==2+2*(len(Path(facts['home']).parents)+1)
    assert all(r['state']=='ConfirmedClosed' for r in native_rows)
    assert trace['completedInOriginalWindow'] and complete[0]['newConfigWriteSucceeded']
    if run!='library01':
        assert native_rows[-1]['row']['key']=='route/config'
        assert {r['fault'] for r in rows['attempt']}=={81,85,82}
        for r in rows['attempt']:
            entries=r['trace']['handles']['rows']
            assert not r['trace']['completedInOriginalWindow']
            assert sum(x['state']=='ConfirmedClosed' for x in entries)=={81:1,85:0,82:len(entries)}[r['fault']]
            assert sum(x['state']=='Unknown' for x in entries)==int(r['fault']==85)
            if r['fault'] in [81,85]:assert entries[-1]['row']['key']=='route/config' and entries[-1]['state']=='Held'
    if rows['stream']:
        assert run=='library03' and len(rows['stream'])==1
        stream=rows['stream'][0]
        if stream['added']:
            assert all(r['state']=='Held' for r in stream['trace']['handles']['rows'])
            block=next(body for name,body in re.findall(r'test ([^\n]+?) \.\.\. (.*?)(?=\ntest |\nfailures:|\ntest result:|\Z)',log,re.S) if name.endswith('wl2_r6_route_close_native_added_ads_after_transfer_refuses_before_any_close'))
            sources=re.findall(r'r4-retained-synthetic-fixture=([^\r\n]+)',block);assert len(sources)==1
            source=Path(sources[0]).resolve();assert source.is_relative_to(Path('C:/Users/15119/AppData/Local/Temp'))
            path=source/'route-home'/'config.json:synthetic-foreign';raw=path.read_bytes();assert raw==b'synthetic unknown stream'
            target=ROOT/'raw-route-streams'/run/'synthetic-foreign.ads.bin';target.parent.mkdir(parents=True,exist_ok=True)
            with target.open('xb') as f:f.write(raw)
            exports.append(dict(source=str(path),evidence=target.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw),originalStreamRetained=True,nativeMetadataCloned=False))
        else:assert stream['writeRawError']==32 and not stream['afterTransferStreamVetoExecuted']
    observations[run]=rows
assert len(observations['library03']['native'])==7
assert len(observations['library03']['stream'])==1
with (ROOT/'actual-observations.json').open('x',encoding='utf-8') as f:json.dump(dict(runs=observations,alternateStreamExports=exports,firstFailureDetailedPartialTraceUnavailable=True,productionRouteCloseIntegrated=False,typedClosed=False),f,indent=2)
print(json.dumps(dict(primaryLastVerified=True,finalRouteCases=8,adsCreated=observations['library03']['stream'][0]['added'],exportedStreams=len(exports),productionIntegrated=False)))
