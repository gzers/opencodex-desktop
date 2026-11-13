"""Compile actual Windows templates/resources with an explicitly marked EXE.

The default EXE is a fixture; --product-bin-dir uses MSVC development binaries.
Both use a test bundle identity, never a release candidate.
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
import os

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
    parser.add_argument('--product-bin-dir', type=Path)
    parser.add_argument('--stage-source', type=Path)
    args = parser.parse_args()
    work, evidence = args.work.resolve(), args.evidence.resolve()
    if not work.is_relative_to(Path(__file__).resolve().parent / 'target'):
        raise ValueError('generated project must be inside standalone target')
    evidence.mkdir(parents=True, exist_ok=False)
    report = {'passed': False, 'productExecutable': False, 'installed': False,
              'scope': 'native template/resource compilation only', 'commands': []}
    try:
        if args.product_bin_dir:
            for name in ('opencodex-desktop.exe', 'ocxd.exe', 'ocx.exe'):
                if not (args.product_bin_dir.resolve() / name).is_file():
                    raise ValueError('missing exact product build input: ' + name)
        if args.stage_source:
            work.mkdir(parents=True, exist_ok=False)
            source = args.stage_source.resolve()
            staged = json.loads((source / 'stage-result.json').read_bytes())
            resources = source / 'tauri/resources/runtime-bundle'
            descriptor = json.loads((resources / 'runtime-bundle.manifest.json').read_bytes())
            if digest(resources / 'runtime-bundle.manifest.json') != '42539a22dde96977d7575077d3e27e4537fe2794a46c07c565a73cedb7a11a1a' or descriptor['runtimeArchiveSha256'] != 'b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c':
                raise ValueError('exact prior Windows024 stage required')
            if digest(resources / descriptor['archiveName']) != descriptor['archiveSha256'] or digest(resources / 'official-runtime.tgz') != descriptor['runtimeArchiveSha256']:
                raise ValueError('prior staged input bytes changed')
            shutil.copytree(resources, work / 'tauri/resources/runtime-bundle')
            (work / 'stage-result.json').write_text(json.dumps(staged, indent=2))
        staged = json.loads((work / 'stage-result.json').read_bytes())
        if not staged['passed']:
            raise ValueError('real Windows024 stage required')
        project = work / 'tauri'
        (project / 'src').mkdir(exist_ok=True)
        (project / 'src/main.rs').write_text('fn main() {}\n')
        binary_name = 'opencodex-desktop' if args.product_bin_dir else 'ocxd'
        (project / 'Cargo.toml').write_text(f'[package]\nname="{binary_name}"\nversion="0.2.24"\nedition="2021"\n[workspace]\n')
        (project / 'windows').mkdir(exist_ok=True)
        for name in ('installer-hooks.nsh', 'nsis023.nsi', 'wix023.wxs'):
            shutil.copy2(ROOT / 'apps/desktop/tauri/windows' / name, project / 'windows' / name)
        for name in ('LICENSE', 'THIRD-PARTY-NOTICES.txt'):
            shutil.copy2(ROOT / 'apps/desktop/tauri/resources' / name, project / 'resources' / name)
        (project / 'icons').mkdir(exist_ok=True)
        shutil.copy2(ROOT / 'apps/desktop/tauri/icons/windows/icon.ico', project / 'icons/icon.ico')
        executable = project / f'target/release/{binary_name}.exe'
        executable.parent.mkdir(exist_ok=True, parents=True)
        fixture = args.product_bin_dir.resolve() / 'opencodex-desktop.exe' if args.product_bin_dir else Path(__file__).resolve().parent / 'target/debug/native024-fixture.exe'
        shutil.copy2(fixture, executable)
        config = {
            'productName': 'OpenCodeX W1D Proof' if args.product_bin_dir else 'OpenCodeX Desktop', 'version': '0.2.24',
            'identifier': 'com.gzers.opencodex.w1-resource-proof',
            'bundle': {'active': True, 'targets': ['msi', 'nsis'], 'icon': ['icons/icon.ico'],
                       'resources': {'resources/LICENSE': 'resources/LICENSE',
                                     'resources/THIRD-PARTY-NOTICES.txt': 'resources/THIRD-PARTY-NOTICES.txt',
                                     'resources/runtime-bundle/': 'runtime-bundle/'},
                       'windows': {'webviewInstallMode': {'type': 'skip'},
                                   'nsis': {'template': 'windows/nsis023.nsi', 'installerHooks': 'windows/installer-hooks.nsh', 'installMode': 'currentUser', 'compression': 'none'},
                                   'wix': {'template': 'windows/wix023.wxs', 'upgradeCode': '3a2937cc-120b-4f06-a17a-49bf1d77267d'}}}}
        if args.product_bin_dir:
            for name in ('ocxd.exe','ocx.exe'):
                shutil.copy2(args.product_bin_dir.resolve()/name, executable.parent/name)
            provenance={'schemaVersion':1,'package':'opencodex-desktop','version':'0.2.24','entryKind':'installer-native-exe',
                        'files':{name:digest(args.product_bin_dir.resolve()/name) for name in ('ocxd.exe','ocx.exe')}}
            (project/'resources/ocx-native-entry.json').write_text(json.dumps(provenance,indent=2)+'\n')

            report.update(productExecutable=True, scope='actual MSVC dev product/helper template compilation; test bundle identity, not release candidate')
        (project / 'tauri.conf.json').write_text(json.dumps(config, indent=2))
        cli = args.tooling.resolve() / 'node_modules/@tauri-apps/cli/tauri.js'
        command = [str(args.node.resolve()), str(cli), 'bundle', '--ci', '--no-sign', '--bundles', 'msi,nsis', '-vv']
        with (evidence / 'bundle.stdout.log').open('wb') as out, (evidence / 'bundle.stderr.log').open('wb') as err:
            environment = dict(os.environ)
            environment.pop('CARGO_TARGET_DIR', None)
            if args.product_bin_dir:
                environment['RUSTUP_TOOLCHAIN'] = 'stable-x86_64-pc-windows-msvc'
            result = subprocess.run(command, cwd=project, env=environment, stdout=out, stderr=err)
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
