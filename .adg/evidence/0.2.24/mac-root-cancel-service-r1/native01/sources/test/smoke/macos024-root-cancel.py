"""Real official coordinator + same-root storage test in a fresh guarded profile.

No service or production cancellation permission is constructed. Refuses existing
outputs, keeps diagnostic metadata/streams, removes only its copied fixture.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import pwd
import shutil
import signal
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]
TEST = 'infrastructure::macos_client_originals::session_index::root_preparation::preservation::tests::official_guard_preserves_exact_uncaptured_root_and_releases_only_its_marker'


def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''): h.update(block)
    return h.hexdigest()


def write(path, value):
    path.write_text(json.dumps(value, indent=2) + chr(10))


def owned_group(pid):
    rows = subprocess.check_output(['/bin/ps', '-axo', 'pid=,ppid=,pgid=,comm='], text=True)
    return [{'pid': int(p[0]), 'parentPid': int(p[1]), 'groupId': int(p[2]), 'image': p[3]}
            for row in rows.splitlines() if len(p := row.split(None, 3)) == 4 and int(p[2]) == pid]


def run(output, scenario='success', shared_program=None, scope='storage'):
    output = output.absolute(); output.mkdir(parents=True, exist_ok=False)
    argv = ['cargo', 'test', '--lib', '--no-run', '--message-format=json']
    build = subprocess.run(argv, cwd=ROOT / 'apps/desktop/tauri', capture_output=True)
    (output / 'build.stdout.jsonl').write_bytes(build.stdout)
    (output / 'build.stderr.log').write_bytes(build.stderr)
    if build.returncode: raise RuntimeError('same-source native test host build failed')
    artifacts = [json.loads(line)['executable'] for line in build.stdout.decode().splitlines()
                 if line.startswith('{') and json.loads(line).get('reason') == 'compiler-artifact' and json.loads(line).get('executable')]
    if len(artifacts) != 1: raise ValueError('one actual test-host artifact required')
    parent = Path(tempfile.mkdtemp(prefix='ocxd024-root-cancel-', dir='/private/tmp')); parent.chmod(0o700)
    root = parent / 'fixture'; root.mkdir(mode=0o700)
    (root / 'profile').mkdir(mode=0o700); (root / 'resources').mkdir(mode=0o700)
    result = None
    proc = None
    try:
        host = parent / 'test-host'; shutil.copyfile(artifacts[0], host); host.chmod(0o700)
        host_hash = sha(host)
        subprocess.run(['/usr/bin/codesign', '--verify', '--strict', host], check=True, capture_output=True)
        for name in ('runtime-bundle.manifest.json', 'runtime-bundle-0.2.24-darwin-arm64.tar.gz'):
            shutil.copyfile(ROOT / 'apps/desktop/tauri/resources/runtime-bundle' / name, root / 'resources' / name)
        policy = parent / 'native.sb'
        policy.write_text('(version 1) (allow default) (deny network*) (deny file-read* file-write* (subpath ' + json.dumps(pwd.getpwuid(os.getuid()).pw_dir) + ')) (deny file-read* file-write* (subpath "/Library/Keychains")) (deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.secd"))')
        shutil.copyfile(policy, output / 'native.sb')
        test = TEST if scope == 'storage' else 'infrastructure::macos_client_originals::session_index::root_preparation::preservation::service_tests::current_service_movement_and_native_release_bind_a_distinct_cancellation_terminal'
        if scope not in ('storage', 'current-service'): raise ValueError('unknown root cancellation test scope')
        argv = ['/usr/bin/sandbox-exec', '-f', str(policy), str(host), '--exact', test, '--ignored', '--nocapture', '--test-threads=1']
        env = {'HOME': str(root / 'profile'), 'CFFIXED_USER_HOME': str(root / 'profile'), 'PATH': '/usr/bin:/bin', 'TMPDIR': str(parent), 'OCXD024_ROOT_CANCEL_NATIVE': str(root)}
        env['OCXD024_ROOT_CANCEL_CASE'] = scenario
        if shared_program is not None: env['OCXD024_ROOT_CANCEL_REUSE'] = str(shared_program)
        started = time.monotonic()
        with (output / 'stdout.log').open('xb') as stdout, (output / 'stderr.log').open('xb') as stderr:
            proc = subprocess.Popen(argv, cwd=root, env=env, stdout=stdout, stderr=stderr, start_new_session=True)
            proc.wait(timeout=180)
        result = {'argv': argv, 'pid': proc.pid, 'exitCode': proc.returncode, 'elapsedMs': round((time.monotonic() - started) * 1000),
                  'source': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), 'scenario': scenario, 'scope': scope,
                  'hostSha256': host_hash, 'hostUnchanged': sha(host) == host_hash, 'releaseAcceptance': False, 'serviceStarted': False}
        print(json.dumps(result), flush=True)
        print((output / 'stdout.log').read_text()[-2000:], flush=True)
        print((output / 'stderr.log').read_text()[-2000:], flush=True)
    finally:
        if proc and proc.poll() is None:
            os.killpg(proc.pid, signal.SIGTERM)
            try: proc.wait(timeout=5)
            except subprocess.TimeoutExpired: pass
        residues = owned_group(proc.pid) if proc else []
        for name in ('native-preservation-result.json', 'native-preservation-error.json'):
            if (root / name).exists(): shutil.copyfile(root / name, output / name)
        records = root / 'profile/desktop/manager-state'
        if records.exists():
            for path in records.rglob('*'):
                if path.is_file() and (path.suffix == '.json' or path.name == 'client-session-index.sqlite'):
                    target = output / 'raw-metadata' / path.relative_to(records)
                    target.parent.mkdir(parents=True, exist_ok=True); shutil.copyfile(path, target)
        inventory = [{'path': p.relative_to(root).as_posix(), 'bytes': p.stat().st_size, 'sha256': sha(p)} for p in root.rglob('*')
                     if p.is_file() and not p.is_symlink() and not p.is_relative_to(root / 'program') and not p.is_relative_to(root / 'resources')]
        write(output / 'fixture-content-inventory.json', inventory)
        write(output / 'result.json', result or {'failedBeforeRun': True, 'releaseAcceptance': False})
        if not residues: shutil.rmtree(parent)
        write(output / 'cleanup.json', {'ownedRoot': str(parent), 'absent': not parent.exists(), 'ownedGroupResidues': residues, 'serviceStarted': False,
                                       'scope': 'new synthetic profile/program/copied test host only; evidence retained'})
    if result is None or result['exitCode'] or residues: raise RuntimeError('official guarded preservation test or cleanup failed')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--case', choices=['success', 'before_move', 'after_move', 'new_data', 'new_root'], default='success')
    parser.add_argument('--scope', choices=['storage', 'current-service'], default='storage')
    args = parser.parse_args()
    run(args.output, args.case, scope=args.scope)
