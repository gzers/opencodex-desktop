#!/usr/bin/env python3
"""Real .app/PID/WKWebView/AX acceptance; never launches or quits a daily app."""
import argparse
import ctypes
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import plistlib
import pwd
import re
import shlex
import shutil
import signal
import subprocess
import sys
import threading
import time
import traceback
import uuid

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
BUTTON_ROLES = ('AXButton', 'AXPopUpButton', 'AXRadioButton')

class Blocked(RuntimeError):
    pass

def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')
    path.chmod(0o600)

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def command(args, **kwargs):
    return subprocess.run([str(x) for x in args], capture_output=True, text=True, timeout=40, **kwargs)

def processes():
    rows = command(['/bin/ps', '-axo', 'pid=,ppid=,pgid=,comm=']).stdout.splitlines()
    return [{'pid': int(p[0]), 'ppid': int(p[1]), 'pgid': int(p[2]), 'command': p[3]}
            for row in rows if len(p := row.split(None, 3)) == 4]

def tree_inventory(root):
    result = {}
    for path in sorted(root.rglob('*')):
        key = str(path.relative_to(root))
        if path.is_symlink():
            if not path.resolve().is_relative_to(root.resolve()):
                raise Blocked('Tool dependency symlink escapes its copied tree: ' + str(path))
            result[key] = {'symlink': os.readlink(path)}
        elif path.is_file(): result[key] = {'sha256': sha(path), 'size': path.stat().st_size}
    return result

def notice_clock_bounds(samples, exits):
    """Conservative native observation intervals, never input-call durations.

    Each AX state was read somewhere between request begin and response end.
    A transition is therefore bracketed by the previous state's request begin
    and the next state's response end. Keep that uncertainty in the verdict.
    """
    first = next((i for i, s in enumerate(samples) if s['notice'] and not s['dialogs']), None)
    if first is None or first == 0:
        raise Blocked('Notice appearance lacks an observed absent->present boundary')
    final_present = max(i for i, s in enumerate(samples) if s['notice'] and not s['dialogs'])
    after = next((i for i in range(final_present + 1, len(samples)) if not samples[i]['notice'] and not samples[i]['dialogs']), None)
    if after is None: raise Blocked('Notice disappearance was not observed')
    if any(not s['active'] for s in samples[first:after + 1]):
        raise Blocked('Candidate lost foreground during notice observation')
    start = [samples[first - 1]['readBegin'], samples[first]['readEnd']]
    end = [samples[final_present]['readBegin'], samples[after]['readEnd']]
    segments, opening = [], None
    for i in range(first, after + 1):
        present = '重启 Codex' in samples[i]['dialogs']
        previous = '重启 Codex' in samples[i - 1]['dialogs'] if i else False
        if present and not previous: opening = i
        if previous and not present:
            if opening is None: raise Blocked('Missing observed confirmation opening')
            segments.append((opening, i)); opening = None
    if opening is not None or len(segments) != len(exits):
        raise Blocked('Observed confirmation transitions do not match the exercised exits')
    pauses = []
    for how, (opened, closed) in zip(exits, segments):
        appearance = [samples[opened - 1]['readBegin'], samples[opened]['readEnd']]
        disappearance = [samples[closed - 1]['readBegin'], samples[closed]['readEnd']]
        duration = [max(0, disappearance[0] - appearance[1]), disappearance[1] - appearance[0]]
        pauses.append({'exit': how, 'appearanceInterval': appearance, 'disappearanceInterval': disappearance,
                       'durationInterval': duration, 'firstPresentSample': opened, 'firstAbsentSample': closed})
    elapsed = [end[0] - start[1], end[1] - start[0]]
    active = [elapsed[0] - sum(p['durationInterval'][1] for p in pauses),
              elapsed[1] - sum(p['durationInterval'][0] for p in pauses)]
    return {'appearanceInterval': start, 'disappearanceInterval': end, 'elapsedIntervalSeconds': elapsed,
            'activeIntervalSeconds': active, 'activeMidpointSeconds': sum(active) / 2, 'confirmationPauses': pauses,
            'boundaryMethod': 'AX read-begin/read-end brackets around actual notice/dialog state transitions; no input-call subtraction',
            'thresholdSeconds': [6.7, 9.3], 'intervalMustFitThreshold': True}

class NoticeAxReader:
    """Read only the owned application's AX notice/dialog presence in process.

    No input, focus, clock, DOM, product state or Swift helper is changed. Avoid
    launching a full-tree/layout exporter for every subsecond timer observation.
    """
    def __init__(self, pid):
        ref = ctypes.c_void_p
        self.cf = ctypes.CDLL('/System/Library/Frameworks/CoreFoundation.framework/CoreFoundation')
        self.ax = ctypes.CDLL('/System/Library/Frameworks/ApplicationServices.framework/ApplicationServices')
        def declare(lib, name, restype, argtypes):
            f = getattr(lib, name); f.restype = restype; f.argtypes = argtypes; return f
        self.release = declare(self.cf, 'CFRelease', None, [ref])
        self.retain = declare(self.cf, 'CFRetain', ref, [ref])
        self.make_string = declare(self.cf, 'CFStringCreateWithCString', ref, [ref, ctypes.c_char_p, ctypes.c_uint32])
        self.string_bytes = declare(self.cf, 'CFStringGetCString', ctypes.c_bool, [ref, ctypes.c_char_p, ctypes.c_long, ctypes.c_uint32])
        self.type_id = declare(self.cf, 'CFGetTypeID', ctypes.c_ulong, [ref])
        self.string_type = declare(self.cf, 'CFStringGetTypeID', ctypes.c_ulong, [])()
        self.array_type = declare(self.cf, 'CFArrayGetTypeID', ctypes.c_ulong, [])()
        self.boolean_type = declare(self.cf, 'CFBooleanGetTypeID', ctypes.c_ulong, [])()
        self.boolean = declare(self.cf, 'CFBooleanGetValue', ctypes.c_bool, [ref])
        self.count = declare(self.cf, 'CFArrayGetCount', ctypes.c_long, [ref])
        self.at = declare(self.cf, 'CFArrayGetValueAtIndex', ref, [ref, ctypes.c_long])
        self.copy = declare(self.ax, 'AXUIElementCopyAttributeValue', ctypes.c_int, [ref, ref, ctypes.POINTER(ref)])
        self.create = declare(self.ax, 'AXUIElementCreateApplication', ref, [ctypes.c_int])
        self.timeout = declare(self.ax, 'AXUIElementSetMessagingTimeout', ctypes.c_int, [ref, ctypes.c_float])
        self.keys = {key: self.make_string(None, key.encode(), 0x08000100) for key in [
            'AXRole', 'AXSubrole', 'AXTitle', 'AXDescription', 'AXChildren', 'AXWindows', 'AXFrontmost']}
        self.app, self.web = self.create(pid), None
        self.timeout(self.app, .75)
        self.web = self.find_web(self.app)
        if not self.web:
            self.close(); raise Blocked('Owned AXWebArea is unavailable for timing observation')

    def attr(self, element, key):
        value = ctypes.c_void_p()
        if self.copy(element, self.keys[key], ctypes.byref(value)) != 0: return None
        return value.value

    def text(self, element, key):
        value = self.attr(element, key)
        if not value: return ''
        try:
            if self.type_id(value) != self.string_type: return ''
            buffer = ctypes.create_string_buffer(2048)
            return buffer.value.decode('utf-8') if self.string_bytes(value, buffer, len(buffer), 0x08000100) else ''
        finally: self.release(value)

    def find_web(self, element, depth=0):
        if depth > 8: return None
        role = self.text(element, 'AXRole')
        if role == 'AXWebArea': return self.retain(element)
        if role.startswith('AXMenu'): return None
        children = self.attr(element, 'AXWindows' if depth == 0 else 'AXChildren')
        if not children: return None
        try:
            if self.type_id(children) != self.array_type: return None
            for i in range(min(self.count(children), 100)):
                found = self.find_web(self.at(children, i), depth + 1)
                if found: return found
        finally: self.release(children)

    def sample(self):
        active = self.attr(self.app, 'AXFrontmost')
        if not active: raise Blocked('Candidate AX frontmost observation unavailable')
        try: front = self.type_id(active) == self.boolean_type and self.boolean(active)
        finally: self.release(active)
        if self.text(self.web, 'AXRole') != 'AXWebArea': raise Blocked('Owned AXWebArea was invalidated')
        notice, dialogs, visited = False, [], 0
        def walk(element, depth):
            nonlocal notice, visited
            visited += 1
            if depth > 30 or visited > 2000: raise Blocked('Timing AX traversal bound exceeded')
            role = self.text(element, 'AXRole')
            if role.startswith('AXMenu'): return
            if role == 'AXGroup' and self.text(element, 'AXSubrole') in ['AXApplicationDialog', 'AXApplicationAlertDialog']:
                dialogs.append(self.text(element, 'AXTitle')); return
            if role in ['AXButton', 'AXPopUpButton']:
                if self.text(element, 'AXTitle') == '立即重启 Codex' or self.text(element, 'AXDescription') == '立即重启 Codex': notice = True
                return
            if role in ['AXStaticText', 'AXImage', 'AXTextField', 'AXCheckBox', 'AXHeading', 'AXRadioButton']: return
            children = self.attr(element, 'AXChildren')
            if not children: return
            try:
                if self.type_id(children) != self.array_type: return
                for i in range(self.count(children)): walk(self.at(children, i), depth + 1)
            finally: self.release(children)
        walk(self.web, 0)
        return {'active': front, 'notice': notice, 'dialogs': dialogs, 'focused': [], 'axNodesVisited': visited}

    def close(self):
        if self.web: self.release(self.web); self.web = None
        if self.app: self.release(self.app); self.app = None
        for value in self.keys.values(): self.release(value)
        self.keys = {}

class NativeRun:
    def __init__(self, args):
        self.args = args
        self.ready = bool(args.node and args.npm_cli and args.official_entry)
        self.app = Path(args.app).expanduser().resolve(strict=True)
        self.info = plistlib.loads((self.app / 'Contents/Info.plist').read_bytes())
        self.exe = self.app / 'Contents/MacOS' / self.info['CFBundleExecutable']
        expected_version = getattr(args, 'expected_version', '0.2.21')
        if expected_version not in ['0.2.21', '0.2.22']:
            raise ValueError('Unadmitted native acceptance version')
        if self.info['CFBundleIdentifier'] != 'com.gzers.opencodex.desktop' or self.info['CFBundleShortVersionString'] != expected_version:
            raise ValueError('Expected the original OpenCodeX ' + expected_version + ' app identity')
        self.container = Path('/private/tmp') / ('ocx021-mac-native-' + str(uuid.uuid4()))
        self.container.mkdir(mode=0o700)
        self.root = self.container / 'root'
        self.root.mkdir(mode=0o700)
        self.out = Path(args.out).resolve() / ('ready' if self.ready else '') / self.container.name
        self.out.mkdir(parents=True, mode=0o700)
        self.home, self.tmp = self.root / 'home', self.root / 'tmp'
        for path in [self.home, self.tmp, self.root / 'opencodex-home', self.root / 'manager-state']:
            path.mkdir(mode=0o700)
        self.env = {
            'PATH': '/usr/bin:/bin:/usr/sbin:/sbin', 'LANG': 'en_US.UTF-8',
            'USER': pwd.getpwuid(os.getuid()).pw_name, 'LOGNAME': pwd.getpwuid(os.getuid()).pw_name,
            'HOME': str(self.home), 'CFFIXED_USER_HOME': str(self.home),
            'TMP': str(self.tmp), 'TMPDIR': str(self.tmp) + '/', 'TEMP': str(self.tmp),
            'XDG_CONFIG_HOME': str(self.home / '.config'), 'XDG_CACHE_HOME': str(self.home / '.cache'),
            'OPENCODEX_SANDBOX': '1', 'OPENCODEX_SANDBOX_ROOT': str(self.root),
            'OPENCODEX_SANDBOX_BOUNDARY': str(self.container),
            'OPENCODEX_HOME': str(self.root / 'opencodex-home'), 'CODEX_HOME': str(self.home / '.codex'),
        }
        self.proc = None
        self.direct_groups = []
        self.descendants = {}
        self.stop_observer = threading.Event()
        self.helper = self.container / 'ax'
        self.capwin = self.container / 'capwin'
        self.report = {'schemaVersion': 1, 'stage': args.stage, 'result': 'incomplete',
                       'app': str(self.app), 'executable': str(self.exe), 'bundleInfo': self.info,
                       'binarySha256Before': sha(self.exe), 'bundleInfoSha256Before': sha(self.app / 'Contents/Info.plist'),
                       'root': str(self.root), 'boundary': str(self.container), 'environment': self.env,
                       'rootSource': 'Fresh UUID root; explicit process environment; startup-bound boundary; no daily data inspection',
                       'sourceHeadAtRun': command(['git', '-C', REPO, 'rev-parse', 'HEAD']).stdout.strip(),
                       'sourceDirtyAtRun': command(['git', '-C', REPO, 'status', '--short', '--untracked-files=no']).stdout,
                       'scope': args.scope, 'readyMode': self.ready, 'expectedBuildCommit': args.expected_build_commit,
                       'scriptSha256': sha(Path(__file__)), 'axHelperSourceSha256': sha(HERE / 'ocx021-macos-native.swift'),
                       'signedFinalCommitClaimed': False, 'scopeCompleted': False, 'checks': [], 'startedAt': dt.datetime.now(dt.timezone.utc).isoformat()}
        self.report['codesign'] = command(['/usr/bin/codesign', '-dv', '--verbose=4', self.app]).stderr
        self.flush()

    def flush(self):
        write_json(self.out / 'report.json', self.report)
        write_json(self.out / 'run-context.json', {k: self.report.get(k) for k in (
            'stage', 'result', 'app', 'executable', 'pid', 'processGroup', 'root', 'boundary',
            'rootSource', 'environment', 'permissions', 'binarySha256Before', 'codesign',
            'sourceHeadAtRun', 'startupBuildCommit', 'signedFinalCommitClaimed', 'scope', 'readyMode', 'toolInputs',
            'realToolVersions', 'nativeEnvironmentGate', 'checks')})
        print(json.dumps({'out': str(self.out), 'root': str(self.root), 'pid': self.proc.pid if self.proc else None,
                          'checks': self.report['checks'][-1:]}, ensure_ascii=False), flush=True)

    def check(self, name, status, **facts):
        self.report['checks'].append(dict(name=name, status=status, **facts)); self.flush()

    def prepare(self):
        for source, dest in [(HERE / 'ocx021-macos-native.swift', self.helper), (HERE / 'lib/capwin.swift', self.capwin)]:
            compiled = command(['/usr/bin/xcrun', 'swiftc', source, '-o', dest], env=self.env)
            (self.out / (dest.name + '-compile.log')).write_text(compiled.stdout + compiled.stderr)
            if compiled.returncode: raise RuntimeError('Swift helper compilation failed: ' + dest.name)
        status = json.loads(command([self.helper, 'status'], env=self.env).stdout)
        self.report['permissions'] = status
        self.check('native-permissions', 'pass' if status['axTrusted'] and status['screenCaptureAllowed'] and not status['locked'] else 'blocked', **status)
        preferences = json.loads((REPO / 'apps/desktop/tauri/config/preferences.defaults.json').read_text())
        preferences.update(auto_panel=False, launch_with_codex=False, app_update_auto_check=False, cold_sync=False, cli_enabled=False, startup_cleanup=False)
        write_json(self.root / 'manager-state/preferences.json', preferences)
        provider = {'adapter': 'openai-chat', 'baseUrl': 'https://native021.invalid/v1', 'authMode': 'none',
                    'models': ['vendor/a:8b', 'vendor-a:8b', 'Mixed_Case/V2'],
                    'selectedModels': ['vendor/a:8b', 'vendor-a:8b'], 'newModelPolicy': 'off',
                    'note': 'Synthetic fixture; never query this endpoint', 'enabled': False}
        self.before = {'providers': {'native021-old': provider}, 'defaultProvider': 'native021-old',
                       'modelPickerOrder': ['native021-old/vendor/a:8b', 'native021-old/vendor-a:8b', 'native021-old/Mixed_Case/V2'],
                       'disabledModels': ['native021-old/Mixed_Case/V2'], 'native021Opaque': {'keep': 42}}
        write_json(self.root / 'opencodex-home/config.json', self.before)
        write_json(self.out / 'config-before.json', self.before)
        if self.ready: self.prepare_tools()
        # The candidate can read its own immutable bundle, never the personal home.
        userhome = pwd.getpwuid(os.getuid()).pw_dir
        literal = lambda value: json.dumps(str(value))
        self.profile = self.container / 'candidate.sb'
        self.profile.write_text('(version 1)\n(allow default)\n(deny network*)\n' +
            '(deny file-read* (require-all (subpath ' + literal(userhome) + ') (require-not (subpath ' + literal(self.app) + '))))\n' +
            '(deny file-write* (subpath ' + literal(userhome) + '))\n' +
            '(deny file-read* file-write* (subpath "/Library/Keychains"))\n' +
            '(deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.secd"))\n' +
            ('(deny file-write* (subpath ' + literal(self.tools) + '))\n' if self.ready else ''))
        shutil.copyfile(self.profile, self.out / 'candidate.sb')
        # Admission tests hit only a reserved documentation IP. The kernel must
        # refuse before any connection; no vendor, WebDAV or application request.
        guard = command(['/usr/bin/sandbox-exec', '-f', self.profile, '/usr/bin/nc', '-v', '-G', '1', '-z', '192.0.2.1', '443'], env=self.env)
        (self.out / 'network-guard.log').write_text(guard.stdout + guard.stderr)
        if guard.returncode == 0 or 'Operation not permitted' not in guard.stderr:
            raise Blocked('Kernel network-denial guard was not confirmed')
        self.check('kernel-network-denial', 'pass', returncode=guard.returncode, error=guard.stderr.strip())
        if self.ready: self.probe_tools()
        if not status['axTrusted'] or not status['screenCaptureAllowed'] or status['locked']:
            raise Blocked('Locked session or missing AX/screen-capture permission; UI checks not passed')
        self.flush()

    def prepare_tools(self):
        """Copy real code/binaries; wrappers fix paths only, never produce output."""
        inputs = {}
        for name in ['node', 'npm_cli', 'official_entry', 'bun']:
            raw = getattr(self.args, name)
            if not raw: continue
            source = Path(raw)
            if not source.is_absolute(): raise Blocked('--' + name.replace('_', '-') + ' must be absolute')
            source = source.resolve(strict=True)
            if not source.is_file(): raise Blocked('Tool input is not a file: ' + str(source))
            inputs[name] = {'requested': raw, 'path': str(source), 'sha256': sha(source), 'size': source.stat().st_size}
        self.report['toolInputs'] = inputs
        self.tools = self.root / 'tools'
        self.tools.mkdir(mode=0o700)
        node = Path(inputs['node']['path'])
        npm_cli = Path(inputs['npm_cli']['path'])
        official_entry = Path(inputs['official_entry']['path'])
        for name in ['node', 'bun']:
            if name not in inputs: continue
            path = Path(inputs[name]['path'])
            with path.open('rb') as source: magic = source.read(4)
            if magic not in [b'\xcf\xfa\xed\xfe', b'\xfe\xed\xfa\xcf', b'\xca\xfe\xba\xbe', b'\xbe\xba\xfe\xca'] or path.stat().st_size < 1_000_000:
                raise Blocked(name + ' must be a real macOS binary, not a version shim')
        npm_root = npm_cli.parent.parent
        npm_package = json.loads((npm_root / 'package.json').read_text())
        if npm_cli.name != 'npm-cli.js' or npm_package.get('name') != 'npm':
            raise Blocked('--npm-cli must be the npm package original bin/npm-cli.js')
        official = official_entry.parent.parent
        manifest = json.loads((official / 'package.json').read_text())
        if manifest.get('name') != '@bitkyc08/opencodex' or (official / manifest['bin']['ocx']).resolve() != official_entry:
            raise Blocked('--official-entry must be the published @bitkyc08/opencodex manifest ocx entry')
        source_modules = official.parent.parent
        if source_modules.name != 'node_modules': raise Blocked('Official entry needs its cached node_modules dependencies; no install is performed')
        self.report['toolInputs']['npm_cli']['packageVersion'] = npm_package['version']
        self.report['toolInputs']['official_entry']['packageVersion'] = manifest['version']
        # Copy package code and its complete dependencies; the source cache stays untouched.
        self.copied_node = self.tools / 'node'
        shutil.copy2(node, self.copied_node)
        self.npm_root = self.tools / 'npm'
        self.modules = self.tools / 'official/node_modules'
        for src, dest in [(npm_root, self.npm_root), (source_modules, self.modules)]:
            before = tree_inventory(src)
            shutil.copytree(src, dest, symlinks=True)
            assert tree_inventory(dest) == before, 'Tool tree copy differs from original: ' + str(src)
            label = 'npm' if src == npm_root else 'official-node-modules'
            write_json(self.out / (label + '-code-inventory.json'), before)
        self.copied_npm = self.npm_root / npm_cli.relative_to(npm_root)
        self.copied_official = self.modules / official_entry.relative_to(source_modules)
        self.copied_bun = None
        if self.args.bun:
            self.copied_bun = self.tools / 'bun'
            shutil.copy2(inputs['bun']['path'], self.copied_bun)
        else:
            for candidate in [self.modules / 'bun/bin/bun.exe', self.modules / 'bun/bin/bun']:
                if candidate.is_file() and candidate.stat().st_size >= 1_000_000:
                    self.copied_bun = candidate; break
            if self.copied_bun is None:
                raise Blocked('Cached official Bun is not ready; supply --bun with a real binary. No lazy installation is allowed.')
            original_bun = source_modules / self.copied_bun.relative_to(self.modules)
            self.report['toolInputs']['bundled_bun'] = {'path': str(original_bun.resolve()), 'sha256': sha(original_bun),
                                                       'size': original_bun.stat().st_size, 'selection': 'official bundled runtime'}
        self.bin = self.home / '.local/bin'
        self.bin.mkdir(parents=True, mode=0o700)
        (self.bin / 'node').symlink_to(self.copied_node)
        (self.bin / 'bun').symlink_to(self.copied_bun)
        # All mutable inputs are fixed inside this run, including npm config/cache.
        for name in ['empty-user.npmrc', 'empty-global.npmrc']:
            (self.root / name).write_text('')
        fixed = {k: v for k, v in self.env.items() if k in [
            'HOME', 'CFFIXED_USER_HOME', 'OPENCODEX_HOME', 'CODEX_HOME', 'XDG_CONFIG_HOME', 'XDG_CACHE_HOME', 'TMPDIR', 'TMP', 'TEMP', 'LANG']}
        fixed.update(PATH=str(self.bin) + ':/usr/bin:/bin:/usr/sbin:/sbin',
                     XDG_DATA_HOME=str(self.home / '.local/share'), XDG_STATE_HOME=str(self.home / '.local/state'),
                     npm_config_userconfig=str(self.root / 'empty-user.npmrc'), npm_config_globalconfig=str(self.root / 'empty-global.npmrc'),
                     npm_config_cache=str(self.root / 'npm-cache'), npm_config_update_notifier='false')
        if self.args.bun: fixed['OPENCODEX_BUN_PATH'] = str(self.copied_bun)
        prefix = '#!/bin/sh\nexec /usr/bin/env -i ' + shlex.join([k + '=' + v for k, v in fixed.items()]) + ' '
        for name, entry in [('npm', self.copied_npm), ('ocx', self.copied_official)]:
            path = self.bin / name
            path.write_text(prefix + shlex.join([str(self.copied_node), str(entry)]) + ' "$@"\n')
            path.chmod(0o700)
        self.tool_hashes = {str(p): sha(p) for p in [self.copied_node, self.copied_npm, self.copied_official, self.copied_bun]}
        assert sha(self.copied_node) == inputs['node']['sha256']
        assert sha(self.copied_npm) == inputs['npm_cli']['sha256']
        assert sha(self.copied_official) == inputs['official_entry']['sha256']
        write_json(self.out / 'ready-tools.json', {'inputs': inputs, 'discoveryDirectory': str(self.bin), 'fixedEnvironment': fixed,
            'copiedHashes': self.tool_hashes, 'npmWrapper': (self.bin / 'npm').read_text(), 'officialWrapper': (self.bin / 'ocx').read_text(),
            'runtimeSourceSeeded': False, 'packagesInstalled': False, 'sourceTreesUnmodified': True})
        self.flush()

    def owned_probe(self, argv, label, seconds=25):
        """A short tool probe owns its process group, including Node->Bun children."""
        child = subprocess.Popen(['/usr/bin/sandbox-exec', '-f', str(self.profile), *map(str, argv)], env=self.env,
                                 cwd=self.root, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True, text=True)
        record = {'pid': child.pid, 'pgid': child.pid, 'label': label, 'argv': list(map(str, argv))}
        self.direct_groups.append(record)
        stdout, stderr = '', ''
        try:
            stdout, stderr = child.communicate(timeout=seconds)
            record.update(returncode=child.returncode, stdout=stdout.strip(), stderr=stderr.strip())
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGKILL)
            stdout, stderr = child.communicate(timeout=5)
            record.update(returncode=child.returncode, timeout=True)
            raise Blocked('Real tool probe timed out: ' + label)
        except BaseException:
            if child.poll() is None: os.killpg(child.pid, signal.SIGKILL)
            stdout, stderr = child.communicate(timeout=5)
            record.update(returncode=child.returncode, interrupted=True)
            raise
        finally:
            (self.out / (label + '.stdout')).write_text(stdout)
            (self.out / (label + '.stderr')).write_text(stderr)
            remaining = [p for p in processes() if p['pgid'] == child.pid]
            if remaining:
                os.killpg(child.pid, signal.SIGKILL)
                time.sleep(.1)
            record['remaining'] = [p for p in processes() if p['pgid'] == child.pid]
            write_json(self.out / 'owned-tool-probes.json', self.direct_groups)
        if child.returncode != 0: raise Blocked(label + ' failed: ' + stderr[-1200:])
        return record

    def probe_tools(self):
        versions = {}
        for name in ['node', 'npm', 'bun', 'ocx']:
            record = self.owned_probe([self.bin / name, '--version'], name + '-version')
            if not re.search(r'(?<!\d)\d+\.\d+\.\d+', record['stdout']): raise Blocked('Original ' + name + ' did not return a real version')
            versions[name] = record['stdout']
        self.report['realToolVersions'] = versions
        write_json(self.out / 'tool-versions.json', versions)
        self.check('ready-real-tool-versions', 'pass', versions=versions, codeBytesUnmodified=True,
                   proof='Original binaries/CLI copied with equal SHA; fixed-environment wrappers; OS network denied; no service commands')

    def observe_descendants(self):
        while not self.stop_observer.is_set():
            try:
                rows = processes()
                known = {self.proc.pid, *self.descendants.keys()}
                while True:
                    children = [p for p in rows if p['ppid'] in known and p['pid'] not in known and p['pid'] != 80916]
                    if not children: break
                    for p in children:
                        known.add(p['pid']); self.descendants[p['pid']] = p
            except Exception: pass
            self.stop_observer.wait(.15)

    def launch(self):
        self.log = open(self.out / 'candidate-stdout.log', 'wb')
        self.proc = subprocess.Popen(['/usr/bin/sandbox-exec', '-f', str(self.profile), str(self.exe)], env=self.env,
                                     cwd=self.root, stdout=self.log, stderr=subprocess.STDOUT, start_new_session=True)
        self.report['pid'] = self.proc.pid
        self.report['processGroup'] = os.getpgid(self.proc.pid)
        if self.ready:
            self.observer = threading.Thread(target=self.observe_descendants, daemon=True)
            self.observer.start()
        write_json(self.out / 'owned-process.json', {'pid': self.proc.pid, 'exe': str(self.exe), 'root': str(self.root), 'helper': str(self.helper), 'capwin': str(self.capwin)})
        time.sleep(2)
        if self.proc.poll() is not None: raise RuntimeError('Candidate exited: ' + str(self.proc.returncode))
        # Only our environment is printed; no global ps eww or daily process inspection.
        observed = command(['/bin/ps', 'eww', '-p', self.proc.pid, '-o', 'pid=,pgid=,command=']).stdout
        (self.out / 'process-environment.txt').write_text(observed)
        required = ['HOME', 'TMP', 'TMPDIR', 'OPENCODEX_SANDBOX', 'OPENCODEX_SANDBOX_ROOT', 'OPENCODEX_SANDBOX_BOUNDARY']
        if not all(k + '=' + self.env[k] in observed for k in required): raise RuntimeError('Process-level sandbox environment not confirmed')
        self.check('process-sandbox-admission', 'pass', pid=self.proc.pid, pgid=os.getpgid(self.proc.pid), required=required)
        deadline = time.monotonic() + 25
        while time.monotonic() < deadline:
            snap = self.snapshot('startup')
            if any(n.get('role') == 'AXWebArea' for n in snap['nodes']): break
            if self.proc.poll() is not None: raise RuntimeError('Candidate exited during window creation')
            time.sleep(0.6)
        windows = [w for w in snap['windows'] if w['onscreen'] and w['layer'] == 0]
        self.check('native-window', 'pass' if windows and any(n.get('role') == 'AXWebArea' for n in snap['nodes']) else 'blocked', windows=windows, axNodes=len(snap['nodes']))
        if windows: self.capture('startup', snap)
        for path in self.root.rglob('app.log'):
            shutil.copyfile(path, self.out / ('startup-' + path.parent.name + '.log'))
        log = self.root / 'logs/app.log'
        if log.exists():
            startup = next((line for line in log.read_text().splitlines() if 'startup version=' in line), '')
            self.report['startupEvent'] = startup
            self.report['startupBuildCommit'] = startup.split('commit=')[-1] if 'commit=' in startup else None
        if self.args.expected_build_commit:
            assert self.report.get('startupBuildCommit') == self.args.expected_build_commit, 'Embedded startup build commit differs from the expected final commit'
            self.check('expected-build-commit', 'pass', expected=self.args.expected_build_commit)
        self.binding_before = (self.root / 'manager-state/data-root.json').read_bytes()
        binding = json.loads(self.binding_before)
        assert binding.get('structure_version') == '1'
        assert Path(binding.get('active_data_root', self.root)).resolve() == self.root
        assert binding.get('opencodex_home_mode', 'inside') == 'inside'
        self.check('startup-root-binding', 'pass', binding=binding, resolvedActiveRoot=str(self.root),
                   resolvedOfficialHome=str(self.root / 'opencodex-home'), startup=self.report.get('startupEvent'))
        self.ax('focus')
        self.flush()

    def ax(self, verb, *args):
        if verb != 'snapshot':
            # Never record typed values (even synthetic passwords) in the action journal.
            with (self.out / 'actions.jsonl').open('a') as journal:
                journal.write(json.dumps({'at': time.monotonic(), 'verb': verb, 'pathOrKey': args[0] if args else None}, ensure_ascii=False) + '\n')
        result = command([self.helper, verb, self.proc.pid, self.exe, *args], env=self.env)
        if result.returncode: raise Blocked('AX ' + verb + ': ' + result.stdout + result.stderr)
        return json.loads(result.stdout)

    def snapshot(self, name=None):
        result = self.ax('snapshot')
        if name: write_json(self.out / (name + '-ax.json'), result)
        return result

    def capture(self, name, snap=None):
        snap = snap or self.snapshot(name)
        windows = [w for w in snap['windows'] if w['onscreen'] and w['layer'] == 0 and w['bounds'].get('Width', 0) > 500]
        if not windows: raise RuntimeError('No visible candidate-owned main window')
        window = windows[0]
        result = command([self.capwin, window['id'], self.out / (name + '.png')], env=self.env)
        (self.out / (name + '-capture.log')).write_text(result.stdout + result.stderr)
        if result.returncode: raise RuntimeError('Candidate-window screenshot failed')
        return window

    def wait(self, predicate, seconds=8, name=None):
        end = time.monotonic() + seconds
        while True:
            snap = self.snapshot()
            if snap.get('locked'): raise Blocked('Screen locked during native acceptance')
            value = predicate(snap)
            if value:
                if name: write_json(self.out / (name + '-ax.json'), snap)
                return snap
            if time.monotonic() > end:
                self.snapshot('timeout-' + (name or 'target'))
                raise AssertionError('Native AX target/state not reached: ' + (name or str(predicate)))
            time.sleep(0.12)

    @staticmethod
    def matches(snap, label, roles=BUTTON_ROLES):
        webroots = [n['path'] + '.' for n in snap['nodes'] if n.get('role') == 'AXWebArea']
        return [n for n in snap['nodes'] if any(n['path'].startswith(p) for p in webroots) and n.get('role') in roles and
                (n.get('title') == label or n.get('description') == label)]

    def press(self, label, index=0):
        snap = self.wait(lambda s: len(self.matches(s, label)) > index, name='target')
        n = self.matches(snap, label)[index]
        assert n.get('enabled', True), 'Disabled control: ' + label
        self.ax('press', n['path'], label)

    def set_text(self, label, value, secure=False):
        snap = self.wait(lambda s: self.matches(s, label, ('AXTextField',)), name='input')
        n = self.matches(snap, label, ('AXTextField',))[0]
        self.ax('type' if secure else 'set', n['path'], value, label)
        if secure: time.sleep(.3)  # Allow real queued key events to reach the secure WKWebView field.
        if not secure:
            self.wait(lambda s: any(n.get('value') == value for n in self.matches(s, label, ('AXTextField',))), name='input-readback')

    @staticmethod
    def dialogs(snap):
        return [n for n in snap['nodes'] if n.get('subrole') in ('AXApplicationDialog', 'AXApplicationAlertDialog')]

    @staticmethod
    def text(snap):
        return '\n'.join(str(n.get('value') or n.get('title') or '') for n in snap['nodes'])

    def dismiss(self, how='x'):
        if how == 'x': self.press('关闭')
        elif how == 'cancel': self.press('取消')
        elif how == 'escape': self.ax('key', '53')
        else: raise ValueError(how)
        return self.wait(lambda s: not self.dialogs(s), name='dismiss-' + how)

    def channels(self):
        self.press('模型')
        self.press('渠道模型')
        return self.wait(lambda s: self.matches(s, '编辑'), name='channel-list')

    def node_detection(self):
        self.channels()
        before = (self.root / 'opencodex-home/config.json').read_bytes()
        scenes = []
        for how in ['x', 'escape']:
            self.press('检测 native021-old 服务节点')
            snap = self.wait(lambda s: self.dialogs(s), name='node-detection-' + how)
            dialog = self.dialogs(snap)[0]
            web = next(n for n in snap['nodes'] if n.get('role') == 'AXWebArea')['frame']
            box = dialog['frame']
            dx = box['x'] + box['width'] / 2 - web['x'] - web['width'] / 2
            dy = box['y'] + box['height'] / 2 - web['y'] - web['height'] / 2
            assert abs(dx) <= 3 and abs(dy) <= 3, 'Detection is not centered in the real WKWebView'
            assert len(self.matches(snap, '关闭')) == 1
            assert not self.matches(snap, '取消') and not self.matches(snap, '保存')
            assert '未检测' in self.text(snap)
            self.capture('node-detection-' + how, snap)
            time.sleep(1)
            assert '未检测' in self.text(self.snapshot('node-detection-unmeasured-' + how))
            self.dismiss(how)
            assert (self.root / 'opencodex-home/config.json').read_bytes() == before
            scenes.append({'exit': how, 'dialog': box, 'webview': web, 'centerDelta': [dx, dy], 'status': '未检测'})
        self.check('node-detection-centered-unmeasured', 'pass', scenes=scenes,
                   proof='AX invocation opens observation only; no measurement action; kernel denies network')

    def edit_cancel(self):
        self.channels(); self.press('编辑')
        self.wait(lambda s: self.matches(s, '编辑明细'), name='channel-editor')
        self.press('编辑明细')
        self.wait(lambda s: self.dialogs(s), name='edit-before')
        self.set_text('路由别名', 'native021-cancelled-alias')
        snap = self.snapshot('edit-cancel'); self.capture('edit-cancel', snap)
        assert len(self.matches(snap, '关闭')) == 1 and self.matches(snap, '取消')
        self.dismiss('cancel')
        assert all(n.get('value') != 'native021-cancelled-alias' for n in self.snapshot()['nodes'])
        self.press('编辑明细')
        snap = self.wait(lambda s: self.dialogs(s), name='edit-reopened')
        assert all(n.get('value') != 'native021-cancelled-alias' for n in snap['nodes'])
        self.dismiss('x'); self.press('返回渠道列表')
        assert json.loads((self.root / 'opencodex-home/config.json').read_bytes()) == self.before
        self.check('public-edit-cancel', 'pass', typedAliasDiscarded=True, reopenedClean=True, fileUnchanged=True, headerX=1)

    def view_modal(self, provider='native021-old'):
        self.press('同步')
        self.press('文件同步')
        snap = self.wait(lambda s: self.matches(s, '展开 渠道与模型') or self.matches(s, '收起 渠道与模型'))
        if self.matches(snap, '展开 渠道与模型'): self.press('展开 渠道与模型')
        self.press(provider)
        snap = self.wait(lambda s: self.dialogs(s), name='view-no-footer')
        dialog = self.dialogs(snap)[0]
        buttons = [n for n in snap['nodes'] if n['path'].startswith(dialog['path'] + '.') and n.get('role') == 'AXButton']
        assert [n.get('title') for n in buttons] == ['关闭'], 'Read-only detail has footer/action controls'
        assert not any(n.get('subrole') == 'AXLandmarkContentInfo' for n in snap['nodes'])
        self.capture('view-no-footer', snap)
        self.dismiss('x')
        self.check('public-view-no-footer-x', 'pass', surface='production configuration object detail',
                   title=dialog['title'], dialogButtons=['关闭'], footerLandmarks=0, proof='AX + candidate-window PNG')

    def rename_save(self, target):
        self.channels(); self.press('编辑')
        self.wait(lambda s: self.matches(s, '渠道名称', ('AXTextField',)))
        self.set_text('渠道名称', target)
        snap = self.wait(lambda s: any(n.get('value') == target for n in self.matches(s, '渠道名称', ('AXTextField',))), name='rename-draft')
        assert self.matches(snap, '保存渠道草稿')[0]['enabled']
        self.ax('focus')
        nav = self.matches(self.snapshot(), '概览')[0]
        self.ax('move', nav['path'], '概览')
        snap = self.snapshot()
        if not snap.get('active'): raise Blocked('Candidate could not remain foreground for timing')
        # Begin read-only sampling before Save: the previous clock started only
        # after Save's wait/AX round trips and lost part of the real countdown.
        self.start_notice_observation()
        try:
            self.press('保存渠道草稿')
            return self.wait(lambda s: self.matches(s, '立即重启 Codex'), name='save-notice-first')
        except BaseException:
            self.stop_notice_observation()
            raise

    def start_notice_observation(self):
        self.notice_reader = NoticeAxReader(self.proc.pid)
        self.notice_origin = time.monotonic()
        self.notice_samples, self.notice_error = [], None
        self.notice_stop = threading.Event()
        sampled = threading.Event()
        def observe():
            try:
                while not self.notice_stop.is_set():
                    begin = time.monotonic()
                    state = self.notice_reader.sample()
                    end = time.monotonic()
                    self.notice_samples.append({'readBegin': begin - self.notice_origin, 'readEnd': end - self.notice_origin,
                        'monotonicReadBegin': begin, 'monotonicReadEnd': end, 'time': end - self.notice_origin,
                        **state})
                    sampled.set()
                    self.notice_stop.wait(.01)
            except BaseException as error:
                self.notice_error = error
                sampled.set()
        self.notice_thread = threading.Thread(target=observe, daemon=True)
        self.notice_thread.start()
        if not sampled.wait(timeout=10):
            self.stop_notice_observation(); raise Blocked('No initial notice timing sample')
        if self.notice_error:
            self.stop_notice_observation(); raise self.notice_error
        if self.notice_samples[-1]['notice']:
            self.stop_notice_observation(); raise Blocked('A prior notice was already present before the measured Save')

    def stop_notice_observation(self):
        self.notice_stop.set()
        self.notice_thread.join(timeout=10)
        if self.notice_thread.is_alive(): raise Blocked('Timing observer did not stop')
        if self.notice_reader:
            self.notice_reader.close(); self.notice_reader = None

    def notice_timing(self, confirm=False):
        operations, exits, measurement = [], [], None
        start = self.notice_origin
        try:
            snap = self.snapshot('notice-start-' + str(confirm))
            assert '8秒' in self.text(snap)
            assert not any(n.get('focused') and n.get('title') in ('立即重启 Codex', '稍后') for n in snap['nodes']), 'Save notice stole focus'
            self.capture('notice-start-' + str(confirm), snap)
        except BaseException:
            self.stop_notice_observation()
            raise
        def sample():
            if self.notice_error: raise self.notice_error
            s = self.snapshot()
            if not s.get('active'): raise Blocked('Candidate lost foreground; eight-second elapsed result is invalid')
            if s.get('locked'): raise Blocked('Screen locked during notice timing')
            return s
        try:
            if confirm:
                # Consume real time relative to the observed appearance, not a
                # late timer-method entry. The observer keeps sampling during AX.
                first = next(s for s in self.notice_samples if s['notice'])
                until = start + first['readEnd'] + 1.6
                while time.monotonic() < until: sample(); time.sleep(.1)
                for how in ['x', 'escape', 'cancel']:
                    pause_start = time.monotonic()
                    actual_exit = how
                    self.press('立即重启 Codex')
                    self.wait(lambda s: any(d.get('title') == '重启 Codex' for d in self.dialogs(s)), name='restart-confirm-' + how)
                    self.capture('restart-confirm-' + how)
                    until = time.monotonic() + 1.4
                    while time.monotonic() < until:
                        s = sample()
                        assert any(d.get('title') == '重启 Codex' for d in self.dialogs(s))
                        time.sleep(.1)
                    try:
                        self.dismiss(how)
                    except AssertionError:
                        if how != 'escape': raise
                        self.snapshot('restart-confirm-escape-unclosed')
                        self.capture('restart-confirm-escape-unclosed')
                        self.check('save-notice-confirm-escape', 'fail', reason='PID-directed Escape did not close the real restart confirmation within 8 seconds; captured before recovery with header X. No restart was confirmed.')
                        self.dismiss('x')
                        actual_exit = 'header-X-recovery-after-Escape-timeout'
                    resumed = sample()
                    assert self.matches(resumed, '立即重启 Codex'), 'Cancel lost the saved notification'
                    operations.append({'requestedExit': how, 'actualExit': actual_exit, 'start': pause_start - start, 'end': time.monotonic() - start,
                                       'usedAsPausedDuration': False})
                    exits.append(actual_exit)
                    # No blur/focus/move after cancellation: the product must resume itself.
            while time.monotonic() - start < 45:
                s = sample()
                if not self.matches(s, '立即重启 Codex'): break
                time.sleep(.1)
            else: raise AssertionError('Saved notification failed to collapse after confirmation cancellation')
            # Ensure the read-only observer itself recorded the disappearance;
            # its brackets, not this slower action-side poll, determine the end.
            until = time.monotonic() + 3
            while time.monotonic() < until:
                tail = self.notice_samples[-2:]
                if len(tail) == 2 and all(not s['notice'] and not s['dialogs'] for s in tail): break
                time.sleep(.02)
            self.stop_notice_observation()
            if self.notice_error: raise self.notice_error
            measurement = notice_clock_bounds(self.notice_samples, exits)
            low, high = measurement['activeIntervalSeconds']
            if not (6.7 <= low <= high <= 9.3):
                if high < 6.7 or low > 9.3:
                    raise AssertionError('Native countdown interval is outside 6.7..9.3 seconds: ' + str([low, high]))
                raise Blocked('Native timing uncertainty crosses unchanged 6.7..9.3 threshold: ' + str([low, high]))
            elapsed = sum(measurement['elapsedIntervalSeconds']) / 2
            active = measurement['activeMidpointSeconds']
            self.check('save-notice-confirm-cancel-remainder' if confirm else 'save-notice-eight-seconds', 'pass',
                       elapsedSeconds=elapsed, estimatedActiveSeconds=active, confirmationPauses=measurement['confirmationPauses'],
                       confirmationExitResults=operations,
                       measurement=measurement, clock='real monotonic AX observation brackets; full active interval fits unchanged threshold',
                       restartConfirmed=False, manualBlurAfterCancel=False, trace='notice-trace-' + str(confirm) + '.json')
        finally:
            self.stop_notice_observation()
            write_json(self.out / ('notice-trace-' + str(confirm) + '.json'), {'schemaVersion': 2,
                'monotonicOrigin': start, 'samples': self.notice_samples, 'inputOperationSpans': operations,
                'measurement': measurement, 'observerError': str(self.notice_error) if self.notice_error else None,
                'legacyOperationSpansAreNotDialogPauses': True,
                'observationMethod': 'read-only in-process native AX attributes; no frames/screenshots/process launch in the sampler',
                'remainingCounterExposedByAx': False})

    def renamed_readback(self, original, target):
        config = self.root / 'opencodex-home/config.json'
        actual = json.loads(config.read_bytes())
        expected = json.loads(json.dumps(original))
        old = original['defaultProvider']
        expected['providers'][target] = expected['providers'].pop(old)
        expected['defaultProvider'] = target
        for key in ['modelPickerOrder', 'disabledModels']:
            expected[key] = [target + v[len(old):] if v.startswith(old + '/') else v for v in expected[key]]
        assert actual == expected, 'Direct rename did not preserve exact model IDs/references/unrelated data'
        write_json(self.out / ('config-after-' + target + '.json'), actual)
        assert not (self.root / 'manager-state/provider-rename-pending.json').exists()
        self.channels(); self.press('编辑')
        snap = self.wait(lambda s: self.matches(s, '渠道名称', ('AXTextField',)), name='rename-reopened-' + target)
        assert self.matches(snap, '渠道名称', ('AXTextField',))[0]['value'] == target
        runtime_fields = self.matches(snap, '运行渠道标识', ('AXTextField',))
        assert not runtime_fields or runtime_fields[0]['value'] == target
        assert not self.matches(snap, '保存渠道草稿')[0]['enabled']
        self.capture('rename-reopened-' + target, snap)
        self.press('返回渠道列表')
        self.check('channel-rename-file-readback-' + target, 'pass', provider=target, file=str(config),
                   configSha256=sha(config), modelIds=actual['providers'][target]['models'], exactReferencesPreserved=True,
                   reopenedClean=True, systemKeychainTest=False)
        return actual

    def webdav(self):
        self.press('同步'); self.press('WebDAV 同步'); self.press('连接设置')
        self.wait(lambda s: self.dialogs(s), name='connection-initial')
        self.ax('focus')
        for label, value in [('服务地址', 'https://webdav.native021.invalid'), ('独立配置同步目录', 'native021-only'),
                             ('用户名', 'synthetic-native021'), ('连接密码', 'synthetic-native021-webdav-password')]:
            self.set_text(label, value, secure=label == '连接密码')
        self.press('保存连接')
        self.wait(lambda s: not self.dialogs(s), name='connection-saved')
        path = self.root / 'manager-state/sync-endpoints.json'
        config = json.loads(path.read_bytes())
        endpoint = config['endpoints'][0]
        assert endpoint['url'].rstrip('/') == 'https://webdav.native021.invalid'
        assert endpoint['username'] == 'synthetic-native021' and endpoint['remote_path'] == 'native021-only'
        write_json(self.out / 'saved-webdav-endpoint.json', config)
        before = path.read_bytes()
        self.press('连接设置')
        snap = self.wait(lambda s: '已配置 · 未检测' in self.text(s), name='webdav-configured-untested')
        self.capture('webdav-configured-untested', snap)
        assert self.matches(snap, '服务地址', ('AXTextField',))[0]['value'].rstrip('/') == endpoint['url'].rstrip('/')
        assert not any(token in self.text(snap) for token in ['检测成功', '最近同步成功', '连接失败'])
        self.check('webdav-configured-untested', 'pass', state='已配置 · 未检测', savedBy='real AX form -> native IPC', actualEndpointFile=str(path), networkTest=False, sync=False)
        # Empty candidates deliberately render nothing after their local backend
        # read. AX proves settled absence, not a DOM loading-event observation.
        time.sleep(1)
        snap = self.snapshot('webdav-empty-candidates'); self.capture('webdav-empty-candidates', snap)
        assert not any(x in self.text(snap) for x in ['导入的WebDAV连接', '启用导入的WebDAV连接', '正在读取导入连接'])
        assert not (self.root / 'manager-state/imported-sync-connections.json').exists()
        self.check('webdav-empty-candidates', 'pass', visibleCandidateRows=0, localReadSettledSeconds=1,
                   proof='Fresh owned root + AX absence after local load; loading transition is not instrumented')
        self.dismiss('x'); assert path.read_bytes() == before
        self.press('概览')
        snap = self.wait(lambda s: '0 / 3' in self.text(s) or '已配置 · 未检测' in self.text(s), name='webdav-overview')
        self.capture('webdav-overview', snap)
        if '已配置 · 未检测' in self.text(snap):
            scene = self.webdav_scene(snap)
            assert not any(token in self.text(snap) for token in ['最近检测成功', '最近同步成功', '连接失败', '连接成功'])
            self.check('webdav-overview-card', 'pass', state='已配置 · 未检测', gate=self.report.get('nativeEnvironmentGate'),
                       noRemoteSuccessOrFailureClaim=True, networkTest=False, sync=False, scene=scene)
        else: self.check('webdav-overview-card', 'blocked', reason='Real isolated root has no Node.js/npm/runtime; production environment gate hides overview status cards. No DEV state or mock bypass.')

    @staticmethod
    def webdav_scene(snap):
        marker = next(n for n in snap['nodes'] if n.get('role') == 'AXStaticText' and n.get('value') == '线上版本')
        row = marker['path'].rsplit('.', 1)[0]
        card = row.rsplit('.', 1)[0]
        values = [n['value'] for n in snap['nodes'] if n['path'].startswith(row + '.') and n.get('role') == 'AXStaticText' and n.get('value')]
        card_values = [n.get('value') for n in snap['nodes'] if n['path'].startswith(card + '.') and n.get('role') == 'AXStaticText']
        assert values == ['线上版本', '—', '尚未读取'], 'Overview fabricated or lost the unobserved remote-version state: ' + str(values)
        assert 'WebDAV 同步' in card_values and '连接状态' in card_values and '已配置 · 未检测' in card_values
        return {'connectionState': '已配置 · 未检测', 'remoteVersionRow': values, 'cardAxPath': card}

    def ready_environment(self):
        self.press('概览')
        try:
            self.wait(lambda s: self.matches(s, '查看运行详情，包含运行环境'), seconds=15, name='ready-overview')
        except AssertionError as exc:
            snap = self.snapshot('ready-gate-unreachable'); self.capture('ready-gate-unreachable', snap)
            raise Blocked('Production environment remains gated after real tools were provisioned: ' + self.text(snap)[-2600:]) from exc
        self.press('查看运行详情，包含运行环境')
        snap = self.wait(lambda s: self.dialogs(s), name='ready-environment-detail')
        values = [n.get('value') for n in snap['nodes'] if n.get('role') == 'AXStaticText' and n.get('value')]
        checks = []
        for label in ['Node.js', 'npm', 'OpenCodex']:
            index = values.index(label)
            state = values[index + 1]
            if state != '已发现': raise Blocked('Production environment ' + label + ' is ' + str(state) + '; source fallback is not gate=ready')
            checks.append({'name': label, 'value': state})
        # Only all three found results map to ready. A missing_ocx source fallback
        # renders "运行来源", which is explicitly rejected above. Observe the
        # candidate's production IPC result via AX, without reading/writing Pinia
        # or replacing the backend response. This is not a raw DTO export.
        if '完整环境指引' in self.text(snap): raise Blocked('Production runtime detail still exposes the blocked environment guide')
        self.capture('ready-environment-detail', snap)
        runtime_path = self.root / 'manager-state/runtime.json'
        runtime = json.loads(runtime_path.read_bytes())
        expected_entry = self.bin / 'ocx'
        assert runtime['source'] == 'discovered' and Path(runtime['path']).resolve() == expected_entry.resolve(), 'Manager did not select the owned real CLI through production discovery'
        write_json(self.out / 'production-runtime-source.json', runtime)
        gate = {'gate': 'ready', 'producer': 'candidate discover_environment(request=null)',
                'observationMethod': 'production response rendered as three AX found rows; raw DTO is not exported by AX',
                'checks': checks, 'sourceFallbackUsed': False, 'runtimeSource': runtime,
                'runtimeSourceSeeded': False, 'discoveryDirectory': str(self.bin), 'realVersions': self.report['realToolVersions']}
        write_json(self.out / 'native-environment-gate.json', gate)
        self.report['nativeEnvironmentGate'] = gate
        self.check('native-discover-environment-ready', 'pass', **gate)
        self.dismiss('x')
        self.capture('ready-overview-before-webdav')

    def overview(self):
        if not self.ready: raise Blocked('--scope overview requires --node, --npm-cli and --official-entry together')
        self.ready_environment()
        self.webdav()
        self.ready_environment()
        assert any(c['name'] == 'webdav-overview-card' and c['status'] == 'pass' for c in self.report['checks']), 'Ready overview WebDAV card was not actually verified'
        snap = self.snapshot('ready-overview-final'); self.capture('ready-overview-final', snap)
        scene = self.webdav_scene(snap)
        self.check('AC05-ready-overview', 'pass', gate='ready', webdav='已配置 · 未检测', environmentInjected=False,
                   officialServiceStarted=False, webdavObservationRequested=False, candidateStage=self.args.stage, scene=scene)

    def run_case(self, name, callback):
        try: callback()
        except Exception as exc:
            self.check(name, 'blocked' if isinstance(exc, Blocked) else 'fail', reason=str(exc))
            (self.out / (name + '-exception.log')).write_text(traceback.format_exc())
            try:
                snap = self.snapshot(name + '-failure'); self.capture(name + '-failure', snap)
                if self.dialogs(snap): self.dismiss('x')
            except Exception: pass

    def verify(self):
        if self.args.scope == 'overview':
            self.run_case('ready-overview', self.overview)
            return
        if self.ready: self.run_case('ready-environment', self.ready_environment)
        if self.args.scope == 'notice':
            def notice_only():
                self.rename_save('native021-notice'); self.notice_timing(confirm=True)
            self.run_case('save-notice-cancellation', notice_only)
            self.run_case('channel-rename-readback', lambda: self.renamed_readback(self.before, 'native021-notice'))
            return
        self.run_case('node-detection', self.node_detection)
        self.run_case('edit-cancel', self.edit_cancel)
        self.run_case('view-no-footer', self.view_modal)
        def first_save():
            self.rename_save('native021-new'); self.notice_timing()
        self.run_case('save-notice-baseline', first_save)
        self.run_case('channel-rename-readback', lambda: self.renamed_readback(self.before, 'native021-new'))
        current = json.loads((self.root / 'opencodex-home/config.json').read_bytes())
        def second_save():
            self.rename_save('native021-confirm'); self.notice_timing(confirm=True)
        self.run_case('save-notice-cancellation', second_save)
        self.run_case('channel-confirm-readback', lambda: self.renamed_readback(current, 'native021-confirm'))
        self.run_case('webdav', self.webdav)

    def close(self):
        if self.proc:
            # Popen handle + unreaped owned group; never pgrep/pkill/app-name quit.
            if self.proc.poll() is None:
                os.killpg(self.proc.pid, signal.SIGTERM)
                try: self.proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(self.proc.pid, signal.SIGKILL); self.proc.wait(timeout=5)
            self.log.close()
            self.report['ownedProcessExit'] = self.proc.returncode
            # Check the whole owned group even if its leader already exited.
            def group_members():
                rows = command(['/bin/ps', '-axo', 'pid=,ppid=,pgid=,comm=']).stdout.splitlines()
                return [{'pid': int(p[0]), 'ppid': int(p[1]), 'pgid': int(p[2]), 'command': p[3]}
                        for row in rows if len(p := row.split(None, 3)) == 4 and int(p[2]) == self.proc.pid]
            remaining = group_members()
            if remaining:
                # A re-used group leader with another executable is not owned.
                leader = next((p for p in remaining if p['pid'] == self.proc.pid), None)
                if leader and leader['command'] != str(self.exe):
                    self.check('owned-process-group-cleanup', 'blocked', reason='PGID was reused; refusing to signal an unrelated leader', remaining=remaining)
                else:
                    os.killpg(self.proc.pid, signal.SIGTERM)
                    until = time.monotonic() + 3
                    while group_members() and time.monotonic() < until: time.sleep(.1)
                    if group_members(): os.killpg(self.proc.pid, signal.SIGKILL)
                    remaining = group_members()
                    self.check('owned-process-group-cleanup', 'pass' if not remaining else 'blocked', pgid=self.proc.pid, remaining=remaining)
            else: self.check('owned-process-group-cleanup', 'pass', pgid=self.proc.pid, remaining=[])
        if self.ready:
            self.stop_observer.set()
            if hasattr(self, 'observer'): self.observer.join(timeout=3)
            descendants = list(self.descendants.values())
            groups = {p['pgid'] for p in descendants if p['pgid'] not in [0, 1, 80916, os.getpgrp(), self.proc.pid if self.proc else None]}
            cleanup = []
            for pgid in groups:
                remaining = [p for p in processes() if p['pgid'] == pgid]
                if remaining:
                    # Only a directly observed descendant is eligible for cleanup.
                    if any(p['pid'] not in self.descendants for p in remaining):
                        cleanup.append({'pgid': pgid, 'status': 'blocked-reused-or-unobserved', 'remaining': remaining}); continue
                    os.killpg(pgid, signal.SIGTERM)
                    time.sleep(.15)
                    if any(p['pgid'] == pgid for p in processes()): os.killpg(pgid, signal.SIGKILL)
                cleanup.append({'pgid': pgid, 'remaining': [p for p in processes() if p['pgid'] == pgid]})
            write_json(self.out / 'owned-descendants-cleanup.json', {'observedDescendants': descendants, 'groups': cleanup, 'directVersionProbes': self.direct_groups})
            direct_pgids = {p['pgid'] for p in self.direct_groups}
            remaining = [p for p in processes() if p['pid'] in self.descendants or p['pgid'] in groups | direct_pgids]
            self.check('owned-tool-descendants-cleanup', 'pass' if not remaining else 'blocked', remaining=remaining)
            if hasattr(self, 'tool_hashes'):
                unchanged = all(sha(Path(path)) == value for path, value in self.tool_hashes.items())
                unchanged = unchanged and all(sha(Path(value['path'])) == value['sha256'] for value in self.report['toolInputs'].values())
                self.check('ready-original-tools-unchanged', 'pass' if unchanged else 'fail', copiedHashes=self.tool_hashes,
                           sourceInputHashes={k: v['sha256'] for k, v in self.report['toolInputs'].items()})
        if hasattr(self, 'binding_before'):
            binding = (self.root / 'manager-state/data-root.json').read_bytes()
            self.check('startup-root-source-unchanged', 'pass' if binding == self.binding_before else 'fail',
                       source='Process environment + startup binding; no root rebind invoked', bindingSha256=hashlib.sha256(binding).hexdigest())
        for source in [self.root / 'logs/app.log', self.root / 'manager-state/model-channel-layout.json']:
            if source.exists(): shutil.copyfile(source, self.out / ('final-' + source.name))
        self.report['binarySha256After'] = sha(self.exe)
        self.report['bundleInfoSha256After'] = sha(self.app / 'Contents/Info.plist')
        self.check('candidate-identity-unchanged', 'pass' if self.report['binarySha256Before'] == self.report['binarySha256After'] and self.report['bundleInfoSha256Before'] == self.report['bundleInfoSha256After'] else 'fail')
        self.report['finishedAt'] = dt.datetime.now(dt.timezone.utc).isoformat()
        self.report['sourceHeadAtEnd'] = command(['git', '-C', REPO, 'rev-parse', 'HEAD']).stdout.strip()
        statuses = [c['status'] for c in self.report['checks']]
        if self.report.get('errorKind') == 'blocked': self.report['result'] = 'partial-blocked'
        elif self.report.get('error') or 'fail' in statuses: self.report['result'] = 'fail'
        elif 'blocked' in statuses: self.report['result'] = 'partial-blocked'
        elif not self.args.hold_seconds:
            self.report['result'] = 'pass' if self.report['scopeCompleted'] else 'partial-blocked'
        self.flush()

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--app', required=True, help='Original .app path, debug or signed final; never rewritten')
    docs = REPO.parent / 'governance-main'
    p.add_argument('--out', default=str((docs if (docs / '.git').exists() else REPO) / '.adg/evidence/0.2.21/macos-native-20261104'))
    p.add_argument('--stage', choices=['debug-preverify', 'signed-final-reverify'], default='debug-preverify')
    p.add_argument('--expected-build-commit', help='Optional exact embedded startup commit admission, useful for the signed final original')
    p.add_argument('--expected-version', choices=['0.2.21', '0.2.22'], default='0.2.21')
    p.add_argument('--scope', choices=['all', 'notice', 'overview'], default='all', help='Each scope always creates a fresh owned root')
    p.add_argument('--node', help='Absolute path to a real unmodified macOS Node binary; all three ready inputs are required')
    p.add_argument('--npm-cli', help='Absolute path to the original npm bin/npm-cli.js with its package dependencies')
    p.add_argument('--official-entry', help='Absolute cached node_modules/@bitkyc08/opencodex manifest ocx entry; no install')
    p.add_argument('--bun', help='Optional real macOS Bun binary, selected through the official launcher runtime-path support')
    p.add_argument('--hold-seconds', type=int, default=0, help='Bounded interactive exploration of the owned PID')
    args = p.parse_args()
    provided = [args.node, args.npm_cli, args.official_entry]
    if any(provided) and not all(provided): p.error('Ready inputs --node, --npm-cli and --official-entry must all be supplied')
    if args.scope == 'overview' and not all(provided): p.error('--scope overview requires all three real tool inputs')
    if args.bun and not all(provided): p.error('--bun requires the three explicit ready inputs')
    run = NativeRun(args)
    def interrupted(signum, frame):
        raise KeyboardInterrupt('Native acceptance interrupted by signal ' + str(signum))
    signal.signal(signal.SIGTERM, interrupted)
    try:
        run.prepare(); run.launch()
        if not args.hold_seconds:
            run.verify(); run.report['scopeCompleted'] = True
        deadline = time.monotonic() + min(max(args.hold_seconds, 0), 1800)
        while time.monotonic() < deadline and run.proc.poll() is None: time.sleep(0.5)
        run.report['result'] = 'incomplete'
    except (Exception, KeyboardInterrupt) as exc:
        run.report['error'] = str(exc)
        run.report['errorKind'] = 'blocked' if isinstance(exc, (Blocked, KeyboardInterrupt)) else 'fail'
        run.check('run-prerequisite', run.report['errorKind'], reason=str(exc))
        (run.out / 'exception.log').write_text(traceback.format_exc())
        run.report['result'] = 'blocked'
    finally: run.close()
    return 0 if run.report['result'] == 'pass' else (1 if run.report['result'] == 'fail' else 2)

if __name__ == '__main__': sys.exit(main())
