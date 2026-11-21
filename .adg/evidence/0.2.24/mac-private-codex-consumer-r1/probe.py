import hashlib
import importlib.util
import json
import os
from pathlib import Path
import pwd
import queue
import threading
import shutil
import signal
import subprocess
import sys
import tempfile
import time

base = Path(__file__).resolve().parent
repo = Path('/private/tmp/ocxd024-integration-source.sTfP3jjz/code')
inputs = Path("/private/tmp/ocxd024-mac-handoff.MSUgo1UC/docs/.adg/evidence/0.2.24/channel-name-rollback-r1/run01")
codex_cli = Path("/Applications/ChatGPT.app/Contents/Resources/codex-cli/bin/codex")
codex_binary = codex_cli.parent.parent / "CodexCLI.app/Contents/MacOS/codex"
version = Path('/private/tmp/ocxd024-gui-root-cancel-q7_i7k2k/profile/Library/Application Support/OpenCodeX Desktop/runtime/versions/2.99.0')
old_fixture = version.parents[6]
node = version / 'dependencies/node/bin/node'
bun = version / 'dependencies/bun/bun'
entry = version / 'opencodex/node_modules/@bitkyc08/opencodex/bin/ocx.mjs'
recipe = Path('/private/tmp/ocxd024-mac-gui-restore.IedffWBs/restore-logic.py')
exec(compile(recipe.read_bytes(), str(recipe), 'exec'))
spec = importlib.util.spec_from_file_location('helpers', repo / 'test/smoke/macos024-root-cancel.py')
helpers = importlib.util.module_from_spec(spec)
spec.loader.exec_module(helpers)

def write(target, data):
    target.write_text(json.dumps(data, indent=2, ensure_ascii=False) + chr(10))

def command(args):
    return subprocess.run(args, check=True, capture_output=True, timeout=6)

def rows():
    return subprocess.check_output(['/bin/ps', '-axo', 'pid=,pgid=,command='], text=True).splitlines()

def list_models(sandbox, env, root, output, phase, expected):
    incoming = queue.Queue()
    codex_env = {**env, 'CODEX_ELECTRON_USER_DATA_PATH':str(root/'electron-data')}
    with (output/f'codex-{phase}.stderr.log').open('xb') as error_log:
        process = subprocess.Popen([*sandbox, str(codex_cli), 'app-server', '--listen', 'stdio://'], env=codex_env, cwd=root, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=error_log)
        def read():
            for line in process.stdout:
                incoming.put(line)
            incoming.put(None)
        reader = threading.Thread(target=read, daemon=True)
        reader.start()
        result = {'pid': process.pid}
        def send(payload):
            process.stdin.write((json.dumps(payload)+'\n').encode())
            process.stdin.flush()
        def receive(request_id):
            deadline = time.monotonic() + 8
            while True:
                line = incoming.get(timeout=max(0.01, deadline-time.monotonic()))
                if line is None:
                    raise RuntimeError(f'app-server ended before reply {request_id}')
                response = json.loads(line)
                if response.get('id') == request_id:
                    if 'error' in response:
                        raise RuntimeError(str(response['error']))
                    return response['result']
                assert time.monotonic() < deadline
        try:
            send({'id':1,'method':'initialize','params':{'clientInfo':{'name':'ocxd_fixture','title':'Isolated OpenCodex fixture','version':'0.2.24'}}})
            initialized = receive(1)
            result['reportedHome'] = initialized.get('codexHome')
            assert Path(result['reportedHome']) == Path(env['CODEX_HOME']), 'unexpected app-server HOME'
            send({'method':'initialized'})
            cursor = None
            discovered = []
            while True:
                request_id = 2 + len(discovered)
                send({'id':request_id,'method':'model/list','params':{'limit':100,'cursor':cursor}})
                reply = receive(request_id)
                discovered.extend(reply['data'])
                cursor = reply.get('nextCursor')
                if not cursor:
                    break
                assert 0 < len(discovered) < 500
            rows = [row for row in discovered if row.get('id') in expected or row.get('model') in expected]
            result['matches'] = rows
            result['models'] = {row.get('model') if row.get('model') in expected else row['id']: row['displayName'] for row in rows}
            assert len(rows) == len(result['models']), 'duplicated routed model in app-server response'
        finally:
            process.stdin.close()
            try:
                process.wait(timeout=4)
            except subprocess.TimeoutExpired:
                process.terminate()
                process.wait(timeout=2)
            reader.join(timeout=1)
            result['exit'] = process.returncode
            write(output/f'codex-{phase}-transport.json', result)
        assert process.returncode == 0
        return result

def run(label):
    output = base / label
    output.mkdir(mode=0o700)
    root = Path(tempfile.mkdtemp(prefix='ocxd024-cli-fixture-', dir='/private/tmp'))
    profile = root / 'profile'
    home = profile / '.opencodex'
    tmp = root / 'tmp'
    home.mkdir(mode=0o700, parents=True)
    tmp.mkdir(mode=0o700)
    redirected = root / 'private-clients'
    redirected.mkdir(mode=0o700)
    for client_root in (profile, redirected):
        seed_clients(client_root, command)
        command(['/usr/bin/xattr', '-w', 'org.opencodex.fixture', 'grok-original', client_root/'.grok/config.toml'])
    if label == 'backup-absent':
        (redirected / '.grok/config.toml.bak-opencodex').unlink()
    config = home / 'config.json'
    phases = [json.loads((inputs/f'phase-{i}.json').read_text()) for i in (1,2,3)]
    for phase in phases:
        phase.update(hostname='127.0.0.1', port=0)
        assert all(provider['apiKey'] == 'synthetic-only-never-return' and not provider['liveModels'] for provider in phase['providers'].values())
    write(config, phases[0])
    config.chmod(0o600)
    coordinator_paths = helpers.coordinators(redirected) + helpers.coordinators(profile)
    assert not any(os.path.lexists(str(p) + suffix) for p in coordinator_paths for suffix in ('', '-journal', '-wal', '-shm'))
    account = pwd.getpwuid(os.getuid())
    policy = root / 'fixture.sb'
    policy.write_text('(version 1) (allow default) (deny network*) (allow network-bind (local ip "localhost:*")) (allow network-inbound (local ip "localhost:*")) (allow network-outbound (remote ip "localhost:*")) (deny file-read* file-write* (subpath ' + json.dumps(account.pw_dir) + ')) (deny file-write* (subpath ' + json.dumps(str(old_fixture)) + ')) (deny file-read* file-write* (subpath "/Library/Keychains")) (deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.secd"))')
    with policy.open('a') as handle:
        handle.write(' (deny file-write* (subpath "/Applications/ChatGPT.app"))')
        for client in (profile / '.codex', profile / '.grok'):
            handle.write(' (deny file-read* file-write* (subpath ' + json.dumps(str(client)) + '))')
    shutil.copyfile(policy, output/'fixture.sb')
    env = {'HOME': str(profile), 'CFFIXED_USER_HOME': str(profile), 'OPENCODEX_HOME': str(home), 'CODEX_HOME': str(redirected/'.codex'), 'GROK_HOME': str(redirected/'.grok'), 'TMPDIR': str(tmp)+'/', 'USER': account.pw_name, 'LOGNAME': account.pw_name, 'PATH': '/usr/bin:/bin:/usr/sbin:/sbin', 'OPENCODEX_BUN_PATH': str(bun), 'LANG': 'en_US.UTF-8'}
    sandbox = ['/usr/bin/sandbox-exec', '-f', str(policy)]
    argv = [*sandbox, str(node), str(entry)]
    guards = {}
    probes = {'externalDial': ['/usr/bin/nc', '-v', '-G', '1', '-z', '192.0.2.1', '443']}
    for client_name in ('.codex', '.grok'):
        config_path = profile / client_name / 'config.toml'
        probes[client_name + '-read'] = ['/bin/cat', str(config_path)]
        probes[client_name + '-write'] = ['/bin/sh', '-c', 'printf guard >> "$1"', 'probe', str(config_path)]
        probes[client_name + '-private-read'] = ['/bin/cat', str(redirected / client_name / 'config.toml')]
    for name, args in probes.items():
        result = subprocess.run([*sandbox, *args], env=env, cwd=root, capture_output=True, timeout=4)
        guards[name] = {'exit':result.returncode,'stderr':result.stderr.decode()}
    write(output/'guards.json', guards)
    for name, guard in guards.items():
        if name.endswith('-private-read'):
            assert guard['exit'] == 0 and not guard['stderr'], name
        else:
            assert guard['exit'] and 'Operation not permitted' in guard['stderr'], name
    def snapshot(phase):
        write(output/(phase+'.json'), {
            label: {key: capture_tree(parent/key, command) for key in ('.codex','.grok')}
            for label, parent in [('default', profile), ('redirected', redirected)]
        })
    refresh = root / 'official-probe.mjs'
    refresh.write_text((inputs/'official-probe.mjs').read_text().replace("assert(profile.startsWith('/private/tmp/ocxd024-name-refresh-')||profile.startsWith('/private/tmp/ocxd024-name-rollback-'))", "assert(profile.startsWith('/private/tmp/ocxd024-cli-fixture-'))").replace("path.join(profile,'opencodex')", "path.join(profile,'.opencodex')").replace("path.join(profile,'.codex')", "path.join(path.dirname(profile),'private-clients','.codex')"))
    snapshot('original')
    info = {'fixture':str(root),'cliDiagnosticOnly':True,'accepted':False}
    ports = set()
    start = None
    raised = None
    try:
        with (output/'start.log').open('xb') as log:
            started = time.monotonic()
            start = subprocess.Popen([*argv,'start'], cwd=root, env=env, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
            info['launcherPid'] = start.pid
            while 'Client startup work complete.' not in (output/'start.log').read_text(errors='replace'):
                if start.poll() is not None or time.monotonic()-started >= 30:
                    raise RuntimeError('official startup completion absent')
                time.sleep(.05)
            info['clientStartupMs'] = round((time.monotonic()-started)*1000,3)
            snapshot('ready')
            group = [row.split(None, 2) for row in rows() if len(row.split(None, 2))==3 and row.split(None, 2)[1]==str(start.pid)]
            info['startGroup'] = group
            assert group
            for pid, *_ in group:
                listener = subprocess.run(['/usr/sbin/lsof','-nP','-a','-p',pid,'-iTCP','-sTCP:LISTEN','-Fn'],capture_output=True,text=True,timeout=3)
                ports.update(int(line.rsplit(':',1)[-1]) for line in listener.stdout.splitlines() if line.startswith('n') and line.rsplit(':',1)[-1].isdigit())
            info['ports'] = sorted(ports)
            assert ports
            try:
                for number, (phase, name) in enumerate(zip(phases, ['备份渠道','修改后渠道','备份渠道']), start=1):
                    fixture = root/f'phase-{number}.json'
                    write(fixture, phase)
                    with (output/f'refresh-{number}.log').open('xb') as refresh_log:
                        sync = subprocess.run([*sandbox, str(bun), str(refresh), str(entry.parents[1]), str(fixture), str(output/f'refresh-{number}.json')], env=env, cwd=root, stdin=subprocess.DEVNULL, stdout=refresh_log, stderr=subprocess.STDOUT, timeout=10)
                    assert sync.returncode == 0, f'refresh {number} exit {sync.returncode}'
                    evidence = json.loads((output/f'refresh-{number}.json').read_text())
                    expected = {row['slug']: row['displayName'] for row in evidence['results'][0]['routed']}
                    assert set(expected.values()) == {name+' · vendor／custom', '人工名称'}
                    result = list_models(sandbox, env, root, output, number, expected)
                    write(output/f'codex-{number}.json', result)
                    assert result['models'] == expected, f'app-server phase {number} returned different model names'
                info['privateModelListVerified'] = True
            except Exception as exc:
                info['modelListFirstError'] = repr(exc)
            snapshot('pre-stop')
            before_stop = time.monotonic()
            with (output/'stop.log').open('xb') as stop_log:
                stop = subprocess.run([*argv,'stop'], cwd=root, env=env, stdin=subprocess.DEVNULL, stdout=stop_log, stderr=subprocess.STDOUT, timeout=10)
            start.wait(timeout=max(.01,10-(time.monotonic()-before_stop)))
            info.update(stopMs=round((time.monotonic()-before_stop)*1000,3), stopExit=stop.returncode, launcherExit=start.returncode)
            snapshot('stopped')
            assert stop.returncode == start.returncode == 0
            phases = {phase: json.loads((output/(phase+'.json')).read_text()) for phase in ('original','ready','pre-stop','stopped')}
            baseline = phases['original']
            assert all(phase['default'] == baseline['default'] for phase in phases.values())
            for client_name in ('.codex', '.grok'):
                original, ready, stopped = (phases[name]['redirected'][client_name]['config.toml'] for name in ('original','ready','stopped'))
                assert ready['sha256'] != original['sha256'] and ready['inode'] != original['inode'], client_name
                assert stopped['sha256'] == original['sha256'], client_name
            info.update(defaultTreesIdentical=True, redirectedConfigsChangedAndRestored=True)
    except Exception as exc:
        raised=exc
        info['firstError']=str(exc)
    finally:
        residues = [row.strip() for row in rows() if str(root) in row or (start and len(row.split(None,2))==3 and row.split(None,2)[1]==str(start.pid))]
        listener_checks = {port:subprocess.run(['/usr/sbin/lsof','-nP','-iTCP:'+str(port),'-sTCP:LISTEN'],capture_output=True,text=True,timeout=3) for port in sorted(ports)}
        info.update(residues=residues,listeners={port:{'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr} for port,p in listener_checks.items()})
        if raised is None and not residues and all(p.returncode == 1 and not p.stderr for p in listener_checks.values()):
            helpers.clean_coordinators(coordinator_paths,output)
            shutil.rmtree(root)
        info['fixtureAbsent'] = not root.exists()
        write(output/'run.json',info)
    if raised is not None:
        raise raised
    assert info['fixtureAbsent']
    assert info.get('privateModelListVerified'), info.get('modelListFirstError')

write(base/'inputs.json',{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in (node,bun,entry,recipe,codex_cli,codex_binary,Path(__file__))})
for case in ('backup-existing',):
    run(case)
