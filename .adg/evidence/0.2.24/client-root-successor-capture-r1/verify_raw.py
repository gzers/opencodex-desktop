from pathlib import Path
import base64, hashlib, json, sqlite3, tempfile, uuid

root = Path(__file__).resolve().parent
sha = lambda raw: hashlib.sha256(raw).hexdigest()
log = (root / 'capture-delivery.log').read_text()
rows = [json.loads(line.split('SUCCESSOR_CAPTURE_EXIT_EVIDENCE ', 1)[1])
        for line in log.splitlines() if 'SUCCESSOR_CAPTURE_EXIT_EVIDENCE ' in line]
assert len(rows) == 3
verified = []
for row in rows:
    assert row['accepted'] is False and row['exitCode'] == 91 and row['reopenedCaptureRefused']
    with tempfile.TemporaryDirectory(prefix='ocxd024-capture-offline-') as replay:
        replay = Path(replay)
        seen = set()
        for entry in row['files']:
            path = Path(entry['path'])
            assert not path.is_absolute() and '..' not in path.parts and str(path) not in seen
            seen.add(str(path))
            raw = base64.b64decode(entry['rawBase64'], validate=True)
            assert sha(raw) == entry['sha256']
            destination = replay / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(raw)
        manager = replay / 'desktop/manager-state'
        db = sqlite3.connect((manager / 'client-session-index.sqlite').as_uri() + '?mode=ro&immutable=1', uri=True)
        raw_events = [v[0] for v in db.execute('select document from events order by revision')]
        events = [json.loads(v) for v in raw_events]
        head = json.loads(db.execute('select document from current_session').fetchone()[0])
        initial = json.loads(db.execute('select document from binding').fetchone()[0])
        db.close()
        captured = row['point'] == 'captured'
        assert head == row['head'] == events[-1]
        assert len(events) == head['revision'] == (8 if captured else 7)
        assert [v['revision'] for v in events] == list(range(1, len(events) + 1))
        assert [v['state'] for v in events[:7]] == ['preparing', 'capture_reserved', 'captured', 'root_retirement_prepared', 'root_absent', 'preparing', 'capture_reserved']
        assert events[0]['ticket'] == initial['originalRoot']['id'] == events[4]['ticket']
        old_refs = events[4]['references']
        old = manager / 'client-originals' / old_refs['attempt']
        old_frame = (old / 'original.json').read_bytes()
        assert sha(old_frame) == old_refs['originalSha256']
        assert json.loads(old_frame)['originalRootAbsence'] == initial['originalRoot']
        assert sha((old / 'startup-scope.json').read_bytes()) == old_refs['scopeSha256']
        move = events[4]['rootRetirement']
        plan = (old / ('root-retirement-' + move['plan']['id'] + '.json')).read_bytes()
        assert sha(plan) == move['plan']['sha256'] and json.loads(plan)['source'] == events[2]
        readback = old / ('root-retirement-' + move['plan']['id'] + '-Retire-readback.json')
        assert sha(readback.read_bytes()) == events[4]['completionSha256']
        preparing = events[5]
        generation = preparing['rootGeneration']
        assert generation['preparingRevision'] == 6 and generation['predecessorRevision'] == 5
        assert generation['predecessorSha256'] == sha(raw_events[4])
        active = json.loads(bytes(generation['binding']))
        old_scope = json.loads(bytes(initial['scope']))
        new_scope = json.loads(bytes(active['scope']))
        assert old_scope[:8] == new_scope[:8] and old_scope[8] != new_scope[8]
        origin = active['originalRoot']
        assert str(uuid.UUID(preparing['ticket'])) == origin['id'] == origin['generationSlot'] == preparing['ticket']
        assert origin['id'] != old_refs['attempt']
        slot = manager / 'client-root-generations' / origin['id']
        for name, field in [('intent.json', 'intentSha256'), ('created.json', 'createdSha256'), ('published.json', 'publishedSha256')]:
            assert sha((slot / name).read_bytes()) == origin[field]
        assert json.loads((slot / 'created.json').read_bytes())['rootIdentity'] == new_scope[8]
        assert all(v['ticket'] == preparing['ticket'] and 'rootGeneration' not in v for v in events[6:])
        attempt = manager / 'client-originals' / preparing['ticket']
        if row['point'] == 'reserved':
            assert not attempt.exists() and head['references'] is None
        else:
            frame_raw = (attempt / 'original.json').read_bytes()
            frame = json.loads(frame_raw)
            assert frame['schemaVersion'] == 5 and frame['originalRootAbsence'] == origin
            assert frame['attempt'] == origin['id'] and frame['clientRoot'] == new_scope[8]
            assert frame['phase'] == 'original' and frame['previousSha256'] is None
            assert frame['tree']['entries'] == [] and frame['tree']['totalBytes'] == 0
            if captured:
                refs = head['references']
                assert sha(frame_raw) == refs['originalSha256'] and refs['phases'] == [] and refs['liveSha256'] is None
                scope_raw = (attempt / 'startup-scope.json').read_bytes()
                assert sha(scope_raw) == refs['scopeSha256']
                scope = json.loads(scope_raw)
                assert scope['originalSha256'] == refs['originalSha256'] and scope['attempt'] == origin['id']
                assert scope['clientIdentity'] == new_scope[8] and scope['home'] == new_scope[5]
            else:
                assert head['references'] is None and not (attempt / 'startup-scope.json').exists()
        verified.append({'point': row['point'], 'allRawBytesVerified': True, 'revision': head['revision'],
                         'independentOriginalRegistered': captured, 'offlineReplayRemoved': True})
publication = [json.loads(line.split('SUCCESSOR_CAPTURE_PUBLICATION_EVIDENCE ', 1)[1])
               for line in log.splitlines() if 'SUCCESSOR_CAPTURE_PUBLICATION_EVIDENCE ' in line]
assert len(publication) == 4
assert {(v['point'], v['change']) for v in publication} == {(p, c) for p in ['references_sql_written', 'references_committed'] for c in ['deadline', 'root-identity']}
for row in publication:
    assert row['accepted'] is False and row['returnedError'] and row['originalRetained']
    assert row['committed'] == (row['point'] == 'references_committed')
    assert row['retainedRevision'] == (8 if row['committed'] else 7)
print(json.dumps({'accepted': False, 'verified': verified, 'publicationBoundaryAssertions': publication,
                  'scope': 'offline raw SQLite/JSON supplement; no OS-service or repeated Start acceptance'}, indent=2))
