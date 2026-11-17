"""PID-scoped real app rename/save/reopen in a fresh OS-guarded private profile.

Only synthetic channels/keys. No service, model API, real WebDAV or daily HOME.
Keeps commands/AX/config evidence and removes only its own app/profile/processes.
"""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import plistlib
import pwd
import shutil
import signal
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]
_spec = importlib.util.spec_from_file_location('first_supply024', Path(__file__).with_name('macos024-first-supply.py'))
base = importlib.util.module_from_spec(_spec); _spec.loader.exec_module(base)


def write(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + chr(10))


def run(app, expected_sha, output):
    app = app.resolve(strict=True)
    if app.name != base.APP or base.sha(app / 'Contents/MacOS/opencodex-desktop') != expected_sha:
        raise ValueError('explicit exact sealed app required')
    output = output.absolute(); output.mkdir(parents=True, mode=0o700, exist_ok=False)
    root = Path(tempfile.mkdtemp(prefix='ocxd024-native-model-save-', dir='/private/tmp')); root.chmod(0o700)
    profile = root / 'profile'; profile.mkdir(mode=0o700)
    temporary = root / 'tmp'; temporary.mkdir(mode=0o700)
    copied = root / base.APP
    helper, capture = root / 'ax', root / 'capture'
    commands, snapshots, processes = [], [], []
    proc, log = None, None
    report = {'inputApp': str(app), 'inputSha256': expected_sha, 'releaseAcceptance': False,
              'dailyConfigurationModified': False, 'serviceActionRequested': False, 'result': 'incomplete',
              'scope': 'real Desktop file save and application reopen; not external Codex GUI or service restart'}

    def command(argv, timeout=30, env=None, required=True):
        p = subprocess.run([str(x) for x in argv], cwd=root, env=env, capture_output=True, timeout=timeout)
        commands.append({'argv': [str(x) for x in argv], 'exitCode': p.returncode,
                         'stdout': p.stdout.decode(errors='replace')[:18000], 'stderr': p.stderr.decode(errors='replace')[:18000]})
        if p.returncode and required: raise RuntimeError('native command failed: ' + str(argv[0]) + ' / ' + p.stderr.decode(errors='replace')[:400])
        return p

    def snapshot(label):
        if proc.poll() is not None: raise RuntimeError('candidate exited before observation')
        p = command([helper, 'snapshot', proc.pid, exe], 12, required=False)
        if p.returncode == 3 and proc.poll() is None:
            # macOS may not have registered the just-created GUI process yet.
            # Reobserve THIS live handle; never start a replacement process.
            return None
        if p.returncode: raise RuntimeError('candidate AX observation failed')
        value = json.loads(p.stdout)
        if value.get('locked'): raise RuntimeError('desktop locked')
        snapshots.append(label); write(output / (str(len(snapshots)) + '-' + label + '.json'), value)
        return value

    def wait_for(label, predicate, seconds=30):
        end = time.monotonic() + seconds
        while time.monotonic() < end:
            value = snapshot(label)
            if value is not None and predicate(value): return value
            time.sleep(.6)
        raise RuntimeError('observation timeout: ' + label)

    def matches(value, label, role=None):
        return [n for n in value['nodes'] if (role is None or n.get('role') == role)
                and label in (n.get('title'), n.get('description')) and n.get('enabled', True)]

    def press(label):
        value = wait_for('find-' + label, lambda s: bool(matches(s, label, 'AXButton')))
        found = matches(value, label, 'AXButton')
        if len(found) != 1: raise RuntimeError('ambiguous candidate button: ' + label)
        command([helper, 'press', proc.pid, exe, found[0]['path'], label])

    def end_process():
        nonlocal proc, log
        if proc and proc.poll() is None:
            os.killpg(proc.pid, signal.SIGTERM)
            proc.wait(timeout=10)
        if proc:
            residues = base.owned_group(proc.pid)
            processes.append({'pid': proc.pid, 'exitCode': proc.returncode, 'residues': residues})
            if residues: raise RuntimeError('own process group residues remain')
        if log: log.close()
        proc, log = None, None

    try:
        command(['/usr/bin/ditto', '--rsrc', '--extattr', '--acl', app, copied], 120)
        exe = copied / 'Contents/MacOS/opencodex-desktop'
        command(['/usr/bin/codesign', '--verify', '--deep', '--strict', copied])
        for source, dest in [(ROOT / 'test/smoke/ocx021-macos-native.swift', helper), (ROOT / 'test/smoke/lib/capwin.swift', capture)]:
            command(['/usr/bin/xcrun', 'swiftc', source, '-o', dest], 90)
        status = json.loads(command([helper, 'status']).stdout); report['permissions'] = status
        if status.get('locked') or not status.get('axTrusted'): raise RuntimeError('unlocked AX desktop required')
        _, config = base.seed_preferences(profile)
        initial = {'hostname': '127.0.0.1', 'port': 0, 'defaultProvider': 'GuiSource',
                   'defaultModel': 'GuiSource/vendor/custom', 'defaultModelAliases': False,
                   'providers': {'GuiSource': {'name': 'GuiSource', 'adapter': 'openai-chat', 'authMode': 'key',
                    'apiKey': 'synthetic-ui-key-never-used', 'baseUrl': 'https://fixture.invalid/v1',
                    'models': ['gpt-6-sol', 'vendor/custom'], 'selectedModels': ['gpt-6-sol', 'vendor/custom'],
                    'modelContextWindows': {'vendor/custom': 4096}, 'liveModels': False, 'defaultAliases': False,
                    'unknownFixture': {'keep': True}}}}
        write(config, initial); config.chmod(0o600)
        policy = root / 'candidate.sb'
        policy.write_text('(version 1) (allow default) (deny network*) (deny file-read* file-write* (subpath ' + json.dumps(pwd.getpwuid(os.getuid()).pw_dir) + ')) (deny file-read* file-write* (subpath "/Library/Keychains")) (deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.secd"))')
        shutil.copyfile(policy, output / 'candidate.sb')
        env = {'HOME': str(profile), 'CFFIXED_USER_HOME': str(profile), 'CODEX_HOME': str(profile / '.codex'),
               'PATH': '/usr/bin:/bin:/usr/sbin:/sbin', 'TMPDIR': str(temporary) + '/', 'LANG': 'en_US.UTF-8',
               'USER': pwd.getpwuid(os.getuid()).pw_name, 'LOGNAME': pwd.getpwuid(os.getuid()).pw_name}
        denied = subprocess.run(['/usr/bin/sandbox-exec', '-f', str(policy), '/usr/bin/nc', '-v', '-G', '1', '-z', '192.0.2.1', '443'], env=env, capture_output=True)
        report['networkGuard'] = {'exitCode': denied.returncode, 'stderr': denied.stderr.decode(errors='replace')}
        if denied.returncode == 0 or b'Operation not permitted' not in denied.stderr: raise RuntimeError('network denial unproven')
        for generation in (1, 2):
            log = (output / ('application-' + str(generation) + '.log')).open('xb')
            proc = subprocess.Popen(['/usr/bin/sandbox-exec', '-f', str(policy), str(exe)], cwd=root, env=env,
                                    stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
            print(json.dumps({'generation': generation, 'pid': proc.pid, 'phase': 'started'}), flush=True)
            current = profile / 'Library/Application Support/OpenCodeX Desktop/runtime/current.json'
            wait_for('startup-' + str(generation), lambda s: current.is_file() and any(n.get('value') == '运行状态与常用操作：环境明细收进运行详情。' or n.get('title') == '版本升级' for n in s['nodes']), 180)
            press('模型')
            wait_for('models-' + str(generation), lambda s: bool(matches(s, '读取现有渠道', 'AXButton')))
            if generation == 1:
                press('读取现有渠道')
                wait_for('read-channels', lambda s: any(n.get('value') == 'GuiSource' for n in s['nodes']))
                press('编辑')
                value = wait_for('channel-editor', lambda s: bool(matches(s, '渠道名称', 'AXTextField')))
                node = matches(value, '渠道名称', 'AXTextField')
                if len(node) != 1: raise RuntimeError('channel name field ambiguous')
                command([helper, 'type', proc.pid, exe, node[0]['path'], '界面中文渠道', '渠道名称'])
                wait_for('renamed-input', lambda s: any(n.get('description') == '渠道名称' and n.get('value') == '界面中文渠道' for n in s['nodes']))
                # Detail save is the explicit single-channel commit. Only the
                # global list action opens a separate confirmation dialog.
                press('保存渠道草稿')
                wait_for('saved-list', lambda s: any(n.get('value') == '界面中文渠道' for n in s['nodes']))
            else:
                wait_for('reopened-channel', lambda s: any(n.get('value') == '界面中文渠道' for n in s['nodes']))
            document = json.loads(config.read_bytes()); rows = document['providers']
            if len(rows) != 1: raise RuntimeError('expected exactly one synthetic provider')
            provider, row = next(iter(rows.items()))
            assert row['name'] == '界面中文渠道'
            assert row['apiKey'] == initial['providers']['GuiSource']['apiKey']
            assert row['models'] == initial['providers']['GuiSource']['models']
            assert row['unknownFixture'] == {'keep': True}
            assert row['modelDisplayNames']['gpt-6-sol'] == '界面中文渠道 · gpt-6-sol'
            assert row['modelDisplayNames']['vendor/custom'] == '界面中文渠道 · vendor／custom'
            assert document['defaultModel'] == provider + '/vendor/custom'
            shutil.copyfile(config, output / ('saved-config-' + str(generation) + '.json'))
            value = snapshot('final-' + str(generation))
            if status.get('screenCaptureAllowed'):
                windows = [w for w in value.get('windows', []) if w.get('onscreen') and w.get('layer') == 0]
                if windows: command([capture, windows[0]['id'], output / ('window-' + str(generation) + '.png')])
            end_process()
        report.update(result='passed', realGuiSave=True, applicationReopen=True, rawModelsAndSyntheticKeyPreserved=True)
    except BaseException as exc:
        report.update(result='failed', error=repr(exc))
        raise
    finally:
        try: end_process()
        except Exception as exc: report['cleanupError'] = repr(exc)
        config_path = profile / '.codex-desktop/opencodex/home/config.json'
        if config_path.is_file(): shutil.copyfile(config_path, output / 'final-synthetic-config.json')
        for name in ('app.log',):
            file = profile / '.codex-desktop/desktop/logs' / name
            if file.is_file(): shutil.copyfile(file, output / ('startup-' + name))
        report['processes'] = processes
        report['sourceAppUnchanged'] = base.sha(app / 'Contents/MacOS/opencodex-desktop') == expected_sha
        report['copiedAppUnchanged'] = (copied / 'Contents/MacOS/opencodex-desktop').is_file() and base.sha(copied / 'Contents/MacOS/opencodex-desktop') == expected_sha
        if not report.get('cleanupError') and all(not p['residues'] for p in processes): shutil.rmtree(root)
        report['temporaryRoot'] = str(root); report['temporaryRootAbsent'] = not root.exists()
        write(output / 'commands.json', commands); write(output / 'summary.json', report)
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--expected-sha256', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(run(args.app, args.expected_sha256, args.output), ensure_ascii=False), flush=True)
