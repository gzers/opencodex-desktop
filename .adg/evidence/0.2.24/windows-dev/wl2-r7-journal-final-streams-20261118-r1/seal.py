from pathlib import Path
import hashlib
import json
import re
import shutil
import subprocess
import sys


RUN = Path(__file__).resolve().parent
CODE = RUN.parents[3]
DOCS = Path(sys.argv[1]).resolve()
EVIDENCE = DOCS / '.adg/evidence/0.2.24/windows-dev'
PREVIOUS = EVIDENCE / 'wl2-r7-journal-locator-20261118-r1'
DEST = EVIDENCE / 'wl2-r7-journal-final-streams-20261118-r1'
BASE = '4b8284f152d4940fd9e7f7b30abd304c909f6a98'
SCHEMAS = CODE / 'apps/desktop/tauri/gen/schemas'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return json.loads(path.read_bytes())


def write(path, value):
    with path.open('x', encoding='utf-8', newline='\n') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write('\n')


def git(*args):
    return subprocess.check_output(['git', *args], cwd=CODE)


def verify_run(name, raw):
    run = RUN / name
    result = read(run / 'result.json')
    sources = read(run / 'source.json')
    assert result['raw'] == raw and result['arguments'] == sources['arguments']
    assert not result['currentR2Touched'] and not result['productionAccepted']
    return (sources['sources'], (run / 'stdout.log').read_text(encoding='utf-8'),
            (run / 'stderr.log').read_text(encoding='utf-8'))


def artifact(name, raw):
    _, _, stderr = verify_run(name, raw)
    matches = re.findall(r'Running unittests .*\((.+\.exe)\)', stderr)
    assert len(matches) == 1, matches
    path = (CODE / matches[0]).resolve(strict=True)
    write(RUN / name / 'artifact.json', dict(pathAtExecution=str(path), bytes=path.stat().st_size,
                                              sha256=sha(path.read_bytes()), raw=raw))


def cleanup():
    _, baseline, baseline_errors = verify_run('baseline', 101)
    _, fixed, _ = verify_run('fixed', 0)
    assert 'completed journal must reject a late directory stream: Object' in baseline + baseline_errors
    assert '5 passed; 0 failed' in fixed
    retained = re.findall(r'r4-retained-synthetic-fixture=(.+)$', baseline, re.MULTILINE)
    removed = re.findall(r'^r7-journal-cleanup-absent=(.+)$', fixed, re.MULTILINE)
    assert len(retained) == 1 and Path(retained[0]).is_dir(), retained
    assert len(removed) == len(set(removed)) == 31 and all(not Path(p).exists() for p in removed), removed
    assert 'r4-retained-synthetic-fixture=' not in fixed
    original = (RUN / 'desktop-schema-original.json').read_bytes()
    assert original.replace(b'\r\n', b'\n') == git('show', f'{BASE}:apps/desktop/tauri/gen/schemas/desktop-schema.json')
    generated = SCHEMAS / 'windows-schema.json'
    assert generated.name not in {row['Name'] for row in read(RUN / 'schema-before.json')}
    assert generated.resolve(strict=True) == generated and generated.is_file() and not generated.is_symlink()
    archive = RUN / 'generated-schema'
    assert generated.drive == archive.drive
    archive.mkdir()
    desktop = SCHEMAS / 'desktop-schema.json'
    prior, new = desktop.read_bytes(), generated.read_bytes()
    (archive / 'desktop-schema-generated.json').write_bytes(prior)
    (archive / 'desktop-schema-original.json').write_bytes(original)
    generated.rename(archive / generated.name)
    desktop.write_bytes(original)
    assert (archive / generated.name).read_bytes() == new and desktop.read_bytes() == original and not generated.exists()
    write(RUN / 'cleanup-readback.json', dict(failingFixtureRetained=retained[0], successfulFixturesVerifiedAbsent=removed,
                                               generatedSchemas=dict(desktopGenerated=sha(prior), desktopRestored=sha(original), windowsMoved=sha(new)),
                                               currentR2Touched=False, previousDeletePolicyRejectionRetried=False))


def evidence():
    head = git('rev-parse', 'HEAD').decode().strip()
    paths = set(read(RUN / 'sources.json'))
    assert set(git('diff', '--name-only', BASE, head).decode().splitlines()) == paths
    verified = []
    for name, raw in [('baseline', 101), ('fixed', 0)]:
        sources, _, _ = verify_run(name, raw)
        assert {row['path'] for row in sources} == paths
        for row in sources:
            path = row['path']
            source = (RUN / name / 'executed-source' / path).read_bytes()
            revision = BASE if name == 'baseline' and path.endswith('/read.rs') else head
            assert len(source) == row['bytes'] and sha(source) == row['sha256']
            assert source.replace(b'\r\n', b'\n') == git('show', f'{revision}:{path}')
            verified.append(dict(run=name, gitRevision=revision, **row))
    DEST.mkdir()
    for name in ['.gitattributes', 'verify_git_payload.py']:
        shutil.copyfile(PREVIOUS / name, DEST / name)
    for name in ['hypothesis.json', 'sources.json', 'schema-before.json', 'run-product-low-memory.ps1', 'seal.py', 'deliver.py',
                 'upstream-fetch.json', 'upstream-fetch.stdout.log', 'upstream-fetch.stderr.log',
                 'negotiation.json', 'negotiation.stdout.log', 'negotiation.stderr.log',
                 'proxy-connect.json', 'proxy-connect.stdout.log', 'proxy-connect.stderr.log', 'cleanup-readback.json']:
        shutil.copyfile(RUN / name, DEST / name)
    shutil.copytree(RUN / 'delivery-code', DEST / 'delivery-code')
    for name in ['baseline', 'fixed']:
        (DEST / name).mkdir()
        for file in ['result.json', 'source.json', 'stdout.log', 'stderr.log', 'artifact.json']:
            shutil.copyfile(RUN / name / file, DEST / name / file)
    shutil.copytree(RUN / 'generated-schema', DEST / 'generated-schema')
    write(DEST / 'source-git-verification.json', dict(code=head, normalizedOnlyCRLF=True, files=verified))
    old = read(PREVIOUS / 'status.json')
    additions = [
        dict(item='Completed initial journal late directory stream baseline', status='MSVC raw101; original reader accepted a late synthetic directory stream; first failure fixture retained'),
        dict(item='Completed journal final directory stream check', status='MSVC raw0; rejects InvalidData, retains both original handles and pending state; 5 affected regressions passed'),
        dict(item='Native fixture and generated schema cleanup', status='31 successful fixture roots verified absent; source checkout generated schema restored'),
    ]
    upstream = read(RUN / 'upstream-fetch.json')
    write(DEST / 'status.json', dict(matrix=old['matrix'] + additions, baselineStatus=(PREVIOUS / 'status.json').relative_to(DOCS).as_posix(),
                                    codeAtTest=head, mainAtTest=upstream['main'], governance=upstream['governance'],
                                    artifact=read(RUN / 'fixed/artifact.json'), productionAccepted=False, originalR2Touched=False,
                                    publicStartOpen=False, privateV2='session 7 disconnected and staged output absent at start',
                                    initialStartWorker='handoff request 7ef162357b02f12722dc79811d10a760b17c7544 pending',
                                    currentR2=dict(executionReady=False, unknownJobHandles=24, raw=50)))


phase = sys.argv[2]
if phase in ['baseline', 'fixed']:
    artifact(phase, 101 if phase == 'baseline' else 0)
elif phase == 'cleanup':
    cleanup()
else:
    assert phase == 'evidence'
    evidence()
