import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time

root = Path(__file__).resolve().parent
phase = sys.argv[1]
assert phase in ('first-probe', 'final')
artifact = json.loads((root / 'test-binary/artifact.json').read_bytes())
exe = Path(artifact['retainedPath'])
sha = hashlib.file_digest(exe.open('rb'), 'sha256').hexdigest()
assert sha == artifact['sha256']
base = Path(tempfile.gettempdir()).resolve()
fixtures_before = set(base.iterdir())
results = []
fixtures = []
filters = ['wl2_r7_pending_attachment_native_same_thread_exact_readback_and_create_only']
if phase == 'final':
    filters.append('wl2_r7_pending_attachment_native_short_write_flush_and_readback_keep_diagnostic_prefix')
for ordinal, name in enumerate(filters):
    directory = root / f'direct-{ordinal:02}'
    directory.mkdir()
    arguments = [str(exe), name, '--test-threads=1', '--nocapture']
    started = time.monotonic()
    with (directory / 'stdout.log').open('xb') as out, (directory / 'stderr.log').open('xb') as err:
        raw = subprocess.run(arguments, stdout=out, stderr=err, timeout=30).returncode
    elapsed = round((time.monotonic() - started) * 1000)
    stdout = (directory / 'stdout.log').read_text(encoding='utf-8')
    stderr = (directory / 'stderr.log').read_text(encoding='utf-8')
    match = re.search(r'(d+) passed; (d+) failed; (d+) ignored;', stdout)
    assert match
    counts = dict(zip(('passed', 'failed', 'ignored'), map(int, match.groups())))
    for marker, item in re.findall(r'r7-pending-attachment-(cleanup-absent|failure-retained)=([^
]+)', stdout + '
' + stderr):
        path = Path(item)
        assert path.parent.resolve() == base and path not in fixtures_before
        fixtures.append({'suite': name, 'path': str(path), 'absent': not os.path.lexists(path), 'marker': marker, 'suiteRaw': raw})
    result = {'arguments': arguments, 'raw': raw, 'elapsedMs': elapsed, 'counts': counts, 'sha256': sha}
    (directory / 'result.json').write_text(json.dumps(result, indent=2) + '
', encoding='utf-8')
    results.append(result)
    print(json.dumps(result), flush=True)
assert sha == hashlib.file_digest(exe.open('rb'), 'sha256').hexdigest()
(root / 'direct-results.json').write_text(json.dumps(results, indent=2) + '
', encoding='utf-8')
(root / 'fixture-state.json').write_text(json.dumps(fixtures, indent=2) + '
', encoding='utf-8')
assert len(fixtures) == (1 if phase == 'first-probe' else 5), fixtures
sys.exit(max(result['raw'] for result in results))
