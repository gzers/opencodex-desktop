"""Isolated full-source producer overlay. Never replaces production archive/pins."""
from pathlib import Path
import hashlib
import json
import os
import shutil
import sys

assert os.name == 'nt' and os.environ.get('GITHUB_ACTIONS') == 'true'
ROOT = Path(__file__).resolve().parents[2]
original, output, report = [Path(p).resolve() for p in sys.argv[1:4]]
boundary = Path(os.environ['RUNNER_TEMP']).resolve()
for p in [original, output, report]:
    p.relative_to(boundary)
assert not output.exists() and not report.exists()
assert json.loads((original / 'package.json').read_bytes())['version'] == '2.97.0-desktop023.lifecycle.1-record-prototype'
shutil.copytree(original, output)
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


def cli(text):
    text = text.replace('import { serviceStayOutExitCode', 'import { acquireWindowsRuntimeLifetime } from "../lib/windows-runtime-lifetime";\nimport { serviceStayOutExitCode', 1)
    return replace_one(text, 'async function handleStart(options: { block?: boolean } = {}) {',
                       'async function handleStart(options: { block?: boolean } = {}) {\n  acquireWindowsRuntimeLifetime(getConfigDir());')


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
                   source=os.environ.get('GITHUB_SHA'), productionPinsChanged=False,
                   scope='Separate disposable producer clone: acquire native lifetime before handleStart side effects; append private binding to record; add per-challenge proof header on original normal health handler. No Stop/client restore code changed.'), stream, indent=2)
