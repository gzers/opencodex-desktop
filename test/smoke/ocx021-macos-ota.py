#!/usr/bin/env python3
"""0.2.20 -> 0.2.21 original macOS OTA; prepare is offline and never launches an app."""
import argparse
import base64
import datetime as dt
import hashlib
import http.server
import json
import os
from pathlib import Path, PurePosixPath
import plistlib
import pwd
import re
import select
import shutil
import signal
import socket
import subprocess
import sys
import tarfile
import threading
import time
import traceback
import urllib.parse
import urllib.request
import uuid

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
DOCS = REPO.parent / 'governance-main'
HISTORY = DOCS / '.adg/evidence/0.2.20/final-candidate-updates-20261104'
BASELINE_SOURCE = 'b1750f5cf78b294b2edb34b9674d8c707ef8f810'
BASELINE_ARCHIVE_SHA = '8fe307fe2f5daaa87c41abbcccc38ef7db346cd5e88ee1456d3285bfb1d8d0f9'
BASELINE_EXE_SHA = '544de246de1a498b8c681c0bd6ce254c7043ffe4b5fe754d67d3477a8fe4370b'
BETA_URL = 'https://github.com/gzers/opencodex-desktop/releases/download/beta/latest.json'
TAG_PREFIX = 'https://github.com/gzers/opencodex-desktop/releases/download/v0.2.21/'
ASSET_NAME = 'OpenCodeX.Desktop_aarch64.app.tar.gz'
NETWORK_HOSTS = {'github.com', 'api.github.com', 'release-assets.githubusercontent.com', 'objects.githubusercontent.com', 'github-releases.githubusercontent.com'}

class Blocked(RuntimeError):
    pass

def dump(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')
    path.chmod(0o600)

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def cmd(argv, **kw):
    return subprocess.run(list(map(str, argv)), capture_output=True, text=True, timeout=40, **kw)

def source_config(source, relative):
    value = cmd(['git', '-C', REPO, 'show', source + ':' + relative])
    if value.returncode: raise Blocked('Public source/config is unavailable locally: ' + source + ':' + relative)
    return json.loads(value.stdout)

def inventory(root):
    files = {}
    for path in sorted(root.rglob('*')):
        relative = str(path.relative_to(root))
        if path.is_symlink():
            if not path.resolve().is_relative_to(root.resolve()): raise Blocked('Bundle symlink escapes its app: ' + relative)
            files[relative] = {'symlink': os.readlink(path)}
        elif path.is_file(): files[relative] = {'sha256': sha(path), 'bytes': path.stat().st_size, 'executable': bool(path.stat().st_mode & 0o111)}
    return files

def unpack(archive, destination):
    destination.mkdir(mode=0o700)
    with tarfile.open(archive, 'r:gz') as tar:
        members = tar.getmembers()
        for member in members:
            path = PurePosixPath(member.name)
            if path.is_absolute() or '..' in path.parts or not path.parts or path.parts[0] != 'OpenCodeX Desktop.app':
                raise Blocked('Archive has a non-app/traversal member: ' + member.name)
            if member.isdev() or member.isfifo() or member.islnk(): raise Blocked('Unsupported archive special/hardlink member')
            if member.issym():
                link = PurePosixPath(member.linkname)
                if link.is_absolute() or '..' in link.parts: raise Blocked('Archive symlink traversal')
        # macOS system Python 3.9 lacks tarfile's later filter argument. All
        # members above are already confined, and the archive was authenticated.
        if hasattr(tarfile, 'data_filter'): tar.extractall(destination, members=members, filter='data')
        else: tar.extractall(destination, members=members)
    return destination / 'OpenCodeX Desktop.app'

def process_rows():
    result = cmd(['/bin/ps', '-axo', 'pid=,ppid=,pgid=,comm='])
    return [{'pid': int(p[0]), 'ppid': int(p[1]), 'pgid': int(p[2]), 'command': p[3]}
            for row in result.stdout.splitlines() if len(p := row.split(None, 3)) == 4]

class Tunnel(http.server.ThreadingHTTPServer):
    daemon_threads = True
    allow_reuse_address = False
    def __init__(self, run):
        self.run = run
        super().__init__(('127.0.0.1', 0), TunnelHandler)

class TunnelHandler(http.server.BaseHTTPRequestHandler):
    """TLS CONNECT only: no TLS termination, manifest substitution, cache or fake response."""
    def log_message(self, *args): pass
    def do_GET(self): self.send_error(403)
    def do_POST(self): self.send_error(403)
    def do_CONNECT(self):
        host, _, port = self.path.rpartition(':')
        host = host.lower()
        record = {'at': time.time(), 'host': host, 'port': port, 'allowed': host in NETWORK_HOSTS and port == '443', 'bytesToRemote': 0, 'bytesFromRemote': 0}
        try:
            if not record['allowed']:
                self.send_error(403); return
            with socket.create_connection((host, 443), timeout=20) as remote:
                self.send_response(200, 'Connection established'); self.end_headers()
                sockets = [self.connection, remote]
                while True:
                    readable, _, _ = select.select(sockets, [], [], 25)
                    if not readable: break
                    for source in readable:
                        data = source.recv(64 * 1024)
                        if not data: return
                        target = remote if source is self.connection else self.connection
                        target.sendall(data)
                        record['bytesToRemote' if target is remote else 'bytesFromRemote'] += len(data)
        except Exception as error: record['errorType'] = type(error).__name__
        finally:
            with self.server.run.tunnel_lock:
                with (self.server.run.out / 'production-tls-tunnels.jsonl').open('a') as stream:
                    stream.write(json.dumps(record) + '\n')

class OtaRun:
    def __init__(self, args):
        self.args = args
        self.container = Path('/private/tmp') / ('ocx021-mac-ota-' + str(uuid.uuid4()))
        self.container.mkdir(mode=0o700)
        self.root = self.container / 'root'; self.root.mkdir(mode=0o700)
        self.home = self.root / 'home'; self.home.mkdir(mode=0o700)
        self.tmp = self.root / 'tmp'; self.tmp.mkdir(mode=0o700)
        self.out = Path(args.out).resolve() / self.container.name; self.out.mkdir(parents=True, mode=0o700)
        self.app = self.container / 'installed/OpenCodeX Desktop.app'
        self.exe = self.app / 'Contents/MacOS/opencodex-desktop'
        self.pid = None; self.old = None; self.restarted = None
        self.owned = {}; self.stop_observer = threading.Event(); self.tunnel_lock = threading.Lock()
        self.env = {'PATH': '/usr/bin:/bin:/usr/sbin:/sbin', 'LANG': 'en_US.UTF-8',
            'USER': pwd.getpwuid(os.getuid()).pw_name, 'LOGNAME': pwd.getpwuid(os.getuid()).pw_name,
            'HOME': str(self.home), 'CFFIXED_USER_HOME': str(self.home), 'TMP': str(self.tmp), 'TEMP': str(self.tmp), 'TMPDIR': str(self.tmp) + '/',
            'CODEX_HOME': str(self.home / '.codex'), 'OPENCODEX_HOME': str(self.root / 'opencodex-home'),
            'XDG_CONFIG_HOME': str(self.home / '.config'), 'XDG_CACHE_HOME': str(self.home / '.cache'),
            'OPENCODEX_SANDBOX': '1', 'OPENCODEX_SANDBOX_ROOT': str(self.root), 'OPENCODEX_SANDBOX_BOUNDARY': str(self.container)}
        self.axbin = self.container / 'ax'; self.monitor = self.container / 'monitor'; self.capwin = self.container / 'capwin'
        self.report = {'schemaVersion': 1, 'mode': args.mode, 'result': 'incomplete', 'from': '0.2.20', 'to': '0.2.21',
            'channel': 'existing production beta', 'container': str(self.container), 'root': str(self.root), 'environment': self.env,
            'app': str(self.app), 'checks': [], 'otaExecuted': False, 'targetSource': args.source_sha,
            'publicationAuthorized': bool(args.parent_beta_published), 'scriptSha256': sha(Path(__file__)),
            'nativeHelperSha256': sha(HERE / 'ocx021-macos-native.swift'), 'monitorSha256': sha(HERE / 'ocx021-macos-ota.swift'),
            'startedAt': dt.datetime.now(dt.timezone.utc).isoformat(),
            'methodOnlyReference': str(HISTORY / 'macos-adjacent-ota/ota-result.json'), 'sameVersionAcceptanceReuse': False}
        self.flush()

    def flush(self):
        dump(self.out / 'report.json', self.report)
        dump(self.out / 'run-context.json', {k: self.report.get(k) for k in ['mode', 'result', 'from', 'to', 'channel', 'root', 'environment', 'app', 'oldPid', 'automaticRestartPid', 'targetSource', 'publicationAuthorized', 'otaExecuted', 'checks']})
        print(json.dumps({'out': str(self.out), 'pid': self.pid, 'result': self.report['result'], 'check': self.report['checks'][-1:]}, ensure_ascii=False), flush=True)

    def check(self, name, status='pass', **facts):
        self.report['checks'].append(dict(name=name, status=status, **facts)); self.flush()

    def verify_signature(self, archive, signature, label):
        public = source_config(BASELINE_SOURCE, 'apps/desktop/tauri/tauri.conf.json')['plugins']['updater']['pubkey']
        key = self.container / 'old-updater-public-key'
        key.write_bytes(base64.b64decode(public, validate=True))
        decoded = self.container / (label + '.minisig')
        decoded.write_bytes(base64.b64decode(signature.read_text().strip(), validate=True))
        tool = shutil.which('minisign')
        if not tool: raise Blocked('minisign not available; no signing secret is needed')
        result = cmd([tool, '-V', '-m', archive, '-p', key, '-x', decoded])
        (self.out / (label + '-updater-signature.log')).write_text(result.stdout + result.stderr)
        if result.returncode: raise Blocked(label + ' fails the old release public-key signature')
        self.check(label + '-old-trust-signature', archiveSha256=sha(archive), signatureSha256=sha(signature), publicKeySource=BASELINE_SOURCE)

    def verify_bundle(self, app, version, source, label):
        info = plistlib.loads((app / 'Contents/Info.plist').read_bytes())
        assert info['CFBundleIdentifier'] == 'com.gzers.opencodex.desktop' and info['CFBundleShortVersionString'] == version
        binary = app / 'Contents/MacOS' / info['CFBundleExecutable']
        assert source.encode() in binary.read_bytes(), 'Expected embedded source absent from original binary'
        result = cmd(['/usr/bin/codesign', '--verify', '--deep', '--strict', app])
        (self.out / (label + '-codesign-verify.log')).write_text(result.stdout + result.stderr)
        if result.returncode: raise Blocked('Original bundle strict signature failed: ' + label)
        detail = cmd(['/usr/bin/codesign', '-dv', '--verbose=4', app])
        (self.out / (label + '-codesign-identity.log')).write_text(detail.stdout + detail.stderr)
        files = inventory(app); dump(self.out / (label + '-bundle-inventory.json'), files)
        self.check(label + '-original-bundle', version=version, embeddedSource=source, binarySha256=sha(binary), fileCount=len(files), signature='deep strict passed')
        return files

    def prepare(self):
        archive = Path(self.args.baseline_archive).resolve(strict=True)
        signature = Path(self.args.baseline_signature).resolve(strict=True)
        assert sha(archive) == BASELINE_ARCHIVE_SHA, 'Baseline must be the exact original public b175/0.2.20 release asset'
        self.verify_signature(archive, signature, 'baseline020')
        self.baseline = unpack(archive, self.container / 'baseline-original')
        self.baseline_inventory = self.verify_bundle(self.baseline, '0.2.20', BASELINE_SOURCE, 'baseline020')
        assert sha(self.baseline / 'Contents/MacOS/opencodex-desktop') == BASELINE_EXE_SHA
        self.app.parent.mkdir(mode=0o700)
        result = cmd(['/usr/bin/ditto', self.baseline, self.app]); assert result.returncode == 0, result.stderr
        assert inventory(self.app) == self.baseline_inventory
        for source, target in [(HERE / 'ocx021-macos-native.swift', self.axbin), (HERE / 'ocx021-macos-ota.swift', self.monitor), (HERE / 'lib/capwin.swift', self.capwin)]:
            built = cmd(['/usr/bin/xcrun', 'swiftc', source, '-o', target], env=self.env)
            (self.out / (target.name + '-compile.log')).write_text(built.stdout + built.stderr)
            assert built.returncode == 0, 'Helper failed to compile: ' + source.name
        permissions = cmd([self.axbin, 'status'], env=self.env)
        self.report['permissions'] = json.loads(permissions.stdout)
        self.check('native-permissions', 'pass' if self.report['permissions']['axTrusted'] and self.report['permissions']['screenCaptureAllowed'] and not self.report['permissions']['locked'] else 'blocked', **self.report['permissions'])
        # A CONNECT tunnel allows only the production GitHub TLS destinations;
        # the app's OS sandbox can connect only to this owned loopback port.
        self.tunnel = Tunnel(self)
        self.proxy_port = self.tunnel.server_address[1]
        self.profile = self.container / 'candidate.sb'
        home = json.dumps(pwd.getpwuid(os.getuid()).pw_dir)
        self.profile.write_text('(version 1)\n(allow default)\n(deny network*)\n' +
            '(allow network-outbound (remote ip "localhost:' + str(self.proxy_port) + '"))\n' +
            '(deny file-read* file-write* (subpath ' + home + '))\n' +
            '(deny file-read* file-write* (subpath "/Library/Keychains"))\n' +
            '(deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.secd"))\n')
        shutil.copyfile(self.profile, self.out / 'candidate.sb')
        guard = cmd(['/usr/bin/sandbox-exec', '-f', self.profile, '/usr/bin/nc', '-v', '-G', '1', '-z', '192.0.2.1', '443'], env=self.env)
        (self.out / 'network-guard.log').write_text(guard.stdout + guard.stderr)
        if guard.returncode == 0 or 'Operation not permitted' not in guard.stderr:
            raise Blocked('OS direct-network denial was not confirmed; see network-guard.log')
        positive = cmd(['/usr/bin/sandbox-exec', '-f', self.profile, '/usr/bin/nc', '-v', '-G', '1', '-z', '127.0.0.1', str(self.proxy_port)], env=self.env)
        (self.out / 'loopback-tunnel-guard.log').write_text(positive.stdout + positive.stderr)
        if positive.returncode: raise Blocked('OS sandbox did not allow the one owned production tunnel port')
        self.check('direct-network-denied', allowedProductionTlsHosts=sorted(NETWORK_HOSTS), localTunnelPort=self.proxy_port, appTraffic='Only real TLS CONNECT, no interception')
        self.seed()
        history = json.loads((HISTORY / 'macos-adjacent-ota/ota-result.json').read_text())
        dump(self.out / 'method-reuse.json', {'reference': str(HISTORY), 'historicalFrom': history['from'], 'historicalTo': history['to'],
            'reusedMethodOnly': ['two actual native old-version saves', 'exact backup preimages', 'production beta signed updater', 'same automatic new process readback', 'whole original signed payload file comparison'],
            'historicalPassNotCountedFor021': True, 'rel23': str(DOCS / 'docs/03-开发实施/REL-OPENCODEX-DESKTOP-23.md')})
        dump(self.out / 'execution-contract.json', {'prepared': True, 'appLaunched': False, 'otaExecuted': False,
            'requiresParentNotification': True, 'requiredInputs': ['--mode execute', '--parent-beta-published', '--source-sha <full40>', '--publication-receipt <parent JSON>', '--target-archive <original release>', '--target-signature <original sidecar>'],
            'requiredReceipt': {'schemaVersion': 1, 'parentNotified': True, 'betaPublished': True, 'tag': 'v0.2.21', 'sourceCommit': '<full40>', 'releaseId': '<integer>', 'archiveSha256': '<64hex>', 'betaManifestUrl': BETA_URL},
            'forbidden': ['new launcher for post-OTA readback', 'modified bundle or signing identity', 'fake IPC or patched updater endpoints', 'daily app restart/quit', 'keychain/signing secrets', 'real provider or sync requests']})
        self.check('prepared-not-executed', baselineOriginal='0.2.20/b175', freshRoot=str(self.root), nativeHelperReusedUnmodified=True, betaNetworkQueried=False, appLaunched=False)

    def seed(self):
        preferences = source_config(BASELINE_SOURCE, 'apps/desktop/tauri/config/preferences.defaults.json')
        preferences.update(auto_panel=False, launch_with_codex=False, app_update_auto_check=False, cold_sync=False, cli_enabled=False, startup_cleanup=False,
            network_proxy_mode='manual', network_proxy_scheme='http', network_proxy_host='127.0.0.1:' + str(self.proxy_port), network_no_proxy='')
        dump(self.root / 'manager-state/preferences.json', preferences)
        config = {'providers': {'ota021-preserve': {'adapter': 'openai-chat', 'authMode': 'none', 'disabled': True,
            'baseUrl': 'https://ota021-never-contact.invalid/v1', 'models': ['vendor/a:8b', 'Mixed_Case/V2'], 'selectedModels': ['vendor/a:8b'],
            'note': 'Owned synthetic adjacent OTA preservation fixture'}}, 'defaultProvider': 'ota021-preserve',
            'modelPickerOrder': ['ota021-preserve/vendor/a:8b', 'ota021-preserve/Mixed_Case/V2'], 'nativeOtaOpaque': {'keep': 21}}
        dump(self.root / 'opencodex-home/config.json', config)
        for rel, text in [('home/.codex/config.toml', '# synthetic owned OTA sentinel\nmodel = "ota021-placeholder"\n'),
                          ('home/.agents/skills/ota021-preserve/SKILL.md', '---\nname: ota021-preserve\ndescription: owned OTA fixture\n---\nSynthetic preservation sentinel.\n')]:
            path = self.root / rel; path.parent.mkdir(parents=True, exist_ok=True); path.write_text(text)
        dump(self.out / 'initial-config.json', config)

    def public_get(self, url, name):
        parsed = urllib.parse.urlparse(url)
        if parsed.scheme != 'https' or parsed.hostname not in NETWORK_HOSTS: raise Blocked('Non-production publication URL refused')
        opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
        request = urllib.request.Request(url, headers={'User-Agent': 'ocx021-owned-native-ota', 'Accept': 'application/json'})
        with opener.open(request, timeout=30) as response:
            if urllib.parse.urlparse(response.url).hostname not in NETWORK_HOSTS: raise Blocked('Publication redirect escaped GitHub')
            raw = response.read(2 * 1024 * 1024 + 1)
        if len(raw) > 2 * 1024 * 1024: raise Blocked('Publication metadata exceeds bound')
        (self.out / name).write_bytes(raw)
        return json.loads(raw)

    def publication(self):
        # This is called only in execute mode, after the parent notification.
        receipt = json.loads(Path(self.args.publication_receipt).read_text())
        required = {'schemaVersion': 1, 'parentNotified': True, 'betaPublished': True, 'tag': 'v0.2.21',
                    'sourceCommit': self.args.source_sha, 'betaManifestUrl': BETA_URL}
        for key, expected in required.items():
            if receipt.get(key) != expected: raise Blocked('Parent publication receipt mismatch: ' + key)
        if not isinstance(receipt.get('releaseId'), int) or not re.fullmatch('[0-9a-f]{64}', receipt.get('archiveSha256', '')):
            raise Blocked('Parent receipt needs the final public release ID and archive SHA-256')
        dump(self.out / 'parent-publication-receipt.json', receipt)
        release = self.public_get('https://api.github.com/repos/gzers/opencodex-desktop/releases/tags/v0.2.21', 'public-release.json')
        if release.get('draft') or release.get('id') != receipt['releaseId'] or release.get('tag_name') != 'v0.2.21':
            raise Blocked('The exact parent-notified public 021 release is unavailable')
        tag = self.public_get('https://api.github.com/repos/gzers/opencodex-desktop/git/ref/tags/v0.2.21', 'public-tag.json')['object']
        if tag['type'] == 'tag':
            tag = self.public_get('https://api.github.com/repos/gzers/opencodex-desktop/git/tags/' + tag['sha'], 'public-annotated-tag.json')['object']
        if tag['type'] != 'commit' or tag['sha'] != self.args.source_sha: raise Blocked('Public tag does not resolve to the parent source SHA')
        manifest = self.public_get(BETA_URL, 'production-beta-manifest.json')
        if manifest.get('version') != '0.2.21': raise Blocked('Production beta does not offer exactly 0.2.21')
        entry = manifest['platforms']['darwin-aarch64']
        if urllib.parse.unquote(entry['url']) != TAG_PREFIX + ASSET_NAME: raise Blocked('Beta updater URL is not the original 021 release asset')
        archive = Path(self.args.target_archive).resolve(strict=True)
        signature = Path(self.args.target_signature).resolve(strict=True)
        digest = sha(archive)
        asset = next(a for a in release['assets'] if a['name'] == ASSET_NAME)
        if digest != receipt['archiveSha256'] or asset.get('digest') != 'sha256:' + digest or asset['size'] != archive.stat().st_size:
            raise Blocked('Local target is not byte-identical to the exact published parent asset')
        if entry['signature'].strip() != signature.read_text().strip(): raise Blocked('Beta manifest and original sidecar signature differ')
        self.verify_signature(archive, signature, 'target021')
        self.target = unpack(archive, self.container / 'target-original')
        self.target_inventory = self.verify_bundle(self.target, '0.2.21', self.args.source_sha, 'target021')
        old_key = source_config(BASELINE_SOURCE, 'apps/desktop/tauri/tauri.conf.json')['plugins']['updater']['pubkey']
        new_key = source_config(self.args.source_sha, 'apps/desktop/tauri/tauri.conf.json')['plugins']['updater']['pubkey']
        if old_key != new_key: raise Blocked('Target changed the inherited updater trust key')
        self.report['publication'] = {'releaseId': release['id'], 'tag': 'v0.2.21', 'sourceCommit': self.args.source_sha,
            'betaEndpoint': BETA_URL, 'publishedAssetSha256': digest, 'originalSidecarSha256': sha(signature)}
        self.check('parent-notified-public-beta-admission', **self.report['publication'])

    def inspect(self, pid):
        result = cmd([self.monitor, 'inspect', self.exe, pid], env=self.env)
        if result.returncode: raise Blocked('Exact owned PID/environment identity unavailable: ' + result.stdout + result.stderr)
        value = json.loads(result.stdout)
        for key in ['HOME', 'TMP', 'TMPDIR', 'CODEX_HOME', 'OPENCODEX_HOME', 'OPENCODEX_SANDBOX', 'OPENCODEX_SANDBOX_ROOT', 'OPENCODEX_SANDBOX_BOUNDARY']:
            if value['environment'].get(key) != self.env[key]: raise Blocked('Automatic process lost isolation key ' + key)
        return value

    def find_apps(self):
        result = cmd([self.monitor, 'find', self.exe], env=self.env)
        if result.returncode: raise Blocked('Owned app enumeration failed: ' + result.stdout + result.stderr)
        return json.loads(result.stdout)

    def observe(self):
        while not self.stop_observer.is_set():
            try:
                rows = process_rows()
                known = {self.old.pid, *self.owned.keys()}
                while True:
                    children = [p for p in rows if p['ppid'] in known and p['pid'] not in known and p['pid'] != 80916]
                    if not children: break
                    for child in children: known.add(child['pid']); self.owned[child['pid']] = child
            except Exception: pass
            self.stop_observer.wait(.15)

    def launch(self):
        permissions = self.report['permissions']
        if not permissions['axTrusted'] or not permissions['screenCaptureAllowed'] or permissions['locked']:
            raise Blocked('Locked or native AX/capture permission missing')
        # Keep this listener alive through the real automatic restart.
        self.tunnel_thread = threading.Thread(target=self.tunnel.serve_forever, daemon=True); self.tunnel_thread.start()
        self.log = open(self.out / 'old-process.log', 'wb')
        self.old = subprocess.Popen(['/usr/bin/sandbox-exec', '-f', str(self.profile), str(self.exe), '-ApplePersistenceIgnoreState', 'YES'],
            env=self.env, cwd=self.root, stdin=subprocess.DEVNULL, stdout=self.log, stderr=subprocess.STDOUT, start_new_session=True)
        self.pid = self.old.pid; self.report['oldPid'] = self.pid; self.report['launchCalls'] = 1
        self.observer = threading.Thread(target=self.observe, daemon=True); self.observer.start()
        time.sleep(2)
        if self.old.poll() is not None: raise Blocked('Old original app exited before native baseline')
        identity = self.inspect(self.pid); dump(self.out / 'old-process-identity.json', identity)
        self.wait(lambda s: any(n.get('role') == 'AXWebArea' for n in s['nodes']), 30, 'old-native-window')
        self.ax('focus'); self.capture('old-native-window')
        log = self.root / 'logs/app.log'
        assert 'startup version=0.2.20 commit=' + BASELINE_SOURCE in log.read_text()
        self.check('old-original-native-launch', pid=self.pid, identity=identity, originalBundleUntouched=inventory(self.app) == self.baseline_inventory)

    def ax(self, verb, *args):
        if not self.pid or self.pid == 80916: raise Blocked('No admitted owned PID')
        if verb != 'snapshot':
            with (self.out / 'native-actions.jsonl').open('a') as journal:
                journal.write(json.dumps({'time': time.monotonic(), 'pid': self.pid, 'verb': verb, 'path': args[0] if args else None}, ensure_ascii=False) + '\n')
        result = cmd([self.axbin, verb, self.pid, self.exe, *args], env=self.env)
        if result.returncode: raise Blocked('Native AX operation failed: ' + result.stdout + result.stderr)
        return json.loads(result.stdout)

    def snapshot(self, name=None):
        value = self.ax('snapshot')
        if value.get('locked'): raise Blocked('Screen locked during OTA; no UI result inferred')
        if name: dump(self.out / (name + '-ax.json'), value)
        return value

    @staticmethod
    def text(snap):
        return '\n'.join(str(n.get('value') or n.get('title') or '') for n in snap['nodes'])

    @staticmethod
    def matches(snap, label):
        webroots = [n['path'] + '.' for n in snap['nodes'] if n.get('role') == 'AXWebArea']
        return [n for n in snap['nodes'] if any(n['path'].startswith(p) for p in webroots) and 'AXPress' in n.get('actions', [])
                and (n.get('title') == label or n.get('description') == label)]

    def wait(self, predicate, seconds=15, name='state'):
        deadline = time.monotonic() + seconds
        while True:
            snap = self.snapshot()
            if predicate(snap):
                dump(self.out / (name + '-ax.json'), snap); return snap
            if time.monotonic() >= deadline:
                self.snapshot(name + '-timeout'); self.capture(name + '-timeout')
                raise Blocked('Native production state not reached: ' + name)
            time.sleep(.25)

    def press(self, label, index=0, seconds=15):
        snap = self.wait(lambda s: len(self.matches(s, label)) > index, seconds, 'control')
        node = self.matches(snap, label)[index]
        if not node.get('enabled', True): raise Blocked('Native control is disabled: ' + label)
        self.ax('press', node['path'], label)

    def capture(self, name):
        snap = self.snapshot(name)
        windows = [w for w in snap['windows'] if w['onscreen'] and w['layer'] == 0 and w['bounds'].get('Width', 0) > 500]
        if not windows: raise Blocked('No onscreen candidate-owned window')
        result = cmd([self.capwin, windows[0]['id'], self.out / (name + '.png')], env=self.env)
        (self.out / (name + '-capture.log')).write_text(result.stdout + result.stderr)
        if result.returncode: raise Blocked('Candidate-only capture failed')

    def baseline_saves(self):
        self.press('模型'); self.press('渠道模型')
        config = self.root / 'opencodex-home/config.json'
        backups = self.root / 'backups/model-config'
        saves = []
        for number in [1, 2]:
            previous = config.read_bytes(); before = set(backups.rglob('config.json')) if backups.exists() else set()
            self.press('ota021-preserve渠道启用状态')
            self.press('编辑'); self.press('保存渠道草稿')
            self.wait(lambda s: self.matches(s, '读取现有渠道'), 15, 'old-native-save-' + str(number))
            # "稍后" only dismisses a save notice; never restart Codex.
            snap = self.snapshot()
            if self.matches(snap, '稍后'): self.press('稍后')
            now = config.read_bytes(); assert now != previous, 'Old UI Save did not write business config'
            added = set(backups.rglob('config.json')) - before
            exact = [p for p in added if p.read_bytes() == previous]
            assert len(exact) == 1, 'Each actual native save needs exactly one exact fresh preimage backup'
            (self.out / ('save-' + str(number) + '-preimage.json')).write_bytes(previous)
            (self.out / ('save-' + str(number) + '-readback.json')).write_bytes(now)
            saves.append({'save': number, 'backup': str(exact[0].relative_to(self.root)), 'preimageSha256': hashlib.sha256(previous).hexdigest(), 'savedSha256': hashlib.sha256(now).hexdigest()})
        self.capture('old-two-native-saves')
        dump(self.out / 'old-two-native-saves.json', saves)
        self.press('设置'); self.press('版本升级')
        self.press('应用更新通道'); self.press('测试通道')
        self.wait(lambda s: '测试通道' in self.text(s), name='old-beta-channel')
        preferences = json.loads((self.root / 'manager-state/preferences.json').read_text())
        assert preferences['app_update_channel'] == 'beta'
        self.capture('old-beta-channel')
        self.business = ['opencodex-home/config.json', 'manager-state/model-channel-layout.json', 'manager-state/preferences.json',
                         'home/.codex/config.toml', 'home/.agents/skills/ota021-preserve/SKILL.md', 'manager-state/data-root.json']
        self.business.extend(str(p.relative_to(self.root)) for p in backups.rglob('*') if p.is_file() and not p.name.endswith('.lock'))
        self.before = {rel: sha(self.root / rel) for rel in self.business}
        dump(self.out / 'before-ota-file-hashes.json', self.before)
        self.check('two-old-native-saves-and-exact-preimages', saves=saves, filesFrozen=len(self.before), channelSavedThroughUi=True)

    def install(self):
        self.report['otaExecuted'] = True; self.flush()
        self.press('检查更新')
        self.wait(lambda s: '0.2.21' in self.text(s) or '更新失败' in self.text(s), 120, 'production-beta-check')
        snap = self.snapshot()
        if self.matches(snap, '桌面管理器'): self.press('桌面管理器')
        elif len(self.matches(snap, '查看详情')) == 2:
            # b175's update-center list has ordered manager/official sections.
            # This is the original 020 UI, not the 021 public-dialog selector.
            assert '桌面管理器' in self.text(snap) and '0.2.21' in self.text(snap)
            self.press('查看详情', index=0)
        elif not self.matches(snap, '确认更新'):
            # The old update-center list has a descriptive manager item label.
            candidates = [n for n in snap['nodes'] if 'AXPress' in n.get('actions', []) and '桌面管理器' in str(n.get('title', '')) and '0.2.21' in str(n.get('title', ''))]
            if len(candidates) != 1: raise Blocked('Exact 021 manager update confirmation entry is not reachable')
            node = candidates[0]; self.ax('press', node['path'], node.get('title'))
        confirm = self.wait(lambda s: self.matches(s, '确认更新'), 45, 'production-target-confirm')
        text = self.text(confirm)
        assert '0.2.20' in text and '0.2.21' in text and '测试通道' in text
        self.capture('production-target-confirm')
        self.press('确认更新')
        pending = self.wait(lambda s: self.matches(s, '重启并应用更新'), 360, 'signed-installed-pending-restart')
        assert '0.2.20' in self.text(pending) and '0.2.21' in self.text(pending)
        self.capture('signed-installed-pending-restart')
        receipt = json.loads((self.root / 'manager-state/update-handoff.json').read_text())
        assert receipt['version'] == '0.2.21' and receipt['identity']['channel'] == 'beta'
        dump(self.out / 'signed-installation-handoff.json', receipt)
        installed = inventory(self.app)
        assert installed == self.target_inventory, 'Installed whole bundle differs from published signed original'
        self.verify_bundle(self.app, '0.2.21', self.args.source_sha, 'installed-before-restart')
        assert self.old.poll() is None, 'Old running process ended before explicit restart'
        self.check('production-signed-download-and-inplace-install', wholeBundleExactlyOriginal=True, oldRunningPid=self.old.pid,
                   handoffProducedBy='production updater only after old-trust verification', changedUpdaterEndpoint=False)

    def restart_readback(self):
        started = time.time()
        self.press('重启并应用更新')
        deadline = time.monotonic() + 90
        while time.monotonic() < deadline:
            candidates = [p for p in self.find_apps() if p['pid'] != self.old.pid and p['launchTime'] >= started - 3]
            if self.old.poll() is not None and len(candidates) == 1:
                self.restarted = self.inspect(candidates[0]['pid']); break
            time.sleep(.25)
        else: raise Blocked('Same-path automatic replacement process was not observed; no manual relaunch is allowed')
        self.pid = self.restarted['pid']; self.report['automaticRestartPid'] = self.pid
        self.owned[self.pid] = dict(self.restarted, ppid=1, command=str(self.exe))
        dump(self.out / 'automatic-restart-identity.json', self.restarted)
        self.wait(lambda s: any(n.get('role') == 'AXWebArea' for n in s['nodes']), 35, 'same-auto-process-mounted')
        self.ax('focus'); self.capture('same-auto-process-mounted')
        assert 'startup version=0.2.21 commit=' + self.args.source_sha in (self.root / 'logs/app.log').read_text()
        assert inventory(self.app) == self.target_inventory
        self.verify_bundle(self.app, '0.2.21', self.args.source_sha, 'automatic-process-installed021')
        self.press('模型'); self.press('渠道模型'); self.press('编辑')
        snap = self.wait(lambda s: '渠道配置' in self.text(s), 20, 'same-auto-process-business-readback')
        assert 'ota021-preserve' in self.text(snap) and 'vendor/a:8b' in self.text(snap) and 'Mixed_Case/V2' in self.text(snap)
        save = self.matches(snap, '保存渠道草稿'); assert len(save) == 1 and not save[0].get('enabled')
        self.capture('same-auto-process-business-readback')
        after = {rel: sha(self.root / rel) for rel in self.business}
        dump(self.out / 'after-ota-file-hashes.json', after)
        assert after == self.before, 'Business data/preferences/fresh preimages changed during OTA'
        assert not (self.root / 'manager-state/update-handoff.json').exists(), 'New process did not consume signed handoff'
        assert self.inspect(self.pid)['launchTime'] == self.restarted['launchTime'] and self.report['launchCalls'] == 1
        self.check('same-automatic-process-native-readback', oldPid=self.old.pid, automaticRestartPid=self.pid,
            sameAutomaticPidThroughoutReadback=True, manualRelaunches=0, exactFilesPreserved=len(after), startupSource=self.args.source_sha,
            signedOriginalWholeBundleEqual=True, pendingHandoffConsumed=True)

    def close(self):
        self.stop_observer.set()
        if hasattr(self, 'observer'): self.observer.join(timeout=3)
        exact = self.find_apps() if self.old else []
        for app in exact:
            if app['pid'] == 80916: raise AssertionError('Protected daily PID is never eligible')
            self.owned[app['pid']] = dict(app, command=str(self.exe))
        if self.old: self.owned[self.old.pid] = {'pid': self.old.pid, 'pgid': self.old.pid, 'command': str(self.exe)}
        cleanup = []
        for pgid in sorted({p['pgid'] for p in self.owned.values()}):
            if pgid in [0, 1, 80916, os.getpgrp()]:
                cleanup.append({'pgid': pgid, 'refused': 'protected/unowned group'}); continue
            members = [p for p in process_rows() if p['pgid'] == pgid]
            if members:
                allowed = all(p['pid'] in self.owned or p['command'] == str(self.exe) for p in members)
                leader = next((p for p in members if p['pid'] == pgid), None)
                if not allowed or (leader and leader['pid'] not in self.owned and leader['command'] != str(self.exe)):
                    cleanup.append({'pgid': pgid, 'refused': 'group reused or has an unobserved process', 'members': members}); continue
                os.killpg(pgid, signal.SIGTERM)
                until = time.monotonic() + 4
                while time.monotonic() < until:
                    if self.old: self.old.poll()
                    if not any(p['pgid'] == pgid for p in process_rows()): break
                    time.sleep(.1)
                if any(p['pgid'] == pgid for p in process_rows()): os.killpg(pgid, signal.SIGKILL)
            if self.old and self.old.poll() is not None: self.old.wait(timeout=5)
            cleanup.append({'pgid': pgid, 'remaining': [p for p in process_rows() if p['pgid'] == pgid]})
        if self.old:
            self.log.close(); self.report['oldExit'] = self.old.poll()
        if hasattr(self, 'tunnel_thread'): self.tunnel.shutdown(); self.tunnel_thread.join(timeout=3)
        if hasattr(self, 'tunnel'): self.tunnel.server_close()
        remaining = [p for p in process_rows() if p['pid'] in self.owned or p['pgid'] in {v['pgid'] for v in self.owned.values()}]
        protected = next((p for p in process_rows() if p['pid'] == 80916), None)
        dump(self.out / 'owned-process-cleanup.json', {'owned': list(self.owned.values()), 'groups': cleanup, 'remaining': remaining,
            'dailyProtectedPid': 80916, 'dailyObserved': protected, 'dailySignalsSent': 0})
        self.check('owned-process-cleanup', 'pass' if not remaining and not any(p.get('refused') for p in cleanup) else 'blocked',
                   remaining=remaining, oldAndAutomaticNewIncluded=True, daily80916SignalsSent=0)
        if self.root.joinpath('logs/app.log').exists(): shutil.copyfile(self.root / 'logs/app.log', self.out / 'final-app.log')
        if hasattr(self, 'baseline'):
            assert inventory(self.baseline) == self.baseline_inventory
            self.check('baseline-original-remained-immutable', binarySha256=sha(self.baseline / 'Contents/MacOS/opencodex-desktop'))
        statuses = [c['status'] for c in self.report['checks']]
        if self.report.get('failureType') == 'fail' or 'fail' in statuses: self.report['result'] = 'fail'
        elif self.report.get('failure') or 'blocked' in statuses: self.report['result'] = 'blocked'
        elif self.args.mode == 'prepare': self.report['result'] = 'prepared-awaiting-parent-publication'
        elif self.report.get('completed'): self.report['result'] = 'pass'
        else: self.report['result'] = 'incomplete'
        self.report['finishedAt'] = dt.datetime.now(dt.timezone.utc).isoformat()
        self.flush()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--mode', choices=['prepare', 'execute'], default='prepare')
    parser.add_argument('--baseline-archive', default='/private/tmp/ocx020-final-updates-b1750f5c/assets/' + ASSET_NAME)
    parser.add_argument('--baseline-signature', default='/private/tmp/ocx020-final-updates-b1750f5c/assets/' + ASSET_NAME + '.sig')
    parser.add_argument('--out', default=str(DOCS / '.adg/evidence/0.2.21/macos-adjacent-ota-20261104'))
    parser.add_argument('--parent-beta-published', action='store_true', help='Set only after the parent explicitly reports final 021 beta publication')
    parser.add_argument('--source-sha', help='Exact parent-provided final 40-hex source commit')
    parser.add_argument('--publication-receipt', help='Parent notification JSON with public release ID/source/archive SHA')
    parser.add_argument('--target-archive', help='Unmodified final public 021 macOS updater archive')
    parser.add_argument('--target-signature', help='Unmodified corresponding updater signature sidecar')
    args = parser.parse_args()
    if args.mode == 'execute':
        if not args.parent_beta_published or not re.fullmatch('[0-9a-f]{40}', args.source_sha or '') or not all([args.publication_receipt, args.target_archive, args.target_signature]):
            parser.error('Execution is closed until parent beta publication notification + exact source SHA + receipt + original target archive/signature are provided')
    elif args.parent_beta_published or any([args.source_sha, args.publication_receipt, args.target_archive, args.target_signature]):
        parser.error('Target/publication inputs belong to execute mode; prepare never checks beta or launches an app')
    run = OtaRun(args)
    def interrupt(signum, frame): raise KeyboardInterrupt('Owned OTA interrupted by signal ' + str(signum))
    signal.signal(signal.SIGTERM, interrupt)
    try:
        run.prepare()
        if args.mode == 'execute':
            run.publication(); run.launch(); run.baseline_saves(); run.install(); run.restart_readback()
            run.report['completed'] = True
    except (Exception, KeyboardInterrupt) as error:
        run.report['failure'] = str(error)
        run.report['failureType'] = 'blocked' if isinstance(error, (Blocked, KeyboardInterrupt)) else 'fail'
        run.check('run-stopped', run.report['failureType'], reason=str(error))
        (run.out / 'failure.log').write_text(traceback.format_exc())
        if run.pid:
            try: run.capture('native-failure')
            except Exception: pass
    finally: run.close()
    return 0 if run.report['result'] in ['pass', 'prepared-awaiting-parent-publication'] else (1 if run.report['result'] == 'fail' else 2)

if __name__ == '__main__': sys.exit(main())
