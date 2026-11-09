"""Isolated exact-source overlay. Never repacks/activates the maintained runtime."""
from pathlib import Path
import hashlib
import json
import os
import shutil
import sys
import tarfile

assert os.name == 'nt' and os.environ.get('GITHUB_ACTIONS') == 'true'
repo = Path(__file__).resolve().parents[2]
original, output, report = map(lambda p: Path(p).resolve(), sys.argv[1:4])
boundary = Path(os.environ['RUNNER_TEMP']).resolve()
for p in [original, output, report]:
    p.relative_to(boundary)
assert not output.exists() and not report.exists()
assert output.parent == original.parent and output.name == 'opencodex-record-prototype'
pin = json.loads((repo / '.github/maintained-runtime.json').read_text())
archive = repo / 'apps/desktop/tauri/resources/runtime-bundle/maintained-runtime.tgz'
assert hashlib.sha256(archive.read_bytes()).hexdigest() == pin['sha256']
path = 'src/config/atomic-write.ts'
with tarfile.open(archive, 'r:gz') as source:
    expected = source.extractfile('package/' + path).read()
assert (original / path).read_bytes() == expected
assert json.loads((original / 'package.json').read_text())['version'] == pin['version']
shutil.copytree(original, output)
candidate = output / path
text = expected.decode()
text = 'import { createWindowsRuntimeRecordTemp, pinWindowsRuntimeRecordPublication } from "../lib/windows-runtime-private-record";\n' + text
start = text.index('function writePrivateTempFile(')
end = text.index('function writePrivateTempFileWith(', start)
segment = text[start:end]
needle = 'const descriptor = openSync(path, "wx", 0o600);'
assert segment.count(needle) == 1
segment = segment.replace(needle, 'const descriptor = createWindowsRuntimeRecordTemp(path, timeoutMemoKey) ?? openSync(path, "wx", 0o600);', 1)
text = text[:start] + segment + text[end:]
start = text.index('function atomicWriteFileToTarget(')
end = text.index('export function atomicWriteFile(', start)
segment = text[start:end]
needle = '  const releaseAcl = admitAtomicAclWork(target, tmp, io === undefined);'
assert segment.count(needle) == 1
segment = segment.replace(needle, needle + '\n  let releaseRecordParents: (() => void) | undefined;', 1)
needle = '  try {\n    if (io) ownsTemp = true;'
assert segment.count(needle) == 1
segment = segment.replace(needle, '  try {\n    if (!io) releaseRecordParents = pinWindowsRuntimeRecordPublication(path);\n    if (io) ownsTemp = true;', 1)
needle = '  } finally { releaseAcl?.(); }'
assert segment.count(needle) == 1
segment = segment.replace(needle, '  } finally { try { releaseRecordParents?.(); } finally { releaseAcl?.(); } }', 1)
text = text[:start] + segment + text[end:]
candidate.write_text(text, encoding='utf-8', newline='')
helper = repo / 'test/smoke/runtime023-windows-private-record.ts'
(output / 'src/lib/windows-runtime-private-record.ts').write_bytes(helper.read_bytes())
files = {}
for relative in [path, 'src/lib/windows-runtime-private-record.ts']:
    data = (output / relative).read_bytes()
    files[relative] = {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}
assert (original / path).read_bytes() == expected
with report.open('x', encoding='utf-8') as stream:
    json.dump({'accepted': False, 'diagnosticOnly': True, 'source': os.environ.get('GITHUB_SHA'),
               'originalArchiveSha256': pin['sha256'], 'originalAtomicWriterBeforeSha256': hashlib.sha256(expected).hexdigest(),
               'originalUnmodified': True, 'productionArchiveChanged': False,
               'scope': 'Separate disposable package clone: sync runtime-port temp creation and publication pins only; no production activation',
               'candidateFiles': files}, stream, indent=2)
