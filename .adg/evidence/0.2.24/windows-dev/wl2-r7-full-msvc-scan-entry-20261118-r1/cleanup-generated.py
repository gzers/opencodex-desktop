from pathlib import Path
import hashlib
import json
import os
import subprocess

ROOT = Path(__file__).resolve().parent
CODE = Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
PREVIOUS = ROOT.parent / 'wl2-r7-bound-preparation-20261118-r1'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write_once(path, data):
    with path.open('xb') as stream:
        stream.write(data)


run = json.loads((ROOT / 'product-scanner/result.json').read_bytes())
assert run['raw'] == 0
log = (ROOT / 'product-scanner/stdout.log').read_text(encoding='utf-8')
assert 'failed owned scanner fixture retained:' not in log
base = set(json.loads((ROOT / 'hypotheses.json').read_text(encoding='utf-8-sig'))['baselineTempRoots'])
remaining = {p.name for p in Path(os.environ['TEMP']).glob('ocx024-r7-image-*') if p.is_dir()}
assert remaining == base

schemas = CODE / 'apps/desktop/tauri/gen/schemas'
tracked = schemas / 'desktop-schema.json'
generated = schemas / 'windows-schema.json'
pristine = (PREVIOUS / 'generated-schema/desktop-schema-original.json').read_bytes()
canonical = subprocess.check_output(['git', 'show', 'HEAD:apps/desktop/tauri/gen/schemas/desktop-schema.json'], cwd=CODE)
assert pristine.replace(b'\r\n', b'\n') == canonical
changed = tracked.read_bytes()
new = generated.read_bytes()
archive = ROOT / 'generated-schema'
archive.mkdir()
write_once(archive / 'desktop-schema-generated.json', changed)
write_once(archive / 'desktop-schema-original.json', pristine)
assert archive.resolve().is_relative_to(ROOT.resolve())
assert generated.resolve() == generated and generated.is_file() and not generated.is_symlink()
assert generated.drive == archive.drive
assert not (archive / generated.name).exists()
generated.rename(archive / generated.name)
assert (archive / generated.name).read_bytes() == new
tracked.write_bytes(pristine)
assert tracked.read_bytes() == pristine and not generated.exists()
result = dict(existingFixtureRoots=sorted(base), newFixtureRootsRemaining=[], selfOwnedFixtureCleanupPassed=True, retainedFixedCopy=True, generatedSchemas=dict(desktopGenerated=sha(changed), desktopRestored=sha(pristine), windowsMoved=sha(new)), previousDeletePolicyRejectionRetried=False)
write_once(ROOT / 'cleanup-readback.json', json.dumps(result, indent=2).encode() + b'\n')
print(json.dumps(result))
