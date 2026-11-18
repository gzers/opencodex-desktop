import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time

root = Path(__file__).resolve().parent
artifact = json.loads((root / 'test-binary/artifact.json').read_bytes())
exe = Path(artifact['retainedPath'])

def digest(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()

def save(path, value):
    with path.open('x', encoding='utf-8', newline='\n') as output:
        output.write(json.dumps(value, indent=2) + '\n')

sha = digest(exe)
assert sha == artifact['sha256']
base = Path(tempfile.gettempdir()).resolve()
before = set(base.iterdir())
name = 'infrastructure::windows_client_originals024::recovery_timing::wl2_r7_preparation_image_scan_observation_keeps_deadline_and_original_error'
arguments = [str(exe), '--exact', name, '--test-threads=1', '--nocapture']
directory = root / 'direct-00'
directory.mkdir()
started = time.monotonic()
with (directory / 'stdout.log').open('xb') as out, (directory / 'stderr.log').open('xb') as err:
    try:
        raw = subprocess.run(arguments, stdout=out, stderr=err, timeout=30).returncode
        timed_out = False
    except subprocess.TimeoutExpired:
        raw = None
        timed_out = True
elapsed = round((time.monotonic() - started) * 1000)
result = {'arguments': arguments, 'raw': raw, 'elapsedMs': elapsed, 'sha256': sha, 'externalTimeout': timed_out}
save(directory / 'execution.json', result)
stdout = (directory / 'stdout.log').read_text(encoding='utf-8')
match = re.search(r'(\d+) passed; (\d+) failed; (\d+) ignored;', stdout)
counts = dict(zip(('passed', 'failed', 'ignored'), map(int, match.groups()))) if match else None
fixtures = []
for item in re.findall(r'r7-preparation-image-cleanup-absent=([^\r\n]+)', stdout):
    path = Path(item)
    assert path.parent.resolve() == base and path not in before
    fixtures.append({'suite': name, 'path': str(path), 'absent': not os.path.lexists(path), 'suiteRaw': raw})
result.update(counts=counts, newRoots=[str(path) for path in sorted(set(base.iterdir()) - before)])
save(directory / 'result.json', result)
save(root / 'direct-results.json', [result])
save(root / 'fixture-state.json', fixtures)
print(json.dumps(result), flush=True)
assert raw == 0 and counts == {'passed': 1, 'failed': 0, 'ignored': 0} and not result['newRoots']
assert len(fixtures) == 1 and fixtures[0]['absent']
assert sha == digest(exe)
observed = re.findall(r'r7-preparation-image-scans=(\[[^\r\n]+\])', stdout)
assert len(observed) == 1
rows = json.loads(observed[0])
assert [row['ordinal'] for row in rows] == [1, 2, 3]
assert [row['completed'] for row in rows] == [True, True, False]
assert rows[-1]['errorCode'] == 'runtime_image_unconfirmed'
save(root / 'observed-scans.json', rows)