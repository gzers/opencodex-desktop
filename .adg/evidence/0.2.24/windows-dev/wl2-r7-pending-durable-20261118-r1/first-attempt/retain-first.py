from pathlib import Path
import hashlib
import json
import shutil
import subprocess
import sys
root = Path(sys.argv[1]).resolve()
assert json.loads((root / 'compile01/result.json').read_bytes())['raw'] == 0
sources = json.loads((root / 'sources.json').read_bytes())['sha256ByPath']
assert all(hashlib.sha256(Path(name).read_bytes()).hexdigest() == sha for name, sha in sources.items())
rows = [json.loads(line) for line in (root / 'compile01/stdout.log').read_bytes().splitlines()]
paths = [Path(r['executable']) for r in rows if r.get('reason') == 'compiler-artifact' and r.get('profile', {}).get('test') and r.get('target', {}).get('name') == 'opencodex_desktop_lib' and r.get('executable')]
assert len(paths) == 1
out = root / 'test-binary'
out.mkdir()
exe = out / paths[0].name
shutil.copyfile(paths[0], exe)
sha = hashlib.sha256(exe.read_bytes()).hexdigest()
assert hashlib.sha256(paths[0].read_bytes()).hexdigest() == sha
report = {'name': exe.name, 'path': str(paths[0]), 'retainedPath': str(exe), 'bytes': exe.stat().st_size, 'sha256': sha}
(out / 'artifact.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
schema = Path('apps/desktop/tauri/gen/schemas/windows-schema.json')
assert schema.is_file() and not subprocess.check_output(['git', 'ls-files', '--', str(schema)])
copy = root / 'generated-windows-schema.json'
shutil.copyfile(schema, copy)
assert copy.read_bytes() == schema.read_bytes()
print(json.dumps(report))
