from pathlib import Path
import hashlib,json,re,sys
ROOT=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
runs=sys.argv[1:]
assert runs
rows=[]
for name in runs:
    text=(ROOT/name/'stdout.log').read_text(encoding='utf-8-sig')
    for line in text.splitlines():
        for marker,kind in [('r6-recovery-chain-native-success=','success'),('r6-recovery-chain-native-interruption=','interruption')]:
            if marker not in line:continue
            data=json.loads(line.split(marker,1)[1]);d=data['durable'];directory=Path(d['directory'])
            current=(directory/'current.json').read_bytes()
            assert len(current)==d['currentActualBytes'] and sha(current)==d['currentActualSha256']
            events=d['immutableRecoveryEvents'];assert events
            for e in events:
                raw=(directory/f"event-{e['sequence']:06}.json").read_bytes();assert json.loads(raw)==e
                assert e['schema']=='windows-client-originals-v3' and e['documentKind']=='stopped-recovery-event'
                if e['nodeId'] is not None:
                    assert e['movement'] and e['movement']['from']!=e['movement']['to']
            originals=[n for n in data['positions']['rows'] if n['kind']=='original']
            currents=[n for n in data['positions']['rows'] if n['kind']=='current']
            assert len(currents)==6 and len([n for n in originals if n['state']=='Restored'])==data.get('originalRestored',3)
            if kind=='success':
                assert d['summary']['retiredCount']==6 and d['summary']['restoredCount']==3
                assert d['summary']['pending'] is None and not d['poisoned']
                assert len(events)==20
                assert all(n['retired'] for n in currents)
                assert json.loads(current)['recovery']==d['summary']
            else:
                fault=data['fault'];restore=data['restore'];assert d['poisoned']
                assert data['currentRetired']==(6 if restore else int(fault in [41,42,43]))
                assert data['originalRestored']==int(restore and fault in [41,42,43])
                assert sum(n['retired'] for n in currents)==data['currentRetired']
                if fault in [2,43]:assert current==b'{'
                else:
                    tip=json.loads(current);assert tip['sequence']==d['sequence'] and tip['recovery']==d['summary']
                expected='RecoveryPlanIntent' if fault in [1,2,44] else (
                    ('OriginalRestore' if restore else 'WorkRetire')+('dVerified' if not restore and fault in [42,43] else 'dVerified' if restore and fault in [42,43] else 'Intent'))
                if fault in [42,43]:expected='OriginalRestoredVerified' if restore else 'WorkRetiredVerified'
                assert events[-1]['operation']==expected
            rows.append(dict(run=name,kind=kind,data=data))
assert rows
target=ROOT/('recovery-native-traces-'+ '-'.join(runs)+'.json')
with target.open('x',encoding='utf-8') as out:json.dump(dict(observations=rows,offlineDataOnly=True,nativeAuthority=False),out,indent=2)
print(json.dumps(dict(nativeObserved=len(rows),success=sum(r['kind']=='success' for r in rows),interruption=sum(r['kind']=='interruption' for r in rows))))
