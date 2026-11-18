import hashlib
import json
import pathlib
import shutil
import subprocess
import sys
import time

root = pathlib.Path(__file__).resolve().parent
cargo = sys.argv[1]
target = pathlib.Path(sys.argv[2])


def save(path, value):
    with path.open('x', encoding='utf-8', newline='\n') as output:
        json.dump(value, output, indent=2, ensure_ascii=False)
        output.write('\n')


def digest(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def retain(path, directory):
    copy = directory / path.name
    with path.open('rb') as inp, copy.open('xb') as out:
        shutil.copyfileobj(inp, out)
    sha = digest(copy)
    assert digest(path) == sha
    return {'name': path.name, 'path': str(path), 'retainedPath': str(copy),
            'bytes': copy.stat().st_size, 'sha256': sha}


previous = root.parent / 'r7-worker-result-20261118-r1'
base = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
changed = subprocess.check_output(['git', 'diff', '--name-only', 'HEAD'], text=True).splitlines()
assert len(changed) == 3 and all('windows_client_originals024/' in f for f in changed)
inputs = {name: digest(pathlib.Path(name)) for name in changed}
save(root / 'sources.json', {'base': base, 'sha256ByPath': inputs})

results = []
for name in ('compile01',):
    directory = root / name
    directory.mkdir()
    args = json.loads((previous / ('compile01' if name == 'compile01' else name) / 'result.json').read_text(encoding='utf-8'))['arguments']
    start = time.monotonic()
    with (directory / 'stdout.log').open('xb') as out, (directory / 'stderr.log').open('xb') as err:
        raw = subprocess.run([cargo, *args], stdout=out, stderr=err).returncode
    elapsed = round((time.monotonic() - start) * 1000)
    report = {'arguments': args, 'raw': raw, 'elapsedMs': elapsed}
    save(directory / 'result.json', report)
    results.append(report)
    print(f'{name} raw={raw} elapsedMs={elapsed}', flush=True)
    if raw:
        continue
    if name == 'compile01':
        rows = [json.loads(line) for line in (directory / 'stdout.log').read_bytes().splitlines()]
        paths = [pathlib.Path(row['executable']) for row in rows if row.get('reason') == 'compiler-artifact'
                 and row.get('profile', {}).get('test') and row.get('target', {}).get('name') == 'opencodex_desktop_lib'
                 and row.get('executable')]
        assert len(paths) == 1
        output = root / 'test-binary'
        output.mkdir()
        save(output / 'artifact.json', retain(paths[0], output))
    else:
        output = root / 'product-binary'
        output.mkdir()
        binaries = target / 'x86_64-pc-windows-msvc' / 'debug'
        save(root / 'product-artifacts.json', [retain(binaries / exe, output) for exe in
                                              ('ocx.exe', 'ocxd.exe', 'opencodex-desktop.exe')])

assert {name: digest(pathlib.Path(name)) for name in changed} == inputs
sys.exit(any(result['raw'] for result in results))
