from pathlib import Path
import base64,hashlib,json,sqlite3,tempfile,uuid

root=Path(__file__).resolve().parent
sha=lambda raw:hashlib.sha256(raw).hexdigest()
log=(root/'rollover-final.log').read_text()
def rows(marker):
    return [json.loads(line.split(marker+' ',1)[1]) for line in log.splitlines() if marker+' ' in line]
def replay(row,callback):
    assert row['accepted'] is False
    with tempfile.TemporaryDirectory(prefix='ocxd024-lineage-offline-') as d:
        d=Path(d);seen=set()
        for entry in row['files']:
            p=Path(entry['path']);assert not p.is_absolute() and '..' not in p.parts and str(p) not in seen
            seen.add(str(p));raw=base64.b64decode(entry['rawBase64'],validate=True);assert sha(raw)==entry['sha256']
            target=d/p;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw)
        m=d/'desktop/manager-state'
        c=sqlite3.connect((m/'client-session-index.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True)
        raw_events=[r[0] for r in c.execute('SELECT document FROM events ORDER BY revision')]
        events=[json.loads(v) for v in raw_events]
        head=json.loads(c.execute('SELECT document FROM current_session').fetchone()[0])
        binding=json.loads(c.execute('SELECT document FROM binding').fetchone()[0]);c.close()
        assert head==row['head']==events[-1]
        assert len(events)==head['revision'] and [h['revision'] for h in events]==list(range(1,len(events)+1))
        result=callback(d,m,binding,events,raw_events)
    result['offlineReplayRemoved']=True
    return result

def generation(m,binding,events,raw_events,start):
    preparing=events[start];ticket=preparing['ticket']
    assert str(uuid.UUID(ticket))==ticket and preparing['state']=='preparing'
    if start==0:
        active=binding;assert 'rootGeneration' not in preparing and active['originalRoot']['id']==ticket
        slot=m/'client-root-reservation'
    else:
        source=preparing['rootGeneration'];prior=events[start-1]
        assert prior['state']=='root_absent' and source['preparingRevision']==start+1
        assert source['predecessorRevision']==start and source['predecessorSha256']==sha(raw_events[start-1])
        active=json.loads(bytes(source['binding']));assert active['originalRoot']['generationSlot']==ticket
        slot=m/'client-root-generations'/ticket
    origin=active['originalRoot'];assert origin['id']==ticket and 'namespace' not in origin
    assert set(origin)==({'id','intentSha256','createdSha256','publishedSha256'}|({'generationSlot'} if start else set()))
    scope=json.loads(bytes(active['scope']));first=json.loads(bytes(binding['scope']));assert scope[:8]==first[:8]
    for filename,field in [('intent.json','intentSha256'),('created.json','createdSha256'),('published.json','publishedSha256')]:
        assert sha((slot/filename).read_bytes())==origin[field]
    intent=json.loads((slot/'intent.json').read_bytes());created=json.loads((slot/'created.json').read_bytes());published=json.loads((slot/'published.json').read_bytes())
    assert intent['id']==ticket and intent['originalRootAbsent'] and created['rootEmpty']
    assert created['intentSha256']==origin['intentSha256'] and published['createdSha256']==origin['createdSha256']
    assert created['rootIdentity']==published['rootIdentity']==scope[8]
    reference=json.loads(((slot if start else m)/'client-root-intent.json').read_bytes())
    assert reference['id']==ticket and reference['intentSha256']==origin['intentSha256']
    return ticket,origin,scope

def closed(m,binding,events,raw_events,start):
    ticket,origin,scope=generation(m,binding,events,raw_events,start)
    segment=events[start:start+5]
    assert [h['state'] for h in segment]==['preparing','capture_reserved','captured','root_retirement_prepared','root_absent']
    assert all(h['ticket']==ticket and 'rootGeneration' not in h for h in segment[1:])
    head=segment[-1];refs=head['references'];assert refs==segment[2]['references']==segment[3]['references']
    assert refs['attempt']==ticket and refs['phases']==[] and refs['liveSha256'] is None
    vault=m/'client-originals'/ticket
    frame_raw=(vault/'original.json').read_bytes();assert sha(frame_raw)==refs['originalSha256']
    frame=json.loads(frame_raw);assert frame['schemaVersion']==5 and frame['originalRootAbsence']==origin
    assert frame['attempt']==ticket and frame['clientRoot']==scope[8] and frame['phase']=='original'
    assert frame['tree']['entries']==[] and frame['tree']['totalBytes']==0 and frame['previousSha256'] is None
    scope_raw=(vault/'startup-scope.json').read_bytes();assert sha(scope_raw)==refs['scopeSha256']
    startup=json.loads(scope_raw);assert startup['attempt']==ticket and startup['originalSha256']==refs['originalSha256']
    assert startup['clientIdentity']==scope[8] and startup['home']==scope[5]
    movement=head['rootRetirement'];assert movement==segment[3]['rootRetirement'] and movement['sourceRevision']==start+3
    plan=(vault/('root-retirement-'+movement['plan']['id']+'.json')).read_bytes();assert sha(plan)==movement['plan']['sha256']
    plan=json.loads(plan);assert plan['source']==segment[2] and plan['origin']==origin and 'namespace' not in plan['origin']
    readback=vault/('root-retirement-'+movement['plan']['id']+'-Retire-readback.json')
    assert sha(readback.read_bytes())==head['completionSha256']
    return ticket,scope[8]

four=rows('ROOT_ROLLOVER_FOUR_EVIDENCE');assert len(four)==1
assert four[0]['generations']==4 and four[0]['oldBindingEventsAndArchivesUnchanged'] and four[0]['productionStartStillRefused']
def verify_four(d,m,binding,events,raw_events):
    assert len(events)==20
    validated=[closed(m,binding,events,raw_events,s) for s in [0,5,10,15]]
    assert len({v[0] for v in validated})==4 and len({json.dumps(v[1],sort_keys=True) for v in validated})==4
    slots={p.name for p in (m/'client-root-generations').iterdir()}
    assert slots=={v[0] for v in validated[1:]}
    assert {p.name for p in (m/'client-originals').iterdir()}=={v[0] for v in validated}
    return {'kind':'four-storage-closes','generations':4,'allIndependentProvenanceScopesPlansAndReadbacksVerified':True}
results=[replay(four[0],verify_four)]
exits=rows('ROOT_ROLLOVER_EXIT_EVIDENCE');assert len(exits)==3
for row in exits:
    assert row['exitCode']==91 and row['rootRetained'] and row['noRecreatedCustody']
    committed=row['point']=='generation_committed';assert row['committed']==committed
    def verify_exit(d,m,binding,events,raw_events):
        assert len(events)==(11 if committed else 10)
        first=closed(m,binding,events,raw_events,0);second=closed(m,binding,events,raw_events,5)
        assert json.loads((d/'predecessor.json').read_bytes())==events[9]
        actual={p.name for p in (m/'client-root-generations').iterdir()}
        assert len(actual)==2 and second[0] in actual
        pending=actual-{second[0]};assert len(pending)==1
        assert {p.name for p in (m/'client-originals').iterdir()}=={first[0],second[0]}
        if committed:
            ticket,origin,scope=generation(m,binding,events,raw_events,10)
            assert ticket in pending and scope[8]!=first[1] and scope[8]!=second[1]
            assert events[-1]['state']=='preparing' and events[-1]['references'] is None
        else:
            assert events[-1]['state']=='root_absent' and all(h['ticket'] not in pending for h in events)
        if row['point']=='generation_head_updated':assert (m/'client-session-index.sqlite-journal').exists()
        return {'kind':'third-generation-process-exit','point':row['point'],'currentRevision':events[-1]['revision'],
                'newGenerationIndependentlyRegistered':committed,'twoPriorClosesVerified':True,'observedPendingIsNotAnAuthority':True}
    results.append(replay(row,verify_exit))
print(json.dumps({'accepted':False,'verified':results,'scope':'raw SQLite/JSON corroboration of owned real filesystem/process tests; no service Start/normal Stop or power-loss acceptance'},indent=2))
