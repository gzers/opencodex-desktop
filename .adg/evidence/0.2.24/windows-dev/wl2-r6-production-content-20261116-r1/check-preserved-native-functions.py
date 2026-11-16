from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
sha=lambda b:hashlib.sha256(b).hexdigest()
prior=lambda p:subprocess.check_output(['git','-C',str(CODE),'show',f'c259886cf:{p}']).decode('utf-8').replace('\r\n','\n')
proofs=[]
path='apps/desktop/tauri/src/infrastructure/windows_diagnostic_roles024.rs'
new=(CODE/path).read_text(encoding='utf-8');old=prior(path)
begin=new.index('    /// Actual retained generations')
end=new.index('    pub(crate) fn check(&self, owner:',begin)
assert new[:begin]+new[end:]==old
proofs.append(dict(path=path,allOriginalFunctionsUnchanged=True,addedFactoryOnly=True,sourceSha256=sha(new.encode())))
path='apps/desktop/tauri/src/infrastructure/windows_official_lifecycle024/actions.rs'
new=(CODE/path).read_text(encoding='utf-8');old=prior(path)
begin=new.index('    pub(crate) fn original_stop_action(');end=new.index('    pub(super) fn bind_stop_action(',begin)
assert new[:begin]+new[end:]==old
proofs.append(dict(path=path,allOriginalActionWindowAndNativeActionFunctionsUnchanged=True,addedOwnerStopSelectorOnly=True,sourceSha256=sha(new.encode())))
for leaf,old_sig,new_sig in [
 ('v3/journal/batches.rs','    fn recover_native_batch(','    pub(super) fn recover_native_batch('),
 ('v3/journal/recovery.rs','    fn prepare_native_recovery(','    pub(super) fn prepare_native_recovery('),
 ('v3/journal/history.rs','    fn seal_original_history(','    pub(super) fn seal_original_history('),
 ('v3/journal/snapshots.rs','    fn stopped_source(','    pub(super) fn stopped_source('),
 ('v3/journal/recovery/final_release.rs','    fn release_native_recovery(','    pub(in crate::infrastructure::windows_client_originals024::v3::journal) fn release_native_recovery('),
]:
    path=(BASE/leaf).relative_to(CODE).as_posix();new=(CODE/path).read_text(encoding='utf-8');old=prior(path)
    assert new.count(new_sig)==1 and new.replace(new_sig,old_sig)==old,path
    proofs.append(dict(path=path,allOriginalNativeAndCanonicalBodiesUnchanged=True,onlyAncestorModuleVisibilityChanged=True,sourceSha256=sha(new.encode())))
path=(BASE/'tree/anchored/observation/series.rs').relative_to(CODE).as_posix()
new=(CODE/path).read_text(encoding='utf-8');old=prior(path)
tail='impl AnchoredTree {'
assert new[new.index(tail):]==old[old.index(tail):]
proofs.append(dict(path=path,completeExistingBothTreeBarrierAndSharedPolicyCallByteIdentical=True,privateNativeQualifiedWrapperAdded=True,sourceSha256=sha(new.encode())))
core=(BASE/'tree/anchored/capture.rs').read_text(encoding='utf-8')
assert '#[cfg(test)]\n    pub(in crate::infrastructure::windows_client_originals024) fn inspect_original_retirement' in core
with (ROOT/'preserved-native-functions.json').open('x',encoding='utf-8') as out:json.dump(dict(sourceBase='c259886cf',
    proofs=proofs,oldUnassessedProductionPreviewRemovedByCfgTest=True,sharedAlgorithmsAndBudgetsModified=False,
    actualProductionFactoriesExecuted=False),out,indent=2)
print(json.dumps(dict(oldFunctionSetsUnchanged=len(proofs),unassessedPreviewTestOnly=True,productionExecution=False)))
