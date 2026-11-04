#!/usr/bin/env python3
"""AC-021-02, macOS only. Default: read-only package/source inventory.

--prepare: verify OS isolation, then run isolated official help/schema probes.
--run: also build the current production save owner and run old/new clients.
--run --manual-client: disable automatic integration and explicitly hand off
the published catalog builder's output to an owned Codex configuration. This
does NOT verify the normal automatic injection path or the desktop GUI.
--run --automatic-chain: start an owned integration-ON installation; after the
real save, POST /api/sync uses the same producer as the true restart route.
Combine it with the unchanged production Rust route/guard tests. Never call a
global restart route or manually generate a post-rename catalog.
No package installation, inherited credentials, real model service or WebDAV.
All generated runtime/build files live in a new system temporary directory.
Evidence output must be inside the explicitly permitted governance directory.
"""
import argparse
import base64
import errno
import hashlib
import hmac
import http.server
import json
import os
from pathlib import Path
import plistlib
import queue
import re
import shlex
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import threading
import time
import urllib.request
import urllib.error

CODE = Path(__file__).resolve().parents[2]
EVIDENCE = CODE.parent / 'governance-main/.adg/evidence/0.2.21/owned-client-routing-20261104'
OFFICIAL = Path('/tmp/ocx020-owned-official-284/installation/node_modules/@bitkyc08/opencodex')
CODEX = Path('/tmp/ocx020-owned-codex-0166/node_modules/@openai/codex-darwin-arm64/vendor/aarch64-apple-darwin/bin/codex')
CACHE_TARGET = Path('/private/var/folders/fq/l0j4lwrs5ld6gpfx2pppv2640000gn/T/ocx-g0-rename-20261104-jlhlzrp6/target')
KEY = 'fixture-owned-ac02-key'
MARKER = 'OCX021_LOCAL_FIXTURE_OK'
RAW_MODELS = ['owned-routing-model', 'owned-second-model', 'owned-hidden-model']

# Product inputs only. In particular ui/tests, ui/qa, test/** and CI/native
# tools may advance independently and cannot invalidate the product identity.
FREEZE_PRODUCT_PATHS = (
    'apps/desktop/tauri/src',
    'apps/desktop/tauri/Cargo.toml', 'apps/desktop/tauri/Cargo.lock',
    'apps/desktop/tauri/build.rs', 'apps/desktop/tauri/tauri.conf.json',
    'apps/desktop/tauri/config',
    'apps/desktop/ui/src', 'apps/desktop/ui/index.html',
    'apps/desktop/ui/vite.config.ts', 'apps/desktop/ui/tsconfig.json',
    'apps/desktop/ui/package.json', 'apps/desktop/ui/package-lock.json',
)

class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, response, code, message, headers, new_url): return None

def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b''): h.update(chunk)
    return h.hexdigest()

def dump(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')

def sources():
    base = CODE / 'apps/desktop/tauri'
    paths = sorted(set(list((base / 'src').rglob('*.rs')) +
                       [base / p for p in ['Cargo.toml', 'Cargo.lock', 'build.rs', 'tauri.conf.json']]))
    return {str(p.relative_to(CODE)): sha(p) for p in paths if p.is_file()}

def verify_source_freeze(requested):
    """Read-only exact commit/product check; runs even if HEAD == freeze."""
    if not isinstance(requested, str) or not re.fullmatch(r'[0-9a-f]{40}', requested):
        raise ValueError('a complete lowercase 40-hex user-confirmed freeze SHA is required; nothing launched')
    def git(*args):
        return subprocess.check_output(['git', '-c', 'core.fsmonitor=false', *args], cwd=CODE)
    resolved = git('rev-parse', '--verify', requested + '^{commit}').decode().strip()
    if resolved != requested:
        raise ValueError('resolved freeze commit differs from the exact requested SHA; nothing launched')
    # Always compare the worktree (including staged/unstaged tracked edits).
    # Merely comparing HEAD would miss a dirty worktree at that same commit.
    changed = git('diff', '--name-only', '-z', requested, '--', *FREEZE_PRODUCT_PATHS)
    untracked = git('ls-files', '--others', '--exclude-standard', '-z', '--', *FREEZE_PRODUCT_PATHS)
    if changed or untracked:
        paths = sorted(set(item.decode() for item in (changed + untracked).split(b'\0') if item))
        raise ValueError('product inputs differ from explicit freeze: ' + ', '.join(paths) + '; nothing launched')
    tracked = git('ls-files', '-z', '--', *FREEZE_PRODUCT_PATHS)
    current = {}
    for item in tracked.split(b'\0'):
        if not item: continue
        relative = item.decode()
        path = CODE / relative
        if not path.is_file():
            raise ValueError('frozen product input is missing: ' + relative + '; nothing launched')
        current[relative] = sha(path)
    tree = git('ls-tree', '-r', '-z', requested, '--', *FREEZE_PRODUCT_PATHS)
    return {'sourceFreezeHead': requested, 'observedHead': git('rev-parse', 'HEAD').decode().strip(),
        'productMatchesFreezeCommit': True, 'productPathScope': list(FREEZE_PRODUCT_PATHS),
        'freezeProductTreeListingSha256': hashlib.sha256(tree).hexdigest(),
        'trackedProductFileCount': len(current), 'currentProductSha256': current,
        'testOnlyHeadAdvanceAllowed': True, 'worktreeCheckedEvenWhenHeadMatches': True,
        'untrackedProductInputsAllowed': False,
        'unrelatedUntrackedUiTestsNativeAndCiToolsAffectIdentity': False}

def inventory(official, codex, node):
    records = {}
    for name, base, lock in [
        ('opencodex', official, official.parents[2] / 'package-lock.json'),
        ('codex', codex.parents[4] / 'codex', codex.parents[6] / 'package-lock.json'),
    ]:
        package = base / 'package.json'
        records[name] = {'directory': str(base), 'exists': base.is_dir()}
        if package.is_file(): records[name]['package'] = json.loads(package.read_text())
        if lock.is_file():
            d = json.loads(lock.read_text())
            records[name]['lock'] = {'path': str(lock), 'sha256': sha(lock), 'packages': {
                k: v for k, v in d.get('packages', {}).items()
                if '@bitkyc08/opencodex' in k or '@openai/codex' in k}}
    records['executables'] = {str(p): {'exists': p.is_file(), 'sha256': sha(p) if p.is_file() else None}
                              for p in [node, codex, official / 'bin/ocx.mjs']}
    records['production'] = sources()
    return records

def owned_coordinator_files(root):
    # The published POSIX resolver intentionally ignores HOME/TMPDIR. Its files
    # are keyed by canonical CODEX_HOME, so admit only this new home's digest.
    namespace = Path('/private/tmp') / ('opencodex-runtime-v1-' + str(os.getuid()))
    digest = hashlib.sha256(str((root / 'home/.codex').resolve()).encode()).hexdigest()
    return [namespace / domain / (digest + '.sqlite' + suffix)
            for domain in ['native-write-locks', 'catalog-write-locks']
            for suffix in ['', '-journal', '-wal', '-shm']]

def sandbox_policy(root, ports=(), build=False, owned_coordinators=False):
    # Do not resolve or read default client paths. Runtime denies the real user
    # tree, with narrow exceptions for interpreter dependencies only.
    q = lambda p: json.dumps(str(p))
    lines = ['(version 1)', '(allow default)', '(deny network*)', '(deny file-write*)',
             f'(allow file-write* (subpath {q(root)}) (literal "/dev/null") (literal "/dev/tty"))',
             '(deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.securityd.xpc") (global-name "com.apple.security.agent"))']
    if build:
        for p in ['.codex', '.opencodex', '.npmrc', '.cargo/config', '.cargo/config.toml', '.cargo/credentials', '.cargo/credentials.toml',
                  'Library/Keychains', 'Library/Application Support/Codex',
                  'Library/Application Support/com.openai.codex',
                  'Library/Application Support/com.gzers.opencodex.desktop']:
            lines.append(f'(deny file-read* (subpath {q(Path("/Users/ezio") / p)}))')
    else:
        lines += ['(deny file-read* (subpath "/Users/ezio"))',
                  f'(allow file-read* (subpath {q(Path(sys.executable).resolve().parents[1])}) (literal {q(Path(shutil.which("node")).resolve())}))',
                  f'(deny file-read* (literal {q(root / "forbidden-canary")}))']
        for port in ports:
            lines += [f'(allow network-outbound (remote ip "localhost:{port}"))',
                      f'(allow network-bind (local ip "localhost:{port}"))',
                      f'(allow network-inbound (local ip "localhost:{port}"))']
        if owned_coordinators:
            files = owned_coordinator_files(root)
            namespace = files[0].parent.parent
            lines.append(f'(deny file-read* (subpath {q(namespace)}))')
            for directory in [namespace, files[0].parent, files[4].parent]:
                lines.append(f'(allow file-read-metadata (literal {q(directory)}))')
            for path in files:
                lines.append(f'(allow file-read* file-write* (literal {q(path)}))')
    lines.append('(deny file-read* (subpath "/Library/Keychains"))')
    return '\n'.join(lines) + '\n'

class Rpc:
    def __init__(self, runner, label):
        self.r, self.label, self.rows, self.q = runner, label, [], queue.Queue()
        self.err = (runner.root / (label + '.stderr')).open('w')
        self.proc = runner.spawn([str(runner.codex), 'app-server'], stderr=self.err)
        def read():
            for line in self.proc.stdout:
                try: self.q.put(json.loads(line))
                except ValueError: self.q.put({'unparsed': line})
        threading.Thread(target=read, daemon=True).start()
        self.call(1, 'initialize', {'clientInfo': {'name': 'ocx021-owned-routing', 'version': '0.0.0'},
                                    'capabilities': {'experimentalApi': True}})
        self.proc.stdin.write(json.dumps({'method': 'initialized'}) + '\n'); self.proc.stdin.flush()
    def call(self, ident, method, params):
        message = {'id': ident, 'method': method, 'params': params}
        self.rows.append({'send': message})
        self.proc.stdin.write(json.dumps(message) + '\n'); self.proc.stdin.flush()
        deadline = time.monotonic() + 35
        while time.monotonic() < deadline:
            row = self.q.get(timeout=max(.01, deadline - time.monotonic()))
            self.rows.append({'receive': row})
            if row.get('id') == ident:
                if 'error' in row: raise RuntimeError(f'{method}: {row["error"]}')
                return row['result']
        raise TimeoutError(method)
    def close(self):
        self.r.stop(self.proc); self.err.close()
        dump(self.r.out / (self.label + '.json'), self.rows)
        shutil.copyfile(self.r.root / (self.label + '.stderr'), self.r.out / (self.label + '.stderr'))

class Runner:
    def __init__(self, args):
        self.args = args
        self.freeze_identity = verify_source_freeze(args.source_freeze_head) if args.source_freeze_head else None
        self.code_head = (self.freeze_identity['observedHead'] if self.freeze_identity else
            subprocess.check_output(['git', '-c', 'core.fsmonitor=false', 'rev-parse', 'HEAD'], cwd=CODE, text=True).strip())
        self.freeze_product_delta = ''
        self.official, self.codex, self.node = args.official.resolve(), args.codex.resolve(), args.node.resolve()
        self.out = args.evidence.resolve() / time.strftime('run-%Y%m%d-%H%M%S')
        if not self.out.is_relative_to(EVIDENCE.resolve()): raise ValueError('evidence outside authorized directory')
        self.out.mkdir(parents=True, exist_ok=False)
        if self.freeze_identity: dump(self.out / 'source-freeze-before-launch.json', self.freeze_identity)
        self.root = Path(tempfile.mkdtemp(prefix='ocx021-owned-routing-')).resolve()
        self.work = self.root / 'work'
        self.children = []
        self.phase = 'before-rename'
        self.catalog_handoffs = []
        self.result = {'scope': 'AC-021-02 owned CLI directory and local routing', 'status': 'preparing',
                       'runtimeRoot': str(self.root), 'codeRoot': str(CODE), 'codeHead': self.code_head,
                       'sourceFreezeHead': args.source_freeze_head, 'evidence': str(self.out),
                       'productSourcesMatchFreezeCommit': not self.freeze_product_delta,
                       'dailyConfigAccess': False, 'nativeCredentialStoreAccess': False,
                       'dailyClientRestarted': False, 'realProviderOrWebdavRequests': False,
                       'desktopGuiVerified': False, 'commands': []}
        self.result['automaticClientIntegrationVerified'] = False
        self.result['manualClientSetup'] = args.manual_client
        self.result['automaticChainProbe'] = args.automatic_chain
        self.result['smokeSourceSha256'] = {p.name: sha(p) for p in sorted(Path(__file__).parent.glob('ocx021-owned-routing.*')) if p.is_file()}
        for path in ['home/.codex', 'home/.config', 'home/.cache', 'home/.local/share', 'home/.local/state',
                     'official', 'manager', 'work', 'tmp', 'cargo-home', 'build']:
            (self.root / path).mkdir(parents=True, exist_ok=True)
        self.env = {
            'HOME': str(self.root / 'home'), 'USERPROFILE': str(self.root / 'home'),
            'CODEX_HOME': str(self.root / 'home/.codex'), 'CODEX_SQLITE_HOME': str(self.root / 'home/.codex'),
            'OPENCODEX_HOME': str(self.root / 'official'),
            'XDG_CONFIG_HOME': str(self.root / 'home/.config'), 'XDG_CACHE_HOME': str(self.root / 'home/.cache'),
            'XDG_DATA_HOME': str(self.root / 'home/.local/share'), 'XDG_STATE_HOME': str(self.root / 'home/.local/state'),
            'TMPDIR': str(self.root / 'tmp'), 'TMP': str(self.root / 'tmp'), 'TEMP': str(self.root / 'tmp'),
            'PATH': str(self.node.parent) + ':/usr/bin:/bin:/usr/sbin:/sbin', 'CI': '1', 'TERM': 'dumb',
            'LANG': 'en_US.UTF-8', 'NO_PROXY': '127.0.0.1', 'no_proxy': '127.0.0.1',
            'CODEX_CLI_PATH': str(self.codex), 'OPENCODEX_SANDBOX': '1',
            'OPENCODEX_SANDBOX_ROOT': str(self.root / 'manager'), 'OPENCODEX_SANDBOX_BOUNDARY': str(self.root),
            'OCX021_CONTAINER': str(self.root), 'OTEL_SDK_DISABLED': 'true', 'DO_NOT_TRACK': '1',
            'OCX_TEST_HOME_GUARD': '1', 'OCX_REAL_HOME': '/Users/ezio',
        }
        self.sb = self.root / 'runtime.sb'; self.sb.write_text(sandbox_policy(self.root))
        (self.root / 'forbidden-canary').write_text('synthetic forbidden read')
        (self.root / 'home/.codex/config.toml').write_text(
            'cli_auth_credentials_store = "file"\ncheck_for_update_on_startup = false\nweb_search = "disabled"\n'
            '[analytics]\nenabled = false\n[feedback]\nenabled = false\n')
        dump(self.out / 'environment.json', self.env)
        dump(self.out / 'inventory.json', inventory(self.official, self.codex, self.node))
        # Wrapper runs the unmodified published launcher, not a replacement runtime.
        # Production cache_refresh intentionally env_clear()s. This explicit
        # owned runtime wrapper must re-establish every temporary client path.
        exports = ''.join('export ' + k + '=' + shlex.quote(v) + '\n' for k, v in self.env.items())
        (self.root / 'ocx').write_text('#!/bin/sh\n' + exports + 'exec ' + shlex.join([str(self.node), str(self.official / 'bin/ocx.mjs')]) + ' "$@"\n')
        (self.root / 'ocx').chmod(0o700)
    def log(self, msg): print(msg, flush=True)
    def verify_frozen_product(self, label):
        if not self.freeze_identity: return
        actual = verify_source_freeze(self.args.source_freeze_head)
        dump(self.out / ('source-freeze-' + label + '.json'), actual)
        if actual['currentProductSha256'] != self.freeze_identity['currentProductSha256']:
            raise RuntimeError('product bytes changed after the explicit source freeze; stop without retry')
    def command(self, argv, name, timeout=35, policy=None, env=None, check=True):
        if name.endswith('-manual-handoff'):
            if self.args.automatic_chain and self.phase != 'before-rename':
                raise RuntimeError('automatic chain forbids manual catalog generation after rename')
            self.catalog_handoffs.append({'name': name, 'phase': self.phase})
        cmd = ['/usr/bin/sandbox-exec', '-f', str(policy or self.sb), *map(str, argv)]
        start = time.monotonic()
        try:
            p = subprocess.run(cmd, env=env or self.env, cwd=self.work, stdin=subprocess.DEVNULL,
                               capture_output=True, text=True, timeout=timeout)
            status, stdout, stderr = p.returncode, p.stdout, p.stderr
        except subprocess.TimeoutExpired as e:
            status = 124
            stdout = (e.stdout or b'').decode(errors='replace') if isinstance(e.stdout, bytes) else (e.stdout or '')
            stderr = (e.stderr or b'').decode(errors='replace') if isinstance(e.stderr, bytes) else (e.stderr or '')
        (self.out / (name + '.stdout')).write_text(stdout)
        (self.out / (name + '.stderr')).write_text(stderr)
        self.result['commands'].append({'name': name, 'argv': cmd, 'exitCode': status, 'seconds': round(time.monotonic() - start, 3)})
        dump(self.out / 'result.json', self.result)
        self.log(f'{name}: exit {status}')
        if check and status: raise RuntimeError(f'{name} exited {status}; see {name}.stderr')
        return status, stdout, stderr
    def spawn(self, argv, **kwargs):
        p = subprocess.Popen(['/usr/bin/sandbox-exec', '-f', str(self.sb), *argv],
                             env=self.env, cwd=self.work, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                             text=True, start_new_session=True, **kwargs)
        self.children.append(p)
        self.result['commands'].append({'name': 'spawn', 'argv': argv, 'pid': p.pid, 'policy': str(self.sb)})
        return p
    def stop(self, p):
        if p.poll() is None:
            os.killpg(p.pid, signal.SIGTERM)
            try: p.wait(timeout=12)
            except subprocess.TimeoutExpired:
                os.killpg(p.pid, signal.SIGKILL); p.wait(timeout=5)
    def prepare(self):
        assert sys.platform == 'darwin', 'fail closed: only tested macOS Seatbelt isolation is supported'
        for path in [self.official / 'bin/ocx.mjs', self.codex, self.node]: assert path.is_file(), path
        code = 'import socket,errno;from pathlib import Path\n' + \
               f'p=Path({str(self.root / "forbidden-canary")!r})\n' + \
               'try:p.read_bytes();raise AssertionError("sandbox read escaped")\n' + \
               'except PermissionError:pass\n' + \
               's=socket.socket();assert s.connect_ex(("127.0.0.1",9)) in (errno.EPERM,errno.EACCES);print("file-read and network denied")\n'
        self.command([sys.executable, '-I', '-c', code], 'sandbox-canary')
        shutil.copyfile(self.sb, self.out / 'probe.sb')
        self.command([self.codex, '--version'], 'codex-version')
        self.command([self.codex, '--help'], 'codex-help')
        self.command([self.codex, 'exec', '--help'], 'codex-exec-help')
        self.command([self.codex, 'app-server', '--help'], 'codex-app-server-help')
        self.command([self.codex, 'debug', 'models', '--bundled'], 'codex-bundled-models')
        catalog = json.loads((self.out / 'codex-bundled-models.stdout').read_text())
        assert any('base_instructions' in row for row in catalog['models'])
        self.bundled = catalog
        dump(self.root / 'native-bundled.json', catalog)
        self.command([self.root / 'ocx', '--version'], 'opencodex-version')
        self.command([self.codex, 'app-server', 'generate-json-schema', '--out', self.root / 'schema'], 'app-server-schema')
        for pattern in ['*ModelList*', '*Initialize*']:
            for path in (self.root / 'schema').rglob(pattern):
                if path.is_file(): shutil.copyfile(path, self.out / path.name)
        self.result['isolationVerifiedBeforeOfficialLaunch'] = True
    def build(self):
        self.verify_frozen_product('before-build')
        if self.args.reuse_build:
            previous = self.args.reuse_build.resolve()
            if not previous.is_relative_to(EVIDENCE.resolve()): raise ValueError('build provenance outside evidence')
            result = json.loads((previous / 'result.json').read_text())
            before = json.loads((previous / 'source-after-build.json').read_text())
            if before != sources(): raise RuntimeError('cannot reuse build: production source hashes changed')
            if result['smokeSourceSha256']['ocx021-owned-routing.rs'] != sha(Path(__file__).with_suffix('.rs')):
                raise RuntimeError('cannot reuse build: Rust helper changed')
            original = Path(result['runtimeRoot']) / 'target/debug/ocx021-owned-routing'
            if sha(original) != result['helperSha256']: raise RuntimeError('cannot reuse build: binary digest changed')
            self.helper = self.root / 'build/ocx021-owned-routing'
            shutil.copy2(original, self.helper)
            self.result.update(buildExitCode=0, buildReusedFrom=str(previous), sourceChangesDuringBuild=[], helperSha256=sha(self.helper))
            if self.args.automatic_chain:
                assert result['codeHead'] == self.args.source_freeze_head
                assert result['productionTrueRouteGuards']['exitCode'] == 0
                assert result['productionGuardsSource']['sha256'] == sha(CODE / 'apps/desktop/tauri/src/modules/model_config/cache_refresh_tests.rs')
                assert sha(Path(result['productionGuardsSource']['binary'])) == result['productionGuardsSource']['binarySha256']
                self.result['productionTrueRouteGuards'] = result['productionTrueRouteGuards']
                self.result['productionGuardsSource'] = result['productionGuardsSource']
                self.result['productionGuardsEvidenceReusedFrom'] = str(previous)
                for name in ['production-true-route-guards.stdout', 'production-true-route-guards.stderr', 'true-route-guards.sb']:
                    shutil.copyfile(previous / name, self.out / name)
            dump(self.out / 'source-before-build.json', before); dump(self.out / 'source-after-build.json', before)
            for name in ['helper-Cargo.toml', 'helper-Cargo.lock']:
                shutil.copyfile(previous / name, self.out / name)
            self.log('Reused production build after matching all source, helper and binary hashes')
            return
        manifest = self.root / 'build/Cargo.toml'
        manifest.write_text('[package]\nname = "ocx021-owned-routing"\nversion = "0.0.0"\nedition = "2021"\n'
                            '[dependencies]\nopencodex-desktop = { path = ' + json.dumps(str(CODE / 'apps/desktop/tauri')) + ' }\n'
                            'serde_json = "1"\ntokio = { version = "1", features = ["rt"] }\nkeyring = { version = "3", default-features = false }\n'
                            'axum = { version = "0.8", default-features = false, features = ["http1", "json", "tokio"] }\n'
                            'base64 = "0.22.1"\nhmac = "0.12"\nsha2 = "0.10"\n'
                            '[dev-dependencies]\ntempfile = "3"\nreqwest = { version = "0.12", default-features = false }\n'
                            '[profile.dev]\ndebug = 0\nincremental = false\n')
        (manifest.parent / 'src').mkdir()
        shutil.copyfile(Path(__file__).with_suffix('.rs'), manifest.parent / 'src/main.rs')
        shutil.copyfile(Path(__file__).with_name('ocx021-owned-routing.commit.rs'),
                        manifest.parent / 'src/production-commit-simulation.rs')
        guards_source = CODE / 'apps/desktop/tauri/src/modules/model_config/cache_refresh_tests.rs'
        (manifest.parent / 'src/production-cache-refresh-guards.rs').write_text(
            '#[path = ' + json.dumps(str(guards_source)) + ']\nmod production;\n')
        shutil.copyfile(CODE / 'apps/desktop/tauri/Cargo.lock', manifest.with_name('Cargo.lock'))
        target = self.root / 'target'
        if self.args.cache_target.is_dir():
            self.log('Cloning dependency build cache into the new temporary root')
            subprocess.run(['/bin/cp', '-cR', str(self.args.cache_target), str(target)], check=True, timeout=50)
        toolchain = Path('/Users/ezio/.rustup/toolchains/stable-aarch64-apple-darwin/bin')
        env = dict(self.env, CARGO_HOME=str(self.root / 'cargo-home'), CARGO_NET_OFFLINE='true',
                   CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS='6', CARGO_INCREMENTAL='0', RUSTC=str(toolchain / 'rustc'))
        (self.root / 'cargo-home/registry').symlink_to('/Users/ezio/.cargo/registry', target_is_directory=True)
        policy = self.root / 'build.sb'; policy.write_text(sandbox_policy(self.root, build=True))
        shutil.copyfile(policy, self.out / 'build.sb')
        before = sources(); dump(self.out / 'source-before-build.json', before)
        self.log('Building production-linked save helper offline')
        build_args = [toolchain / 'cargo', 'build', '--offline', '--manifest-path', manifest, '--message-format=json-render-diagnostics']
        compile_guards = self.args.automatic_chain and not getattr(self.args, 'production_parent', False)
        if compile_guards: build_args.extend(['--bins', '--tests'])
        status, build_stdout, _ = self.command(build_args,
                                    'build', timeout=600, policy=policy, env=env, check=False)
        after = sources(); dump(self.out / 'source-after-build.json', after)
        drift = [p for p in before.keys() | after.keys() if before.get(p) != after.get(p)]
        self.result['sourceChangesDuringBuild'] = drift
        self.result['buildExitCode'] = status
        shutil.copyfile(manifest, self.out / 'helper-Cargo.toml')
        if manifest.with_name('Cargo.lock').exists(): shutil.copyfile(manifest.with_name('Cargo.lock'), self.out / 'helper-Cargo.lock')
        if status: raise RuntimeError('production-linked build failed; inspect source drift and build stderr before rerun')
        if drift: raise RuntimeError('production sources changed during compilation; rerun against a stable source snapshot')
        self.verify_frozen_product('after-build')
        self.helper = target / 'debug/ocx021-owned-routing'
        self.result['helperSha256'] = sha(self.helper)
        if compile_guards:
            artifacts = []
            for line in build_stdout.splitlines():
                try: row = json.loads(line)
                except ValueError: continue
                if row.get('reason') == 'compiler-artifact' and row.get('profile', {}).get('test') and row.get('executable'):
                    artifacts.append(row['executable'])
            assert len(artifacts) == 1, artifacts
            self.guards = Path(artifacts[0])
            self.result['productionGuardsSource'] = {'path': str(guards_source), 'sha256': sha(guards_source),
                'binary': str(self.guards), 'binarySha256': sha(self.guards),
                'method': 'unchanged production test file imported by generated path; actual production library linked'}
    def run_true_route_guards(self):
        if self.result.get('productionGuardsEvidenceReusedFrom'):
            self.log('Reused 17/17 production true-route guard evidence with matching source/binary hashes')
            return
        # These existing tests allocate their own attested loopback fixture ports.
        # This dedicated process allows loopback only, no real user files/keychain.
        policy = self.root / 'true-route-guards.sb'
        policy.write_text(sandbox_policy(self.root) +
            '(allow network-outbound (remote ip "localhost:*"))\n'
            '(allow network-bind (local ip "localhost:*"))\n'
            '(allow network-inbound (local ip "localhost:*"))\n')
        shutil.copyfile(policy, self.out / 'true-route-guards.sb')
        status, stdout, _ = self.command([self.guards, 'production_true_route_guards::production::', '--nocapture', '--test-threads=1'],
            'production-true-route-guards', timeout=60, policy=policy, check=False)
        passed = re.search(r'test result: ok\. (\d+) passed;', stdout)
        self.result['productionTrueRouteGuards'] = {'exitCode': status,
            'passed': int(passed.group(1)) if passed else 0,
            'restartEndpoint': 'attested synthetic local fixture only; not the official live restart route'}
        if status or not passed or int(passed.group(1)) == 0:
            raise RuntimeError('production true-route guard tests did not pass')
    def start_official(self, label):
        stdout = (self.root / (label + '.stdout')).open('w'); stderr = (self.root / (label + '.stderr')).open('w')
        p = subprocess.Popen(['/usr/bin/sandbox-exec', '-f', str(self.sb), str(self.root / 'ocx'),
                              'start', '--port', str(self.proxy_port)],
                             env=self.env, cwd=self.work, stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr, start_new_session=True)
        self.children.append(p)
        self.result['commands'].append({'name': label, 'pid': p.pid, 'argv': [str(self.root / 'ocx'), 'start', '--port', str(self.proxy_port)]})
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            if p.poll() is not None: raise RuntimeError(f'{label} exited {p.returncode}')
            try:
                with self.http.open(f'http://127.0.0.1:{self.proxy_port}/readyz', timeout=1) as response:
                    if response.status == 200: break
            except Exception: pass
            time.sleep(.15)
        else: raise TimeoutError(label + ' readiness')
        self.log(label + ': ready (published OpenCodex launcher)')
        return p, stdout, stderr
    def close_official(self, item, label):
        p, stdout, stderr = item
        self.stop(p); stdout.close(); stderr.close()
        for suffix in ['.stdout', '.stderr']: shutil.copyfile(self.root / (label + suffix), self.out / (label + suffix))
    def client_models(self, label):
        if self.args.manual_client:
            sidecar = self.root / 'manual-catalog.mjs'
            shutil.copyfile(Path(__file__).with_suffix('.mjs'), sidecar)
            catalog = self.root / 'home/.codex/opencodex-catalog.json'
            bun = self.official.parents[1] / 'bun/bin/bun.exe'
            self.command([bun, sidecar, self.official, catalog, self.root / 'native-bundled.json'], label + '-manual-handoff')
            # Only client wiring is manual. Actual rows come from the published
            # builder after reading the saved provider configuration.
            config = self.root / 'home/.codex/config.toml'
            config.write_text('cli_auth_credentials_store = "file"\ncheck_for_update_on_startup = false\nweb_search = "disabled"\n'
                'model_provider = "ocx021_owned"\nmodel_catalog_json = ' + json.dumps(str(catalog)) + '\n'
                '[analytics]\nenabled = false\n[feedback]\nenabled = false\n'
                '[model_providers.ocx021_owned]\nname = "Owned loopback fixture"\nwire_api = "responses"\nrequires_openai_auth = false\n'
                f'base_url = "http://127.0.0.1:{self.proxy_port}/v1"\n')
            shutil.copyfile(catalog, self.out / (label + '-manual-catalog.json'))
            shutil.copyfile(config, self.out / (label + '-manual-config.toml'))
        rpc = Rpc(self, label)
        try:
            result = rpc.call(2, 'model/list', {'includeHidden': True, 'limit': 1000})
            dump(self.out / (label + '-models.json'), result)
            return [row.get('id', row.get('model')) for row in result['data']]
        finally: rpc.close()
    def run(self):
        self.build()
        self.verify_frozen_product('before-owned-runtime')
        if self.args.automatic_chain: self.run_true_route_guards()
        self.seen = []
        runner = self
        class Fixture(http.server.BaseHTTPRequestHandler):
            def log_message(self, *args): pass
            def do_GET(self):
                if self.path != '/v1/models': self.send_error(404); return
                body = json.dumps({'object': 'list', 'data': [{'id': m, 'object': 'model', 'owned_by': 'owned-fixture'} for m in RAW_MODELS]}).encode()
                self.send_response(200); self.send_header('Content-Type', 'application/json'); self.end_headers(); self.wfile.write(body)
            def do_POST(self):
                body = json.loads(self.rfile.read(int(self.headers.get('Content-Length', 0))))
                row = {'path': self.path, 'model': body.get('model'), 'stream': body.get('stream'),
                       'syntheticAuthorization': self.headers.get('Authorization') == 'Bearer ' + KEY,
                       'method': 'POST', 'peer': self.client_address[0], 'body': body}
                runner.seen.append(row); dump(runner.out / 'fixture-requests.json', runner.seen)
                if self.path != '/v1/responses': self.send_error(404); return
                msg = {'id': 'msg_owned_ac02', 'type': 'message', 'role': 'assistant', 'status': 'completed',
                       'content': [{'type': 'output_text', 'text': MARKER, 'annotations': []}]}
                response = {'id': 'resp_owned_ac02', 'object': 'response', 'created_at': int(time.time()),
                            'status': 'completed', 'model': body.get('model'), 'output': [msg],
                            'usage': {'input_tokens': 1, 'output_tokens': 1, 'total_tokens': 2}}
                self.send_response(200)
                if not body.get('stream'):
                    self.send_header('Content-Type', 'application/json'); self.end_headers(); self.wfile.write(json.dumps(response).encode()); return
                self.send_header('Content-Type', 'text/event-stream'); self.send_header('Connection', 'close'); self.end_headers()
                for seq, event in enumerate([
                    {'type': 'response.created', 'response': dict(response, status='in_progress', output=[])},
                    {'type': 'response.output_item.added', 'output_index': 0, 'item': dict(msg, status='in_progress', content=[])},
                    {'type': 'response.content_part.added', 'item_id': msg['id'], 'output_index': 0, 'content_index': 0, 'part': {'type': 'output_text', 'text': '', 'annotations': []}},
                    {'type': 'response.output_text.delta', 'item_id': msg['id'], 'output_index': 0, 'content_index': 0, 'delta': MARKER},
                    {'type': 'response.output_text.done', 'item_id': msg['id'], 'output_index': 0, 'content_index': 0, 'text': MARKER},
                    {'type': 'response.content_part.done', 'item_id': msg['id'], 'output_index': 0, 'content_index': 0, 'part': msg['content'][0]},
                    {'type': 'response.output_item.done', 'output_index': 0, 'item': msg},
                    {'type': 'response.completed', 'response': response},
                ]):
                    event['sequence_number'] = seq
                    self.wfile.write(('event: ' + event['type'] + '\ndata: ' + json.dumps(event) + '\n\n').encode()); self.wfile.flush()
        fixture = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Fixture)
        fixture.daemon_threads = True
        self.fixture_port = fixture.server_port
        self.http = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())
        sock = socket.socket(); sock.bind(('127.0.0.1', 0)); self.proxy_port = sock.getsockname()[1]; sock.close()
        threading.Thread(target=fixture.serve_forever, daemon=True).start()
        self.sb.write_text(sandbox_policy(self.root, [self.proxy_port, self.fixture_port]))
        if self.args.allow_owned_coordinator_files:
            files = owned_coordinator_files(self.root)
            for path in files:
                if path.exists() or path.is_symlink():
                    raise RuntimeError('new owned coordinator key unexpectedly exists; refusing to read/reuse it')
            for directory in {p.parent for p in files} | {files[0].parent.parent}:
                state = directory.lstat()
                assert not directory.is_symlink() and directory.is_dir()
                assert state.st_uid == os.getuid() and state.st_mode & 0o777 == 0o700
            self.sb.write_text(sandbox_policy(self.root, [self.proxy_port, self.fixture_port], owned_coordinators=True))
            dump(self.out / 'owned-coordinator-files.json', {'syntheticHome': self.env['CODEX_HOME'],
                'source': str(self.official / 'src/codex/user-identity.ts'),
                'scope': 'only new CODEX_HOME SHA256 exact SQLite files and SQLite sidecars; shared namespace read denied',
                'allExactFilesAbsentBeforeLaunch': True, 'files': [str(p) for p in files]})
            self.result['ownedCoordinatorExactFiles'] = [str(p) for p in files]
        shutil.copyfile(self.sb, self.out / 'runtime.sb')
        self.result['loopbackPorts'] = {'proxy': self.proxy_port, 'fixture': self.fixture_port}
        try:
            self.command([sys.executable, '-I', '-c',
                'import socket,errno;'
                f's=socket.create_connection(("127.0.0.1",{self.fixture_port}),timeout=2);s.close();'
                's=socket.socket();assert s.connect_ex(("127.0.0.1",9)) in (errno.EPERM,errno.EACCES);'
                'print("exact fixture port allowed; unlisted loopback port denied")'], 'runtime-port-canary')
            config = {'port': self.proxy_port, 'hostname': '127.0.0.1', 'defaultProvider': 'Krill-1',
                      'providers': {'Krill-1': {'adapter': 'openai-responses', 'authMode': 'key',
                          'baseUrl': f'http://127.0.0.1:{self.fixture_port}/v1', 'apiKey': KEY,
                          'models': RAW_MODELS, 'selectedModels': RAW_MODELS[:2], 'liveModels': False,
                          'allowPrivateNetwork': True, 'newModelPolicy': 'off'}},
                      'modelPickerOrder': ['Krill-1/' + m for m in RAW_MODELS[:2]],
                      'disabledModels': ['Krill-1/' + RAW_MODELS[2]],
                      'claudeCode': {'systemEnv': False, 'enabled': False},
                      'clientIntegrations': {'codex': not self.args.manual_client, 'grok': False, 'claude': False},
                      'syncResumeHistory': False, 'serviceHealthRecovery': {'enabled': False},
                      'tokenGuardian': {'enabled': False}, 'oauthOpenBrowser': False}
            dump(self.root / 'official/config.json', config)
            dump(self.root / 'home/.codex/opencodex-catalog.json', self.bundled)
            if self.args.seed_installed_fixture:
                self.seed_installation()
            if self.args.allow_owned_coordinator_files:
                probe = self.root / 'coordinator-probe.mjs'
                shutil.copyfile(Path(__file__).with_name('ocx021-owned-routing.coordinator.mjs'), probe)
                self.command([self.official.parents[1] / 'bun/bin/bun.exe', probe, self.official, self.root, self.proxy_port],
                             'official-coordinator-preflight')
            if self.args.automatic_chain:
                # Synthetic API auth for the owned Codex client; no OAuth or real account.
                dump(self.root / 'home/.codex/auth.json', {'OPENAI_API_KEY': 'fixture-owned-codex-client-key'})
                (self.root / 'home/.codex/auth.json').chmod(0o600)
            old = self.start_official('official-before')
            if self.args.automatic_chain: self.held_official = old
            try:
                before = self.client_models('codex-before'); self.result['modelsBefore'] = before
                assert all('Krill-1/' + m in before for m in RAW_MODELS[:2]), before
                assert not self.seen, 'directory-only baseline generated a model request'
            finally:
                if not self.args.automatic_chain: self.close_official(old, 'official-before')
            if self.args.automatic_chain:
                self.catalog_before_rename = (self.root / 'home/.codex/opencodex-catalog.json').read_bytes()
                self.wiring_before_rename = (self.root / 'home/.codex/config.toml').read_bytes()
            self.command([self.helper], 'production-save')
            self.phase = 'after-rename'
            saved = json.loads((self.out / 'production-save.stdout').read_text()); assert saved['success']
            self.result['productionSave'] = saved
            shutil.copyfile(self.root / 'official/config.json', self.out / 'owned-config-after-save.json')
            if self.args.automatic_chain:
                self.automatic_chain()
                return
            new = self.start_official('official-after')
            try:
                after = self.client_models('codex-after'); self.result['modelsAfter'] = after
                assert all('Krill/' + m in after for m in RAW_MODELS[:2]), after
                assert not any(isinstance(m, str) and m.startswith('Krill-1/') for m in after), after
                for f in ['config.toml', 'opencodex-catalog.json', 'opencodex.config.toml']:
                    p = self.root / 'home/.codex' / f
                    if p.exists(): shutil.copyfile(p, self.out / ('loaded-' + f))
                status, stdout, stderr = self.command([self.codex, 'exec', '--json', '--skip-git-repo-check',
                    '--sandbox', 'read-only', '--ephemeral', '--model', 'Krill/' + RAW_MODELS[0],
                    '--cd', self.work, '--output-last-message', self.root / 'last-message.txt',
                    'Return the fixture text. Do not call any tools.'], 'codex-local-turn', timeout=75, check=False)
                self.result['wireExitCode'] = status
                self.result['fixturePosts'] = len(self.seen)
                self.result['rawModelPreservedAtEndpoint'] = bool(self.seen) and all(row['model'] == RAW_MODELS[0] for row in self.seen)
                self.result['syntheticCredentialRouted'] = bool(self.seen) and all(row['syntheticAuthorization'] for row in self.seen)
                last = self.root / 'last-message.txt'
                self.result['fixtureResponseConsumedByCodex'] = last.is_file() and last.read_text().strip() == MARKER
                if last.exists(): shutil.copyfile(last, self.out / 'codex-last-message.txt')
                assert self.result['rawModelPreservedAtEndpoint'], self.seen
                assert self.result['syntheticCredentialRouted']
                assert status == 0 and self.result['fixtureResponseConsumedByCodex'], (status, stderr[-1500:])
                self.result['automaticClientIntegrationVerified'] = not self.args.manual_client
                self.result['status'] = ('passed-manual-client-directory-and-local-routing' if self.args.manual_client
                                         else 'passed-cli-directory-and-local-routing')
            finally: self.close_official(new, 'official-after')
        finally:
            if self.args.automatic_chain and hasattr(self, 'held_official'):
                self.close_official(self.held_official, 'official-before')
            fixture.shutdown(); fixture.server_close()
    def seed_installation(self):
        # File-only fixture accepted explicitly by the user. This never installs,
        # bootstraps, edits or stops a launchd job and is not OS ownership proof.
        state = {'version': 1, 'codexHome': self.env['CODEX_HOME'],
                 'opencodexHome': self.env['OPENCODEX_HOME'],
                 'codexSqliteHome': self.env['CODEX_SQLITE_HOME'], 'revision': 0,
                 'launcherPath': str(self.root / 'ocx')}
        definition = {'Label': 'com.opencodex.proxy',
            'ProgramArguments': [str(self.node), str(self.official / 'bin/ocx.mjs'), 'start', '--port', str(self.proxy_port)],
            'WorkingDirectory': str(self.work),
            'EnvironmentVariables': {k: self.env[k] for k in ['HOME', 'CODEX_HOME', 'CODEX_SQLITE_HOME', 'OPENCODEX_HOME']}}
        path = self.root / 'home/Library/LaunchAgents/com.opencodex.proxy.plist'
        path.parent.mkdir(parents=True, exist_ok=True); path.write_bytes(plistlib.dumps(definition))
        dump(self.root / 'official/service-state.json', state)
        dump(self.out / 'old-installation-fixture.json', {
            'synthetic': True, 'seededBeforeRename': True, 'serviceManagerMutated': False,
            'source': 'published service/state-record.mjs v1 parser and service-manager-probe.ts inspectLaunchd homes contract',
            'state': state, 'definition': definition,
            'limitation': 'file-only ownership fixture; not proof of an isolated launchd namespace or GUI restart'})
        self.result['oldInstallationFixtureSynthetic'] = True
    def automatic_chain(self):
        catalog = self.root / 'home/.codex/opencodex-catalog.json'
        before = self.catalog_before_rename
        config_before = self.wiring_before_rename
        self.result.update(oldInstallationSeededBeforeRename=self.args.seed_installed_fixture,
                           initialNativeCatalogSeededBeforeRename=True,
                           oldRoutedCatalogGeneratedByOfficialStartup=True,
                           manualCatalogCallsAfterRename=sum(h['phase'] == 'after-rename' for h in self.catalog_handoffs),
                           oldClientWiringGeneratedByOfficialStartup=True, manualPostRenameCatalogHandoff=False,
                           automaticClientIntegrationVerified=False, automaticDirectoryGatePassed=False,
                           releaseGatePassed=False, globalRestartEndpointCalled=False,
                           producerProof='official POST /api/sync -> syncModelsToCodex, also called by runCodexRestart.defaultSyncCatalog',
                           productionTrueRouteInvokedAgainstOfficialRuntime=False)
        current = json.loads((self.root / 'official/config.json').read_text())
        assert current['clientIntegrations']['codex'] is True
        assert 'Krill' in current['providers'] and 'Krill-1' not in current['providers']
        # This same official process was started while the provider was Krill-1;
        # /api/sync must reload the actual saved file, not reuse its boot object.
        status, refresh = self.sync_official_producer()
        self.result['officialProducerSync'] = {'httpStatus': status, 'result': refresh}
        self.result['catalogChangedByOfficialProducer'] = catalog.read_bytes() != before
        self.result['clientConfigChangedByOfficialProducer'] = (self.root / 'home/.codex/config.toml').read_bytes() != config_before
        for filename in ['opencodex-catalog.json', 'config.toml', 'opencodex.config.toml']:
            p = self.root / 'home/.codex' / filename
            if p.is_file(): shutil.copyfile(p, self.out / ('automatic-' + filename))
        cache = self.root / 'home/.codex/models_cache.json'
        if cache.is_file(): shutil.copyfile(cache, self.out / 'automatic-models_cache.json')
        if not (status == 200 and refresh.get('status') == 'applied' and refresh.get('ok') is True
                and refresh.get('catalogWritten') is True and refresh.get('refreshOutcome') == 'committed'):
            self.result['status'] = 'official-producer-refused-or-unconfirmed'
            raise RuntimeError('official producer did not confirm an applied, committed catalog write; see official-producer-sync.json')
        # No generator and no fallback after rename. The ONLY directory producer
        # called is the one shared with the true restart route.
        # Reopen only an owned Codex app-server; never request a global restart.
        models = self.client_models('codex-after-official-producer')
        self.result['modelsAfterOfficialProducer'] = models
        wanted = ['Krill/' + m for m in RAW_MODELS[:2]]
        fresh = all(m in models for m in wanted) and not any(m.startswith('Krill-1/') for m in models)
        self.result['automaticCatalogRegenerationVerified'] = fresh and self.result['catalogChangedByOfficialProducer']
        if not self.result['automaticCatalogRegenerationVerified']:
            self.result['status'] = 'automatic-chain-release-gate-failed'
            raise RuntimeError('official producer did not generate the renamed directory loaded by new Codex; no manual fallback attempted')
        self.result['automaticDirectoryGatePassed'] = True
        assert not self.catalog_handoffs, 'automatic mode must never invoke manual catalog sidecar'
        status, _, _ = self.command([self.codex, 'exec', '--json', '--skip-git-repo-check',
            '--sandbox', 'read-only', '--ephemeral', '--model', wanted[0], '--cd', self.work,
            '--output-last-message', self.root / 'automatic-last-message.txt',
            'Return the fixture text. Do not call any tools.'], 'automatic-chain-local-turn', timeout=75, check=False)
        last = self.root / 'automatic-last-message.txt'
        consumed = last.is_file() and last.read_text().strip() == MARKER
        endpoint_reached = len(self.seen) == 1 and self.seen[0]['path'] == '/v1/responses'
        raw_model_preserved = endpoint_reached and self.seen[0]['model'] == RAW_MODELS[0]
        key_matched = endpoint_reached and self.seen[0]['syntheticAuthorization']
        usage_file = self.root / 'official/usage.jsonl'
        route_rows = [json.loads(line) for line in usage_file.read_text().splitlines() if line.strip()] if usage_file.is_file() else []
        matching = [row for row in route_rows if row.get('requestedModel') == wanted[0]]
        dump(self.out / 'owned-routing-usage.json', matching)
        selected_new_provider = len(matching) == 1 and matching[0].get('provider') == 'Krill'
        routed = raw_model_preserved and key_matched and selected_new_provider
        self.result.update(wireExitCode=status, fixturePosts=len(self.seen), fixtureResponseConsumedByCodex=consumed,
                           localEndpointReached=endpoint_reached, rawModelPreservedAtEndpoint=raw_model_preserved,
                           syntheticCredentialRouted=key_matched, liveNewProviderSelected=selected_new_provider)
        if last.is_file(): shutil.copyfile(last, self.out / 'automatic-last-message.txt')
        if status == 0 and consumed and endpoint_reached and not routed:
            self.result['status'] = 'blocked-stale-live-provider-routing'
        assert status == 0 and consumed and routed, 'automatic catalog must also route successfully'
        self.result.update(status='passed-true-route-shared-producer-and-local-routing',
                           automaticClientIntegrationVerified=True, automaticDirectoryGatePassed=True,
                           syntheticCredentialRouted=routed,
                           trueRouteProducerCompositeVerified=self.result['productionTrueRouteGuards']['exitCode'] == 0)
    def sync_official_producer(self):
        # Explicit allowed path only; no restart route or generic remote URL API.
        path = self.root / 'official/runtime-port.json'
        record_bytes = path.read_bytes(); record = json.loads(record_bytes)
        assert record['port'] == self.proxy_port and record['pid'] > 0
        challenge = base64.urlsafe_b64encode(os.urandom(32)).decode().rstrip('=')
        request = urllib.request.Request(f'http://127.0.0.1:{self.proxy_port}/healthz',
            headers={'x-opencodex-attestation-challenge': challenge})
        with self.http.open(request, timeout=8) as response:
            health = json.load(response)
            proof = response.headers.get('x-opencodex-attestation-proof', '')
        expected = base64.urlsafe_b64encode(hmac.new(record['attestationSecret'].encode(),
            f'opencodex-local-management-v1\n{challenge}\n{record["pid"]}\n{record["port"]}'.encode(), hashlib.sha256).digest()).decode().rstrip('=')
        assert hmac.compare_digest(proof, expected), 'owned official runtime attestation failed'
        assert health['service'] == 'opencodex' and health['pid'] == record['pid']
        assert path.read_bytes() == record_bytes, 'runtime changed before credential read'
        token = (self.root / 'official/admin-api-token').read_text().strip()
        assert re.fullmatch(r'ocx_admin_[A-Za-z0-9_-]{43}', token)
        request = urllib.request.Request(f'http://127.0.0.1:{self.proxy_port}/api/sync', data=b'', method='POST',
            headers={'Authorization': 'Bearer ' + token, 'Content-Length': '0'})
        try: response = self.http.open(request, timeout=55)
        except urllib.error.HTTPError as error: response = error
        with response:
            status = response.code; body = json.load(response)
        assert path.read_bytes() == record_bytes, 'runtime changed during official sync'
        dump(self.out / 'official-producer-sync.json', {'method': 'POST', 'path': '/api/sync',
            'attestationVerifiedBeforeCredentialRead': True, 'httpStatus': status, 'result': body})
        self.result['commands'].append({'name': 'official-producer-sync', 'method': 'POST',
            'url': f'http://127.0.0.1:{self.proxy_port}/api/sync', 'httpStatus': status, 'credential': 'owned generated token, not logged'})
        return status, body
    def finish(self):
        for p in reversed(self.children): self.stop(p)
        for f in self.root.glob('official-*.*'):
            if f.suffix in ['.stdout', '.stderr']: shutil.copyfile(f, self.out / f.name)
        self.result['allOwnedChildrenReaped'] = all(p.poll() is not None for p in self.children)
        self.result['catalogHandoffs'] = self.catalog_handoffs
        if hasattr(self, 'proxy_port'):
            port_states = {}
            for port in [self.proxy_port, self.fixture_port]:
                with socket.socket() as sock:
                    sock.settimeout(.5); port_states[str(port)] = sock.connect_ex(('127.0.0.1', port))
            self.result['ownedPortConnectResultsAfterCleanup'] = port_states
        final_sources = sources()
        dump(self.out / 'source-after-run.json', final_sources)
        built_sources = self.out / 'source-after-build.json'
        if built_sources.is_file():
            built = json.loads(built_sources.read_text())
            drift = [p for p in built.keys() | final_sources.keys() if built.get(p) != final_sources.get(p)]
            self.result['sourceChangesAfterBuild'] = drift
            if drift and self.result['status'].startswith('passed-'):
                self.result.update(status='source-drift-invalidated-result', automaticClientIntegrationVerified=False,
                                   automaticDirectoryGatePassed=False, trueRouteProducerCompositeVerified=False,
                                   productionPrepareIntegrationPassed=False)
        try:
            self.verify_frozen_product('after-run')
            if self.freeze_identity: self.result['exactProductFreezeVerifiedAfterRun'] = True
        except Exception as error:
            self.result.update(status='product-freeze-mismatch-invalidated-result',
                productFreezeError=str(error), exactProductFreezeVerifiedAfterRun=False,
                automaticClientIntegrationVerified=False, automaticDirectoryGatePassed=False,
                productionPrepareIntegrationPassed=False, releaseGatePassed=False)
        dump(self.out / 'result.json', self.result)
        self.log('Evidence: ' + str(self.out))

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--official', type=Path, default=OFFICIAL)
    parser.add_argument('--codex', type=Path, default=CODEX)
    parser.add_argument('--node', type=Path, default=Path(shutil.which('node') or '/missing-node'))
    parser.add_argument('--evidence', type=Path, default=EVIDENCE)
    parser.add_argument('--cache-target', type=Path, default=CACHE_TARGET, help='clone cache only; never write original')
    parser.add_argument('--manual-client', action='store_true', help='explicit manual isolated client setup; automatic injection remains unverified')
    parser.add_argument('--reuse-build', type=Path, help='reuse an evidence run only after production/helper/binary hashes all match')
    parser.add_argument('--automatic-chain', action='store_true', help='seed only old install; diagnose real production automatic refresh without any post-rename catalog handoff')
    parser.add_argument('--seed-installed-fixture', action='store_true', help='explicit user-authorized file-only old installation fixture; never mutates a live service manager')
    parser.add_argument('--source-freeze-head', help='exact user-authorized final core HEAD; required for the final automatic run')
    parser.add_argument('--allow-owned-coordinator-files', action='store_true', help='allow only new temporary CODEX_HOME digest SQLite lock files; no existing namespace DBs')
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--prepare', action='store_true'); mode.add_argument('--run', action='store_true')
    args = parser.parse_args()
    if args.manual_client and args.automatic_chain: parser.error('manual and automatic-chain modes are exclusive')
    if args.seed_installed_fixture and not args.automatic_chain: parser.error('old installation seed is restricted to automatic-chain mode')
    if args.allow_owned_coordinator_files and not args.automatic_chain: parser.error('exact coordinator exception requires automatic-chain mode')
    if args.automatic_chain and args.run and not args.source_freeze_head:
        parser.error('final automatic run requires explicit --source-freeze-head after user freeze')
    if not args.prepare and not args.run:
        print(json.dumps(inventory(args.official, args.codex, args.node.resolve()), ensure_ascii=False, indent=2)); return 0
    runner = Runner(args)
    try:
        runner.prepare()
        if args.run: runner.run()
        else: runner.result['status'] = 'isolation-and-official-interface-prepared'
    except Exception as e:
        if runner.result['status'] == 'preparing': runner.result['status'] = 'failed'
        runner.result['error'] = str(e)
        runner.log('FAILED: ' + str(e)); return 1
    finally: runner.finish()
    return 0

if __name__ == '__main__': sys.exit(main())
