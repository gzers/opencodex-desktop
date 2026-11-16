from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
sha=lambda b:hashlib.sha256(b).hexdigest()
rows=[]
paths=list(BASE.rglob('*.rs'))
for method,decl,allowed in [
 ('retire_current_file','tree/anchored/observation/retirement.rs',{'tree/anchored/observation/retirement/witness.rs','tree/anchored/observation/retirement_tests.rs'}),
 ('restore_original_file','tree/anchored/observation/retirement.rs',{'tree/anchored/observation/retirement/witness.rs','tree/anchored/observation/retirement_tests.rs'}),
 ('prepare_native_recovery','v3/journal/recovery.rs',{'v3/journal/recovery.rs'}),
 ('execute_native_recovery','v3/journal/recovery.rs',{'v3/journal/recovery.rs'}),
]:
    definition=BASE/decl;text=definition.read_text(encoding='utf-8')
    assert re.search(r'^    fn '+method+r'\(',text,re.M),method
    assert not re.search(r'pub[^\n]*fn '+method+r'\(',text),method
    callers=[]
    for p in paths:
        for line_number,line in enumerate(p.read_text(encoding='utf-8').splitlines(),1):
            if re.search(r'\.'+method+r'\s*\(',line):
                rel=p.relative_to(BASE).as_posix();assert rel in allowed,(method,rel,line_number)
                callers.append(dict(path=rel,line=line_number))
    assert callers
    rows.append(dict(method=method,private=True,definition=decl,moduleSha256=sha(definition.read_bytes()),actualCallers=callers))
driver=(BASE/'v3/journal/recovery.rs').read_text(encoding='utf-8')
for wrapper in ['component_prepare_recovery_for_test','component_execute_recovery_for_test','component_reacquire_same_window_for_test']:
    assert re.search(r'#\[cfg\(test\)\]\s+pub\([^\n]+\) fn '+wrapper,driver)
assert 'self.source_action.is_some()' in driver
witness=(BASE/'tree/anchored/observation/retirement/witness.rs').read_text(encoding='utf-8')
assert '#[derive' not in witness and 'from_value' not in witness and 'from_slice' not in witness
assert "trees: &'a mut [AnchoredTree; 2]" in witness
assert 'pub(in crate::infrastructure::windows_client_originals024::tree::anchored) fn recovery_witness' in witness
gate=(BASE/'mod.rs').read_text(encoding='utf-8').split('pub(crate) fn require_durable_capture()',1)[1].split('/// Fixed NEW-session',1)[0]
assert 'Err(' in gate and 'Ok(' not in gate
with (ROOT/'execution-boundary.json').open('x',encoding='utf-8') as out:
    json.dump(dict(methods=rows,privateNativeDriversOnlyTestEntrypoints=True,witnessOnlyBorrowsNativeTrees=True,noSerdeWitness=True,publicStartOpen=False,actualStopContentFactoryJoined=False),out,indent=2)
print(json.dumps(dict(privateMethods=len(rows),publicStartOpen=False,productionRecoveryEntrypoint=False)))
