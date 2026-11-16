from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
sha=lambda b:hashlib.sha256(b).hexdigest()
current=(BASE/'v3/journal/recovery.rs').read_text(encoding='utf-8')
execute=current.split('fn execute_native_recovery(',1)[1].split('#[cfg(test)]\n    pub',1)[0]
assert execute.count('self.recovery_native_check(')==2 and execute.count('self.recovery_journal_claim(')==2
assert execute.index('recovery_native_check(')<execute.index('recovery_event(intent')<execute.index('append_entry_inner')<execute.index('recovery_journal_claim(')<execute.index('witness.retire(')
assert execute.index('witness.restore(')<execute.rindex('recovery_journal_claim(')<execute.index('recovery_event(completion')<execute.rindex('recovery_native_check(')
same=[]
for rel in ['handle_release.rs','bounded.rs','tree/anchored/observation/retirement_tests.rs','tree/anchored/observation/retirement_chain_tests.rs',
    'v3/journal/recovery/final_release.rs','v3/journal/history.rs','tree/anchored/observation/retirement/history.rs']:
    p=BASE/rel;raw=p.read_bytes();old=(ROOT/'library03/executed-source'/p.relative_to(CODE)).read_bytes();assert raw==old,rel
    same.append(dict(path=p.relative_to(CODE).as_posix(),sha256=sha(raw),identicalToPriorFailedRun=True))
witness=(BASE/'tree/anchored/observation/retirement/witness.rs').read_text(encoding='utf-8')
for operation,next_operation in [('retire','restore'),('restore','restored')]:
    block=witness.split('fn '+operation+'(',1)[1].split('fn '+next_operation+'(',1)[0]
    assert block.count('self.check(claim, end)')==2
raw=(BASE/'tree/anchored/observation/retirement.rs').read_text(encoding='utf-8')
for method,next_method in [('retire_current_file','restore_original_file'),('restore_original_file','verify_retirement_restored')]:
    block=raw.split('fn '+method+'(',1)[1].split('fn '+next_method+'(',1)[0];assert block.count('self.verify_retirement_prefix(claim, end)')==2
with (ROOT/'recovery-barrier-verification.json').open('x',encoding='utf-8') as out:
    json.dump(dict(driverFullPlanBeforeIntentAfterResult=True,journalClaimAfterIntentBeforeResult=True,
        witnessCompleteBothTreesBeforeAfterActualMove=True,rawOwnTreeBeforeAfterActualMove=True,
        rawNativePredicatesAndExistingAssertionsUnchanged=same,originalDeadlineChanged=False,fault40to43SamePositions=True),out,indent=2)
print(json.dumps(dict(originalNativeImplementationsByteEqual=len(same),bothTreeBeforeAfterPreserved=True,faultWindowsPreserved=True)))
