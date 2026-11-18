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

def sha256(path):
    with path.open('rb') as file:
        return hashlib.file_digest(file, 'sha256').hexdigest()

def save(path, value):
    with path.open('x', encoding='utf-8', newline='\n') as output:
        output.write(json.dumps(value, indent=2) + '\n')

sha = sha256(exe)
assert sha == artifact['sha256']
base = Path(tempfile.gettempdir()).resolve()
results = []
fixtures = []
filters = (
    'wl2_r7_pending_attachment_native_same_thread_exact_readback_and_create_only',
    'wl2_r7_pending_attachment_native_short_write_flush_and_readback_keep_diagnostic_prefix',
)
for ordinal, name in enumerate(filters):
    before = set(base.iterdir())
    directory = root / f'direct-{ordinal:02}'
    directory.mkdir()
    arguments = [str(exe), name, '--test-threads=1', '--nocapture']
    started = time.monotonic()
    with (directory / 'stdout.log').open('xb') as out, (directory / 'stderr.log').open('xb') as err:
        raw = subprocess.run(arguments, stdout=out, stderr=err, timeout=30).returncode
    elapsed = round((time.monotonic() - started) * 1000)
    result = {'arguments': arguments, 'raw': raw, 'elapsedMs': elapsed, 'sha256': sha}
    save(directory / 'execution.json', result)
    stdout = (directory / 'stdout.log').read_text(encoding='utf-8')
    stderr = (directory / 'stderr.log').read_text(encoding='utf-8')
    match = re.search(r'(\d+) passed; (\d+) failed; (\d+) ignored;', stdout)
    assert match
    counts = dict(zip(('passed', 'failed', 'ignored'), map(int, match.groups())))
    for marker, item in re.findall(r'r7-pending-attachment-(cleanup-absent|failure-retained)=([^\r\n]+)', stdout + '\n' + stderr):
        path = Path(item)
        assert path.parent.resolve() == base and path not in before
        fixtures.append({'suite': name, 'path': str(path), 'absent': not os.path.lexists(path), 'marker': marker, 'suiteRaw': raw})
    result.update(counts=counts, newRoots=[str(path) for path in sorted(set(base.iterdir()) - before)])
    save(directory / 'result.json', result)
    results.append(result)
    print(json.dumps(result), flush=True)
assert sha == sha256(exe)
save(root / 'direct-results.json', results)
save(root / 'fixture-state.json', fixtures)
assert all(row['raw'] == 0 and row['counts'] == {'passed': 1, 'failed': 0, 'ignored': 0} and not row['newRoots'] for row in results), results
assert len(fixtures) == 5 and all(row['absent'] for row in fixtures), fixtures
