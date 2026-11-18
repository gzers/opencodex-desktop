from pathlib import Path
import hashlib
import json
import shutil
import subprocess
import sys
import time

root = Path(__file__).resolve().parent
cargo, mode, prior, attempt = sys.argv[1:]
assert mode in ('test', 'products')
run = root / attempt
run.mkdir()

def save(path, value):
    with path.open('x', encoding='utf-8') as out:
        json.dump(value, out, indent=2)
        out.write('\n')

def digest(path):
    with Path(path).open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()

sources = sorted(set(subprocess.check_output(["git", "diff", "--name-only", "HEAD"], text=True).splitlines() + subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard"], text=True).splitlines()))
base = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
source_hashes = {path: digest(path) for path in sources}
old = Path(prior) / ('compile01' if mode == 'test' else 'bins01') / 'result.json'
args = json.loads(old.read_bytes())['arguments']
if mode == 'products' and '--message-format=json' not in args:
    args += ['--message-format=json']
started = time.monotonic()
with (run / 'stdout.log').open('xb') as out, (run / 'stderr.log').open('xb') as err:
    raw = subprocess.run([cargo, *args], stdout=out, stderr=err).returncode
save(run / 'result.json', dict(raw=raw, elapsedMs=round((time.monotonic() - started) * 1000), cargo=cargo, arguments=args, base=base, sourceHashes=source_hashes))
assert source_hashes == {path: digest(path) for path in sources}
if raw:
    raise SystemExit(raw)
records = []
for line in (run / 'stdout.log').read_bytes().splitlines():
    row = json.loads(line)
    target = row.get('target', {})
    expected = target.get('name') == 'opencodex_desktop_lib' and row.get('profile', {}).get('test') if mode == 'test' else target.get('name') in ('ocx', 'ocxd', 'opencodex-desktop') and target.get('kind') == ['bin']
    if row.get('reason') == 'compiler-artifact' and row.get('executable') and expected:
        original = Path(row['executable'])
        retained = run / original.name
        shutil.copyfile(original, retained)
        sha = digest(retained)
        assert sha == digest(original)
        records.append(dict(name=retained.name, path=str(retained), bytes=retained.stat().st_size, sha256=sha))
assert len(records) == (1 if mode == 'test' else 3), records
save(run / 'artifacts.json', records)
print(json.dumps(records), flush=True)

