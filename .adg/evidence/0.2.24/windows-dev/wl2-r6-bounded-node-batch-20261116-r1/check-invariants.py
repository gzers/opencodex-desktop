from pathlib import Path
import hashlib,json,subprocess,re
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
sha=lambda b:hashlib.sha256(b).hexdigest()
baseline='d20103fee084a3e63f728bf7c6664af65405e456'
old=lambda rel:subprocess.check_output(['git','-C',str(CODE),'show',f'{baseline}:{rel}']).decode('utf-8').replace('\r\n','\n')
unchanged=[]
for rel in ['apps/desktop/tauri/Cargo.toml','apps/desktop/tauri/Cargo.lock','apps/desktop/tauri/src/modules/runtime/client_artifact_policy.rs',
 'apps/desktop/tauri/src/modules/runtime/official_lifecycle024.rs','apps/desktop/tauri/src/modules/runtime/windows_lifecycle024_contract.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/bounded.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/handle_release.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/coordination.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/coordination/segments.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree/anchored/observation/retirement_tests.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree/anchored/observation/retirement_release_tests.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree/anchored/capture_tests.rs']:
    p=CODE/rel;raw=p.read_bytes();assert raw.replace(b'\r\n',b'\n').decode()==old(rel),rel
    unchanged.append(dict(path=rel,bytes=len(raw),sha256=sha(raw),byteEquivalentWithExplicitCrlfNormalization=True))
anchored=(BASE/'tree/anchored.rs').read_text(encoding='utf-8')
assert anchored.replace('pub(super) mod batches;\n','')==old((BASE/'tree/anchored.rs').relative_to(CODE).as_posix())
native=(BASE/'tree/anchored/observation/retirement.rs').read_text(encoding='utf-8')
prior=old((BASE/'tree/anchored/observation/retirement.rs').relative_to(CODE).as_posix())
body=lambda text:text.split('fn ordinary(',1)[1].split('#[cfg(test)]\n#[path',1)[0]
assert body(native)==body(prior),'all old retirement/native file predicates unchanged'
new_journal=(BASE/'v3/journal.rs').read_text(encoding='utf-8');old_journal=old((BASE/'v3/journal.rs').relative_to(CODE).as_posix())
new_body=new_journal.split('        if native::current_identity()? != self.token {',1)[1].split('    pub(in super::super) fn check(',1)[0]
old_body=old_journal.split('        if native::current_identity()? != self.token {',1)[1].split('    pub(in super::super) fn check(',1)[0]
# Only the new current.batch comparison is added; all old native and record
# comparisons below the original token gate remain literally unchanged.
assert new_body.replace('            || tip.batch != self.progress.batch_summary()\n','')==old_body
assert 'if self.poisoned || end > authorized_end' in new_journal and 'before(authorized_end)?' in new_journal
batch=(BASE/'v3/batches.rs').read_text(encoding='utf-8')
compact=re.sub(r'\s+','',batch)
assert 'raw.len()>1024*1024' in compact and 'e.count>512' in compact and 'e.rows.len()>512' in compact
assert 'projected.sequence = self.sequence;' in batch and 'projected.tip = self.tip.clone();' in batch
assert '*self = projected;' in batch and 'self.apply(plan, event)' in batch and 'self.apply_recovery(plan, event)' in batch
assert batch.count('pub(super) enum RowEvent')==1
row= batch.split('pub(super) enum RowEvent',1)[1].split('#[derive',1)[0];assert 'Batch(' not in row
for file,method in [('v3.rs','fn apply('),('v3/segments.rs','fn apply_lease('),('v3/actions.rs','fn apply_action('),
    ('v3/observations.rs','fn apply_work('),('v3/snapshots.rs','fn apply_snapshot('),('v3/recovery.rs','fn apply_recovery('),
    ('v3/history.rs','fn apply_history('),('v3/handles.rs','fn apply_handles(')]:
    text=(BASE/file).read_text(encoding='utf-8').split(method,1)[1]
    assert 'if self.batch.is_some()' in text[:650],(file,method)
witness=(BASE/'tree/anchored/batches.rs').read_text(encoding='utf-8')
assert '#[derive' not in witness and 'fn observed(' in witness
assert 'pub(in crate::infrastructure::windows_client_originals024::tree::anchored) fn observed' in witness
assert 'file' not in witness.split('pub(in crate::infrastructure::windows_client_originals024) struct VerifiedBatchValue',1)[1].split('impl VerifiedBatchValue',1)[0].lower()
gate=(BASE/'mod.rs').read_text(encoding='utf-8').split('pub(crate) fn require_durable_capture()',1)[1].split('/// Fixed NEW-session',1)[0]
assert 'Err(' in gate and 'Ok(' not in gate
with (ROOT/'invariant-verification.json').open('x',encoding='utf-8') as out:
    json.dump(dict(sourceBase=baseline,unchangedSafetyAndOriginalAssertionFiles=unchanged,
        rawAnchoredMutationImplementationUnchanged=True,rawRetirementImplementationUnchanged=True,
        allPriorJournalNativeComparisonsRetainedWithOnlyNewBatchCheck=True,pendingBatchBlocksOldDataPaths=True,
        noRecursiveBatchRow=True,projectionNeverCopiesNativeObjects=True,originalDeadlinesAndBoundsUnchanged=True,
        nativeWitnessNoSerdeOrClone=True,publicStartOpen=False,productionReadyStopContentQualificationJoined=False),out,indent=2)
print(json.dumps(dict(unchangedFiles=len(unchanged),rawNativeChecksPreserved=True,originalDeadlinePreserved=True,publicStartOpen=False)))
