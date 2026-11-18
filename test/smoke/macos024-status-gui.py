#!/usr/bin/env python3
"""One packaged status IPC observation on a real reserved partial root.

Two GUI launches are setup/sampling, never lifecycle actions. No cancellation,
Start/Stop, model calls, business sync or daily HOME access. Failures retain
the exact GUI handle and fixture for directed inspection.
"""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import pwd
import shutil
import signal
import sqlite3
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


base = load('status_supply', 'macos024-first-supply.py')
coord = load('status_coord', 'macos024-root-cancel.py')


def run(args):
    app = args.app.resolve(strict=True)
    binary = app / 'Contents/MacOS/opencodex-desktop'
    assert app.name == base.APP and base.sha(binary) == args.sha256
    output = args.out.absolute()
    output.mkdir(parents=True, mode=0o700, exist_ok=False)
    shutil.copyfile(Path(__file__), output / 'executed-runner.py')
    root = Path(tempfile.mkdtemp(prefix='ocxd024-gui-root-cancel-', dir='/private/tmp'))
    root.chmod(0o700)
    profile, temp = root / 'profile', root / 'tmp'
    profile.mkdir(mode=0o700)
    temp.mkdir(mode=0o700)
    copied, helper = root / base.APP, root / 'ax'
    exe = copied / 'Contents/MacOS/opencodex-desktop'
    processes, commands, observations = [], [], []
    proc, log, active_log = None, None, None
    paths = coord.coordinators(profile)
    report = dict(temporaryRoot=str(root), appSha256=args.sha256, seedHostSha256=base.sha(args.seed),
                  result='incomplete', releaseAcceptance=False, serviceActionRequested=False,
                  hypothesis='real packaged status IPC returns without executor starvation on partial root',
                  statusBudgetMs=5000, dailyConfigurationModified=False)

    def write(name, value):
        base.write(output / name, value)

    def command(argv, timeout=30, env=None, required=True):
        result = subprocess.run([str(x) for x in argv], cwd=root, env=env, capture_output=True, timeout=timeout)
        commands.append(dict(argv=[str(x) for x in argv], exitCode=result.returncode,
                             stdout=result.stdout.decode(errors='replace'), stderr=result.stderr.decode(errors='replace')))
        if required and result.returncode:
            raise RuntimeError('native command failed: ' + str(argv[0]))
        return result

    def snapshot(label):
        assert proc.poll() is None
        result = command([helper, 'snapshot', proc.pid, exe], timeout=12)
        value = json.loads(result.stdout)
        assert not value.get('locked')
        observations.append(label)
        write(str(len(observations)) + '-' + label + '.json', value)
        return value

    def wait(label, predicate, seconds):
        end = time.monotonic() + seconds
        while time.monotonic() < end:
            value = snapshot(label)
            if predicate(value):
                return value
            time.sleep(.25)
        raise RuntimeError('observation timeout: ' + label)

    def find(value, label):
        return [n for n in value['nodes'] if n.get('role') == 'AXButton'
                and label in (n.get('title'), n.get('description')) and n.get('enabled', True)]

    def press(label):
        value = wait('find-' + label, lambda v: len(find(v, label)) in ((1, 2) if label == '关闭' else (1,)), 15)
        nodes = find(value, label)
        node = max(nodes, key=lambda n: n.get('frame', {}).get('y', -1)) if label == '关闭' else nodes[0]
        command([helper, 'press', proc.pid, exe, node['path'], label])

    def end_gui():
        nonlocal proc, log
        if proc.poll() is None:
            os.killpg(proc.pid, signal.SIGTERM)
            proc.wait(timeout=10)
        group = base.owned_group(proc.pid)
        processes.append(dict(pid=proc.pid, exitCode=proc.returncode, residues=group))
        assert not group
        log.close()
        proc, log = None, None

    try:
        assert not any(os.path.lexists(str(p) + suffix) for p in paths for suffix in ('', '-journal', '-wal', '-shm'))
        write('coordinator-before.json', dict(exactPathsAbsent=[str(p) for p in paths]))
        command(['/usr/bin/ditto', '--rsrc', '--extattr', '--acl', app, copied], timeout=120)
        command(['/usr/bin/codesign', '--verify', '--deep', '--strict', copied])
        command(['/usr/bin/xcrun', 'swiftc', ROOT / 'test/smoke/ocx021-macos-native.swift', '-o', helper], timeout=90)
        permissions = json.loads(command([helper, 'status']).stdout)
        write('desktop-permissions.json', permissions)
        assert not permissions.get('locked') and permissions.get('axTrusted')
        _, config = base.seed_preferences(profile)
        config.write_text('{"hostname":"127.0.0.1","port":0,"providers":{}}\n')
        config.chmod(0o600)
        policy = root / 'candidate.sb'
        policy.write_text('(version 1) (allow default) (deny network*) (allow network-bind (local ip "localhost:*")) (allow network-inbound (local ip "localhost:*")) (allow network-outbound (remote ip "localhost:*")) (deny file-read* file-write* (subpath ' + json.dumps(pwd.getpwuid(os.getuid()).pw_dir) + ')) (deny file-read* file-write* (subpath "/Library/Keychains")) (deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.secd"))')
        shutil.copyfile(policy, output / 'candidate.sb')
        env = dict(HOME=str(profile), CFFIXED_USER_HOME=str(profile), CODEX_HOME=str(profile / '.codex'),
                   PATH='/usr/bin:/bin:/usr/sbin:/sbin', TMPDIR=str(temp) + '/', LANG='en_US.UTF-8',
                   USER=pwd.getpwuid(os.getuid()).pw_name, LOGNAME=pwd.getpwuid(os.getuid()).pw_name, OCXD024_STATUS_TRACE='1')
        denied = command(['/usr/bin/sandbox-exec', '-f', policy, '/usr/bin/nc', '-v', '-G', '1', '-z', '192.0.2.1', '443'], env=env, required=False)
        assert denied.returncode and b'Operation not permitted' in denied.stderr

        def launch(generation):
            nonlocal proc, log, active_log
            active_log = output / ('application-' + str(generation) + '.log')
            log = active_log.open('xb')
            proc = subprocess.Popen(['/usr/bin/sandbox-exec', '-f', str(policy), str(exe)],
                                    cwd=root, env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
            write('live.json', dict(root=str(root), pid=proc.pid, executable=str(exe), generation=generation))
            deadline = time.monotonic() + 12
            while time.monotonic() < deadline:
                focused = command([helper, 'focus', proc.pid, exe], required=False)
                if focused.returncode == 0:
                    break
                assert focused.returncode == 3 and proc.poll() is None
                time.sleep(.3)
            else:
                raise RuntimeError('exact GUI registration timeout')
            current = profile / 'Library/Application Support/OpenCodeX Desktop/runtime/current.json'
            pending = current.parent / '.selection-pending.json'
            # current.json is published before activation is fully settled.
            # Never kill a preparation process in that intermediate state.
            wait('startup-' + str(generation), lambda v: current.is_file() and not pending.exists()
                 and len(find(v, '刷新状态')) == 1
                 and any('运行状态与常用操作' in str(n.get('value', '')) for n in v['nodes']), 180)

        launch(1)
        end_gui()
        seed = root / 'seed-host'
        shutil.copyfile(args.seed, seed)
        seed.chmod(0o700)
        command(['/usr/bin/codesign', '--verify', '--strict', seed])
        command(['/usr/bin/sandbox-exec', '-f', policy, seed, '--exact', 'prepare_exact_owned_first_root_for_real_gui', '--ignored', '--nocapture'],
                timeout=60, env=dict(env, OCXD024_GUI_ROOT_CANCEL_SEED=str(root), OCXD024_GUI_ROOT_PARTIAL_SEED='1'))
        shutil.copyfile(root / 'seed-result.json', output / 'seed-result.json')
        index = profile / '.codex-desktop/desktop/manager-state/client-session-index.sqlite'

        def head():
            connection = sqlite3.connect(index.as_uri() + '?mode=ro', uri=True, timeout=0)
            try:
                return json.loads(connection.execute('select document from current_session where singleton=1').fetchone()[0])
            finally:
                connection.close()

        before = head()
        write('head-before.json', before)
        assert before['state'] == 'capture_reserved'
        launch(2)
        press('诊断')
        press('检查客户端恢复')
        press('关闭')
        # Exactly one user refresh; no retry to turn a busy/unknown into pass.
        prior = active_log.read_text(errors='replace').count('"stage":"command.received"')
        clicked = time.monotonic()
        press('刷新状态')
        wait('manual-refresh-returned', lambda v: len(find(v, '刷新状态')) == 1, 6)
        report['uiObservationElapsedMs'] = int((time.monotonic() - clicked) * 1000)
        phases = [json.loads(line.split('STATUS024_PHASE ', 1)[1]) for line in active_log.read_text(errors='replace').splitlines() if line.startswith('STATUS024_PHASE ')]
        write('phases.json', phases)
        commands_seen = [p for p in phases if p['stage'] == 'command.received']
        assert len(commands_seen) > prior, 'native action must reach real Tauri status command'
        latest = commands_seen[-1]['elapsedUs']
        acquired = [p for p in phases if p['stage'] == 'collector.acquired' and p['elapsedUs'] >= latest]
        returned = [p for p in phases if p['stage'] == 'collector.return' and p['elapsedUs'] >= latest]
        report['manualRefresh'] = dict(commandReceived=True, acquired=bool(acquired), returned=bool(returned),
                                     observedBusyOrUnknown=not bool(acquired))
        if acquired:
            assert len(acquired) == len(returned) == 1 and returned[0]['thread'] == acquired[0]['thread']
            elapsed = (returned[0]['elapsedUs'] - latest) / 1000
            report['manualRefresh']['backendElapsedMs'] = elapsed
            assert elapsed < 5000, 'original Status5 must not accept a late read'
        assert head() == before
        write('head-after.json', head())
        assert not (config.parent / 'runtime-port.json').exists()
        assert not (config.parent / 'ocx.pid').exists()
        # This proves return/latency only; an error or busy status is never a
        # successful status fact, Start permission or lifecycle acceptance.
        report.update(result='observation_completed', canonicalUnchanged=True, noRuntimeRecord=True,
                      statusSuccessNotAsserted=True)
        end_gui()
    except BaseException as error:
        report.update(result='failed', error=repr(error), retainedExactGui=proc is not None and proc.poll() is None)
        if proc is not None and proc.poll() is None:
            command(['/usr/bin/sample', proc.pid, '2', '-file', output / 'failure-sample.txt'], timeout=15, required=False)
        raise
    finally:
        manager = profile / '.codex-desktop/desktop/manager-state'
        if manager.exists():
            shutil.copytree(manager, output / 'raw-manager', symlinks=True)
        report['processes'] = processes
        report['sourceAppUnchanged'] = base.sha(binary) == args.sha256
        report['copiedAppUnchanged'] = exe.is_file() and base.sha(exe) == args.sha256
        if report['result'] == 'observation_completed' and proc is None:
            try:
                coord.clean_coordinators(paths, output)
                shutil.rmtree(root)
            except Exception as error:
                report['cleanupError'] = repr(error)
        report['temporaryRootAbsent'] = not root.exists()
        write('commands.json', commands)
        write('summary.json', report)
    if report.get('cleanupError'):
        raise RuntimeError('exact fixture cleanup incomplete; original evidence retained')
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--sha256', required=True)
    parser.add_argument('--seed', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    print(json.dumps(run(parser.parse_args())))
