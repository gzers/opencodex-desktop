from pathlib import Path
import hashlib,json,sys
ROOT=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
observations=[]
for run in sys.argv[1:]:
    text=(ROOT/run/'stdout.log').read_text(encoding='utf-8-sig')
    for line in text.splitlines():
        for marker,kind in [('r6-batch-native-complete=','complete'),('r6-batch-native-initial-fault=','initial'),('r6-batch-native-recovery-fault=','recovery')]:
            if marker not in line:continue
            data=json.loads(line.split(marker,1)[1]);batch=data['batch'];directory=Path(batch['directory']);phase_bytes=[]
            for phase in batch['immutableBatchPhases']:
                path=directory/f"event-{phase['sequence']:06}.json";raw=path.read_bytes();assert json.loads(raw)==phase and len(raw)<=1024*1024
                assert phase['documentKind']=='bounded-node-batch-event' and len(phase['rows'])<=512
                start=phase['prefix']-len(phase['rows']) if phase['operation']=='PrefixVerified' else 0
                for ordinal,row in enumerate(phase['rows'],start):
                    assert row['ordinal']==ordinal;inner=row['event']
                    assert inner['sequence']==phase['sequence'] and inner['previousEventDigest']==phase['previousEventDigest']
                    assert inner['sessionId']==phase['sessionId'] and inner['claimKey']==phase['claimKey'] and inner['leaseEpoch']==phase['leaseEpoch'] and inner['deadlineScope']==phase['deadlineScope']
                    assert inner['rootId'] in ['.codex','.grok'] and len(inner['nodeId'])==64
                phase_bytes.append(dict(path=str(path),bytes=len(raw),sha256=sha(raw),sequence=phase['sequence'],kind=phase['kind'],operation=phase['operation'],count=phase['count'],prefix=phase['prefix']))
            current=(directory/'current.json').read_bytes();assert len(current)<=4096
            if kind=='complete':
                assert batch['physicalEventFiles']==56 and batch['pending'] is None and not batch['poisoned']
                assert data['completion']['completedInOriginalWindow'] and not data['typedClosed']
                assert json.loads(current)['handles']['released']
                assert len(batch['immutableBatchPhases'])==10
                assert all(n['committed'] for n in batch['retainedNativeBatches'])
            else:
                fault=data['fault'];assert batch['poisoned'] and batch['pending'] is not None
                durable=data['durable'];assert len(current)==durable['currentActualBytes'] and sha(current)==durable['currentActualSha256']
                if kind=='initial':
                    complete_work=fault>=140
                    expected=0 if fault in [100,140] else fault-100 if fault in [101,102,103] else fault-140 if fault in [141,142,143] else 1 if fault in [112,152] else 3
                    assert (data['working'] if complete_work else data['moved'])==expected
                    if complete_work:assert data['moved']+data['working']==3
                    assert batch['pending']['prefix']==int(fault in [112,152])
                    partial=fault in [111,151]
                else:
                    expected=0 if fault==120 else fault-120 if fault in range(121,127) else 1 if fault==132 else 6 if data['retire'] else 3
                    assert (data['currentRetired'] if data['retire'] else data['originalRestored'])==expected
                    if not data['retire']:assert data['currentRetired']==6
                    assert batch['pending']['prefix']==int(fault==132)
                    partial=fault==131
                if partial:assert current==b'{'
                else:assert json.loads(current)['batch']==batch['pending']
                assert not batch['retainedNativeBatches'][-1]['committed']
                assert batch['retainedNativeBatches'][-1]['actualVerifiedPrefix']==expected
                for row in data.get('positions',{}).get('rows',[]):
                    path=row.get('path')
                    if not path:continue
                    p=Path(path)
                    if p.is_file() and 'sha256' in row:assert sha(p.read_bytes())==row['sha256']
            observations.append(dict(run=run,kind=kind,observation=data,actualCanonicalPhases=phase_bytes,offlineDataOnly=True,nativeAuthority=False))
assert observations
with (ROOT/('native-traces-'+'-'.join(sys.argv[1:])+'.json')).open('x',encoding='utf-8') as out:json.dump(dict(observations=observations,nativeAuthority=False),out,indent=2)
print(json.dumps(dict(observedNativeCases=len(observations),complete=sum(r['kind']=='complete' for r in observations),initial=sum(r['kind']=='initial' for r in observations),recovery=sum(r['kind']=='recovery' for r in observations))))
