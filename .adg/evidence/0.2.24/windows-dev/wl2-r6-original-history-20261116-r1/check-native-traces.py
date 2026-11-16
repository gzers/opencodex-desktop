from pathlib import Path
import hashlib,json,sys
ROOT=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
def encoded(v):return json.dumps(v,ensure_ascii=False,separators=(',',':')).encode()
rows=[]
for run in sys.argv[1:]:
    for line in (ROOT/run/'stdout.log').read_text(encoding='utf-8-sig').splitlines():
        marker='r6-original-history-native='
        if marker not in line:continue
        data=json.loads(line.split(marker,1)[1]);d=data['durable'];fault=data['fault']
        directory=Path(d['directory']);raw=(directory/'current.json').read_bytes()
        assert len(raw)==d['currentActualBytes'] and sha(raw)==d['currentActualSha256']
        assert d['actualReleaseReceiptRetained'] and d['leaseState']=='Released' and not data['Closed']
        for event in d['immutableRecoveryEvents']+d['immutableLeaseEvents']+d['immutableHistoryEvents']:
            path=directory/f"event-{event['sequence']:06}.json";assert json.loads(path.read_bytes())==event
            if event['documentKind']=='stopped-recovery-event':
                if event['nodeId'] is None:assert event['rootId'] is None
                else:assert event['rootId'] in ['.codex','.grok'] and event['movement']['from'].split('/')[0] in [event['rootId'],'custody']
        assert d['summary']['retiredCount']==6 and d['summary']['restoredCount']==3
        current_nodes=[n for n in data['positions']['rows'] if n['kind']=='current']
        assert len(current_nodes)==6 and all(n['retired'] for n in current_nodes)
        assert all(n['state'] in ['Original','Restored'] for n in data['positions']['rows'] if n['kind']=='original')
        history=d['immutableHistoryEvents'];assert len(history)==(2 if fault in [0,74,75,76] else 1)
        assert history[0]['operation']=='OriginalHistoryIntent'
        context=history[0]['context'];plan=d['immutableRecoveryEvents'][0]['plan']
        archive_rows=[]
        for index,root in enumerate(plan['roots']):
            profile=Path(root['originalTree']['root']).parent
            leaf='codex' if index==0 else 'grok'
            path=profile/f".ocxd024-history-{context['sessionId']}-{leaf}"
            if not path.exists():continue
            manifest=path/'snapshot.json';members=[]
            for p in sorted(path.iterdir()):
                if p.is_dir():
                    assert fault==76 and p.name=='synthetic-history-foreign';members.append(dict(path=str(p),foreignDirectoryRetained=True));continue
                assert p.is_file();b=p.read_bytes();members.append(dict(path=str(p),bytes=len(b),sha256=sha(b)))
            archived=dict(directory=str(path),members=members,complete=manifest.exists())
            if manifest.exists():
                metadata=json.loads(manifest.read_bytes());assert metadata['schema']=='windows-anchored-original-history-v3'
                assert metadata['context']==context and not metadata['runtimeAuthority']
                original=root['originalTree'];source=metadata['source']
                assert source['root']==original['root']
                assert len(source['nodes'])==len(original['nodes'])
                for node,original_node in zip(source['nodes'],original['nodes']):
                    assert node['relative']==original_node['relative'] and node['fileId']==original_node['fileId']
                    assert node['security']==original_node['security'] and node['children']==original_node['children']
                    assert node['bytes']==original_node['length'] and node['sha256']==original_node['contentDigest']
                source_ids={(n['volume'],n['fileId']) for n in source['nodes']};copies=set()
                parts=metadata['parts'];expected={'snapshot.json'}
                for part in parts:
                    p=path/part['name'];b=p.read_bytes();assert sha(b)==part['sha256'] and len(b)==part['bytes']
                    copy=(part['volume'],part['fileId']);assert copy not in source_ids and copy not in copies;copies.add(copy)
                    assert part['security']['fileId']==part['fileId'];expected.add(part['name'])
                actual={p.name for p in path.iterdir()}
                if fault==76:
                    assert index!=0 or actual==expected|{'synthetic-history-foreign'}
                    if index==1:assert actual==expected
                else:assert expected==actual
                for i,node in enumerate(source['nodes']):
                    if node['directory']:continue
                    group=[p for p in parts if p['node']==i];assert group
                    group.sort(key=lambda p:p['begin']);b=b''.join((path/p['name']).read_bytes() for p in group)
                    assert len(b)==node['bytes'] and sha(b)==node['sha256']
                archived.update(sourceOriginalIdentities=sorted(source_ids),archiveCopyIdentities=sorted(copies),metadataSha256=sha(manifest.read_bytes()))
                if fault in [0,74,75,76]:
                    ref=history[-1]['references'][index];assert ref['context']==context
                    assert ref['metadataSha256']==sha(manifest.read_bytes()) and ref['rootId']==('.codex' if index==0 else '.grok')
                    assert ref['schema']=='windows-anchored-original-history-reference-v3' and not ref['runtimeAuthority']
                    assert (ref['volume'],ref['fileId']) not in source_ids|copies
                    assert (ref['volume'],ref['metadataId']) not in source_ids|copies
            archive_rows.append(archived)
        assert len(archive_rows)==data['actualHistoryArchives']+data.get('pendingHistoryDirectories',0)
        if fault in [0,76]:
            assert d['historySummary']['linkedReferences'] is not None and not d['poisoned']
        else:
            assert d['historySummary']['linkedReferences'] is None and d['poisoned']
        if fault==75:assert raw==b'{'
        else:
            tip=json.loads(raw);assert tip['history']==d['historySummary'] and tip['sequence']==d['sequence']
        rows.append(dict(run=run,observation=data,actualOriginalArchives=archive_rows,offlineDataOnly=True))
assert rows
with (ROOT/('native-traces-'+'-'.join(sys.argv[1:])+'.json')).open('x',encoding='utf-8') as out:
    json.dump(dict(observations=rows,nativeAuthority=False),out,indent=2)
print(json.dumps(dict(nativeObserved=len(rows),faults=[r['observation']['fault'] for r in rows],archiveIdentitiesSeparate=True)))
