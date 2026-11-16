from pathlib import Path
import subprocess,json,hashlib,re
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE='13c3485805f16e9ff94bf43ca35e748b5740338d'
rows=[]
for path in ['apps/desktop/tauri/Cargo.toml','apps/desktop/tauri/Cargo.lock','apps/desktop/tauri/src/modules/runtime/client_artifact_policy.rs','apps/desktop/tauri/src/modules/runtime/official_lifecycle024.rs','apps/desktop/tauri/src/modules/runtime/windows_lifecycle024_contract.rs']:
    old=subprocess.check_output(['git','-C',str(CODE),'show',f'{BASE}:{path}'])
    raw=(CODE/path).read_bytes();assert old.replace(b'\r\n',b'\n')==raw.replace(b'\r\n',b'\n'),path
    rows.append(dict(path=path,bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest(),baselineEquivalent=True))
frame=(CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree/anchored/observation/retirement_tests.rs').read_text(encoding='utf-8')
assert 'from_secs(10)' not in frame,'Tests must inherit original fixture cutoff, no fresh nested10s window'
assert not re.search(r'\bdebug_assert\b|cfg\(debug_assertions\)',frame)
with (ROOT/'unchanged-build-and-contract-inputs.json').open('x',encoding='utf-8') as out:
    json.dump(dict(baseline=BASE,unchangedInputs=rows,releaseProfileParametersChanged=False,nativeTestsInheritOriginalFixtureDeadline=True,newTestDebugOnlyAssertions=False,nativeReleaseDirectedScope='Only same existing-artifact test under original same cutoff, no release publication/W2/dualSID'),out,indent=2)
print(json.dumps(dict(unchangedInputs=len(rows),releaseProfileChanged=False)))
