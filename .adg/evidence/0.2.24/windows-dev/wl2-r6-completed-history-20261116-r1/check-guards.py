from pathlib import Path
import hashlib,json,subprocess,re
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
sha=lambda b:hashlib.sha256(b).hexdigest()
baseline='d20103fee084a3e63f728bf7c6664af65405e456'
path='apps/desktop/tauri/src/infrastructure/windows_client_originals024/v3/journal.rs'
old=subprocess.check_output(['git','-C',str(CODE),'show',f'{baseline}:{path}']).decode().replace('\r\n','\n')
new=(CODE/path).read_text(encoding='utf-8')
start='        if native::current_identity()? != self.token {'
stop='    pub(in super::super) fn check('
old_native=old[old.index(start):old.index(stop)]
new_native=new[new.index(start):new.index(stop)]
assert old_native==new_native,'full native journal comparison body must remain exact'
assert 'if self.poisoned || end > authorized_end' in new and 'before(authorized_end)?;' in new
append=new.split('fn append_entry_inner(',1)[1];assert append.index('self.check(end)?')<append.index('Record::create(')
completed=(BASE/'v3/journal/completed.rs').read_text(encoding='utf-8')
assert '#[derive' not in completed and 'Deserialize' not in completed and 'self.deadline =' not in completed
assert 'Duration::from_secs(5)' in completed and 'when > Instant::now()' in completed
assert completed.count('.matches(self)?')==2 and completed.count('self.check_records(end)?')==2
assert re.search(r'#\[cfg\(test\)\]\s+pub\([^\n]+\) fn component_completed_history_at_for_test',completed)
callers=[]
for p in BASE.rglob('*.rs'):
    for n,line in enumerate(p.read_text(encoding='utf-8').splitlines(),1):
        if '.check_records(' in line:callers.append(dict(path=p.relative_to(BASE).as_posix(),line=n))
assert {r['path'] for r in callers}=={'v3/journal.rs','v3/journal/completed.rs'}
handles=(BASE/'v3/journal/handles.rs').read_text(encoding='utf-8')
capture='CompletedOriginals::capture(self, end)?'
assert capture in handles and handles.index('OriginalHandlesReleasedVerified')<handles.index(capture)
assert handles.index('self.close_completion = Some(')>handles.rindex('.check_originals(end)?')
unchanged=[]
for rel in ['handle_release.rs','bounded.rs','coordination.rs','coordination/segments.rs','tree/anchored/observation/retirement/handoff.rs','v3/journal/recovery/final_release.rs','v3/journal/history.rs']:
    p=BASE/rel;raw=p.read_bytes();prior=subprocess.check_output(['git','-C',str(CODE),'show',f'{baseline}:{p.relative_to(CODE).as_posix()}'])
    assert raw.replace(b'\r\n',b'\n')==prior.replace(b'\r\n',b'\n'),rel
    unchanged.append(dict(path=p.relative_to(CODE).as_posix(),bytes=len(raw),sha256=sha(raw),baselineEquivalent=True))
recovery=(BASE/'v3/journal/recovery.rs').read_text(encoding='utf-8')
assert 'if self.poisoned || end > self.deadline' in recovery
execute=recovery.split('fn execute_native_recovery(',1)[1].split('#[cfg(test)]\n    pub',1)[0]
assert execute.count('self.recovery_native_check(')==2 and execute.count('self.recovery_journal_claim(')==2
assert execute.index('self.append_entry_inner(')<execute.index('witness.retire(')<execute.rindex('self.append_entry_inner(')
assert 'self.append_entry_inner(' in new and 'self.events.push(record)' in new
witness=(BASE/'tree/anchored/observation/retirement/witness.rs').read_text(encoding='utf-8')
for method,next_method in [('retire','restore'),('restore','restored')]:
    block=witness.split('fn '+method+'(',1)[1].split('fn '+next_method+'(',1)[0]
    assert block.count('self.check(claim, end)')==2
gate=(BASE/'mod.rs').read_text(encoding='utf-8').split('pub(crate) fn require_durable_capture()',1)[1].split('/// Fixed NEW-session',1)[0]
assert 'Err(' in gate and 'Ok(' not in gate
with (ROOT/'guard-verification.json').open('x',encoding='utf-8') as out:
    json.dump(dict(baseline=baseline,exactCommonNativeBodySha256=sha(old_native.encode()),fullJournalNativeComparisonsUnchanged=True,
        mutationStillOriginalCutoff=True,noGenericDeadlineSetter=True,receiptNonSerdeNonClone=True,
        receiptAfterCanonicalResultAndNativePostconditions=True,fullRecordComparisonCallers=callers,
        unchangedNativeFiles=unchanged,publicStartOpen=False,productionStatusOrTypedClosedJoined=False),out,indent=2)
print(json.dumps(dict(fullNativeComparisonBodyIdentical=True,unchangedNativeFiles=len(unchanged),mutationCutoffReset=False,publicStartOpen=False)))
