import hashlib
import json
import os
from pathlib import Path
import select
import shutil
import subprocess
import sys
import tempfile
import uuid

out = Path(__file__).resolve().parent
repo = Path('/private/tmp/ocxd024-integration-source.sTfP3jjz/code')
helper = repo / 'apps/desktop/tauri/src/infrastructure/client_config_guard299.mjs'
version = Path('/private/tmp/ocxd024-gui-root-cancel-q7_i7k2k/profile/Library/Application Support/OpenCodeX Desktop/runtime/versions/2.99.0')
package = version / 'opencodex/node_modules/@bitkyc08/opencodex'
bun = version / 'dependencies/bun/bun'
fixture = Path(tempfile.mkdtemp(prefix='ocxd024-config-lock-fixture-', dir='/private/tmp'))
profile = fixture / 'profile'
home, codex, grok = [profile / name for name in ('.opencodex', '.codex', '.grok')]
for path in (home, codex, grok):
    path.mkdir(mode=0o700, parents=True)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def file_info(path):
    return {'inode': path.stat().st_ino, 'sha256': sha(path.read_bytes()),
            'syntheticXattr': 'org.opencodex.fixture' in os.listxattr(path)}


def argv(target, operation):
    params = dict(schema='ocxd-config-write-guard/2', operation=operation, packageRoot=str(package),
                  profile=str(profile), home=str(home), codex=str(codex), target=str(target))
    return [str(bun), '--eval', helper.read_text(), '--', json.dumps(params)]


env = dict(HOME=str(profile), CODEX_HOME=str(codex), GROK_HOME=str(grok),
           OPENCODEX_HOME=str(home), PATH='/usr/bin:/bin', LANG='C.UTF-8')
report = {'accepted': False, 'diagnosticOnly': True, 'inputs': {str(p): sha(p.read_bytes()) for p in
          (helper, bun, package/'src/codex/config-write-lock.ts', package/'src/codex/prompt-lock.ts')}, 'cases': {}}


def run_case(label, target):
    config = target / 'config.toml'
    config.write_text('fixture = true\n')
    config.chmod(0o600)
    os.setxattr(config, 'org.opencodex.fixture', b'original')
    operation = str(uuid.uuid4())
    result = report['cases'][label] = {'before': file_info(config)}
    with subprocess.Popen(argv(target, operation), env=env, cwd=fixture, text=True,
                          stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE) as holder:
        try:
            assert select.select([holder.stdout], [], [], 5)[0], 'holder did not respond'
            held = result['held'] = json.loads(holder.stdout.readline())
            assert held['kind'] == 'held' and held['pid'] == holder.pid and held['operation'] == operation
            lock = target / 'config.toml.ocx-write.lock'
            lock_before = file_info(lock)
            contender = subprocess.run(argv(target, str(uuid.uuid4())), env=env, cwd=fixture,
                                       capture_output=True, text=True, input='', timeout=5)
            result['cooperatingContender'] = {'exit': contender.returncode, 'event': json.loads(contender.stdout)}
            assert contender.returncode == 1 and result['cooperatingContender']['event']['phase'] == 'acquire'
            writer_source = 'from pathlib import Path; import sys; p=Path(sys.argv[1]); temp=p.with_suffix(".fixture-stage"); temp.write_bytes(p.read_bytes()); temp.chmod(0o600); temp.replace(p)'
            writer = subprocess.run([sys.executable, '-I', '-c', writer_source, str(config)], env=env,
                                    cwd=fixture, check=True, timeout=5)
            result['independentWriterExit'] = writer.returncode
            result['after'] = file_info(config)
            result['sameLockAfterReplace'] = file_info(lock) == lock_before
            assert result['before']['sha256'] == result['after']['sha256']
            assert result['before']['inode'] != result['after']['inode']
            assert result['before']['syntheticXattr'] and not result['after']['syntheticXattr']
            assert result['sameLockAfterReplace'] and holder.poll() is None
            stdout, stderr = holder.communicate(json.dumps(dict(kind='release', operation=operation))+'\n', timeout=5)
            result.update(release=json.loads(stdout), exit=holder.returncode, lockAbsentAfterRelease=not lock.exists())
            assert not stderr and result['exit'] == 0 and result['release']['kind'] == 'released'
            assert result['lockAbsentAfterRelease']
        finally:
            if holder.poll() is None:
                holder.kill()
                holder.wait(timeout=5)


try:
    for label, target in (('codex', codex), ('grok', grok)):
        run_case(label, target)
except Exception as error:
    report['firstError'] = repr(error)
    raise
finally:
    shutil.rmtree(fixture)
    report['fixtureAbsent'] = not fixture.exists()
    (out / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
