"""Attest read-only profile inputs; never extracts or modifies runtime code."""
import hashlib
import json
import os
from pathlib import Path
import sys
import tarfile

if os.name != 'nt' or os.environ.get('GITHUB_ACTIONS') != 'true':
    raise SystemExit('Ephemeral Windows Actions required')
repo = Path(__file__).resolve().parents[2]
package = Path(sys.argv[1]).resolve()
output = Path(sys.argv[2]).resolve()
boundary = Path(os.environ['RUNNER_TEMP']).resolve()
package.relative_to(boundary)
output.relative_to(boundary)
pin = json.loads((repo / '.github/maintained-runtime.json').read_text())
archive = repo / 'apps/desktop/tauri/resources/runtime-bundle/maintained-runtime.tgz'
assert hashlib.sha256(archive.read_bytes()).hexdigest() == pin['sha256']
files = {}
with tarfile.open(archive, 'r:gz') as source:
    for relative in ['src/service.ts', 'src/service/windows-scheduler.ts',
                     'src/service/orchestration.ts', 'src/lib/winsw.ts',
                     'src/codex/history-job.ts', 'src/codex/history-worker.ts',
                     'src/codex/inject/restore.ts']:
        expected = source.extractfile('package/' + relative).read()
        actual = (package / relative).read_bytes()
        assert actual == expected, 'Profile source differs from original pinned bytes'
        files[relative] = dict(bytes=len(actual), sha256=hashlib.sha256(actual).hexdigest())
with output.open('x', encoding='utf-8') as stream:
    json.dump(dict(accepted=False, diagnosticOnly=True, runtimeVersion=pin['version'],
                   archiveSha256=pin['sha256'], sourceFilesEqualOriginalArchive=files,
                   runtimeInstrumented=False, productStarted=False), stream, indent=2)
