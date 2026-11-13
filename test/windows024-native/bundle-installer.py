"""Compile actual Windows templates/resources with an explicitly marked test EXE.

The EXE is the standalone native fixture, not a product or release candidate.
No installer is executed. CLI/tool caches and generated project stay under target.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import traceback

ROOT = Path(__file__).resolve().parents[2]


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--work', type=Path, required=True)
    parser.add_argument('--tooling', type=Path, required=True)
    parser.add_argument('--node', type=Path, required=True)
    parser.add_argument('--evidence', type=Path, required=True)
    args = parser.parse_args()
    work, evidence = args.work.resolve(), args.evidence.resolve()
    if not work.is_relative_to(Path(__file__).resolve().parent / 'target'):
        raise ValueError('generated project must be inside standalone target')
    evidence.mkdir(parents=True, exist_ok=False)
    report = {'passed': False, 'productExecutable': False, 'installed': False,
              'scope': 'native template/resource compilation only', 'commands': []}
    try:
        staged = json.loads((work / 'stage-result.json').read_bytes())
        if not staged['passed']:
            raise ValueError('real Windows024 stage required')
        project = work / 'tauri'
        (project / 'src').mkdir(exist_ok=True)
        (project / 'src/main.rs').write_text('fn main() {}\n')
        (project / 'Cargo.toml').write_text('[package]\nname="ocxd"\nversion="0.2.24"\nedition="2021"\n[workspace]\n')
        (project / 'windows').mkdir(exist_ok=True)
        for name in ('installer-hooks.nsh', 'nsis023.nsi', 'wix023.wxs'):
            shutil.copy2(ROOT / 'apps/desktop/tauri/windows' / name, project / 'windows' / name)
        for name in ('LICENSE', 'THIRD-PARTY-NOTICES.txt'):
            shutil.copy2(ROOT / 'apps/desktop/tauri/resources' / name, project / 'resources' / name)
        (project / 'icons').mkdir(exist_ok=True)
        shutil.copy2(ROOT / 'apps/desktop/tauri/icons/windows/icon.ico', project / 'icons/icon.ico')
        executable = project / 'target/release/ocxd.exe'
        executable.parent.mkdir(exist_ok=True, parents=True)
        fixture = Path(__file__).resolve().parent / 'target/debug/native024-fixture.exe'
        shutil.copy2(fixture, executable)
        config = {
            'productName': 'OpenCodeX Desktop', 'version': '0.2.24',
            'identifier': 'com.gzers.opencodex.w1-resource-proof',
            'bundle': {'active': True, 'targets': ['msi', 'nsis'], 'icon': ['icons/icon.ico'],
                       'resources': {'resources/LICENSE': 'resources/LICENSE',
                                     'resources/THIRD-PARTY-NOTICES.txt': 'resources/THIRD-PARTY-NOTICES.txt',
                                     'resources/runtime-bundle/': 'runtime-bundle/'},
                       'windows': {'webviewInstallMode': {'type': 'skip'},
                                   'nsis': {'template': 'windows/nsis023.nsi', 'installerHooks': 'windows/installer-hooks.nsh', 'installMode': 'currentUser', 'compression': 'none'},
                                   'wix': {'template': 'windows/wix023.wxs', 'upgradeCode': '77eaf3e9-84df-48e4-b493-a267ab705070'}}}}
        (project / 'tauri.conf.json').write_text(json.dumps(config, indent=2))
        cli = args.tooling.resolve() / 'node_modules/@tauri-apps/cli/tauri.js'
        command = [str(args.node.resolve()), str(cli), 'bundle', '--ci', '--no-sign', '--bundles', 'msi,nsis', '-vv']
        with (evidence / 'bundle.stdout.log').open('wb') as out, (evidence / 'bundle.stderr.log').open('wb') as err:
            result = subprocess.run(command, cwd=project, stdout=out, stderr=err)
        report['commands'].append({'argv': command, 'cwd': str(project), 'exitCode': result.returncode})
        if result.returncode:
            raise RuntimeError('native bundler failed; see original stdout/stderr')
        outputs = sorted((project / 'target/release/bundle').glob('*/*'))
        report.update(passed=True, fixtureSha256=digest(fixture), stage=staged,
                      outputs=[{'path': str(p), 'bytes': p.stat().st_size, 'sha256': digest(p)}
                               for p in outputs if p.is_file()])
    except Exception as error:
        report['firstError'] = {'type': type(error).__name__, 'message': str(error)}
        traceback.print_exc()
    finally:
        (evidence / 'result.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    sys.exit(main())
