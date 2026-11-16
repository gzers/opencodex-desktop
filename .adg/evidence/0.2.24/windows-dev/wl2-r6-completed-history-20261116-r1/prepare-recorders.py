from pathlib import Path
ROOT=Path(__file__).resolve().parent
previous=ROOT.parent/'wl2-r6-handle-handoff-20261116-r1'
for name in ['archive_run.py','verify_evidence.py','verify_git_payload.py']:
    raw=(previous/name).read_bytes()
    if name=='archive_run.py':
        assert b'r6-handle-handoff-{name}' in raw
        raw=raw.replace(b'r6-handle-handoff-{name}',b'r6-completed-history-{name}')
    with (ROOT/name).open('xb') as out:out.write(raw)
print('Prepared exact raw artifact/evidence recorders and unique scene archive prefix')
