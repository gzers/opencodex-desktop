from pathlib import Path
import hashlib
import json
import mmap
import re
import shutil
import subprocess
import sys


def load(path):
    return json.loads(path.read_bytes())


def digest(path):
    with path.open('rb') as src:
        return hashlib.file_digest(src, 'sha256').hexdigest()


def git(*args):
    return subprocess.check_output(['git', '-C', str(code), *args])


def write(name, value):
    with (dest / name).open('x', encoding='utf-8', newline='\n') as output:
        json.dump(value, output, indent=2, ensure_ascii=False)
        output.write('\n')


code, docs = (Path(value).resolve() for value in sys.argv[1:])
run = Path(__file__).resolve().parent
evidence = docs / '.adg/evidence/0.2.24/windows-dev'
prior = evidence / 'wl2-r7-enum-hint-20261118-r1'
dest = evidence / 'wl2-r7-partial-owner-20261118-r1'
revision = git('rev-parse', 'HEAD').decode().strip()
base = git('rev-parse', 'HEAD^').decode().strip()
main = git('rev-parse', 'origin/main').decode().strip()
governance = git('rev-parse', 'origin/docs/governance-main').decode().strip()
assert main == '6bafdd342e69ee83e842f4b35a20bafa1320645a'
assert governance == '89294318f8c5010c58c6b2ba6d3cd9e3deecb219'
authorization = '81904340f107f391fec62bf3f282e91e8504b263'
assert base == '766d1f010be60865e7a425dab242ab5d861773b5'
changed = set(git('diff-tree', '--no-commit-id', '--name-only', '-r', revision).decode().splitlines())
schema = 'apps/desktop/tauri/gen/schemas/windows-schema.json'
retained_schema = run / 'generated-windows-schema.json'
assert not (code / schema).exists() and retained_schema.is_file()
assert all(path.startswith((
    'apps/desktop/tauri/src/infrastructure/windows_client_originals024/',
    'apps/desktop/tauri/src/infrastructure/windows_official_lifecycle024/',
    'apps/desktop/tauri/src/infrastructure/windows_owner_host024/',
    'test/windows024-native/',
)) or path == 'apps/desktop/tauri/src/infrastructure/windows_official_lifecycle024.rs' for path in changed)
inputs = None
artifacts = {}
results = {}
for kind, directory in [('test', 'compile-test-05'), ('products', 'compile-products-03')]:
    compiled = run / directory
    result = load(compiled / 'result.json')
    assert result['raw'] == 0 and result['base'] == base
    source_hashes = dict(result['sourceHashes'])
    assert source_hashes.pop(schema) == digest(retained_schema)
    assert changed == set(source_hashes)
    binding = {}
    for path, sha in source_hashes.items():
        original = (code / path).read_bytes()
        blob = git('show', f'{revision}:{path}')
        assert sha == hashlib.sha256(original).hexdigest() and original.replace(b'\r\n', b'\n') == blob, path
        binding[path] = {'workingInputSha256': sha, 'gitBlobSha256': hashlib.sha256(blob).hexdigest()}
    if inputs is not None:
        assert inputs == binding
    inputs = binding
    artifacts[kind] = load(compiled / 'artifacts.json')
    for artifact in artifacts[kind]:
        path = Path(artifact['path'])
        assert path.stat().st_size == artifact['bytes'] and digest(path) == artifact['sha256']
        with path.open('rb') as file, mmap.mmap(file.fileno(), 0, access=mmap.ACCESS_READ) as image:
            contains = image.find(b'isolated-r7-partial-capture-original-worker') >= 0
        assert contains == (kind == 'test'), artifact['name']
        artifact['testBootstrapMarkerPresent'] = contains
    results[kind] = {'batch': directory, 'raw': result['raw'], 'elapsedMs': result['elapsedMs']}

assert load(run / 'compile-test-02/result.json')['raw'] == 101
assert 'E0308' in (run / 'compile-test-02/stdout.log').read_text(encoding='utf-8')
assert results['test']['raw'] == 0 and results['products']['raw'] == 0
assert load(run / 'desktop-data-01/result.json')['raw'] == 0
powershell = load(run / 'desktop-data-01/stdout.log')
assert powershell['passed'] == 11 and powershell['fixtureAbsent']
assert not Path(powershell['fixture']).exists()

fixture_paths = set()
passed = 0
for kind in ('targeted-01', 'targeted-02', 'targeted-03'):
    batch = run / kind
    attempts = load(batch / 'result.json')
    assert len(attempts) == 7 and all(row['raw'] == 0 for row in attempts)
    counts = []
    for path in batch.glob('*.stdout.log'):
        raw = path.read_text(encoding='utf-8')
        matches = re.findall(r'test result: ok\. (\d+) passed; 0 failed; 0 ignored', raw)
        assert len(matches) == 1
        counts.append(int(matches[0]))
        assert '-failed-fixture-retained=' not in raw
        fixture_paths.update(re.findall(r'cleanup-absent=([^\r\n]+)', raw))
    assert len(counts) == 7
    passed = sum(counts)
    assert passed == 13
assert fixture_paths and all(not Path(path).exists() for path in fixture_paths)
environment = load(run / 'environment-gate.json')
assert not any(environment[name] for name in ('controllerAdministrator', 'dedicatedAccountPresent',
                                               'profilePresent', 'publicFixturePresent', 'oldR2LiveAccess'))

status = load(prior / 'status.json')
assert len(status['matrix']) == 277
status.update(code=revision, baselineCode=base, baselineStatus=(prior.relative_to(docs) / 'status.json').as_posix(),
              main=main, governance=governance, latestFetchedGovernance=governance, team=53, imp=192,
              testPassed=passed, nativeTestFailure=None, firstBuildError='E0308')
status['matrix'].extend([
    {'item': 'TEAM52 partial capture fixture bootstrap, fixed package, Original Owner and raw31 arm',
     'status': 'Test image compiled; local scoped bootstrap and worker cases passed. Dedicated ordinary desktop absent; actual fixed-image Start/injection unverified.'},
    {'item': 'TEAM52 original-worker V2 Pending attachment and claim/NLK continuity',
     'status': 'Scoped 13 native tests passed; original thread, exact attachment, policy/chain and failure cases covered by components. Dedicated full production-shaped session pending.'},
    {'item': 'TEAM52 desktop evidence parser and bounded byte export',
     'status': 'PowerShell 5.1 parsed the final launcher and passed 11 local DATA cases, including exact-byte export, membership, wrong thread, budget and digest failures.'},
    {'item': 'TEAM52 Initial Start and recovery image-scan tracing',
     'status': 'Test-only trace compiled. Dedicated actual initial 30s and explicit recovery 10s trace/same-sample Status 5s unverified. Existing fixed-copy data reused.'},
    {'item': 'TEAM52 MSVC products and test image',
     'status': 'Three final development product executables and one test image compiled raw0; see source-binding.json and retained results.'},
    {'item': 'TEAM52 dedicated ordinary desktop preflight and cleanup',
     'status': 'ocxr7owner absent; controller lacks Administrator. No account, Profile or staging created. Local test roots and generated schema verified absent.'},
])

dest.mkdir()
for name in ('.gitattributes', 'verify_git_payload.py'):
    shutil.copyfile(prior / name, dest / name)
with (dest / '.gitattributes').open('ab') as attributes:
    attributes.write(b'*.py whitespace=blank-at-eol,-blank-at-eof,space-before-tab,cr-at-eol\n'
                     b'*.ps1 whitespace=blank-at-eol,-blank-at-eof,space-before-tab,cr-at-eol\n')
for name in ('build.py', 'build.ps1', 'desktop-data.ps1', 'environment-gate.json', 'targeted.py', 'targeted-final.py',
             'targeted-receipt.py', 'package.py', retained_schema.name):
    shutil.copyfile(run / name, dest / name)
for name in ('compile-test-01', 'compile-test-02', 'compile-test-03', 'compile-test-04', 'compile-test-05',
             'compile-products-01', 'compile-products-02', 'compile-products-03',
             'targeted-01', 'targeted-02', 'targeted-03', 'desktop-data-01'):
    target = dest / name
    target.mkdir()
    for path in sorted((run / name).iterdir()):
        if path.is_file() and path.suffix in ('.log', '.json'):
            shutil.copyfile(path, target / path.name)

write('source-binding.json', dict(source=revision, base=base, main=main, governance=governance,
                                  team=53, imp=192, authorizationGovernance=authorization, buildInputs=inputs, generatedSchemaSha256=digest(retained_schema),
                                  artifacts=artifacts, buildResults=results))
write('cleanup.json', dict(verifiedAbsentLocalTestRoots=sorted(fixture_paths), desktopData=powershell,
                           generatedSchemaPath=str(code / schema), generatedSchemaAbsent=True,
                           schemaArchiveSha256=digest(retained_schema), dedicatedAccountCreated=False,
                           dedicatedProfileCreated=False, fixtureStaged=False, currentR2Touched=False))
write('first-error.json', dict(firstRustError='E0308', firstRustRaw=101, rustLog='compile-test-02/stdout.log',
                               detail='Test projection passed io::Result<()> after preparation attachment changed to LeaseFacts.',
                               firstToolError='Python source-edit quoting SyntaxError before write; corrected in subsequent edit.',
                               finalTargetedFailure=None, finalDesktopDataFailure=None,
                               priorOfficialStartError='windows024_originals_required/timed out, created-suspended, 30699 ms',
                               currentR2CommandLineageRaw=64, currentR2JobUnknown=24, currentR2JobRaw=50))
write('status.json', status)
write('payload-manifest.json', {'files': [
    {'path': path.relative_to(dest).as_posix(), 'bytes': path.stat().st_size, 'sha256': digest(path)}
    for path in sorted(dest.rglob('*')) if path.is_file()
]})
print(json.dumps({'revision': revision, 'matrix': len(status['matrix']), 'roots': len(fixture_paths),
                  'artifacts': artifacts, 'buildResults': results}))
