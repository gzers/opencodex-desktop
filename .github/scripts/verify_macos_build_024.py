"""Bind every sealed Mac executable to the actual Cargo build artifacts.

Signing changes Mach-O bytes and __LINKEDIT allocation. Remove signatures and
apply one identical ad-hoc identity only on private temporary copies, compare
their complete bytes, and recheck immutable inputs. Never launch, install,
re-sign or modify the sealed application.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

NAMES = {'opencodex-desktop', 'ocxd', 'ocx'}


def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda: source.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def verify(app, artifacts, project, source):
    app, project = Path(app), Path(project).resolve(strict=True)
    if app.is_symlink() or not app.is_dir() or app.name != 'OpenCodeX Desktop.app':
        raise ValueError('explicit sealed app required')
    if len(source) != 40 or any(c not in '0123456789abcdef' for c in source):
        raise ValueError('exact source commit required')
    observed_source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=project, text=True).strip()
    if source != observed_source or subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=no', '--', '.', '../ui'], cwd=project):
        raise ValueError('build source changed or tracked worktree is dirty')
    binaries = {}
    finished = False
    sources = {name: project / 'src/bin' / (name + '.rs') for name in NAMES}
    sources['opencodex-desktop'] = project / 'src/main.rs'
    for line in Path(artifacts).read_text().splitlines():
        # Tauri logs include frontend output around Cargo's JSON messages.
        if not line.startswith('{'):
            continue
        row = json.loads(line)
        if row.get('reason') == 'build-finished':
            finished = row.get('success') is True
        if row.get('reason') != 'compiler-artifact' or not row.get('executable'):
            continue
        target = row['target']; name = target['name']
        if name not in NAMES:
            continue
        if name in binaries or target['kind'] != ['bin'] or Path(target['src_path']).resolve() != sources[name]:
            raise ValueError('ambiguous or foreign Cargo executable: ' + name)
        binaries[name] = Path(row['executable'])
    if not finished or set(binaries) != NAMES:
        raise ValueError('successful Cargo build of all three executables required')
    macos = app / 'Contents/MacOS'
    if macos.is_symlink() or {p.name for p in macos.iterdir()} != NAMES:
        raise ValueError('sealed executable membership differs')
    subprocess.run(['/usr/bin/codesign', '--verify', '--deep', '--strict', app], check=True, capture_output=True)
    rows = []
    with tempfile.TemporaryDirectory(prefix='ocxd024-build-binding-') as temporary:
        root = Path(temporary)
        for name in sorted(NAMES):
            delivered, compiled = macos / name, binaries[name]
            for path in (delivered, compiled):
                if path.is_symlink() or not path.is_file() or path.stat().st_nlink != 1:
                    raise ValueError('single-link regular executable required')
            before = [sha(compiled), sha(delivered)]
            copies = [root / (name + '-compiled'), root / (name + '-delivered')]
            for original, copied in zip((compiled, delivered), copies):
                shutil.copyfile(original, copied)
                # rustc and Tauri both produce signed arm64 Mach-O files. Absence
                # or failure is unknown, never silently treated as equivalent.
                subprocess.run(['/usr/bin/codesign', '--remove-signature', copied], check=True, capture_output=True)
                subprocess.run(['/usr/bin/codesign', '--force', '--sign', '-', '--identifier',
                                'org.opencodex.build-comparison', '--timestamp=none', copied],
                               check=True, capture_output=True)
            normalized = [sha(p) for p in copies]
            if normalized[0] != normalized[1]:
                raise ValueError('sealed executable differs from compiled source: ' + name)
            if before != [sha(compiled), sha(delivered)]:
                raise ValueError('immutable executable changed during verification')
            rows.append({'name': name, 'compiledSha256': before[0], 'sealedSha256': before[1],
                         'canonicalSignedSha256': normalized[0]})
    if subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=project, text=True).strip() != source:
        raise ValueError('source changed during verification')
    return {'source': source, 'executables': rows, 'status': 'verified-build-binding',
            'copiesRemoved': True, 'applicationLaunched': False, 'releaseAcceptance': False}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--artifacts', type=Path, required=True)
    parser.add_argument('--project', type=Path, required=True)
    parser.add_argument('--source', required=True)
    args = parser.parse_args()
    print(json.dumps(verify(args.app, args.artifacts, args.project, args.source), indent=2))
