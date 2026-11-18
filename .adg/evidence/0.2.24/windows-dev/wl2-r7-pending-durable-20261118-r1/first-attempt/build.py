import hashlib
import json
import pathlib
import shutil
import subprocess
import sys
import time

root = pathlib.Path(__file__).resolve().parent
prior = root.parent / 'r7-journal-readback-stages-20261118-r1'
mode, cargo, target = sys.argv[1:]
assert mode in ('compile01', 'bins01')
target = pathlib.Path(target)

def save(path, value):
    with path.open('x', encoding='utf-8') as output:
        json.dump(value, output, indent=2)
        output.write(chr(10))

def digest(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()

def inputs():
    names = set(subprocess.check_output(['git', '-c', 'core.autocrlf=false', 'diff', '--name-only', 'HEAD'], text=True).splitlines())
    names.update(subprocess.check_output(['git', 'ls-files', '--others', '--exclude-standard'], text=True).splitlines())
    assert names and all('windows_client_originals024/' in name or 'windows_official_lifecycle024' in name for name in names), names
    return {name: digest(pathlib.Path(name)) for name in sorted(names)}

base = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
expected = {'base': base, 'sha256ByPath': inputs()}
if mode == 'compile01':
    save(root / 'sources.json', expected)
else:
    assert json.loads((root / 'sources.json').read_bytes()) == expected
folder = root / mode
folder.mkdir()
args = json.loads((prior / mode / 'result.json').read_bytes())['arguments']
start = time.monotonic()
with (folder / 'stdout.log').open('xb') as out, (folder / 'stderr.log').open('xb') as err:
    raw = subprocess.run([cargo, *args], stdout=out, stderr=err).returncode
elapsed = round((time.monotonic() - start) * 1000)
save(folder / 'result.json', {'arguments': args, 'raw': raw, 'elapsedMs': elapsed})
print(f'{mode} raw={raw} elapsedMs={elapsed}', flush=True)
assert inputs() == expected['sha256ByPath']
if raw:
    sys.exit(raw)

def retain(path, directory):
    copy = directory / path.name
    with path.open('rb') as source, copy.open('xb') as dest:
        shutil.copyfileobj(source, dest)
    sha = digest(copy)
    assert digest(path) == sha
    return {'name': path.name, 'path': str(path), 'retainedPath': str(copy), 'bytes': copy.stat().st_size, 'sha256': sha}

if mode == 'compile01':
    rows = [json.loads(line) for line in (folder / 'stdout.log').read_bytes().splitlines()]
    artifacts = [pathlib.Path(row['executable']) for row in rows if row.get('reason') == 'compiler-artifact' and row.get('profile', {}).get('test') and row.get('target', {}).get('name') == 'opencodex_desktop_lib' and row.get('executable')]
    assert len(artifacts) == 1
    output = root / 'test-binary'
    output.mkdir()
    save(output / 'artifact.json', retain(artifacts[0], output))
else:
    output = root / 'product-binary'
    output.mkdir()
    binaries = target / 'x86_64-pc-windows-msvc' / 'debug'
    save(root / 'product-artifacts.json', [retain(binaries / exe, output) for exe in ('ocx.exe', 'ocxd.exe', 'opencodex-desktop.exe')])
