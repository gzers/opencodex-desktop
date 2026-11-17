"""Explicit temporary profile + unmodified official2.99 save/catalog refresh.

Run only after the Rust file-transaction tests pass. Never starts a service.
The kernel denies daily HOME, Keychain and network; all credentials are synthetic.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import pwd
import shutil
import subprocess
import tarfile
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]


def run(output):
    output = output.absolute()
    output.mkdir(parents=True, exist_ok=False)
    root = Path(tempfile.mkdtemp(prefix='ocxd024-name-refresh-', dir='/private/tmp'))
    root.chmod(0o700)
    results = []
    try:
        argv = ['cargo', 'test', '--lib', 'modules::model_config::config_file::v2::tests::export_logical_channel_official_refresh_inputs', '--', '--ignored', '--exact', '--nocapture']
        p = subprocess.run(argv, cwd=ROOT / 'apps/desktop/tauri', env=dict(os.environ, OCXD024_NAME_REFRESH_INPUTS=str(root)), capture_output=True, timeout=120)
        (output / 'export.stdout.log').write_bytes(p.stdout); (output / 'export.stderr.log').write_bytes(p.stderr)
        if p.returncode or len(list(root.glob('phase-*.json'))) != 3:
            raise RuntimeError('explicit Rust transaction export failed')
        for f in root.glob('phase-*.json'):
            shutil.copyfile(f, output / f.name)
        archive_path = ROOT / 'apps/desktop/tauri/resources/runtime-bundle/runtime-bundle-0.2.24-darwin-arm64.tar.gz'
        if hashlib.sha256(archive_path.read_bytes()).hexdigest() != '5b10c1a888000d54d14783622d9d8330c33f4e1b14a5f163d007337d0424ec7f':
            raise ValueError('only the independently verified official Darwin bundle is allowed')
        with tarfile.open(archive_path) as archive:
            for member in archive:
                target = root / member.name
                if '..' in Path(member.name).parts or not target.is_relative_to(root):
                    raise ValueError('unsafe archive member')
                target.parent.mkdir(parents=True, exist_ok=True)
                if member.issym():
                    target.symlink_to(member.linkname)
                else:
                    if not member.isfile(): raise ValueError('unexpected archive member')
                    with archive.extractfile(member) as source, target.open('xb') as destination:
                        shutil.copyfileobj(source, destination)
                    target.chmod(member.mode)
        script = root / 'probe.mjs'
        shutil.copyfile(ROOT / 'test/smoke/official299-display-refresh.mjs', script)
        shutil.copyfile(script, output / 'official-probe.mjs')
        profile = root / 'profile'; profile.mkdir(mode=0o700)
        policy = root / 'probe.sb'
        policy.write_text('(version 1) (allow default) (deny network*) (deny file-read* file-write* (subpath ' + json.dumps(pwd.getpwuid(os.getuid()).pw_dir) + ')) (deny file-read* file-write* (subpath "/Library/Keychains")) (deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.secd"))')
        shutil.copyfile(policy, output / 'probe.sb')
        env = {'HOME': str(profile), 'CFFIXED_USER_HOME': str(profile), 'OPENCODEX_HOME': str(profile / 'opencodex'), 'CODEX_HOME': str(profile / '.codex'), 'PATH': '/usr/bin:/bin', 'TMPDIR': str(root)}
        for phase in range(1, 4):
            report = root / ('refresh-' + str(phase) + '.json')
            argv = ['/usr/bin/sandbox-exec', '-f', str(policy), str(root / 'runtime/bun/bun'), str(script), str(root / 'runtime/opencodex/node_modules/@bitkyc08/opencodex'), str(root / ('phase-' + str(phase) + '.json')), str(report)]
            started = time.monotonic()
            p = subprocess.run(argv, cwd=root, env=env, capture_output=True, timeout=60)
            (output / ('refresh-' + str(phase) + '.stdout.log')).write_bytes(p.stdout)
            (output / ('refresh-' + str(phase) + '.stderr.log')).write_bytes(p.stderr)
            row = {'phase': phase, 'argv': argv, 'exitCode': p.returncode, 'elapsedMs': round((time.monotonic() - started) * 1000)}
            results.append(row); print(json.dumps(row), flush=True)
            if report.exists(): shutil.copyfile(report, output / report.name)
            if p.returncode: raise RuntimeError('official save/refresh failed at phase ' + str(phase))
    finally:
        for name in ('opencodex/config.json', '.codex/config.toml', '.codex/opencodex-catalog.json'):
            f = root / 'profile' / name
            if f.is_file(): shutil.copyfile(f, output / ('final-' + f.name))
        shutil.rmtree(root)
        (output / 'result.json').write_text(json.dumps({'results': results, 'allPhasesPassed': len(results) == 3 and all(row['exitCode'] == 0 for row in results), 'serviceStarted': False, 'dailyConfigurationModified': False, 'temporaryRootAbsent': not root.exists(), 'temporaryRoot': str(root), 'releaseAcceptance': False}, indent=2) + chr(10))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    run(parser.parse_args().output)
