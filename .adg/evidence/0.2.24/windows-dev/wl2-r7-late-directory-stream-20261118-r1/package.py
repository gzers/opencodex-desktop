import hashlib
import json
import pathlib
import shutil
import subprocess
import sys

code, docs = map(pathlib.Path, sys.argv[1:])
root = pathlib.Path(__file__).resolve().parent.parent
first = root / 'r7-late-stream-20261118-r1'
final = root / 'r7-late-stream-20261118-r2'
evidence = docs / '.adg/evidence/0.2.24/windows-dev'
prior = evidence / 'wl2-r7-r2-loaded-source-review-20261118-r1'
trace = evidence / 'wl2-r7-preparation-scan-trace-20261118-r1'
dest = evidence / 'wl2-r7-late-directory-stream-20261118-r1'

def read(path):
    return json.loads(path.read_bytes())

def digest(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()

def save(name, value):
    with (dest / name).open('x', encoding='utf-8') as output:
        json.dump(value, output, indent=2, ensure_ascii=False)
        output.write(chr(10))

def git(*args):
    return subprocess.check_output(['git', '-C', str(code), *args])

initial_build, build = read(first / 'build.json'), read(final / 'build.json')
test = read(final / 'test.json')
assert initial_build['raw'] == 101 and build['raw'] == 0
assert test['passed'] and not test['remainingFixtureEntries']
assert not (final / 'fixtures').exists()
assert read(final / 'rustfmt.json')['raw'] == 0
sha = git('rev-parse', 'HEAD').decode().strip()
source = code / build['source']
assert build['sourceSha256'] == digest(source)
assert git('rev-parse', 'HEAD^').decode().strip() == build['base']
assert git('show', f'{sha}:{build["source"]}') == source.read_bytes().replace(b'\r\n', b'\n')
assert git('diff', '--name-only', 'HEAD^', 'HEAD').decode().splitlines() == [build['source']]
schema = code / 'apps/desktop/tauri/gen/schemas/windows-schema.json'
assert not schema.exists()
assert (final / 'generated-windows-schema.json').is_file()
artifact = read(final / 'artifact.json')
assert digest(pathlib.Path(artifact['path'])) == artifact['sha256']

dest.mkdir(exist_ok=True)
assert {p.name for p in dest.iterdir()} <= {'.gitattributes'}
if not (dest / '.gitattributes').exists():
    shutil.copyfile(prior / '.gitattributes', dest / '.gitattributes')
assert (dest / '.gitattributes').read_bytes() == (prior / '.gitattributes').read_bytes()
shutil.copyfile(trace / 'verify_git_payload.py', dest / 'verify_git_payload.py')
shutil.copyfile(__file__, dest / 'package.py')
for directory, source_root, names in [
    ('compile-first', first, ['build.json', 'cargo.stdout.log', 'cargo.stderr.log']),
    ('compile-final', final, ['build.json', 'cargo.stdout.log', 'cargo.stderr.log', 'rustfmt.json']),
    ('direct', final, ['run.py', 'test.json', 'test.stdout.log', 'test.stderr.log', 'artifact.json']),
]:
    (dest / directory).mkdir()
    for name in names:
        shutil.copyfile(source_root / name, dest / directory / name)
shutil.copyfile(final / 'generated-windows-schema.json', dest / 'generated-windows-schema.json')
rows = [json.loads(line) for line in (first / 'cargo.stdout.log').read_bytes().splitlines()]
errors = [row['message'] for row in rows
          if row.get('reason') == 'compiler-message' and row.get('message', {}).get('level') == 'error']
assert errors[0]['code']['code'] == 'E0609'
save('first-error.json', {'raw': 101, 'compilerCode': 'E0609', 'diagnostic': errors[0]['rendered']})
save('first-packaging-error.json', {'raw': 1, 'error': 'FileNotFoundError',
                                   'missing': (prior.relative_to(docs) / 'verify_git_payload.py').as_posix(),
                                   'resolution': 'Reuse the verification script from the preparation scan evidence.'})
save('cleanup.json', {'isolatedFixtureRoot': str(final / 'fixtures'), 'fixtureRootAbsent': True,
                      'unexpectedFixtureEntries': [], 'generatedSchemaPath': str(schema),
                      'generatedSchemaAbsent': True,
                      'schemaArchiveSha256': digest(final / 'generated-windows-schema.json'),
                      'currentR2Touched': False})
governance = git('rev-parse', 'origin/docs/governance-main').decode().strip()
governance_basis = '433dd879f894e60d285e4725b49e1b360f3c7038'
status = read(prior / 'status.json')
status.update(code=sha, baselineCode=build['base'], baselineStatus=(prior.relative_to(docs) / 'status.json').as_posix(),
              main=git('rev-parse', 'origin/main').decode().strip(), governance=governance_basis,
              latestFetchedGovernance=governance, team=51, imp=190, testPassed=1, cleanupUniqueRoots=1,
              nativeTestFailure=None, firstBuildError={'raw': 101, 'code': 'E0609'})
assert len(status['matrix']) == 272
status['matrix'].append({'item': 'r7 Windows final fence after a late directory ADS',
                         'status': '1 scoped MSVC test passed. All three ordinary payload jobs completed; a new ADS on an empty directory was readable while ordinary membership remained empty. The scan rejected runtime_image_unconfirmed. Removing the fixture ADS restored exact reference inventory.'})
save('status.json', status)
save('source-binding.json', {'code': sha, 'base': build['base'], 'buildInputSha256': digest(source),
                            'sourcePath': build['source'], 'binarySha256': artifact['sha256'],
                            'onlyTestSourceChanged': True})
save('payload-manifest.json', {'files': [
    {'path': path.relative_to(dest).as_posix(), 'bytes': path.stat().st_size, 'sha256': digest(path)}
    for path in sorted(dest.rglob('*')) if path.is_file()
]})
