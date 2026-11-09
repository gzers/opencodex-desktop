"""Isolated full-source producer overlay. Never replaces production archive/pins."""
from pathlib import Path
import hashlib
import json
import os
import shutil
import sys

assert os.name == 'nt' and os.environ.get('GITHUB_ACTIONS') == 'true'
ROOT = Path(__file__).resolve().parents[2]
assert len(sys.argv) == 5, 'Explicit frozen installed dependency source required'
original, output, report, installed = [Path(p).resolve() for p in sys.argv[1:5]]
boundary = Path(os.environ['RUNNER_TEMP']).resolve()
for p in [original, output, report, installed]:
    p.relative_to(boundary)
assert not output.exists() and not report.exists()
assert json.loads((original / 'package.json').read_bytes())['version'] == '2.97.0-desktop023.lifecycle.1-record-prototype'
# A source-only archive is NOT a dependency supply proof: Bun auto-install
# otherwise resolves missing packages from its cache/network. Copy only the
# exact already-offline-installed dependency tree; the runtime runs --no-install.
policy = json.loads((ROOT / '.github/runtime-platforms-023.json').read_bytes())
assert installed.name == 'opencodex' and installed.parent.name == '@bitkyc08'
assert json.loads((installed / 'package.json').read_bytes())['version'] == policy['runtime']['version']
dependency_root = installed.parents[1]
assert dependency_root.name == 'node_modules'
for name, row in policy['packageLock']['packages'].items():
    if not name.startswith('node_modules/') or name == 'node_modules/@bitkyc08/opencodex':
        continue
    declaration = dependency_root.parent / name / 'package.json'
    if declaration.exists():
        assert json.loads(declaration.read_bytes())['version'] == row['version'], name

def dependency_inventory(root):
    rows = []
    for p in sorted(root.rglob('*')):
        assert not p.is_symlink(), 'Dependency links are not accepted'
        if p == installed or installed in p.parents:
            continue
        if p.is_file():
            raw = p.read_bytes()
            rows.append(dict(path=p.relative_to(root).as_posix(), bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest()))
    assert rows and len(rows) < 100000 and sum(r['bytes'] for r in rows) < 1024**3
    return rows

supplied_dependencies = dependency_inventory(dependency_root)
shutil.copytree(original, output)
assert not (output / 'node_modules').exists()
# Skip the original runtime package only; all supplied dependencies are retained.
shutil.copytree(dependency_root, output / 'node_modules',
                ignore=lambda path, names: ['opencodex'] if Path(path).resolve() == installed.parent else [])
assert dependency_inventory(dependency_root) == supplied_dependencies
candidate_dependencies = dependency_inventory(output / 'node_modules')
assert candidate_dependencies == supplied_dependencies, 'Copied dependency bytes differ'

before = {}
changed = {}


def update(name, modify):
    p = output / name
    raw = p.read_bytes()
    before[name] = hashlib.sha256(raw).hexdigest()
    text = raw.decode()
    rewritten = modify(text)
    assert rewritten != text
    p.write_bytes(rewritten.encode())
    changed[name] = hashlib.sha256(p.read_bytes()).hexdigest()


def replace_one(text, old, new):
    assert text.count(old) == 1, 'Exact source hook preimage required'
    return text.replace(old, new, 1)


helper = 'src/lib/windows-runtime-lifetime.ts'
assert not (output / helper).exists()
(output / helper).write_bytes((ROOT / 'test/smoke/runtime023-windows-runtime-lifetime.ts').read_bytes())
changed[helper] = hashlib.sha256((output / helper).read_bytes()).hexdigest()
boot_helper = 'src/lib/windows-runtime-start-boot.ts'
assert not (output / boot_helper).exists()
(output / boot_helper).write_bytes((ROOT / 'test/smoke/runtime023-windows-start-boot.ts').read_bytes())
changed[boot_helper] = hashlib.sha256((output / boot_helper).read_bytes()).hexdigest()
trace_helper = 'src/lib/desktop-lifecycle-trace.ts'
assert not (output / trace_helper).exists()
(output / trace_helper).write_bytes((ROOT / 'test/smoke/runtime023-lifecycle-trace.ts').read_bytes())
changed[trace_helper] = hashlib.sha256((output / trace_helper).read_bytes()).hexdigest()



def cli(text):
    assert text.startswith('#!/usr/bin/env bun\n')
    text = replace_one(text, '#!/usr/bin/env bun\n', '#!/usr/bin/env bun\nimport "../lib/windows-runtime-start-boot";\n')
    text = text.replace('import { serviceStayOutExitCode', 'import { currentWindowsRuntimeLifetime } from "../lib/windows-runtime-lifetime";\nimport { serviceStayOutExitCode', 1)
    return replace_one(text, 'async function handleStart(options: { block?: boolean } = {}) {',
                       'async function handleStart(options: { block?: boolean } = {}) {\n  if(process.platform==="win32" && !currentWindowsRuntimeLifetime())throw new Error("Desktop service startup has no pre-import lifetime authority");')


def state(text):
    text = 'import { currentWindowsRuntimeLifetime, type WindowsRuntimeLifetime } from "../lib/windows-runtime-lifetime";\n' + text
    text = replace_one(text, 'export type RuntimePortState = {',
                       'export type RuntimePortState = {\n  desktopLifetime?: WindowsRuntimeLifetime;')
    return replace_one(text, '  atomicWriteFile(getRuntimePortPath(), JSON.stringify(state, null, 2) + "\\n");',
                       '  const desktopLifetime=currentWindowsRuntimeLifetime();\n'
                       '  if(desktopLifetime && (desktopLifetime.pid!==state.pid || desktopLifetime.pid!==process.pid))throw new Error("Runtime lifetime PID changed");\n'
                       '  atomicWriteFile(getRuntimePortPath(), JSON.stringify({...state,...(desktopLifetime?{desktopLifetime}: {})}, null, 2) + "\\n");')


def health(text):
    text = 'import { createWindowsRuntimeLifetimeProof } from "../../lib/windows-runtime-lifetime";\n' + text
    return replace_one(text, '          if (proof) response.headers.set(LOCAL_ATTESTATION_PROOF_HEADER, proof);',
                       '          if (proof) response.headers.set(LOCAL_ATTESTATION_PROOF_HEADER, proof);\n'
                       '          const lifetimeProof=createWindowsRuntimeLifetimeProof(localAttestationSecret,challenge,healthPort);\n'
                       '          if(lifetimeProof)response.headers.set("x-opencodex-runtime-lifetime-proof",lifetimeProof);')


update('src/cli/index.ts', cli)
update('src/config/process-state.ts', state)
update('src/server/index/serve-options.ts', health)


def management(text):
    text = 'import { traceSync, traceAsync } from "../lib/desktop-lifecycle-trace";\n' + text
    text = replace_one(text, 'installedServiceRespawnRisk();',
                       'traceSync("manager-risk", () => installedServiceRespawnRisk());')
    text = replace_one(text, 'const refusal = preflightStopTeardownOwnership();',
                       'const refusal = traceSync("ownership-preflight", () => preflightStopTeardownOwnership());')
    text = replace_one(text, 'serviceStop = sibling ? "absent" : stopServiceIfInstalledDetailed();',
                       'serviceStop = sibling ? "absent" : traceSync("manager-stop", () => stopServiceIfInstalledDetailed());')
    return replace_one(text, 'const teardown = await performStopTeardown(url, { ownsReceipt: deferralMatchesReceipt });',
                       'const teardown = await traceAsync("official-teardown", () => performStopTeardown(url, { ownsReceipt: deferralMatchesReceipt }));')


def teardown(text):
    text = 'import { traceSync, traceAsync } from "../lib/desktop-lifecycle-trace";\n' + text
    text = replace_one(text, 'await (await import("../codex/inject")).restoreNativeCodexAsync(options);',
                       'await traceAsync("native-client-restore", async () => (await import("../codex/inject")).restoreNativeCodexAsync(options));')
    return replace_one(text, 'grok = guard.run(() => strip());',
                       'grok = traceSync("grok-restore", () => guard.run(() => strip()));')


update('src/server/management-api.ts', management)
update('src/server/stop-teardown.ts', teardown)

package = json.loads((output / 'package.json').read_bytes())
package['desktop023LifetimeProducerPrototype'] = dict(candidateOnly=True, userAdopted='2026-11-09', source=os.environ.get('GITHUB_SHA'), files=changed, limits='Not released; Desktop Stop consumer and official restore/foreign-user lifecycle still required.')
(output / 'package.json').write_bytes((json.dumps(package, indent=2, ensure_ascii=False)+'\n').encode())
compatibility_path = output / 'src/generated/compatibility-version.json'
compatibility = json.loads(compatibility_path.read_bytes())
entries = {row['path']: row for row in compatibility['files']}
for path in ['package.json', *changed]:
    entries[path] = {**entries.get(path, {'path': path}), 'sha256': hashlib.sha256((output / path).read_bytes()).hexdigest()}
compatibility['files'] = [entries[path] for path in sorted(entries)]
compatibility_path.write_bytes((json.dumps(compatibility, indent=2, ensure_ascii=False)+'\n').encode())
with report.open('x', encoding='utf-8') as stream:
    json.dump(dict(accepted=False, diagnosticOnly=True, candidateOnly=True, originalUnmodified=True,
                   originalSourcePreimages=before, candidateSourcePostimages=changed,
                   dependencies=dict(source='exact frozen offline bundle', autoInstallDisabled=True, count=len(supplied_dependencies), bytes=sum(r['bytes'] for r in supplied_dependencies), inventorySha256=hashlib.sha256(json.dumps(supplied_dependencies,sort_keys=True,separators=(',', ':')).encode()).hexdigest(), originalUnchanged=True, candidateBytesEqual=True),
                   source=os.environ.get('GITHUB_SHA'), productionPinsChanged=False,
                   scope='Separate disposable producer clone: acquire native lifetime before handleStart side effects; append private binding to record; add per-challenge proof header on original normal health handler. Fixed-label monotonic trace observes original manager/restore calls; no verdict, guard or client restore body changed.'), stream, indent=2)
