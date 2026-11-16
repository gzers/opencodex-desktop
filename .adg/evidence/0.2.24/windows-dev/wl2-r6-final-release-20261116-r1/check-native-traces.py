from pathlib import Path
import hashlib,json,sys
ROOT=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
rows=[]
for run in sys.argv[1:]:
    for line in (ROOT/run/'stdout.log').read_text(encoding='utf-8-sig').splitlines():
        marker='r6-final-release-native='
        if marker not in line:continue
        data=json.loads(line.split(marker,1)[1]);d=data['durable'];fault=data['fault']
        current=Path(d['directory'])/'current.json';raw=current.read_bytes()
        assert len(raw)==d['currentActualBytes'] and sha(raw)==d['currentActualSha256']
        assert d['summary']['retiredCount']==6 and d['summary']['restoredCount']==3
        assert d['actualReleaseReceiptRetained']==data['actualLeaseReleased']==(fault==0 or fault>=55)
        assert not data['Closed']
        for event in d['immutableRecoveryEvents']+d['immutableLeaseEvents']:
            path=Path(d['directory'])/f"event-{event['sequence']:06}.json"
            assert json.loads(path.read_bytes())==event
        originals=[n for n in data['positions']['rows'] if n['kind']=='original']
        currents=[n for n in data['positions']['rows'] if n['kind']=='current']
        assert len(currents)==6 and all(n['retired'] for n in currents)
        assert all(n['state'] in ['Restored','Original'] for n in originals)
        anchors=sum(n['state']=='Original' for n in originals)
        assert d['summary']['anchorReleasedCount']==(anchors if fault in [0,56,57,58] else 0)
        if fault==61:
            custody=Path(d['immutableRecoveryEvents'][0]['plan']['roots'][0]['originalTree']['custody'])
            assert (custody/'synthetic-final-release-foreign').is_dir()
        if fault==0:
            assert d['leaseState']=='Released' and d['completeFilesDataOnly'] and not d['poisoned']
            assert d['summary']['wholeTree']==2 and d['summary']['pending'] is None
        else:
            assert d['poisoned'] and not d['completeFilesDataOnly']
            assert d['leaseState']==('Active' if fault<=53 else 'ReleasePrepared')
            assert d['summary']['wholeTree']==(1 if fault<=53 else 2)
            assert (d['summary']['pending'] is None)==(fault>=54)
        if fault in [53,58,60]:assert raw==b'{'
        else:
            tip=json.loads(raw);assert tip['sequence']==d['sequence'] and tip['recovery']==d['summary']
        rows.append(dict(run=run,observation=data))
assert rows
with (ROOT/('native-traces-'+'-'.join(sys.argv[1:])+'.json')).open('x',encoding='utf-8') as out:
    json.dump(dict(observations=rows,offlineDataOnly=True,nativeAuthority=False),out,indent=2)
print(json.dumps(dict(nativeObserved=len(rows),faults=[r['observation']['fault'] for r in rows])))
