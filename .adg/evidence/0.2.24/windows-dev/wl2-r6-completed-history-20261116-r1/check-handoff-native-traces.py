from pathlib import Path
import hashlib,json,sys
ROOT=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
rows=[]
for run in sys.argv[1:]:
    for line in (ROOT/run/'stdout.log').read_text(encoding='utf-8-sig').splitlines():
        marker='r6-original-handle-handoff-native='
        if marker not in line:continue
        data=json.loads(line.split(marker,1)[1]);d=data['durable'];fault=data['fault'];directory=Path(d['directory'])
        raw=(directory/'current.json').read_bytes();assert len(raw)==d['currentActualBytes'] and sha(raw)==d['currentActualSha256']
        assert d['historySummary']['linkedReferences'] is not None and d['leaseState']=='Released' and not data['Closed']
        for event in d['immutableRecoveryEvents']+d['immutableLeaseEvents']+d['immutableHistoryEvents']+d['immutableHandleEvents']:
            assert json.loads((directory/f"event-{event['sequence']:06}.json").read_bytes())==event
        inventory=d['originalNativeHandoff']['handles']['rows'];expected=d['immutableHandleEvents'][0]['rows']
        assert [n['row'] for n in inventory]==expected
        assert len({n['row']['key'] for n in inventory})==len(inventory)
        assert data['closed']==sum(n['state']=='ConfirmedClosed' for n in inventory)
        assert data['held']==sum(n['state']=='Held' for n in inventory)
        assert data['unknown']==sum(n['state']=='Unknown' for n in inventory)
        assert len(d['nativeLeaseCompletionHistories'])==4 and all(r['completedInOriginalWindow'] for r in d['nativeLeaseCompletionHistories'])
        old_roots=d['immutableRecoveryEvents'][0]['plan']['roots']
        for index,root in enumerate(d['originalNativeHandoff']['roots']):
            assert root['root']==old_roots[index]['originalTree']['root'] and root['retainedNativeArchives']==4
            assert len(root['originals'])==len(old_roots[index]['originalTree']['nodes'])
            for i,node in enumerate(root['originals']):
                old=old_roots[index]['originalTree']['nodes'][i]
                assert node['relative']==old['relative'] and node['fileId']==old['fileId'] and node['security']==old['security']
                assert node['sha256']==old['contentDigest'] and node['bytes']==old['length'] and node['children']==old['children']
                for role in ['original','history-source']:
                    row=next(n for n in expected if n['key']==f'tree/{index}/{role}/{i:04}')
                    assert row['fileId']==old['fileId'] and row['volume']==old['volume']
            for node in root['retired']:
                p=Path(node['path']);b=p.read_bytes();assert len(b)==node['bytes'] and sha(b)==node['sha256']
            if fault in [0,86] and index==0:
                assert (Path(root['root'])/'config.toml').read_bytes()==b'synthetic successor live configuration'
        if fault in [0,86]:
            assert data['closed']==len(inventory) and data['held']==0 and data['unknown']==0
            assert d['originalHandleSummary']['released']
        else:
            assert d['poisoned'] and not d['originalHandleSummary']['released']
            assert data['closed']==(0 if fault in [80,85] else 1 if fault==81 else len(inventory))
            assert data['unknown']==int(fault==85)
            if fault==85:assert any(n['state']=='Unknown' and n['rawError'] is not None for n in inventory)
        if fault==84:assert raw==b'{'
        else:
            tip=json.loads(raw);assert tip['handles']==d['originalHandleSummary'] and tip['sequence']==d['sequence']
        if fault==86:
            path=Path(d['immutableHistoryEvents'][1]['references'][0]['directory'])/'synthetic-handoff-foreign';assert path.is_dir()
        rows.append(dict(run=run,observation=data,offlineDataOnly=True,nativeAuthority=False))
assert rows
with (ROOT/('handoff-native-traces-'+'-'.join(sys.argv[1:])+'.json')).open('x',encoding='utf-8') as out:json.dump(dict(observations=rows,nativeAuthority=False),out,indent=2)
print(json.dumps(dict(nativeObserved=len(rows),faults=[r['observation']['fault'] for r in rows],allCanonicalAndPhysicalPayloadsChecked=True)))
