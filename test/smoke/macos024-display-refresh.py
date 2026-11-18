"""Explicit temporary profile + unmodified official2.99 save/catalog refresh.

Run only after the Rust file-transaction tests pass. Never starts a service.
The kernel denies daily HOME, Keychain and network; all credentials are synthetic.
"""
import argparse
import hashlib
import importlib.util
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
_coord_spec = importlib.util.spec_from_file_location('name_coord024', Path(__file__).with_name('macos024-root-cancel.py'))
coord = importlib.util.module_from_spec(_coord_spec)
_coord_spec.loader.exec_module(coord)


def run(output, backup_rollback=False):
    output = output.absolute()
    output.mkdir(parents=True, exist_ok=False)
    root = Path(tempfile.mkdtemp(prefix='ocxd024-name-rollback-' if backup_rollback else 'ocxd024-name-refresh-', dir='/private/tmp'))
    root.chmod(0o700)
    results = []
    verified = False
    cleanup_error = None
    task_coordinators = coord.coordinators(root / 'profile')
    try:
        before = {str(path) + suffix: os.path.lexists(str(path) + suffix)
                  for path in task_coordinators for suffix in ('', '-journal', '-wal', '-shm')}
        (output / 'coordinators-before.json').write_text(json.dumps(before, indent=2) + chr(10))
        if any(before.values()):
            raise RuntimeError('new test profile coordination identity already exists; not adopted')
        function = 'export_logical_channel_backup_rollback_inputs' if backup_rollback else 'export_logical_channel_official_refresh_inputs'
        variable = 'OCXD024_NAME_ROLLBACK_INPUTS' if backup_rollback else 'OCXD024_NAME_REFRESH_INPUTS'
        argv = ['cargo', 'test', '--lib', 'modules::model_config::config_file::v2::tests::' + function, '--', '--ignored', '--exact', '--nocapture']
        p = subprocess.run(argv, cwd=ROOT / 'apps/desktop/tauri', env=dict(os.environ, **{variable: str(root)}), capture_output=True, timeout=120)
        (output / 'export.stdout.log').write_bytes(p.stdout); (output / 'export.stderr.log').write_bytes(p.stderr)
        if p.returncode or len(list(root.glob('phase-*.json'))) != 3:
            raise RuntimeError('explicit Rust transaction export failed')
        for f in root.glob('phase-*.json'):
            shutil.copyfile(f, output / f.name)
        if backup_rollback:
            shutil.copyfile(root / 'rollback-result.json', output / 'native-rollback-result.json')
            if (root / 'phase-1.json').read_bytes() != (root / 'phase-3.json').read_bytes() or (root / 'phase-1.json').read_bytes() == (root / 'phase-2.json').read_bytes():
                raise RuntimeError('native backup/change/rollback must restore exact original bytes')
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
        if backup_rollback:
            before = json.loads((output / 'refresh-1.json').read_bytes())['results']
            after = json.loads((output / 'refresh-3.json').read_bytes())['results']
            if [r['routed'] for r in before] != [r['routed'] for r in after]:
                raise RuntimeError('official catalog routes/display/capabilities changed after rollback')
        verified = True
    finally:
        for name in ('opencodex/config.json', '.codex/config.toml', '.codex/opencodex-catalog.json'):
            f = root / 'profile' / name
            if f.is_file(): shutil.copyfile(f, output / ('final-' + f.name))
        passed = verified and len(results) == 3 and all(row['exitCode'] == 0 for row in results)
        if passed:
            try:
                coord.clean_coordinators(task_coordinators, output)
                shutil.rmtree(root)
            except Exception as error:
                cleanup_error = repr(error)
                passed = False
        (output / 'result.json').write_text(json.dumps({'results': results, 'allPhasesPassed': passed, 'backupRollback': backup_rollback, 'clientGuiVerified': False, 'serviceStarted': False, 'dailyConfigurationModified': False, 'temporaryRootAbsent': not root.exists(), 'temporaryRoot': str(root), 'cleanupError': cleanup_error, 'releaseAcceptance': False}, indent=2) + chr(10))
    if cleanup_error:
        raise RuntimeError('task coordination cleanup incomplete; fixture retained: ' + cleanup_error)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--backup-rollback', action='store_true')
    args = parser.parse_args()
    run(args.output, args.backup_rollback)
