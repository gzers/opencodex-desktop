from pathlib import Path
import hashlib
import json
import shutil
import statistics
import subprocess
import sys


def load(path):
    return json.loads(path.read_bytes())


def digest(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def git(*args):
    return subprocess.check_output(['git', '-C', str(code), *args])


def write(name, value):
    with (dest / name).open('x', encoding='utf-8', newline='\n') as output:
        json.dump(value, output, indent=2, ensure_ascii=False)
        output.write('\n')


code, docs = (Path(arg).resolve() for arg in sys.argv[1:])
run = Path(__file__).resolve().parent
evidence = docs / '.adg/evidence/0.2.24/windows-dev'
prior = evidence / 'wl2-r7-late-directory-stream-20261118-r1'
dest = evidence / 'wl2-r7-enum-hint-20261118-r1'
main = '6bafdd342e69ee83e842f4b35a20bafa1320645a'
governance = '81904340f107f391fec62bf3f282e91e8504b263'
revision = git('rev-parse', 'HEAD').decode().strip()
base = git('rev-parse', 'HEAD^').decode().strip()
changed = git('diff-tree', '--no-commit-id', '--name-only', '-r', revision).decode().splitlines()
inputs = {}
artifacts = {}
for kind in ('test', 'products'):
    directory = run / ('compile-' + kind)
    result = load(directory / 'result.json')
    assert result['raw'] == 0 and result['base'] == base
    assert sorted(changed) == sorted(result['sourceHashes'])
    binding = {}
    for path, sha in result['sourceHashes'].items():
        original = (code / path).read_bytes()
        blob = git('show', f'{revision}:{path}')
        assert sha == hashlib.sha256(original).hexdigest()
        assert original.replace(b'\r\n', b'\n') == blob
        binding[path] = {'workingInputSha256': sha, 'gitBlobSha256': hashlib.sha256(blob).hexdigest()}
    if inputs:
        assert inputs == binding
    inputs = binding
    artifacts[kind] = load(directory / 'artifacts.json')
    for row in artifacts[kind]:
        path = Path(row['path'])
        assert path.stat().st_size == row['bytes'] and digest(path) == row['sha256']

comparison = load(run / 'direct/comparison.json')
assert comparison['valid'] and comparison['fixtureRootAbsent']
assert comparison['candidate'] == artifacts['test'][0]
assert not (run / 'fixtures').exists()
rows = comparison['rows']
assert [row['label'] for row in rows] == ['baseline', 'candidate', 'candidate', 'baseline'] * 3
assert all(row['raw'] == 0 and row['report']['budgetMs'] == 10000 and row['report']['passed'] for row in rows)
assert all(row['report']['metrics']['stages']['windows-file-full-read-hash'][0] == 10586 for row in rows)
assert '18 passed; 0 failed; 3 ignored' in (run / 'direct/candidate-regressions/stdout.log').read_text()


def metrics(samples):
    return {label: {
        'totalMs': statistics.median(row['report']['elapsedUs'] for row in samples if row['label'] == label) / 1000,
        'enumPinMs': statistics.median(row['report']['metrics']['stages']['windows-native-enumeration-pin'][1]
                                        for row in samples if row['label'] == label) / 1_000_000,
    } for label in ('baseline', 'candidate')}


schema = code / 'apps/desktop/tauri/gen/schemas/windows-schema.json'
retained_schema = run / 'generated-windows-schema.json'
assert not schema.exists() and retained_schema.is_file()
dest.mkdir()
for name in ('.gitattributes', 'verify_git_payload.py'):
    shutil.copyfile(prior / name, dest / name)
for path in sorted(run.rglob('*')):
    if path.is_file() and path.suffix in {'.json', '.log', '.py', '.ps1'}:
        copy = dest / path.relative_to(run)
        copy.parent.mkdir(exist_ok=True, parents=True)
        shutil.copyfile(path, copy)

write('source-binding.json', dict(source=revision, base=base, main=main, governance=governance,
                                  team=52, imp=191, buildInputs=inputs, artifacts=artifacts))
write('scan-metrics.json', dict(order=[row['label'] for row in rows], median=metrics(rows),
                                withoutFirstBaselineSample=metrics(rows[1:]),
                                firstBaselineMs=rows[0]['report']['elapsedUs'] / 1000,
                                firstBaselineDisposition='Retained in primary result; sensitivity reported separately.',
                                readbackSamples=12, payloadFilesPerSample=10586,
                                copiedFixtureOnly=True, productionBudgetProven=False))
write('cleanup.json', dict(fixtureRootAbsent=True, copiedFixtureRetained=comparison['fixture'],
                           generatedSchemaPath=str(schema), generatedSchemaAbsent=True,
                           schemaArchiveSha256=digest(retained_schema), oldR2Touched=False))
write('first-error.json', dict(nativeCompileOrTestError=None, priorBatchCompilerError='E0609',
                               firstVerificationToolError='rustfmt was not on PATH',
                               rustfmtResolvedWithMsvcToolchain=True))
status = load(prior / 'status.json')
assert len(status['matrix']) == 273
status.update(code=revision, baselineCode=base, baselineStatus=(prior.relative_to(docs) / 'status.json').as_posix(),
              main=main, governance=governance, latestFetchedGovernance=governance, team=52, imp=191,
              testPassed=18, nativeTestFailure=None, firstBuildError=None)
status['matrix'].extend([
    {'item': 'r7 Windows enumeration hint retains native type/no-reparse and both membership fences',
     'status': '18 passed, including both stale file/directory replacement directions. 3 ignored in the scoped suite.'},
    {'item': 'r7 fixed copied 2.99 whole-tree and manifest comparison',
     'status': '12/12 raw0 under original 10s, 10586 hashes per scan. See scan-metrics.json. Component only.'},
    {'item': 'r7 scanner hint x86_64-pc-windows-msvc development artifacts',
     'status': 'Library tests and three product executables compiled raw0. See source-binding.json.'},
    {'item': 'TEAM52 isolated ordinary native Owner / V2 Pending acceptance',
     'status': 'New isolated ordinary interactive desktop and native test-artifact bootstrap/worker injection pending.'},
])
write('status.json', status)
write('payload-manifest.json', {'files': [
    {'path': path.relative_to(dest).as_posix(), 'bytes': path.stat().st_size, 'sha256': digest(path)}
    for path in sorted(dest.rglob('*')) if path.is_file()
]})
print(json.dumps({'revision': revision, 'matrix': len(status['matrix']), 'median': metrics(rows)}))
