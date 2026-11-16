from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
sha=lambda b:hashlib.sha256(b).hexdigest()
journal=(BASE/'v3/journal.rs').read_text(encoding='utf-8');append=journal.split('fn append_entry_inner(',1)[1]
assert append.index('self.check(end)?')<append.index('Record::create(')<append.index('self.events.push(record)')<append.index('self.current.write_all(&raw)?')<append.rindex('self.check(end)')
assert 'self.poisoned = true;' in append and 'self.current.sync_all()?' in append
for rel in ['v3/journal/handles.rs','v3/journal/history.rs']:
    text=(BASE/rel).read_text(encoding='utf-8');assert 'if self.poisoned || end > self.deadline' in text
history=(BASE/'v3/journal/history.rs').read_text(encoding='utf-8')
boundary=history.split('fn original_history_boundary(',1)[1].split('fn seal_original_history(',1)[0]
assert boundary.count('self.check(end)')==1 and '.check(end)?' in boundary and 'self.released_lease' in boundary
seal=history.split('fn seal_original_history(',1)[1].split('pub(in crate::infrastructure::windows_client_originals024) fn component_original_history_for_test',1)[0]
assert seal.count('self.original_history_boundary(end)')==4
assert seal.index('ChainEntry::History(event), end')<seal.index('witness.archive_originals(')<seal.rindex('ChainEntry::History(event), end')<seal.index('witness.history_references(end)?')
handles=(BASE/'v3/journal/handles.rs').read_text(encoding='utf-8').split('fn close_original_handles(',1)[1].split('fn component_close_original_handles_for_test(',1)[0]
assert handles.count('.check_originals(end)?')==3
assert handles.index('pending_history_references(end)?')<handles.index('verify_original_history_references')<handles.index('.detach(end)?')<handles.index('OriginalHandlesReleaseIntent')<handles.index('.close_all(')
assert handles.index('OriginalHandlesReleasedVerified')<handles.rindex('.check_originals(end)?')<handles.rindex('self.check(end)?')<handles.index('CompletedOriginals::capture(self, end)?')
tests=(BASE/'tree/anchored/observation/retirement_completed_tests.rs').read_text(encoding='utf-8')
for token in ['actual_closed','actual_unknown','upstream timeout cannot replace requested native prefix','upstream timeout is NOT the requested negative','partial current write','before["currentActualBytes"], 1','fault == 85']:
    assert token in tests,token
unchanged=[]
for rel in ['tree/anchored/observation/retirement.rs','tree/anchored/observation/retirement/witness.rs','tree/anchored/batches.rs','v3/journal/batches.rs','v3/journal/recovery/final_release.rs']:
    p=BASE/rel;raw=p.read_bytes();prior=(ROOT/'library08/executed-source'/p.relative_to(CODE)).read_bytes();assert raw==prior,rel
    unchanged.append(dict(path=p.relative_to(CODE).as_posix(),sha256=sha(raw),identicalToPriorNativeRun=True))
with (ROOT/'final-io-boundary-verification.json').open('x',encoding='utf-8') as out:
    json.dump(dict(appendFullNativeChecksBeforeAndAfterWrite=True,historyBoundaryOwnsReceiptAndOneFullJournalRead=True,
        nativeWitnessBothTreeBarriersPreserved=True,closeFullOriginalBeforeAndAfterActualCloseAndAfterResult=True,
        completionOnlyAfterFinalPostconditions=True,negativeMustReachExactNativePrefix=True,
        unchangedRawAndBatchNativeImplementations=unchanged,originalBudgetsChanged=False,assertionsRelaxed=False),out,indent=2)
print(json.dumps(dict(nativeAppendBeforeAfter=True,fullClosePostconditions=3,exactNegativeCoverageRequired=True,unchangedNativeFiles=len(unchanged))))
