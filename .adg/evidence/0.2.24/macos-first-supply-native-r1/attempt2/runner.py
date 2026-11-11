#!/usr/bin/env python3
"""Actual sealed app first-supply under a fresh normal private OS profile.

No OPENCODEX_SANDBOX: supplies via production startup. Kernel denies daily-home
and outbound network/Keychain. Never clicks Start/Sync or changes installed apps.
Keeps evidence, removes only own process group, copy and private profile.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import plistlib
import pwd
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]
APP = 'OpenCodeX Desktop.app'


def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''): h.update(block)
    return h.hexdigest()


def write(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')


def seed_preferences(home):
    """Current v2 field ownership comes from the actual product table, not v1 migration."""
    source = (ROOT / 'apps/desktop/tauri/src/modules/preferences/mod.rs').read_text()
    table = source.split('const DISK_SECTIONS:', 1)[1].split('];', 1)[0]
    groups = re.findall(r'\(\s*"([a-z_]+)"\s*,\s*&\[([^\]]*)\]\s*,?\s*\)', table, re.S)
    defaults = json.loads((ROOT / 'apps/desktop/tauri/config/preferences.defaults.json').read_bytes())
    defaults.update(auto_panel=False, launch_with_codex=False, app_update_auto_check=False,
                    cold_sync=False, cli_enabled=False, startup_cleanup=False)
    document = {'schema_version': 2}
    keys = []
    for section, raw in groups:
        fields = re.findall(r'"([a-z_]+)"', raw)
        keys += fields
        document[section] = {key: defaults[key] for key in fields}
    if len(groups) != 9 or set(keys) != set(defaults) - {'schema_version'} or len(keys) != len(set(keys)):
        raise ValueError('current product preference sections cannot be projected exactly')
    desktop = home / '.codex-desktop/desktop'
    (desktop / 'manager-state').mkdir(parents=True, mode=0o700)
    # Explicit current-format fixture metadata, same exact shape asserted by
    # data_root::tests. Preseeded preferences must not be an unrecognized root.
    data_root_source=(ROOT/'apps/desktop/tauri/src/modules/data_root/mod.rs').read_text()
    version=re.search(r'pub const STRUCTURE_VERSION: &str = "([0-9]+)";',data_root_source)
    if not version or version.group(1)!='2':raise ValueError('unreviewed data-root schema')
    metadata=desktop/'manager-state/data-root.json'
    write(metadata,{'structure_version':version.group(1),'opencodex_home_mode':'inside'});metadata.chmod(0o600)
    path = desktop / 'manager-state/preferences.json'
    write(path, document); path.chmod(0o600)
    config = home / '.codex-desktop/opencodex/home/config.json'
    config.parent.mkdir(parents=True, mode=0o700)
    config.write_bytes(b'{ "providers":{}, "port":0, "fixtureOnly":true }\n'); config.chmod(0o600)
    return path, config


def owned_group(pid):
    rows = subprocess.check_output(['/bin/ps', '-axo', 'pid=,ppid=,pgid=,comm='], text=True)
    return [{'pid': int(p[0]), 'ppid': int(p[1]), 'pgid': int(p[2]), 'image': p[3]}
            for row in rows.splitlines() if len(p := row.split(None, 3)) == 4 and int(p[2]) == pid]


def run(args):
    if sys.platform != 'darwin': raise ValueError('requires a real macOS desktop')
    source = args.app.resolve(strict=True)
    if source.name != APP or args.app.is_symlink(): raise ValueError('explicit sealed app required')
    info = plistlib.loads((source / 'Contents/Info.plist').read_bytes())
    if (info.get('CFBundleShortVersionString') != '0.2.24'
            or info.get('CFBundleIdentifier') != 'com.gzers.opencodex.desktop'
            or Path(info.get('CFBundleExecutable', '')).name != info.get('CFBundleExecutable')):
        raise ValueError('unexpected actual app identity')
    source_exe = source / 'Contents/MacOS' / info['CFBundleExecutable']
    if sha(source_exe) != args.expected_sha256: raise ValueError('actual input binary differs from reviewed source')
    if args.out.exists(): raise FileExistsError('new evidence directory required')
    args.out.mkdir(parents=True, mode=0o700)
    out = args.out.resolve()
    root = Path(tempfile.mkdtemp(prefix='ocxd024-first-supply-', dir='/private/tmp'))
    root.chmod(0o700)
    profile, temp = root / 'profile', root / 'tmp'
    profile.mkdir(mode=0o700); temp.mkdir(mode=0o700)
    app = root / 'Applications' / APP
    exe = app / 'Contents/MacOS' / info['CFBundleExecutable']
    program = profile / 'Library/Application Support/OpenCodeX Desktop'
    helper, capture = root / 'ax', root / 'capwin'
    commands, observations = [], []
    proc = None; log = None
    report = {'schemaVersion': 1, 'scope': 'actual normal-profile startup supply; no service action',
              'accepted': False, 'result': 'incomplete', 'inputApp': str(source),
              'inputBinarySha256': args.expected_sha256, 'inputSourceDescription': args.source_description,
              'frozenCandidate': False, 'normalStartupSupply': True, 'testSandboxEnabled': False,
              'fixtureDataRoot':'current v2 root initialized only to disable auto online/sync actions; runtime absent',
              'dailyConfigurationModified': False, 'serviceActionRequested': False, 'businessRequests': False,
              'temporaryRoot': str(root), 'scriptSha256': sha(Path(__file__))}

    def command(argv, timeout=30, env=None):
        p = subprocess.run([str(x) for x in argv], capture_output=True, text=True, timeout=timeout, env=env)
        commands.append({'argv': [str(x) for x in argv], 'exitCode': p.returncode,
                         'stdout': p.stdout[:16000], 'stderr': p.stderr[:16000]})
        return p

    started = time.monotonic()
    try:
        app.parent.mkdir()
        p = command(['/usr/bin/ditto', '--rsrc', '--extattr', '--acl', source, app], 120)
        if p.returncode: raise RuntimeError('sealed app copy failed')
        if sha(exe) != args.expected_sha256: raise RuntimeError('app copy changed executable bytes')
        p = command(['/usr/bin/codesign', '--verify', '--deep', '--strict', app])
        if p.returncode: raise RuntimeError('copied app signature not verified')
        resource = app / 'Contents/Resources/runtime-bundle'
        descriptor = json.loads((resource / 'runtime-bundle.manifest.json').read_bytes())
        report['descriptorSha256'] = sha(resource / 'runtime-bundle.manifest.json')
        if report['descriptorSha256'] != 'e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3':
            raise ValueError('explicit reviewed darwin complete official bundle required')
        config_path, config = seed_preferences(profile)
        report['preferencesBeforeSha256'] = sha(config_path); before_config = config.read_bytes()
        for src, target in [(ROOT / 'test/smoke/ocx021-macos-native.swift', helper),
                            (ROOT / 'test/smoke/lib/capwin.swift', capture)]:
            p = command(['/usr/bin/xcrun', 'swiftc', src, '-o', target], 90)
            if p.returncode: raise RuntimeError('PID scoped native observation helper failed to compile')
        permissions = json.loads(command([helper, 'status']).stdout); report['permissions'] = permissions
        if permissions.get('locked') or not permissions.get('axTrusted'):
            report.update(result='blocked', reason='locked or native AX permission unavailable')
            return report
        home = pwd.getpwuid(os.getuid()).pw_dir
        policy = ('(version 1)\n(allow default)\n(deny network*)\n'
                  '(deny file-read* file-write* (subpath ' + json.dumps(home) + '))\n'
                  '(deny file-read* file-write* (subpath "/Library/Keychains"))\n'
                  '(deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.secd"))\n')
        sb = root / 'candidate.sb'; sb.write_text(policy); (out / 'candidate.sb').write_text(policy)
        env = {'PATH': '/usr/bin:/bin:/usr/sbin:/sbin', 'LANG': 'en_US.UTF-8',
               'USER': pwd.getpwuid(os.getuid()).pw_name, 'LOGNAME': pwd.getpwuid(os.getuid()).pw_name,
               'HOME': str(profile), 'CFFIXED_USER_HOME': str(profile),
               'TMPDIR': str(temp) + '/', 'TMP': str(temp), 'TEMP': str(temp),
               'XDG_CONFIG_HOME': str(profile / '.config'), 'XDG_CACHE_HOME': str(profile / '.cache'),
               'CODEX_HOME': str(profile / '.codex')}
        if any(k.startswith('OPENCODEX_SANDBOX') for k in env): raise AssertionError('not a test-sandbox run')
        p = command(['/usr/bin/sandbox-exec', '-f', sb, '/usr/bin/nc', '-v', '-G', '1', '-z', '192.0.2.1', '443'], env=env)
        if p.returncode == 0 or 'Operation not permitted' not in p.stderr: raise RuntimeError('kernel network guard not proved')
        p = command(['/usr/bin/sandbox-exec', '-f', sb, '/bin/cat', Path(home) / '.zshrc'], env=env)
        if p.returncode == 0 or 'Operation not permitted' not in p.stderr: raise RuntimeError('daily-home read denial not proved')
        report['kernelNetworkAndDailyHomeDenied'] = True
        log = (out / 'application.log').open('wb')
        proc = subprocess.Popen(['/usr/bin/sandbox-exec', '-f', str(sb), str(exe)], cwd=root, env=env,
                                stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        report['pid'] = proc.pid
        deadline = time.monotonic() + args.timeout
        capture_states = set(); last = None; previous = None
        current = program / 'runtime/current.json'
        while time.monotonic() < deadline and proc.poll() is None:
            p = command([helper, 'snapshot', proc.pid, exe], 10)
            if p.returncode: time.sleep(.3); continue
            last = json.loads(p.stdout)
            if last.get('locked'): raise RuntimeError('screen locked while observing candidate')
            # Values contain only this isolated app's synthetic data. Never traverse OS menus.
            text = '\n'.join(str(n.get('value') or n.get('title') or n.get('description') or '') for n in last.get('nodes', []))
            state = ('current' if current.is_file() else 'no-current',
                     tuple(word for word in ['准备内置', '校验运行归档', '展开运行包', '核对程序与固定依赖',
                                             '核对程序入口', '准备失败', '已就绪', '就绪', '未就绪'] if word in text))
            if state != previous:
                previous = state
                observations.append({'elapsedSeconds': round(time.monotonic()-started, 3), 'state': state})
                write(out / ('snapshot-' + str(len(observations)) + '.json'), last)
                print(json.dumps(observations[-1], ensure_ascii=False), flush=True)
            if current.is_file() and '未就绪' not in text and ('运行详情' in text or '运行状态与常用操作' in text):
                report['uiEnvironmentReadyObserved'] = True; break
            if '准备失败' in text and not current.exists(): break
            time.sleep(.6)
        if last: write(out / 'last-window-ax.json', last)
        report.update(appStillRunning=proc.poll() is None, processExit=proc.poll(),
                      programRootCreated=program.exists(), currentCreated=current.exists())
        if current.is_file():
            selection = json.loads(current.read_bytes()); report['selection'] = selection
            if selection['version'] != '2.99.0': raise AssertionError('wrong supplied runtime version')
            image = program / 'runtime/versions' / selection['version']
            if sha(image / '.runtime-image.json') != selection['imageSha256']: raise AssertionError('selected image manifest not bound')
            identity = json.loads((image / '.runtime-image.json').read_bytes())
            if identity['officialArchiveSha256'] != descriptor['runtimeArchiveSha256']: raise AssertionError('wrong supplied official archive')
            report['runtimeImage'] = {k: identity[k] for k in ['version', 'platform', 'officialArchiveSha256', 'treeSha256']}
            package = image / 'opencodex/node_modules/@bitkyc08/opencodex/package.json'
            if json.loads(package.read_bytes())['version'] != '2.99.0': raise AssertionError('actual package version differs')
            report['tools'] = {}
            node = image / 'dependencies/node/bin/node'
            for name, argv in [('node', [node, '--version']), ('npm', [node, image / 'dependencies/npm/bin/npm-cli.js', '--version']),
                               ('ocx', [node, image / 'opencodex/node_modules/@bitkyc08/opencodex' / identity['bin'], '--version'])]:
                p = command(['/usr/bin/sandbox-exec', '-f', sb, *argv], env=env, timeout=15)
                report['tools'][name] = {'exitCode': p.returncode, 'version': p.stdout.strip()}
                if p.returncode: raise AssertionError(name + ' version query failed')
            if report['tools']['node']['version'] != 'v26.8.1' or report['tools']['npm']['version'] != '11.19.0' or report['tools']['ocx']['version'] != 'opencodex 2.99.0':
                raise AssertionError('actual tool version output differs from pinned resource')
            write(out / 'runtime-selection.json', selection); write(out / 'runtime-image.json', identity)
        for p in [profile / '.codex-desktop/desktop/logs/app.log', profile / '.codex-desktop/desktop/manager-state/data-root.json']:
            if p.is_file(): shutil.copy2(p, out / ('startup-' + p.name))
        report['configPreserved'] = config.read_bytes() == before_config
        report['runtimePortRecordAbsent'] = not (config.parent / 'runtime-port.json').exists()
        report['sourceAppBinaryUnchanged'] = sha(source_exe) == args.expected_sha256
        report['copiedBinaryUnchanged'] = sha(exe) == args.expected_sha256
        if last and permissions.get('screenCaptureAllowed'):
            windows = [w for w in last.get('windows', []) if w.get('onscreen') and w.get('layer') == 0 and w.get('bounds', {}).get('Width', 0) > 500]
            if windows: report['captureExit'] = command([capture, windows[0]['id'], out / 'last-window.png'], env=env).returncode
        report['result'] = 'passed' if (report.get('uiEnvironmentReadyObserved') and report.get('tools')
                                and report['configPreserved'] and report['runtimePortRecordAbsent']
                                and report['sourceAppBinaryUnchanged'] and report['copiedBinaryUnchanged']) else 'failed'
    except BaseException as exc:
        report.update(result='failed', error=repr(exc))
    finally:
        if proc:
            if proc.poll() is None:
                os.killpg(proc.pid, signal.SIGTERM)
                try: proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    report['guiForcedCleanup'] = True
                    os.killpg(proc.pid, signal.SIGKILL); proc.wait(timeout=5)
            report['ownedGuiExit'] = proc.returncode
            report['ownedGroupResidues'] = owned_group(proc.pid)
        if log: log.close()
        write(out / 'commands.json', commands); write(out / 'observations.json', observations)
        if not report.get('ownedGroupResidues'): shutil.rmtree(root)
        report['temporaryRootAbsent'] = not root.exists()
        report['elapsedSeconds'] = round(time.monotonic()-started, 3)
        write(out / 'summary.json', report)
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--expected-sha256', required=True)
    parser.add_argument('--source-description', required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--timeout', type=int, default=180)
    args = parser.parse_args()
    if not re.fullmatch(r'[a-f0-9]{64}', args.expected_sha256) or not 30 <= args.timeout <= 300:
        raise ValueError('explicit SHA256 and bounded30..300s observation required')
    result = run(args)
    print(json.dumps(result, ensure_ascii=False), flush=True)
    sys.exit(0 if result['result'] == 'passed' and result.get('temporaryRootAbsent') else 1)
