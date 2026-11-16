from pathlib import Path
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
sha=lambda b:hashlib.sha256(b).hexdigest()
rows=[]
for method,definition,allowed in [
 ('seal_original_history','v3/journal/history.rs',{'v3/journal/history.rs'}),
 ('release_native_recovery','v3/journal/recovery/final_release.rs',{'v3/journal/recovery/final_release.rs'}),
 ('close_original_handles','v3/journal/handles.rs',{'v3/journal/handles.rs'}),
]:
    path=BASE/definition;text=path.read_text(encoding='utf-8');assert re.search(r'^    fn '+method+r'\(',text,re.M)
    assert not re.search(r'pub[^\n]*fn '+method+r'\(',text)
    callers=[]
    for p in BASE.rglob('*.rs'):
        for number,line in enumerate(p.read_text(encoding='utf-8').splitlines(),1):
            if re.search(r'\.'+method+r'\s*\(',line):
                rel=p.relative_to(BASE).as_posix();assert rel in allowed,(method,rel,number);callers.append(dict(path=rel,line=number))
    assert callers;rows.append(dict(method=method,definition=definition,moduleSha256=sha(path.read_bytes()),private=True,actualCallers=callers))
driver=(BASE/'v3/journal/handles.rs').read_text(encoding='utf-8')
assert re.search(r'#\[cfg\(test\)\]\s+pub\([^\n]+\) fn component_close_original_handles_for_test',driver)
assert 'self.source_action.is_some()' in driver
close=(BASE/'handle_release.rs').read_text(encoding='utf-8')
assert 'file.into_raw_handle()' in close and 'CloseHandle(raw)' in close
assert 'from_raw_handle' not in close and 'SetHandleInformation(file.as_raw_handle(), 2, 2)' in close
assert close.count('SetHandleInformation(')==1
assert re.search(r'#\[cfg\(test\)\]\s+pub\([^\n]+\) fn protect_first_for_test',close)
gate=(BASE/'mod.rs').read_text(encoding='utf-8').split('pub(crate) fn require_durable_capture()',1)[1].split('/// Fixed NEW-session',1)[0]
old_gate=subprocess.check_output(['git','-C',str(CODE),'show','b30cc9132:apps/desktop/tauri/src/infrastructure/windows_client_originals024/mod.rs']).decode().split('pub(crate) fn require_durable_capture()',1)[1].split('/// Fixed NEW-session',1)[0]
assert gate==old_gate and 'Err(' in gate and 'Ok(' not in gate
unchanged=[]
for path in ['apps/desktop/tauri/Cargo.toml','apps/desktop/tauri/Cargo.lock','apps/desktop/tauri/src/modules/runtime/client_artifact_policy.rs','apps/desktop/tauri/src/modules/runtime/official_lifecycle024.rs','apps/desktop/tauri/src/modules/runtime/windows_lifecycle024_contract.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/bounded.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree/anchored/observation/retirement_tests.rs']:
    raw=(CODE/path).read_bytes();old=subprocess.check_output(['git','-C',str(CODE),'show',f'b30cc9132:{path}'])
    assert raw.replace(b'\r\n',b'\n')==old.replace(b'\r\n',b'\n'),path
    unchanged.append(dict(path=path,sha256=sha(raw),bytes=len(raw),baselineEquivalent=True))
test=(BASE/'tree/anchored/observation/retirement_handoff_tests.rs').read_text(encoding='utf-8');assert 'from_secs' not in test and 'Instant::now() +' not in test and 'debug_assert' not in test
with (ROOT/'execution-boundary.json').open('x',encoding='utf-8') as out:
    json.dump(dict(methods=rows,unchangedInputs=unchanged,publicStartOpen=False,typedProductionCloseQualified=False,
        closeOnlyActualFileOwnership=True,noFileReconstructionOrNativeCloseRetry=True,testProtectionNeverCleared=True,
        original10sFixtureCutoff=True,releaseProfileChanged=False),out,indent=2)
print(json.dumps(dict(privateDrivers=len(rows),unchangedInputs=len(unchanged),publicStartOpen=False,unknownCloseRetried=False)))
