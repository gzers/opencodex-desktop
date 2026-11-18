from pathlib import Path
import hashlib
import json
import re
import shutil
import subprocess
import sys


ROOT = Path(__file__).resolve().parent
CODE = ROOT.parents[3]
DOCS = Path(sys.argv[1]).resolve() / '.adg/evidence/0.2.24/windows-dev'
PHASE = sys.argv[2]
BASE = '0816047f30f667e56bea4167ec012f6e12bfcbdd'
PREVIOUS = DOCS / 'wl2-r7-full-msvc-scan-entry-20261118-r1'
DEST = DOCS / 'wl2-r7-journal-locator-20261118-r1'
SCHEMAS = 'apps/desktop/tauri/gen/schemas'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return json.loads(path.read_bytes())


def write(path, value):
    with path.open('x', newline='\n', encoding='utf-8') as out:
        json.dump(value, out, ensure_ascii=False, indent=2)
        out.write('\n')


def git(*args):
    return subprocess.check_output(['git', *args], cwd=CODE)


def read_run(name):
    run = ROOT / name
    result, sources = read(run / 'result.json'), read(run / 'source.json')
    assert result['arguments'] == sources['arguments']
    assert not result['currentR2Touched'] and not result['productionAccepted']
    return result, sources['sources'], (run / 'stdout.log').read_text(encoding='utf-8'), (run / 'stderr.log').read_text(encoding='utf-8')


if PHASE == 'after01':
    result, _, _, stderr = read_run(PHASE)
    matches = re.findall(r'Running unittests .*\((.+\.exe)\)', stderr)
    assert len(matches) == 1, matches
    exe = (CODE / matches[0]).resolve(strict=True)
    write(ROOT / PHASE / 'artifact.json', dict(pathAtExecution=str(exe), bytes=exe.stat().st_size, sha256=sha(exe.read_bytes()), raw=result['raw']))
    sys.exit(0)

before, before_sources, red, red_err = read_run('before02')
after, after_sources, green, _ = read_run('after01')
assert before['raw'] == 101 and 'same original directory path must stay stable at initializer fault' in red + red_err
assert after['raw'] == 0 and '4 passed; 0 failed' in green
retained = re.findall(r'\br4-retained-synthetic-fixture=(.+)$', red, re.MULTILINE)
removed = re.findall(r'^r7-journal-cleanup-absent=(.+)$', green, re.MULTILINE)
assert len(retained) == 1 and Path(retained[0]).is_dir(), retained
assert len(set(removed)) == len(removed) == 30 and all(not Path(item).exists() for item in removed), removed
assert 'r4-retained-synthetic-fixture=' not in green
regression = read(ROOT / 'related/result.json')
regression_log = (ROOT / 'related/stdout.log').read_text(encoding='utf-8')
assert regression['raw'] == 0 and '3 passed; 0 failed' in regression_log
assert regression['sha256'] == read(ROOT / 'after01/artifact.json')['sha256']
related_removed = re.findall(r'^r7-v2-reference-cleanup-absent=(.+)$', regression_log, re.MULTILINE)
assert len(set(related_removed)) == len(related_removed) == 2 and all(not Path(item).exists() for item in related_removed), related_removed

if PHASE == 'cleanup':
    schemas = CODE / SCHEMAS
    tracked, generated = schemas / 'desktop-schema.json', schemas / 'windows-schema.json'
    pristine = (PREVIOUS / 'generated-schema/desktop-schema-original.json').read_bytes()
    assert pristine.replace(b'\r\n', b'\n') == git('show', f'HEAD:{SCHEMAS}/desktop-schema.json')
    assert generated.name not in {row['Name'] for row in read(ROOT / 'schema-before.json')}
    assert generated.resolve(strict=True) == generated and generated.is_file() and not generated.is_symlink()
    archive = ROOT / 'generated-schema'
    assert generated.drive == archive.drive
    archive.mkdir()
    changed, new = tracked.read_bytes(), generated.read_bytes()
    (archive / 'desktop-schema-generated.json').write_bytes(changed)
    (archive / 'desktop-schema-original.json').write_bytes(pristine)
    generated.rename(archive / generated.name)
    assert (archive / generated.name).read_bytes() == new
    tracked.write_bytes(pristine)
    assert tracked.read_bytes() == pristine and not generated.exists()
    write(ROOT / 'cleanup-readback.json', dict(failingFixtureRetained=retained[0], successfulFixturesVerifiedAbsent=removed + related_removed, generatedSchemas=dict(desktopGenerated=sha(changed), desktopRestored=sha(pristine), windowsMoved=sha(new)), currentR2Touched=False, previousDeletePolicyRejectionRetried=False))
    sys.exit(0)

assert PHASE == 'evidence'
head = git('rev-parse', 'HEAD').decode().strip()
paths = set(read(ROOT / 'sources.json'))
assert set(git('diff', '--name-only', BASE, head).decode().splitlines()) == paths
verified = []
for run, rows in [('before02', before_sources), ('after01', after_sources)]:
    assert {row['path'] for row in rows} == paths
    for row in rows:
        snap = (ROOT / run / 'executed-source' / row['path']).read_bytes()
        revision = BASE if run == 'before02' and row['path'].endswith('/read.rs') else head
        assert len(snap) == row['bytes'] and sha(snap) == row['sha256']
        path = row['path']
        assert snap.replace(b'\r\n', b'\n') == git('show', f'{revision}:{path}')
        verified.append(dict(run=run, gitRevision=revision, **row))

DEST.mkdir()
for file in ['.gitattributes', 'verify_git_payload.py']:
    shutil.copyfile(PREVIOUS / file, DEST / file)
for file in ['hypothesis.json', 'sources.json', 'sources-malformed.json', 'schema-before.json', 'run-product-low-memory.ps1', 'cleanup-readback.json', 'seal.py', 'fetch-http11.json', 'fetch-http11.stdout.log', 'fetch-http11.stderr.log']:
    shutil.copyfile(ROOT / file, DEST / file)
shutil.copytree(ROOT / 'generated-schema', DEST / 'generated-schema')
shutil.copytree(ROOT / 'related', DEST / 'related')
for run in ['before02', 'after01']:
    (DEST / run).mkdir()
    for file in ['source.json', 'result.json', 'stdout.log', 'stderr.log', 'artifact.json']:
        shutil.copyfile(ROOT / run / file, DEST / run / file)
write(DEST / 'source-git-verification.json', dict(code=head, normalizedOnlyCRLF=True, files=verified))
write(DEST / 'runner-preflight.json', dict(run='before', cargoStarted=False, firstFailure='source input joined two paths into one path', malformedInput='sources-malformed.json', correctedInput='sources.json', originalOutputFiles=[], cleanupCheckerAttempts=[dict(raw=1, error='AssertionError: expected 3 tests, actual filter ran 4'), dict(raw=1, error='AssertionError: failed fixture identifier occurred after the Rust test name')], changedFilesBeforeErrors=False, testsRerun=False))
