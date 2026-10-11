from pathlib import Path
import subprocess
import json
import hashlib
import plistlib
import shutil
from datetime import datetime, timezone

source = Path('/private/tmp/ocx-0110-clock-audit')
code = Path('/Users/ezio/.codex/worktrees/ocx-0110-main')
identity = json.loads((source / 'source-identity.json').read_text())
build = json.loads((source / 'macos-build.json').read_text())
assert build['exit_code'] == 0
assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=code).decode().strip() == identity['source_sha']
bundle = code / 'apps/desktop/tauri/target/release/bundle'
apps = list((bundle / 'macos').glob('*.app'))
dmgs = list((bundle / 'dmg').glob('*_0.1.10_aarch64.dmg'))
assert len(apps) == len(dmgs) == 1, (apps, dmgs)
original = source / 'candidate/original'
original.mkdir(parents=True, exist_ok=False)
subprocess.run(['ditto', str(apps[0]), str(original / apps[0].name)], check=True)
shutil.copy2(dmgs[0], original / dmgs[0].name)
app = original / apps[0].name
with (app / 'Contents/Info.plist').open('rb') as f:
    info = plistlib.load(f)
exe = app / 'Contents/MacOS' / info['CFBundleExecutable']
assert info['CFBundleShortVersionString'] == info['CFBundleVersion'] == '0.1.10'
assert info['CFBundleIdentifier'] == 'com.gzers.opencodex.desktop'
assert identity['source_sha'].encode() in exe.read_bytes(), 'missing source identity'
commands = {}
for name, argv in [
    ('macos-file', ['file', str(exe)]),
    ('macos-dmg-verify', ['hdiutil', 'verify', str(original / dmgs[0].name)]),
    ('macos-signature-verify', ['codesign', '--verify', '--deep', '--strict', '--verbose=2', str(app)]),
    ('macos-signature-display', ['codesign', '-d', '--verbose=4', str(app)]),
    ('macos-signing-identities', ['security', 'find-identity', '-v', '-p', 'codesigning']),
]:
    r = subprocess.run(argv, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    (source / (name + '.log')).write_bytes(r.stdout)
    commands[name] = {'argv': argv, 'exit_code': r.returncode}
    print(name, r.returncode, r.stdout.decode(errors='replace')[-1000:])
assert commands['macos-dmg-verify']['exit_code'] == 0
assert commands['macos-file']['exit_code'] == 0
assert b'arm64' in (source / 'macos-file.log').read_bytes()
sha = lambda b: hashlib.sha256(b).hexdigest()
assets = [{'path': str(p.relative_to(source)), 'bytes': p.stat().st_size, 'sha256': sha(p.read_bytes())} for p in [original / dmgs[0].name, exe]]
result = {
    'verified_at_utc': datetime.now(timezone.utc).isoformat(),
    'source_sha': identity['source_sha'], 'build': build,
    'info': {k: info[k] for k in ['CFBundleIdentifier', 'CFBundleShortVersionString', 'CFBundleVersion', 'CFBundleExecutable']},
    'full_source_sha_embedded': True, 'assets': assets, 'commands': commands,
    'installed_or_launched': False, 'resigned': False,
    'notarization_verified': False, 'release_accepted': False,
}
(source / 'candidate-verification.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + chr(10))
print(json.dumps(result, ensure_ascii=False))
