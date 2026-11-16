from pathlib import Path
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
sha=lambda b:hashlib.sha256(b).hexdigest()
rows=[]
for method,definition,allowed in [
 ('retire_current_file','tree/anchored/observation/retirement.rs',{'tree/anchored/observation/retirement/witness.rs','tree/anchored/observation/retirement_tests.rs'}),
 ('restore_original_file','tree/anchored/observation/retirement.rs',{'tree/anchored/observation/retirement/witness.rs','tree/anchored/observation/retirement_tests.rs'}),
 ('prepare_native_recovery','v3/journal/recovery.rs',{'v3/journal/recovery.rs'}),
 ('execute_native_recovery','v3/journal/recovery.rs',{'v3/journal/recovery.rs'}),
 ('seal_native_recovery_tree','v3/journal/recovery/final_release.rs',{'v3/journal/recovery/final_release.rs'}),
 ('release_native_recovery','v3/journal/recovery/final_release.rs',{'v3/journal/recovery/final_release.rs'}),
]:
    path=BASE/definition;text=path.read_text(encoding='utf-8')
    assert re.search(r'^    fn '+method+r'\(',text,re.M),method
    assert not re.search(r'pub[^\n]*fn '+method+r'\(',text),method
    callers=[]
    for p in BASE.rglob('*.rs'):
        for number,line in enumerate(p.read_text(encoding='utf-8').splitlines(),1):
            if re.search(r'\.'+method+r'\s*\(',line):
                rel=p.relative_to(BASE).as_posix();assert rel in allowed,(method,rel,number)
                callers.append(dict(path=rel,line=number))
    assert callers
    rows.append(dict(method=method,private=True,definition=definition,moduleSha256=sha(path.read_bytes()),actualCallers=callers))
for definition,wrappers in [('v3/journal/recovery.rs',['component_prepare_recovery_for_test','component_execute_recovery_for_test','component_reacquire_same_window_for_test']),('v3/journal/recovery/final_release.rs',['component_release_recovery_for_test'])]:
    text=(BASE/definition).read_text(encoding='utf-8')
    for method in wrappers:assert re.search(r'#\[cfg\(test\)\]\s+pub\([^\n]+\) fn '+method,text),method
    assert 'self.source_action.is_some()' in text
gate=(BASE/'mod.rs').read_text(encoding='utf-8').split('pub(crate) fn require_durable_capture()',1)[1].split('/// Fixed NEW-session',1)[0]
assert 'Err(' in gate and 'Ok(' not in gate
unchanged=[]
for path in ['apps/desktop/tauri/Cargo.toml','apps/desktop/tauri/Cargo.lock','apps/desktop/tauri/src/modules/runtime/client_artifact_policy.rs','apps/desktop/tauri/src/modules/runtime/official_lifecycle024.rs','apps/desktop/tauri/src/modules/runtime/windows_lifecycle024_contract.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/mod.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/coordination.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/coordination/segments.rs']:
    raw=(CODE/path).read_bytes();old=subprocess.check_output(['git','-C',str(CODE),'show',f'139de1261:{path}'])
    assert raw.replace(b'\r\n',b'\n')==old.replace(b'\r\n',b'\n'),path
    unchanged.append(dict(path=path,bytes=len(raw),sha256=sha(raw),baselineEquivalent=True))
tests=(BASE/'tree/anchored/observation/retirement_release_tests.rs').read_text(encoding='utf-8')
assert 'from_secs' not in tests and 'Instant::now() +' not in tests and 'debug_assert' not in tests
with (ROOT/'execution-boundary.json').open('x',encoding='utf-8') as out:
    json.dump(dict(methods=rows,unchangedInputs=unchanged,publicStartOpen=False,actualStopContentFactoryJoined=False,
        privateNativeDriverTestOnlyEntrypoints=True,original10sFixtureCutoff=True,noNewTimeoutOrDebugOnlyAssertion=True,releaseProfileChanged=False),out,indent=2)
print(json.dumps(dict(privateMethods=len(rows),unchangedInputs=len(unchanged),publicStartOpen=False)))
