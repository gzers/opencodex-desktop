import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import time

run = pathlib.Path(__file__).resolve().parent
fixture = run / 'fixtures'
fixture.mkdir()

def save(name, value):
    with (run / name).open('x', encoding='utf-8') as output:
        json.dump(value, output, indent=2)
        output.write(chr(10))

def digest(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()

build = json.loads((run / 'build.json').read_bytes())
assert build['raw'] == 0
assert digest(pathlib.Path(build['source'])) == build['sourceSha256']
rows = [json.loads(line) for line in (run / 'cargo.stdout.log').read_bytes().splitlines()]
artifacts = [pathlib.Path(row['executable']) for row in rows
             if row.get('reason') == 'compiler-artifact'
             and row.get('profile', {}).get('test')
             and row.get('target', {}).get('name') == 'opencodex_desktop_lib'
             and row.get('executable')]
assert len(artifacts) == 1, artifacts
binary = run / artifacts[0].name
if binary.exists():
    raise FileExistsError(binary)
shutil.copyfile(artifacts[0], binary)
assert digest(binary) == digest(artifacts[0])
save('artifact.json', {'path': str(binary), 'cargoPath': str(artifacts[0]),
                       'bytes': binary.stat().st_size, 'sha256': digest(binary)})

env = dict(os.environ, TEMP=str(fixture), TMP=str(fixture))
test = ('modules::runtime::program_image::windows_tests::'
        'r7_image_pinned_refuses_directory_stream_added_after_payload_reads')
args = ['--exact', test, '--nocapture', '--test-threads=1']
start = time.monotonic()
timed_out = False
with (run / 'test.stdout.log').open('xb') as out, (run / 'test.stderr.log').open('xb') as err:
    try:
        raw = subprocess.run([binary, *args], stdout=out, stderr=err,
                             env=env, timeout=20).returncode
    except subprocess.TimeoutExpired:
        raw = None
        timed_out = True
stdout = (run / 'test.stdout.log').read_text(encoding='utf-8')
remaining = [str(path) for path in fixture.iterdir()]
passed = raw == 0 and '1 passed; 0 failed' in stdout and 'R7_LATE_DIRECTORY_STREAM' in stdout
save('test.json', {'arguments': args, 'raw': raw, 'timeout': timed_out,
                   'elapsedMs': round((time.monotonic() - start) * 1000),
                   'passed': passed, 'remainingFixtureEntries': remaining})
if passed and not remaining:
    fixture.rmdir()
    assert not fixture.exists()
else:
    raise SystemExit(1)
