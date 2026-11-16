from pathlib import Path
import hashlib,json,subprocess,re
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
sha=lambda b:hashlib.sha256(b).hexdigest()
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
rows=[]
for path in ['apps/desktop/tauri/Cargo.toml','apps/desktop/tauri/Cargo.lock','apps/desktop/tauri/src/modules/runtime/client_artifact_policy.rs','apps/desktop/tauri/src/modules/runtime/official_lifecycle024.rs','apps/desktop/tauri/src/modules/runtime/windows_lifecycle024_contract.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/mod.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/coordination/segments.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/v3/journal/leases.rs']:
    raw=(CODE/path).read_bytes();old=subprocess.check_output(['git','-C',str(CODE),'show',f'55a6d0010:{path}'])
    assert raw.replace(b'\r\n',b'\n')==old.replace(b'\r\n',b'\n'),path
    rows.append(dict(path=path,bytes=len(raw),sha256=sha(raw),baselineEquivalent=True))
debug=[r for r in load(ROOT/'library03/source-manifest.json') if r['path'].endswith('.rs')]
release=[r for r in load(ROOT/'library04/source-manifest.json') if r['path'].endswith('.rs')]
assert debug==release
for row in release:
    raw=(CODE/row['path']).read_bytes();assert sha(raw)==row['sha256']
for leaf in ['tree/anchored/observation/retirement_chain_tests.rs','v3/journal/recovery.rs']:
    text=(CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'/leaf).read_text(encoding='utf-8')
    assert 'from_secs(' not in text and 'debug_assert' not in text and 'cfg(debug_assertions)' not in text
case=(CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree/anchored/observation/retirement_chain_tests.rs').read_text(encoding='utf-8')
assert 'setup();' in case and 'let end =' not in case and 'Instant::now() +' not in case
data=dict(baseline='55a6d0010',unchangedInputs=rows,sameDebugAndReleaseRust=release,
    original10sFixtureDeadline=True,noNewTestDeadline=True,noAssertionsRemovedOrDebugOnly=True,
    productionLeaseOwnerRequirementUnchanged=True,releaseProfileChanged=False,
    releaseDirectedScope='Only two debug budget failures, same Rust/source/assertions/original cutoff, no W2 or publication')
with (ROOT/'unchanged-inputs-and-budget.json').open('x',encoding='utf-8') as out:json.dump(data,out,indent=2)
print(json.dumps(dict(unchangedInputs=len(rows),sameRust=len(release),cutoffRenewed=False)))
