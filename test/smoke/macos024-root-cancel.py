"""Real official coordinator tests in a fresh guarded profile.

Storage and current-service scopes are explicit. No official service is started.
Refuses existing outputs, retains evidence and removes only its owned fixture.
"""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import pwd
import shutil
import signal
import sqlite3
import stat
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


def coordinators(profile):
    client = str(profile / '.codex'); state = client + '/state_5.sqlite'
    scope = hashlib.sha256(client.encode()).hexdigest()
    history = hashlib.sha256((str(len(client))+':'+client+str(len(state))+':'+state).encode()).hexdigest()
    namespace = Path('/private/tmp') / ('opencodex-runtime-v1-' + str(os.getuid()))
    return [namespace / role / (key + '.sqlite') for role,key in
            [('native-write-locks',scope),('catalog-write-locks',scope),('history-write-locks',history)]]


def clean_coordinators(paths, output):
    """Exact fresh task-keyed databases only, never shared lifecycle/global L."""
    removed = []
    for path in paths:
        for suffix in ('-journal','-wal','-shm'):
            if os.path.lexists(str(path)+suffix): raise RuntimeError('pending task coordinator sidecar; retained')
        if not os.path.lexists(path): continue
        before = path.lstat()
        if not stat.S_ISREG(before.st_mode) or before.st_uid!=os.getuid() or before.st_nlink!=1 or stat.S_IMODE(before.st_mode)!=0o600:
            raise RuntimeError('task coordinator identity/protection unconfirmed; retained')
        connection = sqlite3.connect(path.as_uri()+'?mode=rw',uri=True,timeout=0)
        try: connection.execute('BEGIN EXCLUSIVE'); connection.execute('ROLLBACK')
        finally: connection.close()
        raw = path.read_bytes(); after = path.lstat()
        if (before.st_dev,before.st_ino,before.st_mode,before.st_uid,before.st_gid)!=(after.st_dev,after.st_ino,after.st_mode,after.st_uid,after.st_gid):
            raise RuntimeError('task coordinator changed during release probe; retained')
        removed.append({'path':str(path),'device':after.st_dev,'inode':after.st_ino,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'rawBase64':base64.b64encode(raw).decode()})
        write(output/'owned-coordinator-bytes.json',removed)
        if path.lstat().st_ino!=after.st_ino or path.read_bytes()!=raw: raise RuntimeError('task coordinator changed before cleanup')
        path.unlink()
    if any(os.path.lexists(path) for path in paths): raise RuntimeError('task coordinators remain')
    write(output/'owned-coordinator-cleanup.json',{'removed':len(removed),'exactPathsAbsent':[str(p) for p in paths], 'preexistingGlobalLifecycleUntouched':True})


def run(output, scenario='success', shared_program=None, scope='storage', partial_point=None):
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
    ordinary_predecessor = scope == 'ordinary-successor-action'
    if not ordinary_predecessor:
        (root / 'profile').mkdir(mode=0o700)
    (root / 'resources').mkdir(mode=0o700)
    task_coordinators = coordinators(root / 'profile')
    if any(os.path.lexists(str(path)+suffix) for path in task_coordinators for suffix in ('','-journal','-wal','-shm')):
        raise RuntimeError('fresh fixture task coordination path already exists; not adopted')
    write(output/'owned-coordinator-before.json',{'exactPathsAbsent':[str(p) for p in task_coordinators]})
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
        if scope not in ('storage', 'current-service', 'user-action', 'partial-capture', 'successor-review', 'successor-action', 'ordinary-successor-action'): raise ValueError('unknown root cancellation test scope')
        argv = ['/usr/bin/sandbox-exec', '-f', str(policy), str(host), '--exact', test, '--ignored', '--nocapture', '--test-threads=1']
        env = {'HOME': str(root / 'profile'), 'CFFIXED_USER_HOME': str(root / 'profile'), 'PATH': '/usr/bin:/bin', 'TMPDIR': str(parent), 'OCXD024_ROOT_CANCEL_NATIVE': str(root)}
        env['OCXD024_ROOT_CANCEL_CASE'] = scenario
        if scope == 'successor-review':
            if scenario != 'success': raise ValueError('successor review requires the exact successful predecessor setup')
            env['OCXD024_SUCCESSOR_CANCEL_REVIEW_ONLY'] = '1'
        if scope in ('successor-action', 'ordinary-successor-action'):
            if scenario != 'success': raise ValueError('successor action requires a successfully closed predecessor')
            env['OCXD024_SUCCESSOR_CANCEL_ACTION'] = '1'
            if ordinary_predecessor:
                env['OCXD024_SUCCESSOR_ORDINARY_PREDECESSOR'] = '1'
        if scope in ('user-action', 'partial-capture', 'successor-action'): env['OCXD024_ROOT_CANCEL_USER_ACTION'] = '1'
        if scope == 'partial-capture':
            if partial_point not in ('directory_created','codex_archived','grok_archived','original_published'): raise ValueError('explicit capture boundary required')
            env['OCXD024_ROOT_CANCEL_PARTIAL_POINT'] = partial_point
        if shared_program is not None: env['OCXD024_ROOT_CANCEL_REUSE'] = str(shared_program)
        started = time.monotonic()
        with (output / 'stdout.log').open('xb') as stdout, (output / 'stderr.log').open('xb') as stderr:
            proc = subprocess.Popen(argv, cwd=root, env=env, stdout=stdout, stderr=stderr, start_new_session=True)
            proc.wait(timeout=180)
        result = {'argv': argv, 'pid': proc.pid, 'exitCode': proc.returncode, 'elapsedMs': round((time.monotonic() - started) * 1000),
                  'source': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), 'scenario': scenario, 'scope': scope,
                  'trackedDiffSha256': hashlib.sha256(subprocess.check_output(['git', 'diff', '--binary', 'HEAD'], cwd=ROOT)).hexdigest(),
                  'hostSha256': host_hash, 'hostUnchanged': sha(host) == host_hash, 'releaseAcceptance': False, 'serviceStarted': False}
        print(json.dumps(result), flush=True)
        print((output / 'stdout.log').read_text()[-2000:], flush=True)
        print((output / 'stderr.log').read_text()[-2000:], flush=True)
    finally:
        if proc and proc.poll() is None and scope != 'successor-review':
            os.killpg(proc.pid, signal.SIGTERM)
            try: proc.wait(timeout=5)
            except subprocess.TimeoutExpired: pass
        residues = owned_group(proc.pid) if proc else []
        for name in ('native-preservation-result.json', 'native-preservation-error.json'):
            if (root / name).exists(): shutil.copyfile(root / name, output / name)
        data = root / ('desktop' if ordinary_predecessor else 'profile/desktop')
        records = data / 'manager-state'
        if records.exists():
            for path in records.rglob('*'):
                if path.is_file() and (path.suffix == '.json' or path.name == 'client-session-index.sqlite'):
                    target = output / 'raw-metadata' / path.relative_to(records)
                    target.parent.mkdir(parents=True, exist_ok=True); shutil.copyfile(path, target)
        if scope == 'partial-capture':
            vault = records / 'client-originals'
            rows, total = [], 0
            if vault.exists():
                for path in sorted(vault.rglob('*')):
                    if path.is_symlink(): raise RuntimeError('partial fixture archive refuses unexpected link')
                    if not path.is_file(): continue
                    raw = path.read_bytes(); total += len(raw)
                    if total > 16 * 1024 * 1024: raise RuntimeError('synthetic partial fixture evidence exceeds budget')
                    info = path.stat()
                    rows.append({'path':path.relative_to(records).as_posix(),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),
                                 'device':info.st_dev,'inode':info.st_ino,'mode':info.st_mode & 0o777,'rawBase64':base64.b64encode(raw).decode()})
            write(output / 'synthetic-partial-capture-bytes.json', {'scope':'task synthetic Grok/Original blobs only; no real service or business credentials','files':rows})
        inventory = [{'path': p.relative_to(root).as_posix(), 'bytes': p.stat().st_size, 'sha256': sha(p)} for p in root.rglob('*')
                     if p.is_file() and not p.is_symlink() and not p.is_relative_to(root / 'program') and not p.is_relative_to(root / 'resources')]
        write(output / 'fixture-content-inventory.json', inventory)
        write(output / 'result.json', result or {'failedBeforeRun': True, 'releaseAcceptance': False})
        retain_successor = scope in ('successor-review', 'successor-action', 'ordinary-successor-action') and (result is None or result.get('exitCode') != 0 or residues)
        if not residues and not retain_successor:
            clean_coordinators(task_coordinators,output)
            shutil.rmtree(parent)
        write(output / 'cleanup.json', {'ownedRoot': str(parent), 'absent': not parent.exists(), 'ownedGroupResidues': residues, 'serviceStarted': False,
                                       'scope': 'new synthetic profile/program/copied test host only; evidence retained',
                                       'failedSuccessorSceneRetained': bool(retain_successor)})
    if result is None or result['exitCode'] or residues: raise RuntimeError('official guarded preservation test or cleanup failed')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--case', choices=['success', 'before_move', 'after_move', 'new_data', 'new_root'], default='success')
    parser.add_argument('--scope', choices=['storage', 'current-service', 'user-action', 'partial-capture', 'successor-review', 'successor-action', 'ordinary-successor-action'], default='storage')
    parser.add_argument('--partial-point', choices=['directory_created','codex_archived','grok_archived','original_published'])
    args = parser.parse_args()
    run(args.output, args.case, scope=args.scope, partial_point=args.partial_point)
