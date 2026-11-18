from pathlib import Path
import hashlib
import json
import sys

root = Path(__file__).resolve().parent
previous = root.parent / 'wl2-r7-r2-readonly-20261117-receive-r1'
live = Path('C:/Users/Public/ocx-r7-r2-readonly-20261117-r1/out01')
expected = json.loads((previous / 'receive-result.json').read_bytes())
matched = []
for row in expected['rawFiles']:
    for path in (live / row['name'], previous / 'raw' / row['name']):
        with path.open('rb') as stream:
            assert path.stat().st_size == row['bytes']
            assert hashlib.file_digest(stream, 'sha256').hexdigest() == row['sha256']
    matched.append(row['name'])
plan = json.loads((previous / 'exact-repair-plan.json').read_bytes())
assert plan['executionReady'] is False and plan['approvalRequested'] is False
print(json.dumps(dict(matched=matched, archive=expected['fullArchive'], repairExecutionReady=False, readOnly=True), indent=2))
